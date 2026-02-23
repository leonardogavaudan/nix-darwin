use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, ExitStatus, Output};

#[cfg(unix)]
use std::os::unix::fs as unix_fs;

struct Config {
    source: PathBuf,
    worktree: PathBuf,
    dry_run: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;

    let source = canonicalize_dir(&config.source, "--source")?;
    let worktree = canonicalize_dir(&config.worktree, "--worktree")?;

    let Some(global_excludes) = get_global_excludes_file()? else {
        println!("No global core.excludesfile configured; nothing to copy.");
        return Ok(());
    };

    if !global_excludes.exists() {
        return Err(format!(
            "global excludes file does not exist: {}",
            global_excludes.display()
        ));
    }

    let global_excludes = canonicalize_existing_path(&global_excludes)
        .map_err(|err| format!("failed to canonicalize global excludes file: {err}"))?;

    let ignored_files = list_globally_ignored_untracked_files(&source, &global_excludes)?;

    if ignored_files.is_empty() {
        println!(
            "No untracked files matched global excludes file {}",
            global_excludes.display()
        );
        return Ok(());
    }

    let mut copied = 0usize;
    let mut skipped_missing = 0usize;
    let mut skipped_directory = 0usize;

    for rel_path in ignored_files {
        let src = source.join(&rel_path);
        if !src.exists() {
            skipped_missing += 1;
            continue;
        }

        let metadata = fs::symlink_metadata(&src)
            .map_err(|err| format!("failed to read metadata for {}: {err}", src.display()))?;

        if metadata.is_dir() {
            skipped_directory += 1;
            continue;
        }

        let dst = worktree.join(&rel_path);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }

        if config.dry_run {
            println!("[dry-run] would copy {}", rel_path.display());
            copied += 1;
            continue;
        }

        copy_file_or_symlink(&src, &dst)?;
        println!("✓ Copied {}", rel_path.display());
        copied += 1;
    }

    println!(
        "Done. Copied {copied} globally-ignored untracked file(s) into {} (skipped missing: {skipped_missing}, skipped directories: {skipped_directory})",
        worktree.display()
    );

    Ok(())
}

fn parse_args() -> Result<Config, String> {
    let mut source: Option<PathBuf> = None;
    let mut worktree: Option<PathBuf> = None;
    let mut dry_run = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --source".to_string())?;
                source = Some(PathBuf::from(value));
            }
            "--worktree" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --worktree".to_string())?;
                worktree = Some(PathBuf::from(value));
            }
            "--dry-run" => {
                dry_run = true;
            }
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            _ => {
                return Err(format!(
                    "unknown argument: {arg}. Run with --help for usage."
                ))
            }
        }
    }

    let source = source.ok_or_else(|| "--source is required".to_string())?;
    let worktree = worktree.ok_or_else(|| "--worktree is required".to_string())?;

    Ok(Config {
        source,
        worktree,
        dry_run,
    })
}

fn print_help() {
    println!("worktree-copy-global-ignored");
    println!();
    println!("Copy untracked files matched by git global core.excludesfile into a worktree.");
    println!();
    println!("USAGE:");
    println!("  worktree-copy-global-ignored --source <repo-root> --worktree <worktree-path> [--dry-run]");
}

fn canonicalize_dir(path: &Path, flag: &str) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("{flag} path does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("{flag} path is not a directory: {}", path.display()));
    }
    canonicalize_existing_path(path)
        .map_err(|err| format!("failed to canonicalize {}: {err}", path.display()))
}

fn canonicalize_existing_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    fs::canonicalize(path)
}

fn get_global_excludes_file() -> Result<Option<PathBuf>, String> {
    let output = Command::new("git")
        .args(["config", "--global", "--path", "--get", "core.excludesfile"])
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;

    match output.status.code() {
        Some(0) => {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() {
                return Ok(None);
            }
            Ok(Some(PathBuf::from(value)))
        }
        Some(1) => Ok(None),
        _ => Err(format_command_failure(
            "git config --global --path --get core.excludesfile",
            &output.status,
            &output.stderr,
        )),
    }
}

fn list_globally_ignored_untracked_files(
    repo_root: &Path,
    global_excludes: &Path,
) -> Result<Vec<PathBuf>, String> {
    let excludes_arg = global_excludes.to_string_lossy().to_string();
    let output = run_git(
        repo_root,
        &[
            "ls-files",
            "-o",
            "-i",
            "--exclude-from",
            &excludes_arg,
            "-z",
        ],
    )?;

    let mut files = Vec::new();
    for chunk in output.stdout.split(|b| *b == 0) {
        if chunk.is_empty() {
            continue;
        }
        let rel = String::from_utf8_lossy(chunk).to_string();
        if rel.is_empty() {
            continue;
        }
        files.push(PathBuf::from(rel));
    }

    Ok(files)
}

fn run_git(repo_root: &Path, args: &[&str]) -> Result<Output, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run git {}: {err}", args.join(" ")))?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(format_command_failure(
            &format!("git {}", args.join(" ")),
            &output.status,
            &output.stderr,
        ))
    }
}

fn format_command_failure(command: &str, status: &ExitStatus, stderr: &[u8]) -> String {
    let code = status
        .code()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "terminated by signal".to_string());
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if stderr.is_empty() {
        format!("{command} failed with exit status {code}")
    } else {
        format!("{command} failed with exit status {code}: {stderr}")
    }
}

fn copy_file_or_symlink(src: &Path, dst: &Path) -> Result<(), String> {
    remove_destination_if_exists(dst)?;

    let metadata = fs::symlink_metadata(src)
        .map_err(|err| format!("failed to read metadata for {}: {err}", src.display()))?;

    if metadata.is_file() {
        fs::copy(src, dst).map_err(|err| {
            format!(
                "failed to copy file {} -> {}: {err}",
                src.display(),
                dst.display()
            )
        })?;
        return Ok(());
    }

    if metadata.file_type().is_symlink() {
        #[cfg(unix)]
        {
            let target = fs::read_link(src)
                .map_err(|err| format!("failed to read symlink {}: {err}", src.display()))?;
            unix_fs::symlink(&target, dst).map_err(|err| {
                format!(
                    "failed to copy symlink {} -> {}: {err}",
                    src.display(),
                    dst.display()
                )
            })?;
            return Ok(());
        }

        #[cfg(not(unix))]
        {
            return Err(format!(
                "cannot copy symlink on this platform: {}",
                src.display()
            ));
        }
    }

    Err(format!(
        "unsupported ignored path type (not file/symlink): {}",
        src.display()
    ))
}

fn remove_destination_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)
        .map_err(|err| format!("failed to read metadata for {}: {err}", path.display()))?;

    if metadata.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
    } else {
        fs::remove_file(path)
            .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
    }

    Ok(())
}
