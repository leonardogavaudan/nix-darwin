use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use clap::Parser;
use html_escape::{decode_html_entities, encode_safe};
use regex::{Captures, Regex};
use reqwest::blocking::Client;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CONFLUENCE_BASE_URL: &str = "https://algolia.atlassian.net/wiki";
const SPACE_KEY: &str = "~712020024fac75264b406693a5228fc3623b5f";
const SPACE_ID: &str = "5909381247";

const GREEN: &str = "\x1b[92m";
const RED: &str = "\x1b[91m";
const YELLOW: &str = "\x1b[93m";
const RESET: &str = "\x1b[0m";

const FILES_TO_SYNC: &[&str] = &[
    "Algolia Index Settings.md",
    "Algolia Product Architecture.md",
    "Classic vs Metis Architecture.md",
    "Algolia Infrastructure and Observability Overview.md",
    "suggested-actions/One-Shot Script.md",
    "suggested-actions/One-Shot Script Tickets.md",
    "suggested-actions/DRR Eligibility Filtering.md",
    "suggested-actions/NeuralSearch Eligibility Filtering.md",
    "suggested-actions/Suggested Actions Status.md",
    "suggested-actions/Lotfi Sprint Handover.md",
    "suggested-actions/Datamixer.md",
    "suggested-actions/Metis App Detection.md",
    "data/Data Warehousing and Algolia Data Repo.md",
    "data/Algolia Data Sources and Available Data.md",
    "data/dim_application.md",
    "data/salesforce-application-mapping.md",
    "data/search-api-raw-logs-objects-vs-writes.md",
    "incidents/2026-01-28 Analytics API 503 Retry Storm.md",
    "offline-evaluations/optim-2247-admin-dashboard-settings.md",
    "go/semantic/phantom-null-rows.md",
];

#[derive(Debug, Parser)]
#[command(name = "sync-to-confluence-rs")]
#[command(about = "Sync markdown files between master vault and Confluence")]
struct Cli {
    #[arg(long, help = "Pull changes from Confluence to local")]
    pull: bool,
    #[arg(help = "Specific files to sync/pull (default: all)")]
    files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Mapping {
    #[serde(default)]
    pages: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    space_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    confluence_base: Option<String>,
    #[serde(flatten)]
    other: BTreeMap<String, serde_yaml::Value>,
}

struct App {
    client: Client,
    auth_header: String,
    master_dir: PathBuf,
    mapping_file: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{RED}Error: {err}{RESET}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let app = App::new()?;

    let files = if cli.files.is_empty() {
        FILES_TO_SYNC
            .iter()
            .map(|file| (*file).to_string())
            .collect::<Vec<_>>()
    } else {
        cli.files.clone()
    };

    println!("{}", "=".repeat(50));
    println!(
        "{} Confluence",
        if cli.pull {
            "Pulling from"
        } else {
            "Pushing to"
        }
    );
    println!("{}", "=".repeat(50));
    println!("Space: {SPACE_KEY}");
    println!("Local: {}", app.master_dir.display());
    println!();

    let mut mapping = app.load_mapping()?;

    if cli.pull {
        for file in files {
            let filename = canonicalize_to_sync_entry(&normalize_md_filename(&file));
            if is_in_sync_list(&filename) {
                if let Err(err) = app.pull_file(&filename, &mapping) {
                    eprintln!("  {RED}✗ Pull failed for {filename}: {err}{RESET}");
                }
            } else {
                println!("{YELLOW}Skipping (not in sync list): {filename}{RESET}");
            }
        }
    } else {
        for file in files {
            let filename = canonicalize_to_sync_entry(&normalize_md_filename(&file));
            let path = app.master_dir.join(&filename);
            if path.exists() {
                if let Err(err) = app.sync_file(&path, &filename, &mut mapping) {
                    eprintln!("  {RED}✗ Sync failed for {filename}: {err}{RESET}");
                }
            } else {
                println!("{YELLOW}Skipping (not found): {}{RESET}", path.display());
            }
        }
    }

    println!();
    println!("{GREEN}Done!{RESET}");

    Ok(())
}

impl App {
    fn new() -> Result<Self> {
        let auth_header = get_auth_header()?;
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .context("failed to build HTTP client")?;

        let master_dir = env::var("MASTER_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join("master"));
        let mapping_file = env::var("SYNC_TO_CONFLUENCE_MAPPING_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                home_dir().join(".config/nix-darwin/scripts/sync-to-confluence-rs/link_mapping.yaml")
            });

        Ok(Self {
            client,
            auth_header,
            master_dir,
            mapping_file,
        })
    }

    fn load_mapping(&self) -> Result<Mapping> {
        if self.mapping_file.exists() {
            let content = fs::read_to_string(&self.mapping_file)
                .with_context(|| format!("failed reading {}", self.mapping_file.display()))?;
            let mut mapping: Mapping =
                serde_yaml::from_str(&content).context("failed parsing link_mapping.yaml")?;
            if mapping.pages.is_empty() {
                mapping.pages = BTreeMap::new();
            }
            Ok(mapping)
        } else {
            Ok(Mapping::default())
        }
    }

    fn save_mapping(&self, mapping: &Mapping) -> Result<()> {
        let content = serde_yaml::to_string(mapping).context("failed serializing mapping YAML")?;
        fs::write(&self.mapping_file, content)
            .with_context(|| format!("failed writing {}", self.mapping_file.display()))
    }

    fn api_request_v1(&self, method: Method, endpoint: &str, data: Option<Value>) -> Result<Value> {
        self.api_request(method, endpoint, data, false)
    }

    fn api_request_v2(&self, method: Method, endpoint: &str, data: Option<Value>) -> Result<Value> {
        self.api_request(method, endpoint, data, true)
    }

    fn api_request(
        &self,
        method: Method,
        endpoint: &str,
        data: Option<Value>,
        v2: bool,
    ) -> Result<Value> {
        let base_path = if v2 { "/api/v2" } else { "/rest/api" };
        let url = format!("{CONFLUENCE_BASE_URL}{base_path}{endpoint}");

        let mut req = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Basic {}", self.auth_header))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(payload) = data {
            req = req.json(&payload);
        }

        let resp = req
            .send()
            .with_context(|| format!("request failed: {url}"))?;

        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        if body.trim().is_empty() {
            return Ok(json!({}));
        }

        let parsed_json = serde_json::from_str::<Value>(&body);

        if status.is_success() {
            Ok(parsed_json.unwrap_or_else(|_| json!({})))
        } else {
            let error = parsed_json.unwrap_or_else(|_| json!(body));
            Ok(json!({ "error": error }))
        }
    }

    fn find_page_by_title(&self, title: &str) -> Result<Option<Value>> {
        let encoded_title = urlencoding::encode(title);
        let endpoint =
            format!("/content?spaceKey={SPACE_KEY}&title={encoded_title}&expand=version");
        let result = self.api_request_v1(Method::GET, &endpoint, None)?;
        if let Some(results) = result.get("results").and_then(Value::as_array) {
            if let Some(first) = results.first() {
                return Ok(Some(first.clone()));
            }
        }
        Ok(None)
    }

    fn get_page_content(&self, page_id: &str) -> Result<Option<String>> {
        let endpoint = format!("/content/{page_id}?expand=body.storage,version");
        let result = self.api_request_v1(Method::GET, &endpoint, None)?;
        if has_error(&result) {
            return Ok(None);
        }
        Ok(result
            .pointer("/body/storage/value")
            .and_then(Value::as_str)
            .map(ToString::to_string))
    }

    fn create_page(&self, title: &str, content: &str, parent_id: Option<&str>) -> Result<Value> {
        let mut payload = json!({
            "type": "page",
            "title": title,
            "space": { "key": SPACE_KEY },
            "body": {
                "storage": {
                    "value": content,
                    "representation": "storage"
                }
            }
        });

        if let Some(parent_id) = parent_id {
            payload
                .as_object_mut()
                .expect("payload should be object")
                .insert("ancestors".to_string(), json!([{ "id": parent_id }]));
        }

        self.api_request_v1(Method::POST, "/content", Some(payload))
    }

    fn update_page(
        &self,
        page_id: &str,
        title: &str,
        content: &str,
        version: i64,
        parent_id: Option<&str>,
    ) -> Result<Value> {
        let mut payload = json!({
            "version": { "number": version + 1 },
            "title": title,
            "type": "page",
            "body": {
                "storage": {
                    "value": content,
                    "representation": "storage"
                }
            }
        });

        if let Some(parent_id) = parent_id {
            payload
                .as_object_mut()
                .expect("payload should be object")
                .insert("ancestors".to_string(), json!([{ "id": parent_id }]));
        }

        self.api_request_v1(Method::PUT, &format!("/content/{page_id}"), Some(payload))
    }

    fn get_or_create_folder(
        &self,
        folder_name: &str,
        mapping: &mut Mapping,
    ) -> Result<Option<String>> {
        let folder_key = format!("_folder_{folder_name}");
        if let Some(existing) = mapping.pages.get(&folder_key) {
            return Ok(Some(existing.clone()));
        }

        println!("{YELLOW}Creating folder: {folder_name}{RESET}");
        let result = self.api_request_v2(
            Method::POST,
            "/folders",
            Some(json!({
                "spaceId": SPACE_ID,
                "title": folder_name,
            })),
        )?;

        if has_error(&result) {
            println!(
                "  {RED}✗ Failed to create folder: {}{RESET}",
                value_to_string(result.get("error"))
            );
            return Ok(None);
        }

        let folder_id = result
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string);

        if let Some(folder_id) = folder_id {
            println!(
                "  {GREEN}✓ Created folder: {CONFLUENCE_BASE_URL}/spaces/{SPACE_KEY}/folder/{folder_id}{RESET}"
            );
            mapping.pages.insert(folder_key, folder_id.clone());
            self.save_mapping(mapping)?;
            return Ok(Some(folder_id));
        }

        Ok(None)
    }

    fn pull_file(&self, filename: &str, mapping: &Mapping) -> Result<()> {
        let title = filename.trim_end_matches(".md");
        let file_path = self.master_dir.join(filename);

        println!("{YELLOW}Pulling: {title}{RESET}");

        let Some((_, page_id)) = find_mapping_entry(mapping, filename) else {
            println!("  {RED}✗ Not in mapping - sync first{RESET}");
            return Ok(());
        };

        let Some(storage_content) = self.get_page_content(&page_id)? else {
            println!("  {RED}✗ Failed to fetch content{RESET}");
            return Ok(());
        };

        let markdown = storage_to_markdown(&storage_content);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
        fs::write(&file_path, markdown)
            .with_context(|| format!("failed writing {}", file_path.display()))?;

        println!("  {GREEN}✓ Pulled to: {}{RESET}", file_path.display());
        Ok(())
    }

    fn sync_file(
        &self,
        file_path: &Path,
        relative_path: &str,
        mapping: &mut Mapping,
    ) -> Result<()> {
        let mapping_key = find_mapping_entry(mapping, relative_path)
            .map(|(key, _)| key)
            .unwrap_or_else(|| slugify_path(relative_path));

        let mut parent_id = None;
        if let Some((folder_name, _)) = relative_path.split_once('/') {
            parent_id = self.get_or_create_folder(folder_name, mapping)?;
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("failed reading {}", file_path.display()))?;

        let (extracted_title, content_without_h1) = extract_title_from_markdown(&content);
        let title = extracted_title.unwrap_or_else(|| {
            file_path
                .file_stem()
                .and_then(|f| f.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

        println!("{YELLOW}Syncing: {title}{RESET}");

        let transformed = transform_links(&content_without_h1, mapping);
        let storage_content = markdown_to_storage(&transformed);

        let mut existing = None;

        if let Some(mapped_page_id) = mapping.pages.get(&mapping_key) {
            let check = self.api_request_v1(
                Method::GET,
                &format!("/content/{mapped_page_id}?expand=version"),
                None,
            )?;
            if !has_error(&check) {
                existing = Some(check);
            }
        }

        if existing.is_none() {
            existing = self.find_page_by_title(&title)?;
        }

        if let Some(existing_page) = existing {
            let page_id = existing_page
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let version = existing_page
                .pointer("/version/number")
                .and_then(Value::as_i64)
                .unwrap_or(0);

            let result = self.update_page(
                &page_id,
                &title,
                &storage_content,
                version,
                parent_id.as_deref(),
            )?;

            if has_error(&result) {
                println!(
                    "  {RED}✗ Failed to update: {}{RESET}",
                    value_to_string(result.get("error"))
                );
            } else {
                println!(
                    "  {GREEN}✓ Updated: {CONFLUENCE_BASE_URL}/spaces/{SPACE_KEY}/pages/{page_id}{RESET}"
                );
                mapping.pages.insert(mapping_key, page_id);
                self.save_mapping(mapping)?;
            }
        } else {
            let result = self.create_page(&title, &storage_content, parent_id.as_deref())?;

            if has_error(&result) {
                println!(
                    "  {RED}✗ Failed to create: {}{RESET}",
                    value_to_string(result.get("error"))
                );
            } else {
                let page_id = result
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();

                println!(
                    "  {GREEN}✓ Created: {CONFLUENCE_BASE_URL}/spaces/{SPACE_KEY}/pages/{page_id}{RESET}"
                );

                if !page_id.is_empty() {
                    mapping.pages.insert(mapping_key, page_id);
                    self.save_mapping(mapping)?;
                }
            }
        }

        Ok(())
    }
}

fn get_auth_header() -> Result<String> {
    let email = env::var("CONFLUENCE_EMAIL").ok();
    let token = env::var("ATLASSIAN_API_TOKEN").ok();

    let (Some(email), Some(token)) = (email, token) else {
        bail!(
            "CONFLUENCE_EMAIL and ATLASSIAN_API_TOKEN must be set\n\nExport them like this:\n  export CONFLUENCE_EMAIL='your.email@algolia.com'\n  export ATLASSIAN_API_TOKEN='your-api-token'\n\nGet your API token at: https://id.atlassian.com/manage-profile/security/api-tokens"
        );
    };

    Ok(STANDARD.encode(format!("{email}:{token}")))
}

fn home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn normalize_md_filename(name: &str) -> String {
    if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    }
}

fn canonicalize_to_sync_entry(filename: &str) -> String {
    let target_slug = slugify_path(filename);
    FILES_TO_SYNC
        .iter()
        .find(|entry| slugify_path(entry) == target_slug)
        .map(|entry| (*entry).to_string())
        .unwrap_or_else(|| filename.to_string())
}

fn is_in_sync_list(filename: &str) -> bool {
    let target_slug = slugify_path(filename);
    FILES_TO_SYNC
        .iter()
        .any(|entry| slugify_path(entry) == target_slug)
}

fn find_mapping_entry(mapping: &Mapping, path: &str) -> Option<(String, String)> {
    if let Some(id) = mapping.pages.get(path) {
        return Some((path.to_string(), id.clone()));
    }

    let slug = slugify_path(path);
    if slug != path {
        if let Some(id) = mapping.pages.get(&slug) {
            return Some((slug, id.clone()));
        }
    }

    None
}

fn slugify_path(path: &str) -> String {
    path.split('/')
        .map(slugify_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn slugify_segment(segment: &str) -> String {
    let mut output = String::with_capacity(segment.len());
    let mut last_was_dash = false;

    for ch in segment.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '.' {
            output.push(lower);
            last_was_dash = false;
        } else if !last_was_dash {
            output.push('-');
            last_was_dash = true;
        }
    }

    output.trim_matches('-').to_string()
}

fn has_error(value: &Value) -> bool {
    value.get("error").is_some()
}

fn value_to_string(value: Option<&Value>) -> String {
    value
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                v.to_string()
            }
        })
        .unwrap_or_else(|| "unknown error".to_string())
}

fn transform_links(content: &str, mapping: &Mapping) -> String {
    let pattern = Regex::new(r#"\[([^\]]+)\]\((\./[^)]+\.md)\)"#).expect("valid regex");

    pattern
        .replace_all(content, |caps: &Captures| {
            let text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let raw_path = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let filename = raw_path
                .replace("%20", " ")
                .trim_start_matches("./")
                .to_string();

            if let Some((_, page_id)) = find_mapping_entry(mapping, &filename) {
                let space_key = mapping.space_key.as_deref().unwrap_or(SPACE_KEY);
                let confluence_base = mapping
                    .confluence_base
                    .as_deref()
                    .unwrap_or(CONFLUENCE_BASE_URL);
                format!("[{text}]({confluence_base}/spaces/{space_key}/pages/{page_id})")
            } else {
                caps.get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string()
            }
        })
        .into_owned()
}

fn extract_title_from_markdown(md_content: &str) -> (Option<String>, String) {
    let lines = md_content.lines().collect::<Vec<_>>();
    let mut in_code_block = false;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            let title = trimmed.trim_start_matches("# ").trim().to_string();
            let mut remaining = lines[..idx]
                .iter()
                .map(|s| (*s).to_string())
                .chain(lines[idx + 1..].iter().map(|s| (*s).to_string()))
                .collect::<Vec<_>>();

            while remaining.first().is_some_and(|line| line.trim().is_empty()) {
                remaining.remove(0);
            }

            return (Some(title), remaining.join("\n"));
        }
    }

    (None, md_content.to_string())
}

fn process_inline(text: &str) -> String {
    let bold_re = Regex::new(r"\*\*([^*]+)\*\*").expect("valid regex");
    let code_re = Regex::new(r"`([^`]+)`").expect("valid regex");
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("valid regex");

    let escaped = encode_safe(text).to_string();
    let bolded = bold_re.replace_all(&escaped, "<strong>$1</strong>");
    let coded = code_re.replace_all(&bolded, "<code>$1</code>");
    link_re
        .replace_all(&coded, "<a href=\"$2\">$1</a>")
        .into_owned()
}

fn flush_list(result: &mut Vec<String>, in_list: &mut bool, list_items: &mut Vec<String>) {
    if *in_list && !list_items.is_empty() {
        result.push("<ul>".to_string());
        for item in list_items.drain(..) {
            result.push(format!("<li>{}</li>", process_inline(&item)));
        }
        result.push("</ul>".to_string());
        *in_list = false;
    }
}

fn flush_table(result: &mut Vec<String>, in_table: &mut bool, table_rows: &mut Vec<String>) {
    if !*in_table || table_rows.is_empty() {
        return;
    }

    result.push("<table><tbody>".to_string());
    let mut rendered_row_index = 0usize;
    for row in table_rows.drain(..) {
        let cells = row
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();

        if cells.is_empty() {
            continue;
        }

        let is_separator_row = cells.iter().all(|cell| {
            !cell.is_empty()
                && cell
                    .chars()
                    .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
        });
        if is_separator_row {
            continue;
        }

        result.push("<tr>".to_string());
        let tag = if rendered_row_index == 0 { "th" } else { "td" };
        for cell in cells {
            result.push(format!("<{tag}>{}</{tag}>", process_inline(cell)));
        }
        result.push("</tr>".to_string());
        rendered_row_index += 1;
    }
    result.push("</tbody></table>".to_string());
    *in_table = false;
}

fn markdown_to_storage(md_content: &str) -> String {
    let mut result = Vec::<String>::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = Vec::<String>::new();
    let mut in_table = false;
    let mut table_rows = Vec::<String>::new();
    let mut in_list = false;
    let mut list_items = Vec::<String>::new();

    for line in md_content.lines() {
        if let Some(lang) = line.strip_prefix("```") {
            flush_list(&mut result, &mut in_list, &mut list_items);
            flush_table(&mut result, &mut in_table, &mut table_rows);

            if in_code_block {
                let lang_attr = if code_lang.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<ac:parameter ac:name=\"language\">{}</ac:parameter>",
                        code_lang
                    )
                };
                let code = code_content.join("\n");
                result.push(format!(
                    "<ac:structured-macro ac:name=\"code\">{lang_attr}<ac:plain-text-body><![CDATA[{code}]]></ac:plain-text-body></ac:structured-macro>"
                ));
                in_code_block = false;
                code_content.clear();
                code_lang.clear();
            } else {
                in_code_block = true;
                code_lang = lang.trim().to_string();
            }
            continue;
        }

        if in_code_block {
            code_content.push(line.to_string());
            continue;
        }

        if line.starts_with('|') {
            flush_list(&mut result, &mut in_list, &mut list_items);
            in_table = true;
            table_rows.push(line.to_string());
            continue;
        } else if in_table {
            flush_table(&mut result, &mut in_table, &mut table_rows);
        }

        let trimmed_start = line.trim_start();
        if trimmed_start.starts_with("- ") || trimmed_start.starts_with("* ") {
            flush_table(&mut result, &mut in_table, &mut table_rows);
            in_list = true;
            list_items.push(trimmed_start[2..].to_string());
            continue;
        } else if in_list {
            flush_list(&mut result, &mut in_list, &mut list_items);
        }

        if let Some(rest) = line.strip_prefix("# ") {
            result.push(format!("<h1>{}</h1>", process_inline(rest)));
        } else if let Some(rest) = line.strip_prefix("## ") {
            result.push(format!("<h2>{}</h2>", process_inline(rest)));
        } else if let Some(rest) = line.strip_prefix("### ") {
            result.push(format!("<h3>{}</h3>", process_inline(rest)));
        } else if let Some(rest) = line.strip_prefix("#### ") {
            result.push(format!("<h4>{}</h4>", process_inline(rest)));
        } else if let Some(rest) = line.strip_prefix("##### ") {
            result.push(format!("<h5>{}</h5>", process_inline(rest)));
        } else if line.trim() == "---" {
            result.push("<hr />".to_string());
        } else if let Some(rest) = line.strip_prefix("> ") {
            result.push(format!(
                "<blockquote><p>{}</p></blockquote>",
                process_inline(rest)
            ));
        } else if line.trim().is_empty() {
            result.push(String::new());
        } else {
            result.push(format!("<p>{}</p>", process_inline(line)));
        }
    }

    if in_code_block {
        let lang_attr = if code_lang.is_empty() {
            String::new()
        } else {
            format!(
                "<ac:parameter ac:name=\"language\">{}</ac:parameter>",
                code_lang
            )
        };
        let code = code_content.join("\n");
        result.push(format!(
            "<ac:structured-macro ac:name=\"code\">{lang_attr}<ac:plain-text-body><![CDATA[{code}]]></ac:plain-text-body></ac:structured-macro>"
        ));
    }

    flush_list(&mut result, &mut in_list, &mut list_items);
    flush_table(&mut result, &mut in_table, &mut table_rows);

    result.join("\n")
}

fn storage_to_markdown(html_content: &str) -> String {
    let mut content = html_content.to_string();

    let code_macro_re =
        Regex::new(r#"(?s)<ac:structured-macro ac:name=\"code\"[^>]*>.*?</ac:structured-macro>"#)
            .expect("valid regex");
    let lang_re = Regex::new(r#"ac:name=\"language\">([^<]+)<"#).expect("valid regex");
    let code_re = Regex::new(r#"(?s)<!\[CDATA\[(.*?)\]\]>"#).expect("valid regex");
    content = code_macro_re
        .replace_all(&content, |caps: &Captures| {
            let block = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            let lang = lang_re
                .captures(block)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                .unwrap_or_default();
            let code = code_re
                .captures(block)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                .unwrap_or_default();
            format!("\n```{lang}\n{code}\n```\n")
        })
        .into_owned();

    let info_re =
        Regex::new(r#"(?s)<ac:structured-macro ac:name=\"info\"[^>]*>.*?</ac:structured-macro>"#)
            .expect("valid regex");
    let info_text_re = Regex::new(r#"<p>([^<]+)</p>"#).expect("valid regex");
    content = info_re
        .replace_all(&content, |caps: &Captures| {
            let block = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            let text = info_text_re
                .captures(block)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                .unwrap_or_default();
            format!("> {text}\n")
        })
        .into_owned();

    let blockquote_re =
        Regex::new(r#"(?s)<blockquote>(.*?)</blockquote>"#).expect("valid regex");
    let bq_text_re = Regex::new(r#"<p>([^<]+)</p>"#).expect("valid regex");
    content = blockquote_re
        .replace_all(&content, |caps: &Captures| {
            let block = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            let text = bq_text_re
                .captures(block)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                .unwrap_or_default();
            format!("> {text}\n")
        })
        .into_owned();

    let table_re = Regex::new(r#"(?s)<table[^>]*>.*?</table>"#).expect("valid regex");
    let row_re = Regex::new(r#"(?s)<tr[^>]*>(.*?)</tr>"#).expect("valid regex");
    let cell_re = Regex::new(r#"(?s)<t[hd][^>]*>(.*?)</t[hd]>"#).expect("valid regex");
    let strip_tags_re = Regex::new(r#"<[^>]+>"#).expect("valid regex");

    content = table_re
        .replace_all(&content, |caps: &Captures| {
            let table_html = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            let mut md_rows = Vec::new();

            for (index, row_caps) in row_re.captures_iter(table_html).enumerate() {
                let row_html = row_caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let mut cleaned_cells = Vec::new();

                for cell_caps in cell_re.captures_iter(row_html) {
                    let cell = cell_caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                    let no_tags = strip_tags_re.replace_all(cell, "");
                    let decoded = decode_html_entities(&no_tags).to_string();
                    let cleaned = decoded.trim().replace('\n', " ");
                    cleaned_cells.push(cleaned);
                }

                if !cleaned_cells.is_empty() {
                    md_rows.push(format!("| {} |", cleaned_cells.join(" | ")));
                    if index == 0 {
                        md_rows.push(format!("|{}|", vec!["---"; cleaned_cells.len()].join("|")));
                    }
                }
            }

            format!("\n{}\n", md_rows.join("\n"))
        })
        .into_owned();

    content = Regex::new(r"<h1>([^<]+)</h1>")
        .expect("valid regex")
        .replace_all(&content, "# $1")
        .into_owned();
    content = Regex::new(r"<h2>([^<]+)</h2>")
        .expect("valid regex")
        .replace_all(&content, "## $1")
        .into_owned();
    content = Regex::new(r"<h3>([^<]+)</h3>")
        .expect("valid regex")
        .replace_all(&content, "### $1")
        .into_owned();
    content = Regex::new(r"<h4>([^<]+)</h4>")
        .expect("valid regex")
        .replace_all(&content, "#### $1")
        .into_owned();
    content = Regex::new(r"<h5>([^<]+)</h5>")
        .expect("valid regex")
        .replace_all(&content, "##### $1")
        .into_owned();

    let ul_re = Regex::new(r#"(?s)<ul[^>]*>.*?</ul>"#).expect("valid regex");
    let li_re = Regex::new(r#"(?s)<li[^>]*>(.*?)</li>"#).expect("valid regex");
    let link_in_li_re =
        Regex::new(r#"<a href=\"([^\"]+)\"[^>]*>([^<]+)</a>"#).expect("valid regex");
    content = ul_re
        .replace_all(&content, |caps: &Captures| {
            let list_html = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            let mut md_items = Vec::new();
            for item_caps in li_re.captures_iter(list_html) {
                let item_html = item_caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let linked = link_in_li_re.replace_all(item_html, "[$2]($1)");
                let stripped = strip_tags_re.replace_all(&linked, "");
                let decoded = decode_html_entities(&stripped).to_string();
                let cleaned = decoded.trim().replace('\n', " ");
                if !cleaned.is_empty() {
                    md_items.push(format!("- {cleaned}"));
                }
            }
            format!("\n{}\n", md_items.join("\n"))
        })
        .into_owned();

    content = Regex::new(r"<hr\s*/?>")
        .expect("valid regex")
        .replace_all(&content, "\n---\n")
        .into_owned();

    content = Regex::new(r#"<a href=\"([^\"]+)\">([^<]+)</a>"#)
        .expect("valid regex")
        .replace_all(&content, "[$2]($1)")
        .into_owned();

    content = Regex::new(r"<strong>([^<]+)</strong>")
        .expect("valid regex")
        .replace_all(&content, "**$1**")
        .into_owned();

    content = Regex::new(r"<code>([^<]+)</code>")
        .expect("valid regex")
        .replace_all(&content, "`$1`")
        .into_owned();

    content = Regex::new(r"(?s)<p>([^<]*(?:<[^>]+>[^<]*)*)</p>")
        .expect("valid regex")
        .replace_all(&content, "$1\n")
        .into_owned();

    content = strip_tags_re.replace_all(&content, "").into_owned();
    content = decode_html_entities(&content).to_string();
    content = Regex::new(r"\n{3,}")
        .expect("valid regex")
        .replace_all(&content, "\n\n")
        .into_owned();

    content.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_first_h1_title() {
        let input = "# My Title\n\nBody line";
        let (title, content) = extract_title_from_markdown(input);

        assert_eq!(title.as_deref(), Some("My Title"));
        assert_eq!(content, "Body line");
    }

    #[test]
    fn transforms_local_links_using_mapping() {
        let mut mapping = Mapping::default();
        mapping
            .pages
            .insert("data/dim_application.md".to_string(), "12345".to_string());

        let input = "Read [this](./data/dim_application.md) now";
        let output = transform_links(input, &mapping);

        assert!(
            output.contains(
                "[this](https://algolia.atlassian.net/wiki/spaces/~712020024fac75264b406693a5228fc3623b5f/pages/12345)"
            ),
            "output was: {output}"
        );
    }

    #[test]
    fn transforms_links_with_slug_mapping_fallback() {
        let mut mapping = Mapping::default();
        mapping
            .pages
            .insert("data/dim-application.md".to_string(), "777".to_string());

        let input = "Read [this](./data/dim_application.md) now";
        let output = transform_links(input, &mapping);

        assert!(
            output.contains(
                "[this](https://algolia.atlassian.net/wiki/spaces/~712020024fac75264b406693a5228fc3623b5f/pages/777)"
            ),
            "output was: {output}"
        );
    }

    #[test]
    fn slugify_path_matches_existing_mapping_style() {
        assert_eq!(
            slugify_path("suggested-actions/Datamixer.md"),
            "suggested-actions/datamixer.md"
        );
        assert_eq!(
            slugify_path("data/dim_application.md"),
            "data/dim-application.md"
        );
        assert_eq!(
            slugify_path("incidents/2026-01-28 Analytics API 503 Retry Storm.md"),
            "incidents/2026-01-28-analytics-api-503-retry-storm.md"
        );
    }

    #[test]
    fn markdown_to_storage_converts_core_blocks() {
        let input = "# Title\n\n- one\n- two\n\n```rust\nfn main() {}\n```";
        let output = markdown_to_storage(input);

        assert!(output.contains("<h1>Title</h1>"));
        assert!(output.contains("<ul>"));
        assert!(output.contains("ac:structured-macro ac:name=\"code\""));
    }

    #[test]
    fn markdown_to_storage_converts_tables_without_separator_row_or_trailing_empty_column() {
        let input = "| query | A tracked searches | A revenue |\n|---|---:|---:|\n| zellige | 1387 | $4,708.17 |";
        let output = markdown_to_storage(input);

        assert!(output.contains("<table><tbody>"), "output was: {output}");
        assert!(output.contains("<th>query</th>"), "output was: {output}");
        assert!(output.contains("<th>A tracked searches</th>"), "output was: {output}");
        assert!(output.contains("<td>zellige</td>"), "output was: {output}");
        assert!(!output.contains("<td>---</td>"), "output was: {output}");
        assert!(!output.contains("<th></th>"), "output was: {output}");
        assert!(!output.contains("<td></td>"), "output was: {output}");
    }

    #[test]
    fn storage_to_markdown_converts_back() {
        let input = r#"<h1>Title</h1><p>Hello <strong>world</strong></p><ul><li>First</li></ul>"#;
        let output = storage_to_markdown(input);

        assert!(output.contains("# Title"));
        assert!(output.contains("**world**"));
        assert!(output.contains("- First"));
    }
}
