use chrono::Local;
use std::cmp::Ordering;
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

const RETENTION_DAYS: u64 = 90;
const TOP_N: usize = 10;

fn main() {
    if let Err(err) = run() {
        eprintln!("resource-logger: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let log_dir = log_dir()?;
    fs::create_dir_all(&log_dir)?;

    let (header, body) = capture_ps()?;
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let full = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let cpu_top = top_by(&body, 1);
    let mem_top = top_by(&body, 3);
    let snapshot = format!(
        "=== {full} ===\n-- CPU --\n{header}\n{}\n-- MEM --\n{header}\n{}\n\n",
        cpu_top.join("\n"),
        mem_top.join("\n")
    );

    let log_file = log_dir.join(format!("{date}.log"));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;
    file.write_all(snapshot.as_bytes())?;

    cleanup_old_logs(&log_dir)?;
    Ok(())
}

fn log_dir() -> io::Result<PathBuf> {
    let home =
        env::var("HOME").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home).join(".local/share/resource-monitor"))
}

fn capture_ps() -> io::Result<(String, Vec<String>)> {
    let output = Command::new("ps")
        .args(["-eo", "pid,pcpu,pmem,rss,comm"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to run ps command",
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(("PID  %CPU %MEM    RSS COMM".to_string(), Vec::new()));
    }

    let mut lines = trimmed.lines();
    let header = lines
        .next()
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| "PID  %CPU %MEM    RSS COMM".to_string());
    let body = lines
        .filter(|line| !line.trim().is_empty())
        .map(std::string::ToString::to_string)
        .collect();

    Ok((header, body))
}

fn top_by(lines: &[String], col: usize) -> Vec<String> {
    let mut sorted = lines.to_vec();
    sorted.sort_by(|a, b| {
        let va = parse_col(a, col);
        let vb = parse_col(b, col);
        vb.partial_cmp(&va).unwrap_or(Ordering::Equal)
    });
    sorted.into_iter().take(TOP_N).collect()
}

fn parse_col(line: &str, col: usize) -> f64 {
    line.split_whitespace()
        .nth(col)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn cleanup_old_logs(log_dir: &PathBuf) -> io::Result<()> {
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(RETENTION_DAYS * 86_400))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if !name.ends_with(".log") || name.starts_with("launchd") {
            continue;
        }

        let modified = match entry.metadata().and_then(|m| m.modified()) {
            Ok(ts) => ts,
            Err(_) => continue,
        };

        if modified < cutoff {
            let _ = fs::remove_file(entry.path());
        }
    }

    Ok(())
}
