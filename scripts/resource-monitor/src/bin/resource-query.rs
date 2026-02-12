use chrono::{Local, NaiveDateTime, TimeZone};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    date: String,
    date_provided: bool,
    time: Option<String>,
    top_n: usize,
}

#[derive(Debug)]
struct Snapshot {
    timestamp: String,
    lines: Vec<String>,
}

fn main() {
    let args = match parse_cli() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("resource-query: {err}");
            std::process::exit(2);
        }
    };

    if let Some(time) = &args.time {
        let ts = format!("{} {time}:00", args.date);
        if to_epoch(&ts).is_none() {
            eprintln!("resource-query: invalid date/time format, expected YYYY-MM-DD HH:MM");
            std::process::exit(2);
        }
    }

    if let Err(err) = run(args) {
        eprintln!("resource-query: {err}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> io::Result<()> {
    let log_dir = log_dir()?;
    let log_path = log_dir.join(format!("{}.log", args.date));

    if !log_path.exists() {
        println!("No log file found for {}", args.date);
        let dates = list_dates(&log_dir)?;
        if !dates.is_empty() {
            println!("Available dates:");
            for date in dates {
                println!("  {date}");
            }
        }
        return Ok(());
    }

    let content = fs::read_to_string(log_path)?;
    let snapshots = parse_snapshots(&content);

    if let Some(time) = args.time {
        match find_closest(&snapshots, &args.date, &time) {
            Some(snapshot) => println!("{}", trim_snapshot(snapshot, args.top_n)),
            None => println!("No snapshots found for {}", args.date),
        }
    } else if args.date_provided {
        let rendered = snapshots
            .iter()
            .map(|snap| trim_snapshot(snap, args.top_n))
            .collect::<Vec<_>>()
            .join("\n\n");
        println!("{rendered}");
    } else {
        match snapshots.last() {
            Some(snapshot) => println!("{}", trim_snapshot(snapshot, args.top_n)),
            None => println!("No snapshots found in today's log"),
        }
    }

    Ok(())
}

fn parse_cli() -> Result<Args, String> {
    let mut positionals: Vec<String> = Vec::new();
    let mut top_n = 10usize;

    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--top" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --top".to_string())?;
                top_n = parse_top(&value)?;
            }
            _ if arg.starts_with("--top=") => {
                let value = arg.trim_start_matches("--top=");
                top_n = parse_top(value)?;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown flag: {arg}"));
            }
            _ => positionals.push(arg),
        }
    }

    if positionals.len() > 2 {
        return Err("too many positional arguments".to_string());
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let date = positionals
        .first()
        .cloned()
        .unwrap_or_else(|| today.clone());
    let time = positionals.get(1).cloned();

    Ok(Args {
        date,
        date_provided: !positionals.is_empty(),
        time,
        top_n,
    })
}

fn parse_top(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("invalid value for --top: {value}. Expected a positive integer."))?;
    if parsed == 0 {
        return Err("invalid value for --top: expected a positive integer.".to_string());
    }
    Ok(parsed)
}

fn print_help() {
    println!(
        "Usage: resource-query [YYYY-MM-DD] [HH:MM] [--top N]

  No args           Show the last snapshot from today
  YYYY-MM-DD        Show all snapshots for that day
  YYYY-MM-DD HH:MM  Show the closest snapshot to that time
  --top N           Show N processes per section (default: 10)"
    );
}

fn log_dir() -> io::Result<PathBuf> {
    let home =
        env::var("HOME").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home).join(".local/share/resource-monitor"))
}

fn list_dates(log_dir: &PathBuf) -> io::Result<Vec<String>> {
    let entries = match fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };

    let mut dates = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".log") && !name.starts_with("launchd") {
                Some(name.trim_end_matches(".log").to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    dates.sort();
    Ok(dates)
}

fn parse_snapshots(content: &str) -> Vec<Snapshot> {
    let mut snapshots = Vec::new();
    let mut current = Vec::new();

    for line in content.lines() {
        if line.starts_with("=== ") && !current.is_empty() {
            snapshots.push(snapshot_from_lines(std::mem::take(&mut current)));
        }
        current.push(line.to_string());
    }

    if !current.is_empty() {
        snapshots.push(snapshot_from_lines(current));
    }

    snapshots
}

fn snapshot_from_lines(lines: Vec<String>) -> Snapshot {
    let timestamp = lines
        .first()
        .and_then(|line| line.strip_prefix("=== "))
        .and_then(|line| line.strip_suffix(" ==="))
        .unwrap_or("")
        .to_string();

    Snapshot { timestamp, lines }
}

fn trim_snapshot(snapshot: &Snapshot, top_n: usize) -> String {
    let mut out = Vec::new();
    let mut count = 0usize;

    for line in &snapshot.lines {
        if line.starts_with("=== ") || line.trim_start().starts_with("PID") {
            out.push(line.clone());
        } else if line.starts_with("-- ") {
            count = 0;
            out.push(line.clone());
        } else if !line.trim().is_empty() && count < top_n {
            count += 1;
            out.push(line.clone());
        } else if line.trim().is_empty() {
            out.push(line.clone());
        }
    }

    out.join("\n").trim_end().to_string()
}

fn find_closest<'a>(snapshots: &'a [Snapshot], date: &str, time: &str) -> Option<&'a Snapshot> {
    let target = to_epoch(&format!("{date} {time}:00"))?;

    snapshots
        .iter()
        .filter_map(|snapshot| {
            to_epoch(&snapshot.timestamp).map(|snap_time| {
                let diff = (snap_time as i128 - target as i128).abs();
                (snapshot, diff)
            })
        })
        .min_by_key(|(_, diff)| *diff)
        .map(|(snapshot, _)| snapshot)
}

fn to_epoch(ts: &str) -> Option<i64> {
    let naive = NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S").ok()?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.timestamp())
}
