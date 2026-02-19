use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const DEFAULT_MAX_COMMITS: usize = 20;
const CACHE_TIMEOUT_SECONDS: u64 = 5;
const IGNORE_PATTERNS: &[&str] = &[
    "options.json",
    "manpage",
    "manual-html",
    "darwin-help",
    "darwin-manual",
    "darwin-system",
    "darwin-uninstaller",
    "darwin-version",
    "darwin-option",
    "darwin-rebuild",
    "home-manager-generation",
    "home-manager-path",
    "home-manager-files",
    "home-manager-fonts",
    "home-manager-applications",
    "home-manager-agents",
    "home-configuration-reference",
    "activation-script",
    "activation-",
    "hm-modules-messages",
    "hm_Library",
    "check-link-targets",
    "system-path",
    "system-applications",
    "user-environment",
    "etc.drv",
    "vars.sh",
    "link.drv",
    "cleanup.drv",
    ".plist",
    "launchd",
    "source.drv",
    "go-modules",
];

#[derive(Debug)]
struct Config {
    flake_dir: PathBuf,
    flake_lock: PathBuf,
    max_commits: usize,
    apply: bool,
}

#[derive(Debug)]
struct CmdResult {
    stdout: String,
    stderr: String,
    status_code: i32,
    command: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {}", red("[ERROR]"), err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let flake_dir = config.flake_dir.to_string_lossy().to_string();

    info("Searching for most recent fully-cached nixpkgs commit...");
    info(&format!("Flake directory: {flake_dir}"));
    info(&format!("Checking up to {} commits", config.max_commits));
    println!();

    let current_rev = get_current_rev(&config)?;
    info(&format!(
        "Current nixpkgs: {}",
        short_rev(&current_rev).unwrap_or(current_rev.as_str())
    ));

    let config_name = get_darwin_config(&config)?;
    info(&format!("Darwin config: {config_name}"));
    println!();

    info("Fetching recent nixpkgs-unstable commits...");
    let commits = fetch_recent_commits(config.max_commits)?;
    if commits.is_empty() {
        return Err("No commits returned from GitHub API".to_string());
    }

    let mut found_commit: Option<String> = None;

    for (index, commit) in commits.iter().enumerate() {
        let short = short_rev(commit).unwrap_or(commit);
        print!(
            "  [{:>2}/{}] Checking {}... ",
            index + 1,
            config.max_commits,
            short
        );
        io::stdout()
            .flush()
            .map_err(|e| format!("stdout flush failed: {e}"))?;

        match check_commit_cached(commit, &config, &config_name) {
            Ok(uncached) if uncached.is_empty() => {
                println!("{}", green("fully cached!"));
                found_commit = Some(commit.clone());
                break;
            }
            Ok(uncached) => {
                println!("{}", yellow("needs building or missing cache"));
                for pkg in uncached.iter().take(5) {
                    println!("        {} {}", red("-"), pkg);
                }
            }
            Err(err) => {
                println!("{}", yellow("check failed"));
                println!("        {} {}", red("-"), err);
            }
        }
    }

    println!();

    if let Some(commit) = found_commit {
        success(&format!("Found fully-cached commit: {commit}"));

        if config.apply {
            info("Updating flake.lock to this commit...");
            update_nixpkgs_to_commit(&config, &commit)?;
            success(&format!(
                "flake.lock updated! Run 'sudo darwin-rebuild switch --flake {}' to apply.",
                flake_dir
            ));
        } else {
            println!();
            info("To apply this commit now, run:");
            println!(
                "  nix flake update nixpkgs --flake {} --override-input nixpkgs github:NixOS/nixpkgs/{}",
                flake_dir, commit
            );
        }
    } else {
        warn(&format!(
            "No fully-cached commit found in the last {} commits.",
            config.max_commits
        ));
    }

    Ok(())
}

fn parse_args() -> Result<Config, String> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut flake_dir = PathBuf::from(home).join(".config/nix-darwin");
    let mut max_commits = DEFAULT_MAX_COMMITS;
    let mut apply = false;

    let args: Vec<String> = env::args().skip(1).collect();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--apply" => {
                apply = true;
                index += 1;
            }
            "--max-commits" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--max-commits requires a value".to_string())?;
                max_commits = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid --max-commits value: {value}"))?;
                index += 2;
            }
            "--flake-dir" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--flake-dir requires a value".to_string())?;
                flake_dir = PathBuf::from(value);
                index += 2;
            }
            "-h" | "--help" => {
                println!(
                    "{}",
                    [
                        "Usage: nix-update-cached-rs [--apply] [--max-commits N] [--flake-dir PATH]",
                        "",
                        "Finds the most recent nixpkgs-unstable commit where all declared packages",
                        "(systemPackages + Home Manager home.packages) are remotely cached.",
                        "",
                        "Options:",
                        "  --apply           Update flake.lock to the found commit",
                        "  --max-commits N   Number of commits to scan (default: 20)",
                        "  --flake-dir PATH  Path to flake directory (default: ~/.config/nix-darwin)",
                    ]
                    .join("\n")
                );
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    if max_commits == 0 {
        return Err("--max-commits must be greater than 0".to_string());
    }

    let flake_lock = flake_dir.join("flake.lock");
    Ok(Config {
        flake_dir,
        flake_lock,
        max_commits,
        apply,
    })
}

fn get_current_rev(config: &Config) -> Result<String, String> {
    let lock = config.flake_lock.to_string_lossy().to_string();
    let result = run_ok("jq", &["-r", ".nodes.nixpkgs.locked.rev // empty", &lock])?;
    let rev = result.stdout.trim().to_string();
    if rev.is_empty() {
        Err(format!(
            "could not read nixpkgs rev from {}",
            config.flake_lock.display()
        ))
    } else {
        Ok(rev)
    }
}

fn get_darwin_config(config: &Config) -> Result<String, String> {
    let flake_ref = format!(
        "{}#darwinConfigurations",
        config.flake_dir.to_string_lossy()
    );
    let nix_eval = run_capture(
        "nix",
        &[
            "eval",
            &flake_ref,
            "--apply",
            "x: builtins.attrNames x",
            "--json",
        ],
    )?;

    if nix_eval.status_code == 0 {
        let names = jq_lines(&nix_eval.stdout, ".[]")?;
        if let Some(name) = names.first() {
            return Ok(name.to_string());
        }
    }

    let host = run_ok("hostname", &["-s"])?;
    let host_name = host.stdout.trim();
    if host_name.is_empty() {
        Err("could not determine darwin configuration name".to_string())
    } else {
        Ok(host_name.to_string())
    }
}

fn fetch_recent_commits(count: usize) -> Result<Vec<String>, String> {
    let url = format!(
        "https://api.github.com/repos/NixOS/nixpkgs/commits?sha=nixpkgs-unstable&per_page={count}"
    );

    let response = run_ok(
        "curl",
        &[
            "-fsSL",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: nix-update-cached-rs",
            &url,
        ],
    )?;

    jq_lines(&response.stdout, ".[].sha")
}

fn check_commit_cached(
    commit: &str,
    config: &Config,
    config_name: &str,
) -> Result<Vec<String>, String> {
    let backup = fs::read_to_string(&config.flake_lock)
        .map_err(|e| format!("failed to read {}: {e}", config.flake_lock.display()))?;

    let check_result = (|| -> Result<Vec<String>, String> {
        update_nixpkgs_to_commit(config, commit)?;

        let build_target = format!(
            "{}#darwinConfigurations.{}.system",
            config.flake_dir.to_string_lossy(),
            config_name
        );

        let dry_run = run_capture("nix", &["build", &build_target, "--dry-run"])?;
        let build_output = format!("{}\n{}", dry_run.stdout, dry_run.stderr);
        let mut missing = parse_uncached_packages(&build_output);

        let declared_paths = collect_declared_package_paths(config, config_name)?;
        for uncached_pkg in check_remote_cache_for_paths(&declared_paths) {
            missing.insert(uncached_pkg);
        }

        Ok(missing.into_iter().collect())
    })();

    let restore_result = fs::write(&config.flake_lock, backup.as_bytes())
        .map_err(|e| format!("failed to restore {}: {e}", config.flake_lock.display()));

    match (check_result, restore_result) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(err), Ok(())) => Err(err),
        (Ok(_), Err(restore_err)) => Err(restore_err),
        (Err(err), Err(restore_err)) => Err(format!("{err}; also {restore_err}")),
    }
}

fn update_nixpkgs_to_commit(config: &Config, commit: &str) -> Result<(), String> {
    let flake_dir = config.flake_dir.to_string_lossy().to_string();
    let override_ref = format!("github:NixOS/nixpkgs/{commit}");
    run_ok(
        "nix",
        &[
            "flake",
            "update",
            "nixpkgs",
            "--flake",
            &flake_dir,
            "--override-input",
            "nixpkgs",
            &override_ref,
        ],
    )?;
    Ok(())
}

fn collect_declared_package_paths(
    config: &Config,
    config_name: &str,
) -> Result<Vec<String>, String> {
    let mut paths = BTreeSet::new();
    for path in eval_system_package_paths(config, config_name)? {
        paths.insert(path);
    }

    match eval_home_manager_package_paths(config, config_name) {
        Ok(home_paths) => {
            for path in home_paths {
                paths.insert(path);
            }
        }
        Err(err) => {
            warn(&format!("Home Manager package check skipped: {err}"));
        }
    }

    Ok(paths.into_iter().collect())
}

fn eval_system_package_paths(config: &Config, config_name: &str) -> Result<Vec<String>, String> {
    let target = format!(
        "{}#darwinConfigurations.{}.config.environment.systemPackages",
        config.flake_dir.to_string_lossy(),
        config_name
    );

    let output = run_ok(
        "nix",
        &[
            "eval",
            "--json",
            &target,
            "--apply",
            "xs: map (p: p.outPath or null) xs",
        ],
    )?;

    let mut paths = jq_lines(&output.stdout, ".[] | select(. != null)")?;
    paths.retain(|p| p.starts_with("/nix/store/"));
    Ok(paths)
}

fn eval_home_manager_package_paths(
    config: &Config,
    config_name: &str,
) -> Result<Vec<String>, String> {
    let users_target = format!(
        "{}#darwinConfigurations.{}.config.home-manager.users",
        config.flake_dir.to_string_lossy(),
        config_name
    );

    let output = run_ok(
        "nix",
        &[
            "eval",
            "--json",
            &users_target,
            "--apply",
            "users: builtins.concatLists (builtins.attrValues (builtins.mapAttrs (_: u: map (p: p.outPath or null) (u.home.packages or [])) users))",
        ],
    )?;

    let mut paths = jq_lines(&output.stdout, ".[] | select(. != null)")?;
    paths.retain(|p| p.starts_with("/nix/store/"));
    Ok(paths)
}

fn check_remote_cache_for_paths(paths: &[String]) -> Vec<String> {
    let mut seen_hashes = HashSet::new();
    let mut missing = BTreeSet::new();

    for path in paths {
        if let Some((hash, name)) = split_store_path(path) {
            if should_ignore_derivation(&name) || !looks_versioned(&name) {
                continue;
            }
            if !seen_hashes.insert(hash.clone()) {
                continue;
            }
            if !is_hash_cached(&hash) {
                missing.insert(format!("{name} (not in cache.nixos.org)"));
            }
        }
    }

    missing.into_iter().collect()
}

fn is_hash_cached(hash: &str) -> bool {
    let url = format!("https://cache.nixos.org/{hash}.narinfo");
    let timeout = CACHE_TIMEOUT_SECONDS.to_string();
    match run_capture(
        "curl",
        &[
            "-sS",
            "-L",
            "--max-time",
            &timeout,
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            &url,
        ],
    ) {
        Ok(result) => result.status_code == 0 && result.stdout.trim() == "200",
        Err(_) => false,
    }
}

fn split_store_path(path: &str) -> Option<(String, String)> {
    let file = path.trim().rsplit('/').next()?;
    let (hash, name) = file.split_once('-')?;
    if hash.is_empty() || name.is_empty() {
        return None;
    }
    Some((hash.to_string(), name.to_string()))
}

fn parse_uncached_packages(output: &str) -> BTreeSet<String> {
    let mut missing = BTreeSet::new();
    let mut in_built_section = false;

    for line in output.lines() {
        if line.contains("will be built") {
            in_built_section = true;
            continue;
        }
        if line.contains("will be fetched") || (in_built_section && line.trim().is_empty()) {
            in_built_section = false;
            continue;
        }
        if in_built_section && line.contains("/nix/store/") {
            let drv = line.trim();
            if should_ignore_derivation(drv) {
                continue;
            }

            if let Some(file_name) = drv.rsplit('/').next() {
                let without_ext = file_name.strip_suffix(".drv").unwrap_or(file_name);
                let package_name = without_ext
                    .split_once('-')
                    .map(|(_, tail)| tail)
                    .unwrap_or(without_ext);
                missing.insert(package_name.to_string());
            }
        }
    }

    missing
}

fn should_ignore_derivation(drv: &str) -> bool {
    IGNORE_PATTERNS.iter().any(|pattern| drv.contains(pattern))
}

fn jq_lines(json_input: &str, query: &str) -> Result<Vec<String>, String> {
    let mut child = Command::new("jq")
        .arg("-r")
        .arg(query)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start jq for query '{query}': {e}"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(json_input.as_bytes())
            .map_err(|e| format!("failed to write JSON to jq stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to read jq output: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "jq query failed ({query}): {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn run_capture(cmd: &str, args: &[&str]) -> Result<CmdResult, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {}: {e}", format_command(cmd, args)))?;

    Ok(CmdResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status_code: output.status.code().unwrap_or(-1),
        command: format_command(cmd, args),
    })
}

fn run_ok(cmd: &str, args: &[&str]) -> Result<CmdResult, String> {
    let result = run_capture(cmd, args)?;
    if result.status_code == 0 {
        Ok(result)
    } else {
        Err(format!(
            "command failed (exit {}): {}\n{}",
            result.status_code,
            result.command,
            result.stderr.trim()
        ))
    }
}

fn format_command(cmd: &str, args: &[&str]) -> String {
    if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{cmd} {}", args.join(" "))
    }
}

fn short_rev(rev: &str) -> Option<&str> {
    if rev.len() >= 7 {
        Some(&rev[..7])
    } else {
        None
    }
}

fn looks_versioned(name: &str) -> bool {
    name.chars().any(|c| c.is_ascii_digit())
}

fn c(code: u8, text: &str) -> String {
    format!("\x1b[0;{code}m{text}\x1b[0m")
}

fn blue(text: &str) -> String {
    c(34, text)
}

fn green(text: &str) -> String {
    c(32, text)
}

fn yellow(text: &str) -> String {
    c(33, text)
}

fn red(text: &str) -> String {
    c(31, text)
}

fn info(msg: &str) {
    println!("{} {msg}", blue("[INFO]"));
}

fn success(msg: &str) {
    println!("{} {msg}", green("[OK]"));
}

fn warn(msg: &str) {
    println!("{} {msg}", yellow("[WARN]"));
}
