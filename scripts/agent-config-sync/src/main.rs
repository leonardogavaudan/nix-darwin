use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

struct Config {
    source: Option<PathBuf>,
    overlays: Option<PathBuf>,
    agents_dir: PathBuf,
    profile: String,
    claude_out: PathBuf,
    codex_out: PathBuf,
    pi_out: PathBuf,
}

#[derive(Default)]
struct Sections {
    shared: String,
    claude: String,
    codex: String,
    pi: String,
}

#[derive(Default, Clone, Deserialize)]
struct OverlayConfig {
    #[serde(default)]
    targets: BTreeMap<String, TargetOverlay>,
}

#[derive(Default, Clone, Deserialize)]
struct TargetOverlay {
    #[serde(default)]
    remove_sections: Vec<String>,
    #[serde(default)]
    section_replacements: BTreeMap<String, String>,
    prepend: Option<String>,
    append: Option<String>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;

    let (sections, source_label, mut overlays) = if let Some(source) = &config.source {
        let sections = load_sections(source)?;
        (
            sections,
            source.display().to_string(),
            OverlayConfig::default(),
        )
    } else {
        let shared_source = config.agents_dir.join("shared/AGENTS.md");
        let profile_source = config.agents_dir.join(&config.profile).join("AGENTS.md");

        if !profile_source.exists() {
            return Err(format!(
                "profile source not found: {}",
                profile_source.display()
            ));
        }

        let shared_sections = if shared_source.exists() {
            load_sections(&shared_source)?
        } else {
            Sections::default()
        };

        let profile_sections = load_sections(&profile_source)?;
        let merged_sections = merge_sections(&shared_sections, &profile_sections);

        let shared_overlays =
            load_overlay_file_if_exists(&config.agents_dir.join("shared/overlays.yaml"))?;
        let profile_overlays = load_overlay_file_if_exists(
            &config
                .agents_dir
                .join(&config.profile)
                .join("overlays.yaml"),
        )?;
        let merged_overlays = merge_overlay_configs(&shared_overlays, &profile_overlays);

        let source_label = if shared_source.exists() {
            format!("{} + {}", shared_source.display(), profile_source.display())
        } else {
            profile_source.display().to_string()
        };

        (merged_sections, source_label, merged_overlays)
    };

    if let Some(extra_overlays) = &config.overlays {
        let extra = load_overlay_file(extra_overlays)?;
        overlays = merge_overlay_configs(&overlays, &extra);
    }

    let claude_doc = apply_target_overlay(
        &render_document(
            "CLAUDE.md",
            &source_label,
            &sections.shared,
            &sections.claude,
        ),
        &overlays,
        "claude",
    )?;

    let codex_doc = apply_target_overlay(
        &render_document(
            "AGENTS.md",
            &source_label,
            &sections.shared,
            &sections.codex,
        ),
        &overlays,
        "codex",
    )?;

    let pi_doc = apply_target_overlay(
        &render_document("AGENTS.md", &source_label, &sections.shared, &sections.pi),
        &overlays,
        "pi",
    )?;

    write_file(&config.claude_out, &claude_doc)
        .map_err(|err| format!("failed to write {}: {err}", config.claude_out.display()))?;
    write_file(&config.codex_out, &codex_doc)
        .map_err(|err| format!("failed to write {}: {err}", config.codex_out.display()))?;
    write_file(&config.pi_out, &pi_doc)
        .map_err(|err| format!("failed to write {}: {err}", config.pi_out.display()))?;

    if config.source.is_none() {
        sync_profile_assets(&config)?;
    }

    println!("Wrote {}", config.claude_out.display());
    println!("Wrote {}", config.codex_out.display());
    println!("Wrote {}", config.pi_out.display());
    Ok(())
}

fn parse_args() -> Result<Config, String> {
    let home = env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let codex_home = env::var("CODEX_HOME").unwrap_or_else(|_| format!("{home}/.config/codex"));
    let pi_home = env::var("PI_CODING_AGENT_DIR").unwrap_or_else(|_| format!("{home}/.pi/agent"));

    let mut source: Option<PathBuf> = None;
    let mut overlays: Option<PathBuf> = None;
    let mut agents_dir = PathBuf::from(format!("{home}/.config/nix-darwin/agents"));
    let mut profile = env::var("AGENT_PROFILE").unwrap_or_else(|_| "personal".to_string());
    let mut claude_out = PathBuf::from(format!("{home}/.claude/CLAUDE.md"));
    let mut codex_out = PathBuf::from(format!("{codex_home}/AGENTS.md"));
    let mut pi_out = PathBuf::from(format!("{pi_home}/AGENTS.md"));

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --source".to_string())?;
                source = Some(PathBuf::from(value));
            }
            "--overlays" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --overlays".to_string())?;
                overlays = Some(PathBuf::from(value));
            }
            "--agents-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --agents-dir".to_string())?;
                agents_dir = PathBuf::from(value);
            }
            "--profile" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --profile".to_string())?;
                profile = value;
            }
            "--claude-out" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --claude-out".to_string())?;
                claude_out = PathBuf::from(value);
            }
            "--codex-out" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --codex-out".to_string())?;
                codex_out = PathBuf::from(value);
            }
            "--pi-out" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --pi-out".to_string())?;
                pi_out = PathBuf::from(value);
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

    if profile.trim().is_empty() {
        return Err("profile cannot be empty".to_string());
    }

    Ok(Config {
        source,
        overlays,
        agents_dir,
        profile,
        claude_out,
        codex_out,
        pi_out,
    })
}

fn print_help() {
    println!("agent-config-sync");
    println!();
    println!("Generate harness-specific instruction files.");
    println!();
    println!("USAGE:");
    println!("  agent-config-sync [--agents-dir PATH] [--profile NAME] [--claude-out PATH] [--codex-out PATH] [--pi-out PATH]");
    println!("  agent-config-sync --source PATH [--overlays PATH] [--claude-out PATH] [--codex-out PATH] [--pi-out PATH]");
    println!();
    println!("DEFAULTS:");
    println!("  --agents-dir  $HOME/.config/nix-darwin/agents");
    println!("  --profile     $AGENT_PROFILE or personal");
    println!("  --claude-out  $HOME/.claude/CLAUDE.md");
    println!("  --codex-out   $CODEX_HOME/AGENTS.md or $HOME/.config/codex/AGENTS.md");
    println!("  --pi-out      $PI_CODING_AGENT_DIR/AGENTS.md or $HOME/.pi/agent/AGENTS.md");
    println!();
    println!("NOTES:");
    println!("  - Layered mode reads:");
    println!("      <agents-dir>/shared/AGENTS.md (optional)");
    println!("      <agents-dir>/shared/overlays.yaml (optional)");
    println!("      <agents-dir>/<profile>/AGENTS.md (required)");
    println!("      <agents-dir>/<profile>/overlays.yaml (optional)");
    println!("  - Legacy mode (--source) reads a single canonical file.");
    println!("  - --overlays can add one extra overlays file in either mode.");
}

fn load_sections(path: &Path) -> Result<Sections, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;

    let has_any_markers = ["SHARED", "CLAUDE", "CODEX", "PI"]
        .iter()
        .any(|name| content.contains(&format!("<!-- BEGIN {name} -->")));

    if !has_any_markers {
        return Ok(Sections {
            shared: content.trim_matches('\n').to_string(),
            claude: String::new(),
            codex: String::new(),
            pi: String::new(),
        });
    }

    Ok(Sections {
        shared: extract_optional_section(&content, "SHARED")?,
        claude: extract_optional_section(&content, "CLAUDE")?,
        codex: extract_optional_section(&content, "CODEX")?,
        pi: extract_optional_section(&content, "PI")?,
    })
}

fn merge_sections(shared: &Sections, profile: &Sections) -> Sections {
    Sections {
        shared: merge_two_sections(&shared.shared, &profile.shared),
        claude: merge_two_sections(&shared.claude, &profile.claude),
        codex: merge_two_sections(&shared.codex, &profile.codex),
        pi: merge_two_sections(&shared.pi, &profile.pi),
    }
}

fn merge_two_sections(left: &str, right: &str) -> String {
    let left = left.trim();
    let right = right.trim();

    match (left.is_empty(), right.is_empty()) {
        (true, true) => String::new(),
        (false, true) => left.to_string(),
        (true, false) => right.to_string(),
        (false, false) => format!("{left}\n\n{right}"),
    }
}

fn extract_optional_section(content: &str, name: &str) -> Result<String, String> {
    let start_marker = format!("<!-- BEGIN {name} -->");
    let end_marker = format!("<!-- END {name} -->");

    let Some(start_index) = content.find(&start_marker) else {
        return Ok(String::new());
    };

    let after_start = start_index + start_marker.len();
    let tail = &content[after_start..];
    let end_rel_index = tail
        .find(&end_marker)
        .ok_or_else(|| format!("missing marker: {end_marker}"))?;

    Ok(tail[..end_rel_index].trim_matches('\n').to_string())
}

fn render_document(title: &str, source_label: &str, shared: &str, specific: &str) -> String {
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(title);
    out.push_str("\n\n");
    out.push_str("> Generated from `");
    out.push_str(source_label);
    out.push_str("` by `agent-config-sync`.\n");
    out.push_str("> Do not edit this file directly; edit the source and re-run the sync tool.\n\n");

    if !shared.is_empty() {
        out.push_str(shared);
        out.push('\n');
    }

    if !specific.is_empty() {
        out.push('\n');
        out.push_str(specific);
        if !specific.ends_with('\n') {
            out.push('\n');
        }
    }

    ensure_trailing_newline(out.trim_end())
}

fn load_overlay_file_if_exists(path: &Path) -> Result<OverlayConfig, String> {
    if !path.exists() {
        return Ok(OverlayConfig::default());
    }

    load_overlay_file(path)
}

fn load_overlay_file(path: &Path) -> Result<OverlayConfig, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read overlays file {}: {err}", path.display()))?;

    let parsed: OverlayConfig = serde_yaml::from_str(&raw)
        .map_err(|err| format!("failed to parse overlays file {}: {err}", path.display()))?;

    normalize_overlay_targets(parsed, path)
}

fn normalize_overlay_targets(config: OverlayConfig, path: &Path) -> Result<OverlayConfig, String> {
    let mut normalized = OverlayConfig::default();

    for (name, target) in config.targets {
        let canonical = canonical_target_name(&name).ok_or_else(|| {
            format!(
                "unknown overlay target '{}' in {} (expected: pi, claude, codex)",
                name,
                path.display()
            )
        })?;

        if normalized
            .targets
            .insert(canonical.to_string(), target)
            .is_some()
        {
            return Err(format!(
                "duplicate overlay target '{}' (canonical '{}') in {}",
                name,
                canonical,
                path.display()
            ));
        }
    }

    Ok(normalized)
}

fn canonical_target_name(name: &str) -> Option<&'static str> {
    match name {
        "pi" => Some("pi"),
        "codex" => Some("codex"),
        "claude" | "claude-code" => Some("claude"),
        _ => None,
    }
}

fn merge_overlay_configs(base: &OverlayConfig, profile: &OverlayConfig) -> OverlayConfig {
    let mut merged = base.clone();

    for (target_name, profile_target) in &profile.targets {
        if let Some(base_target) = merged.targets.get(target_name) {
            merged.targets.insert(
                target_name.clone(),
                merge_target_overlay(base_target, profile_target),
            );
        } else {
            merged
                .targets
                .insert(target_name.clone(), profile_target.clone());
        }
    }

    merged
}

fn merge_target_overlay(base: &TargetOverlay, profile: &TargetOverlay) -> TargetOverlay {
    let mut remove_sections = base.remove_sections.clone();
    remove_sections.extend(profile.remove_sections.clone());

    let mut section_replacements = base.section_replacements.clone();
    for (heading, body) in &profile.section_replacements {
        section_replacements.insert(heading.clone(), body.clone());
    }

    TargetOverlay {
        remove_sections,
        section_replacements,
        prepend: merge_optional_blocks(base.prepend.clone(), profile.prepend.clone()),
        append: merge_optional_blocks(base.append.clone(), profile.append.clone()),
    }
}

fn merge_optional_blocks(left: Option<String>, right: Option<String>) -> Option<String> {
    let left = left
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let right = right
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    match (left, right) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(left), Some(right)) => Some(format!("{left}\n\n{right}")),
    }
}

fn apply_target_overlay(
    document: &str,
    overlays: &OverlayConfig,
    target: &str,
) -> Result<String, String> {
    let Some(target_overlay) = overlays.targets.get(target) else {
        return Ok(ensure_trailing_newline(document.trim_end()));
    };

    apply_overlays(document, target_overlay)
        .map_err(|err| format!("failed to apply overlays for target '{}': {err}", target))
}

fn apply_overlays(document: &str, target: &TargetOverlay) -> Result<String, String> {
    let mut out = document.to_string();

    for heading in &target.remove_sections {
        out = remove_h1_section(&out, heading);
    }

    for (heading, body) in &target.section_replacements {
        out = replace_h2_section(&out, heading, body)?;
    }

    if let Some(prepend) = &target.prepend {
        out = format!("{}\n\n{}", prepend.trim_end(), out.trim_start());
    }

    if let Some(append) = &target.append {
        out = format!("{}\n\n{}", out.trim_end(), append.trim_start());
    }

    Ok(ensure_trailing_newline(out.trim_end()))
}

fn ensure_trailing_newline(text: &str) -> String {
    format!("{text}\n")
}

fn remove_h1_section(text: &str, heading: &str) -> String {
    let heading_line = format!("# {heading}");
    let Some((start, line_end)) = find_line(text, &heading_line) else {
        return text.to_string();
    };

    let end = find_next_heading_start(text, line_end, |line| line.starts_with("# "));

    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..start]);
    out.push_str(&text[end..]);
    out
}

fn replace_h2_section(text: &str, heading: &str, body: &str) -> Result<String, String> {
    let heading_line = format!("## {heading}");
    let Some((start, line_end)) = find_line(text, &heading_line) else {
        return Err(format!("section not found: {heading}"));
    };

    let end = find_next_heading_start(text, line_end, |line| {
        line.starts_with("# ") || line.starts_with("## ")
    });

    let replacement = format!("## {heading}\n\n{}\n\n", body.trim());

    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..start]);
    out.push_str(&replacement);
    out.push_str(&text[end..]);
    Ok(out)
}

fn find_line(text: &str, target: &str) -> Option<(usize, usize)> {
    let mut start = 0usize;

    while start <= text.len() {
        let end = match text[start..].find('\n') {
            Some(index) => start + index,
            None => text.len(),
        };

        let line = text[start..end].trim_end_matches('\r');
        if line == target {
            return Some((start, end));
        }

        if end == text.len() {
            break;
        }

        start = end + 1;
    }

    None
}

fn find_next_heading_start<F>(text: &str, from_line_end: usize, is_heading: F) -> usize
where
    F: Fn(&str) -> bool,
{
    let mut start = if from_line_end < text.len() {
        from_line_end + 1
    } else {
        text.len()
    };

    while start <= text.len() {
        let end = match text[start..].find('\n') {
            Some(index) => start + index,
            None => text.len(),
        };

        let line = text[start..end].trim_end_matches('\r');
        if is_heading(line) {
            return start;
        }

        if end == text.len() {
            break;
        }

        start = end + 1;
    }

    text.len()
}

fn sync_profile_assets(config: &Config) -> Result<(), String> {
    let claude_root = config.claude_out.parent().ok_or_else(|| {
        format!(
            "invalid claude output path: {}",
            config.claude_out.display()
        )
    })?;
    let codex_root = config
        .codex_out
        .parent()
        .ok_or_else(|| format!("invalid codex output path: {}", config.codex_out.display()))?;
    let pi_root = config
        .pi_out
        .parent()
        .ok_or_else(|| format!("invalid pi output path: {}", config.pi_out.display()))?;

    for (target, root) in [
        ("pi", pi_root),
        ("claude", claude_root),
        ("codex", codex_root),
    ] {
        let skill_sources = vec![
            config.agents_dir.join("shared/skills/common"),
            config.agents_dir.join("shared/skills").join(target),
            config
                .agents_dir
                .join(&config.profile)
                .join("skills/common"),
            config
                .agents_dir
                .join(&config.profile)
                .join("skills")
                .join(target),
        ];

        let synced_skills = sync_layered_entries(&skill_sources, &root.join("skills"))?;

        if synced_skills > 0 {
            println!(
                "Synced {synced_skills} skill entr{} for {target}",
                if synced_skills == 1 { "y" } else { "ies" }
            );
        }

        let hook_sources = vec![
            config.agents_dir.join("shared/hooks/common"),
            config.agents_dir.join("shared/hooks").join(target),
            config.agents_dir.join(&config.profile).join("hooks/common"),
            config
                .agents_dir
                .join(&config.profile)
                .join("hooks")
                .join(target),
        ];

        let synced_hooks = sync_layered_entries(&hook_sources, &root.join("hooks"))?;

        if synced_hooks > 0 {
            println!(
                "Synced {synced_hooks} hook entr{} for {target}",
                if synced_hooks == 1 { "y" } else { "ies" }
            );
        }
    }

    Ok(())
}

fn sync_layered_entries(sources: &[PathBuf], destination: &Path) -> Result<usize, String> {
    let mut desired: BTreeMap<String, PathBuf> = BTreeMap::new();

    for source in sources {
        collect_entries(source, &mut desired)?;
    }

    if !destination.exists() && desired.is_empty() {
        return Ok(0);
    }

    fs::create_dir_all(destination)
        .map_err(|err| format!("failed to create {}: {err}", destination.display()))?;

    cleanup_managed_symlinks(destination, &desired, sources)?;

    for (name, source_path) in &desired {
        let link_path = destination.join(name);
        ensure_symlink(&link_path, source_path)
            .map_err(|err| format!("failed to sync {}: {err}", link_path.display()))?;
    }

    Ok(desired.len())
}

fn collect_entries(source: &Path, entries: &mut BTreeMap<String, PathBuf>) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }

    let metadata = fs::metadata(source)
        .map_err(|err| format!("failed to read {}: {err}", source.display()))?;

    if !metadata.is_dir() {
        return Err(format!(
            "expected directory but found file: {}",
            source.display()
        ));
    }

    let dir_entries = fs::read_dir(source)
        .map_err(|err| format!("failed to read {}: {err}", source.display()))?;

    for entry in dir_entries {
        let entry = entry.map_err(|err| format!("failed to read dir entry: {err}"))?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".gitkeep" || name.is_empty() {
            continue;
        }

        entries.insert(name, entry.path());
    }

    Ok(())
}

fn cleanup_managed_symlinks(
    destination: &Path,
    desired: &BTreeMap<String, PathBuf>,
    managed_roots: &[PathBuf],
) -> Result<(), String> {
    let dir_entries = fs::read_dir(destination)
        .map_err(|err| format!("failed to read {}: {err}", destination.display()))?;

    for entry in dir_entries {
        let entry = entry.map_err(|err| format!("failed to read dir entry: {err}"))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if desired.contains_key(&name) {
            continue;
        }

        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };

        if !metadata.file_type().is_symlink() {
            continue;
        }

        let Ok(link_target_raw) = fs::read_link(&path) else {
            continue;
        };

        let link_target = if link_target_raw.is_absolute() {
            link_target_raw
        } else if let Some(parent) = path.parent() {
            parent.join(link_target_raw)
        } else {
            link_target_raw
        };

        if managed_roots
            .iter()
            .any(|root| link_target.starts_with(root))
        {
            remove_path(&path)
                .map_err(|err| format!("failed to remove stale link {}: {err}", path.display()))?;
        }
    }

    Ok(())
}

fn ensure_symlink(link_path: &Path, source_path: &Path) -> io::Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(link_path) {
        if metadata.file_type().is_symlink() {
            if let Ok(existing_target) = fs::read_link(link_path) {
                if existing_target == source_path {
                    return Ok(());
                }
            }
        }

        remove_path(link_path)?;
    }

    create_symlink(source_path, link_path)
}

fn create_symlink(source_path: &Path, link_path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source_path, link_path)
    }

    #[cfg(not(unix))]
    {
        let _ = source_path;
        let _ = link_path;
        Err(io::Error::new(
            io::ErrorKind::Other,
            "symlink-based sync is only supported on unix",
        ))
    }
}

fn remove_path(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || metadata.is_file() {
                fs::remove_file(path)
            } else {
                fs::remove_dir_all(path)
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn write_file(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)?;
    fs::rename(temp_path, path)?;
    Ok(())
}
