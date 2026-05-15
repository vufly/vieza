use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_EZA_REF: &str = "eed27ed05e74542af5852aed40e3dbff87d69c43";
const DEFAULT_VIVID_REF: &str = "165bbbbe9613e4a8b2dad781c8ff1e34fd052d0d";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
enum Subcmd {
    Generate(GenerateArgs),
}

#[derive(Debug)]
struct GenerateArgs {
    out_dir: PathBuf,
    eza_ref: String,
    vivid_ref: String,
    source: Option<PathBuf>,
    vivid_source: Option<PathBuf>,
}

impl Default for GenerateArgs {
    fn default() -> Self {
        Self {
            out_dir: PathBuf::from("generated"),
            eza_ref: DEFAULT_EZA_REF.to_string(),
            vivid_ref: DEFAULT_VIVID_REF.to_string(),
            source: None,
            vivid_source: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FileCategory {
    Image,
    Video,
    Music,
    Lossless,
    Crypto,
    Document,
    Compressed,
    Temp,
    Compiled,
    Build,
    Source,
}

impl FileCategory {
    fn from_rust_name(name: &str) -> Option<Self> {
        Some(match name {
            "Image" => Self::Image,
            "Video" => Self::Video,
            "Music" => Self::Music,
            "Lossless" => Self::Lossless,
            "Crypto" => Self::Crypto,
            "Document" => Self::Document,
            "Compressed" => Self::Compressed,
            "Temp" => Self::Temp,
            "Compiled" => Self::Compiled,
            "Build" => Self::Build,
            "Source" => Self::Source,
            _ => return None,
        })
    }

    fn vivid_group(self) -> &'static str {
        match self {
            Self::Image => "eza_image",
            Self::Video => "eza_video",
            Self::Music => "eza_music",
            Self::Lossless => "eza_lossless",
            Self::Crypto => "eza_crypto",
            Self::Document => "eza_document",
            Self::Compressed => "eza_compressed",
            Self::Temp => "eza_temp",
            Self::Compiled => "eza_compiled",
            Self::Build => "eza_build",
            Self::Source => "eza_source",
        }
    }

    fn theme_key(self) -> &'static str {
        self.vivid_group().strip_prefix("eza_").unwrap()
    }

    fn style(self) -> StyleSpec {
        match self {
            Self::Image => StyleSpec::new(Some("magenta"), &[]),
            Self::Video => StyleSpec::new(Some("magenta"), &["bold"]),
            Self::Music => StyleSpec::new(Some("cyan"), &[]),
            Self::Lossless => StyleSpec::new(Some("cyan"), &["bold"]),
            Self::Crypto => StyleSpec::new(Some("green"), &["bold"]),
            Self::Document => StyleSpec::new(Some("green"), &[]),
            Self::Compressed => StyleSpec::new(Some("red"), &[]),
            Self::Temp => StyleSpec::new(None, &["faint"]),
            Self::Compiled => StyleSpec::new(Some("yellow"), &[]),
            Self::Build => StyleSpec::new(Some("yellow"), &["bold", "underline"]),
            Self::Source => StyleSpec::new(Some("yellow"), &["bold"]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct StyleSpec {
    foreground: Option<&'static str>,
    attrs: &'static [&'static str],
}

impl StyleSpec {
    const fn new(foreground: Option<&'static str>, attrs: &'static [&'static str]) -> Self {
        Self { foreground, attrs }
    }

    fn write_yaml(self, out: &mut String, indent: &str) {
        if let Some(foreground) = self.foreground {
            out.push_str(indent);
            out.push_str("foreground: ");
            out.push_str(foreground);
            out.push('\n');
        }

        match self.attrs {
            [] => {}
            [one] => {
                out.push_str(indent);
                out.push_str("font-style: ");
                out.push_str(one);
                out.push('\n');
            }
            attrs => {
                out.push_str(indent);
                out.push_str("font-style: [");
                out.push_str(&attrs.join(", "));
                out.push_str("]\n");
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct EzaFileTypes {
    filenames: BTreeMap<String, FileCategory>,
    extensions: BTreeMap<String, FileCategory>,
    patterns: BTreeMap<String, FileCategory>,
}

fn main() -> Result<()> {
    match parse_args(env::args().skip(1))? {
        Subcmd::Generate(args) => generate(args),
    }
}

fn parse_args<I>(args: I) -> Result<Subcmd>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter().peekable();
    if matches!(args.peek().map(String::as_str), Some("--help" | "-h")) {
        print_help();
        std::process::exit(0);
    }
    if matches!(args.peek().map(String::as_str), Some("generate")) {
        args.next();
    }

    let mut out = GenerateArgs::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out-dir" => out.out_dir = PathBuf::from(next_value(&mut args, "--out-dir")?),
            "--eza-ref" => out.eza_ref = next_value(&mut args, "--eza-ref")?,
            "--source" => out.source = Some(PathBuf::from(next_value(&mut args, "--source")?)),
            "--vivid-ref" => out.vivid_ref = next_value(&mut args, "--vivid-ref")?,
            "--vivid-source" => {
                out.vivid_source = Some(PathBuf::from(next_value(&mut args, "--vivid-source")?));
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(err(format!("unknown argument: {other}"))),
        }
    }

    Ok(Subcmd::Generate(out))
}

fn next_value<I>(args: &mut I, flag: &str) -> Result<String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| err(format!("missing value for {flag}")))
}

fn print_help() {
    println!(
        "vieza\n\nUsage:\n  vieza generate [--out-dir DIR] [--eza-ref REF] [--source PATH] [--vivid-ref REF] [--vivid-source PATH]\n\nOptions:\n  --out-dir DIR        Output directory (default: generated)\n  --eza-ref REF        eza git ref to fetch\n  --source PATH        Local eza src/info/filetype.rs\n  --vivid-ref REF      vivid git ref to fetch\n  --vivid-source PATH  Local vivid config/filetypes.yml"
    );
}

fn generate(args: GenerateArgs) -> Result<()> {
    let source = match args.source {
        Some(path) => fs::read_to_string(&path)
            .map_err(|e| err(format!("failed to read {}: {e}", path.display())))?,
        None => fetch_eza_filetypes(&args.eza_ref)?,
    };
    let vivid_filetypes = match args.vivid_source {
        Some(path) => fs::read_to_string(&path)
            .map_err(|e| err(format!("failed to read {}: {e}", path.display())))?,
        None => fetch_vivid_filetypes(&args.vivid_ref)?,
    };
    let filetypes = parse_filetypes(&source)?;
    let eza_filetypes = render_vivid_filetypes(&filetypes);
    let filtered_vivid_filetypes = filter_vivid_filetypes(&vivid_filetypes, &filetypes);
    fs::create_dir_all(&args.out_dir)
        .map_err(|e| err(format!("failed to create {}: {e}", args.out_dir.display())))?;

    let theme_path = args.out_dir.join("vieza.yml");
    let eza_db_path = args.out_dir.join("vieza-filetypes.yml");
    let combined_db_path = args.out_dir.join("filetypes.yml");
    let ls_colors_path = args.out_dir.join("LS_COLORS");

    fs::write(&theme_path, render_vivid_theme())?;
    fs::write(&eza_db_path, &eza_filetypes)?;
    fs::write(
        &combined_db_path,
        render_combined_vivid_filetypes(&filtered_vivid_filetypes, &eza_filetypes),
    )?;
    fs::write(
        &ls_colors_path,
        generate_ls_colors_with_vivid(&combined_db_path, &theme_path)?,
    )?;

    Ok(())
}

fn fetch_eza_filetypes(eza_ref: &str) -> Result<String> {
    let url = format!(
        "https://raw.githubusercontent.com/eza-community/eza/{eza_ref}/src/info/filetype.rs"
    );

    fetch_with_command("curl", &["-fsSL", &url])
        .or_else(|_| fetch_with_command("wget", &["-qO-", &url]))
        .map_err(|e| {
            err(format!(
                "failed to fetch {url}; install curl/wget or pass --source: {e}"
            ))
        })
}

fn fetch_vivid_filetypes(vivid_ref: &str) -> Result<String> {
    let url =
        format!("https://raw.githubusercontent.com/sharkdp/vivid/{vivid_ref}/config/filetypes.yml");

    fetch_with_command("curl", &["-fsSL", &url])
        .or_else(|_| fetch_with_command("wget", &["-qO-", &url]))
        .map_err(|e| {
            err(format!(
                "failed to fetch {url}; install curl/wget or pass --vivid-source: {e}"
            ))
        })
}

fn fetch_with_command(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| err(format!("failed to run {program}: {e}")))?;

    if !output.status.success() {
        return Err(err(format!("{program} exited with {}", output.status)));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| err(format!("{program} returned non-UTF-8 output: {e}")))
}

fn parse_filetypes(source: &str) -> Result<EzaFileTypes> {
    let mut result = EzaFileTypes::default();
    parse_map(source, "FILENAME_TYPES", &mut result.filenames)?;
    parse_map(source, "EXTENSION_TYPES", &mut result.extensions)?;
    add_dynamic_eza_rules(&mut result);
    Ok(result)
}

fn add_dynamic_eza_rules(filetypes: &mut EzaFileTypes) {
    // Mirrors FileType::get_file_type rules not represented by eza's phf maps.
    // LS_COLORS suffix matching cannot fully encode eza's `readme*` prefix rule,
    // so include common exact filenames in addition to best-effort patterns.
    add_readme_filename_rules(filetypes);

    for pattern in ["README*", "Readme*", "readme*"] {
        filetypes
            .patterns
            .insert(pattern.to_string(), FileCategory::Build);
    }
    filetypes
        .patterns
        .insert("*~".to_string(), FileCategory::Temp);
    filetypes
        .patterns
        .insert("#*#".to_string(), FileCategory::Temp);
    // Best-effort encoding for eza's `starts_with('#') && ends_with('#')` rule.
    // LS_COLORS can match suffixes, but not both anchors; this catches Emacs-style autosaves.
    filetypes
        .patterns
        .insert("#".to_string(), FileCategory::Temp);
}

fn add_readme_filename_rules(filetypes: &mut EzaFileTypes) {
    let bases = ["README", "Readme", "ReadMe", "readme"];
    let suffixes = [
        "",
        ".txt",
        ".md",
        ".markdown",
        ".mdown",
        ".mkd",
        ".mdx",
        ".rst",
        ".adoc",
        ".asciidoc",
        ".org",
        ".norg",
        ".tex",
        ".typ",
        ".html",
        ".htm",
        ".xml",
        ".pdf",
    ];

    for base in bases {
        for suffix in suffixes {
            filetypes
                .filenames
                .insert(format!("{base}{suffix}"), FileCategory::Build);
        }
    }
}

fn parse_map(
    source: &str,
    map_name: &str,
    target: &mut BTreeMap<String, FileCategory>,
) -> Result<()> {
    let marker = format!("const {map_name}:");
    let start = source
        .find(&marker)
        .ok_or_else(|| err(format!("missing {map_name}")))?;
    let after_marker = &source[start..];
    let body_start = after_marker
        .find("phf_map! {")
        .ok_or_else(|| err(format!("missing phf_map body for {map_name}")))?
        + "phf_map! {".len();
    let body = &after_marker[body_start..];
    let body_end = body
        .find("};")
        .ok_or_else(|| err(format!("missing phf_map end for {map_name}")))?;

    for raw_line in body[..body_end].lines() {
        let line = raw_line.split("//").next().unwrap_or_default().trim();
        if line.is_empty() || !line.starts_with('"') {
            continue;
        }

        let Some((key, rest)) = parse_quoted_key(line) else {
            continue;
        };
        let Some(category_name) = rest.split("FileType::").nth(1) else {
            continue;
        };
        let category_name: String = category_name
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        let Some(category) = FileCategory::from_rust_name(&category_name) else {
            return Err(err(format!(
                "unknown FileType::{category_name} in {map_name}"
            )));
        };
        target.insert(key.to_string(), category);
    }

    Ok(())
}

fn parse_quoted_key(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some((&rest[..end], &rest[end + 1..]))
}

fn render_vivid_theme() -> String {
    let mut out =
        String::from("# Generated by vieza. Uses eza-like styles with terminal ANSI colors.\n\n");
    write_ansi_colors(&mut out);
    write_core_theme(&mut out);
    write_stock_vivid_categories(&mut out);
    out.push_str("\n\neza:\n");
    for category in all_categories() {
        out.push_str("  ");
        out.push_str(category.theme_key());
        out.push_str(":\n");
        category.style().write_yaml(&mut out, "    ");
    }

    out
}

fn write_ansi_colors(out: &mut String) {
    out.push_str("colors:\n");
    for color in ["red", "green", "yellow", "blue", "magenta", "cyan"] {
        out.push_str("  ");
        out.push_str(color);
        out.push_str(": \"ansi:");
        out.push_str(color);
        out.push_str("\"\n");
    }
}

fn write_core_theme(out: &mut String) {
    out.push_str("\ncore:\n");
    for (key, style) in [
        ("normal_text", StyleSpec::new(None, &[])),
        ("regular_file", StyleSpec::new(None, &[])),
        ("reset_to_normal", StyleSpec::new(None, &[])),
        ("directory", StyleSpec::new(Some("blue"), &["bold"])),
        ("symlink", StyleSpec::new(Some("cyan"), &[])),
        ("multi_hard_link", StyleSpec::new(None, &[])),
        ("fifo", StyleSpec::new(Some("yellow"), &[])),
        ("socket", StyleSpec::new(Some("red"), &["bold"])),
        ("door", StyleSpec::new(Some("red"), &["bold"])),
        ("block_device", StyleSpec::new(Some("yellow"), &["bold"])),
        (
            "character_device",
            StyleSpec::new(Some("yellow"), &["bold"]),
        ),
        ("broken_symlink", StyleSpec::new(Some("red"), &[])),
        (
            "missing_symlink_target",
            StyleSpec::new(Some("red"), &["bold"]),
        ),
        (
            "setuid",
            StyleSpec::new(Some("green"), &["bold", "underline"]),
        ),
        (
            "setgid",
            StyleSpec::new(Some("yellow"), &["bold", "underline"]),
        ),
        ("file_with_capability", StyleSpec::new(None, &[])),
        (
            "sticky_other_writable",
            StyleSpec::new(Some("blue"), &["bold", "underline"]),
        ),
        (
            "other_writable",
            StyleSpec::new(Some("blue"), &["underline"]),
        ),
        ("sticky", StyleSpec::new(Some("blue"), &["bold"])),
        ("executable_file", StyleSpec::new(Some("green"), &["bold"])),
    ] {
        out.push_str("  ");
        out.push_str(key);
        out.push_str(":");
        if style.foreground.is_none() && style.attrs.is_empty() {
            out.push_str(" {}\n");
        } else {
            out.push('\n');
            style.write_yaml(out, "    ");
        }
    }
}

fn write_stock_vivid_categories(out: &mut String) {
    out.push_str("\ntext:\n");
    write_style(out, "  special", FileCategory::Build.style());
    write_style(out, "  todo", StyleSpec::new(Some("yellow"), &["bold"]));
    write_style(out, "  licenses", StyleSpec::new(None, &["regular"]));
    write_style(out, "  configuration", StyleSpec::new(None, &["regular"]));
    write_style(out, "  other", StyleSpec::new(None, &["regular"]));

    out.push_str("\nmarkup:\n");
    StyleSpec::new(None, &["regular"]).write_yaml(out, "  ");

    out.push_str("\nprogramming:\n");
    write_style(out, "  source", FileCategory::Source.style());
    write_style(out, "  tooling", StyleSpec::new(None, &["regular"]));

    out.push_str("\nmedia:\n");
    write_style(out, "  image", FileCategory::Image.style());
    write_style(out, "  audio", FileCategory::Music.style());
    write_style(out, "  video", FileCategory::Video.style());
    write_style(out, "  fonts", StyleSpec::new(None, &["regular"]));
    write_style(out, "  3d", FileCategory::Image.style());

    out.push_str("\noffice:\n");
    FileCategory::Document.style().write_yaml(out, "  ");

    out.push_str("\narchives:\n");
    FileCategory::Compressed.style().write_yaml(out, "  ");

    out.push_str("\nexecutable:\n");
    write_style(out, "  windows", StyleSpec::new(Some("green"), &["bold"]));
    write_style(out, "  library", FileCategory::Compiled.style());
    write_style(out, "  linux", FileCategory::Compiled.style());

    out.push_str("\nunimportant:\n");
    FileCategory::Temp.style().write_yaml(out, "  ");
}

fn write_style(out: &mut String, key: &str, style: StyleSpec) {
    out.push_str(key);
    out.push_str(":\n");
    style.write_yaml(out, "    ");
}

fn render_vivid_filetypes(filetypes: &EzaFileTypes) -> String {
    let mut grouped: BTreeMap<FileCategory, Vec<String>> = BTreeMap::new();
    for (name, category) in &filetypes.filenames {
        grouped.entry(*category).or_default().push(name.to_string());
    }
    for (ext, category) in &filetypes.extensions {
        grouped
            .entry(*category)
            .or_default()
            .push(format!(".{ext}"));
    }
    for (pattern, category) in &filetypes.patterns {
        grouped
            .entry(*category)
            .or_default()
            .push(pattern.to_string());
    }

    let mut out = String::from("# Generated by vieza from eza src/info/filetype.rs.\n\n");
    out.push_str("eza:\n");
    for category in all_categories() {
        out.push_str("  ");
        out.push_str(category.theme_key());
        out.push_str(":\n");
        if let Some(entries) = grouped.get(&category) {
            for entry in entries {
                out.push_str("    - ");
                out.push_str(&yaml_quote(entry));
                out.push('\n');
            }
        }
    }
    out
}

fn normalize_vivid_entry(entry: &str) -> String {
    let entry = strip_vivid_syntax(entry);

    if entry.contains('*') {
        entry
    } else {
        format!("*{entry}")
    }
}

fn filter_vivid_filetypes(vivid_filetypes: &str, eza_filetypes: &EzaFileTypes) -> String {
    let rules = OverrideRules::from_eza(filetypes_to_entries(eza_filetypes));
    let mut out = String::new();

    for raw_line in vivid_filetypes.lines() {
        let line_without_comment = raw_line.split('#').next().unwrap_or_default();
        let trimmed = line_without_comment.trim();

        if let Some(entry) = trimmed.strip_prefix("- ") {
            if rules.overrides(entry) {
                continue;
            }
        }

        if let (Some(start), Some(end)) = (raw_line.find('['), raw_line.rfind(']')) {
            if start < end {
                let entries = raw_line[start + 1..end]
                    .split(',')
                    .map(str::trim)
                    .filter(|entry| !entry.is_empty() && !rules.overrides(entry))
                    .collect::<Vec<_>>();

                if entries.is_empty() {
                    continue;
                }

                out.push_str(&raw_line[..start + 1]);
                out.push_str(&entries.join(", "));
                out.push_str(&raw_line[end..]);
                out.push('\n');
                continue;
            }
        }

        out.push_str(raw_line);
        out.push('\n');
    }

    prune_empty_yaml_mappings(&out)
}

fn prune_empty_yaml_mappings(input: &str) -> String {
    let mut lines = input.lines().map(str::to_string).collect::<Vec<_>>();

    loop {
        let mut remove = BTreeSet::new();

        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with('-')
                || !trimmed.ends_with(':')
            {
                continue;
            }

            let indent = line_indent(line);
            let mut next = index + 1;
            while next < lines.len() && lines[next].trim().is_empty() {
                next += 1;
            }

            if next == lines.len() || line_indent(&lines[next]) <= indent {
                remove.insert(index);
            }
        }

        if remove.is_empty() {
            break;
        }

        lines = lines
            .into_iter()
            .enumerate()
            .filter_map(|(index, line)| (!remove.contains(&index)).then_some(line))
            .collect();
    }

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn line_indent(line: &str) -> usize {
    line.chars().take_while(|char| *char == ' ').count()
}

fn filetypes_to_entries(filetypes: &EzaFileTypes) -> Vec<String> {
    filetypes
        .filenames
        .keys()
        .cloned()
        .chain(filetypes.extensions.keys().map(|ext| format!(".{ext}")))
        .chain(filetypes.patterns.keys().cloned())
        .collect()
}

struct OverrideRules {
    exact: BTreeSet<String>,
    patterns: Vec<String>,
}

impl OverrideRules {
    fn from_eza(entries: Vec<String>) -> Self {
        let mut exact = BTreeSet::new();
        let mut patterns = Vec::new();

        for entry in entries {
            if entry.contains('*') {
                patterns.push(entry);
            } else {
                exact.insert(normalize_vivid_entry(&entry));
            }
        }

        Self { exact, patterns }
    }

    fn overrides(&self, entry: &str) -> bool {
        let normalized = normalize_vivid_entry(entry);
        let plain = strip_vivid_syntax(entry);

        self.exact.contains(&normalized)
            || self
                .patterns
                .iter()
                .any(|pattern| wildcard_match(pattern, &plain))
    }
}

fn strip_vivid_syntax(entry: &str) -> String {
    entry
        .trim()
        .trim_end_matches(',')
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let parts = pattern.split('*').collect::<Vec<_>>();
    if parts.len() == 1 {
        return pattern == value;
    }

    let mut rest = value;
    let anchored_start = !pattern.starts_with('*');
    let anchored_end = !pattern.ends_with('*');

    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if index == 0 && anchored_start {
            let Some(next) = rest.strip_prefix(part) else {
                return false;
            };
            rest = next;
            continue;
        }

        let Some(position) = rest.find(part) else {
            return false;
        };
        rest = &rest[position + part.len()..];
    }

    !anchored_end || parts.last().is_none_or(|last| value.ends_with(last))
}

fn render_combined_vivid_filetypes(vivid_filetypes: &str, eza_filetypes: &str) -> String {
    let mut out = String::from(
        "# Generated by vieza. Includes pinned vivid defaults plus eza-derived additions.\n",
    );
    out.push_str(vivid_filetypes.trim_end());
    out.push_str("\n\n");
    out.push_str(eza_filetypes.trim_start());
    out
}

fn generate_ls_colors_with_vivid(database_path: &Path, theme_path: &Path) -> Result<String> {
    let output = Command::new("vivid")
        .arg("-d")
        .arg(database_path)
        .arg("generate")
        .arg(theme_path)
        .output()
        .map_err(|e| err(format!("failed to run vivid: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(err(format!(
            "vivid failed while generating LS_COLORS from {} and {}: {}",
            database_path.display(),
            theme_path.display(),
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| err(format!("vivid returned non-UTF-8 output: {e}")))
}

fn all_categories() -> [FileCategory; 11] {
    [
        FileCategory::Image,
        FileCategory::Video,
        FileCategory::Music,
        FileCategory::Lossless,
        FileCategory::Crypto,
        FileCategory::Document,
        FileCategory::Compressed,
        FileCategory::Temp,
        FileCategory::Compiled,
        FileCategory::Build,
        FileCategory::Source,
    ]
}

fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn err(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(io::Error::other(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
const FILENAME_TYPES: Map<&'static str, FileType> = phf_map! {
    "Cargo.toml" => FileType::Build,
    "id_ed25519" => FileType::Crypto,
};

const EXTENSION_TYPES: Map<&'static str, FileType> = phf_map! {
    "png" => FileType::Image,
    "mp4" => FileType::Video,
    "rs" => FileType::Source,
    "zip" => FileType::Compressed,
};
"#;

    #[test]
    fn parses_eza_filetypes() {
        let parsed = parse_filetypes(SAMPLE).unwrap();
        assert_eq!(parsed.filenames["Cargo.toml"], FileCategory::Build);
        assert_eq!(parsed.filenames["id_ed25519"], FileCategory::Crypto);
        assert_eq!(parsed.extensions["png"], FileCategory::Image);
        assert_eq!(parsed.extensions["rs"], FileCategory::Source);
        assert_eq!(parsed.filenames["README.md"], FileCategory::Build);
        assert_eq!(parsed.filenames["README.rst"], FileCategory::Build);
        assert_eq!(parsed.filenames["ReadMe.html"], FileCategory::Build);
        assert_eq!(parsed.patterns["README*"], FileCategory::Build);
        assert_eq!(parsed.patterns["*~"], FileCategory::Temp);
        assert_eq!(parsed.patterns["#"], FileCategory::Temp);
    }

    #[test]
    fn renders_outputs() {
        let parsed = parse_filetypes(SAMPLE).unwrap();
        let database = render_vivid_filetypes(&parsed);
        assert!(database.contains("- \"Cargo.toml\""));
        assert!(database.contains("- \"README.md\""));
        assert!(database.contains("- \"README.rst\""));
        assert!(database.contains("- \".rs\""));
        assert!(database.contains("- \"README*\""));
        assert!(database.contains("- \"#\""));
    }

    #[test]
    fn renders_core_file_kind_styles_to_match_eza() {
        let theme = render_vivid_theme();
        assert!(theme.contains("directory:\n    foreground: blue\n    font-style: bold"));
        assert!(theme.contains("executable_file:\n    foreground: green\n    font-style: bold"));
    }

    #[test]
    fn filters_entries_already_present_in_vivid_database() {
        let parsed = parse_filetypes(SAMPLE).unwrap();
        let database = filter_vivid_filetypes(
            r#"
text:
  special:
    - README
    - README.md
programming:
  source:
    rust: [.rs]
unimportant:
  build_artifacts:
    java:
      - .class
programming:
  tooling:
    build:
      cargo:
        - Cargo.toml
"#,
            &parsed,
        );

        assert!(!database.contains("- README"));
        assert!(!database.contains("- README.md"));
        assert!(!database.contains("[.rs]"));
        assert!(!database.contains("- Cargo.toml"));
        assert!(database.contains("- .class"));
    }

    #[test]
    fn parses_cli_flags() {
        let Subcmd::Generate(args) = parse_args([
            "generate".to_string(),
            "--out-dir".to_string(),
            "out".to_string(),
            "--eza-ref".to_string(),
            "abc123".to_string(),
            "--source".to_string(),
            "filetype.rs".to_string(),
            "--vivid-ref".to_string(),
            "def456".to_string(),
            "--vivid-source".to_string(),
            "filetypes.yml".to_string(),
        ])
        .unwrap();

        assert_eq!(args.out_dir, PathBuf::from("out"));
        assert_eq!(args.eza_ref, "abc123");
        assert_eq!(args.vivid_ref, "def456");
        assert_eq!(args.source, Some(PathBuf::from("filetype.rs")));
        assert_eq!(args.vivid_source, Some(PathBuf::from("filetypes.yml")));
    }
}
