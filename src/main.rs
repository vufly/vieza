use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
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
    vivid_theme_source: Option<PathBuf>,
    print_ls_colors: bool,
}

impl Default for GenerateArgs {
    fn default() -> Self {
        Self {
            out_dir: PathBuf::from("generated"),
            eza_ref: DEFAULT_EZA_REF.to_string(),
            vivid_ref: DEFAULT_VIVID_REF.to_string(),
            source: None,
            vivid_source: None,
            vivid_theme_source: None,
            print_ls_colors: false,
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

    fn ansi_codes(self) -> String {
        let mut codes: Vec<&str> = self
            .attrs
            .iter()
            .filter_map(|attr| match *attr {
                "bold" => Some("1"),
                "faint" => Some("2"),
                "italic" => Some("3"),
                "underline" => Some("4"),
                _ => None,
            })
            .collect();

        if let Some(code) = self.foreground.and_then(|color| match color {
            "red" => Some("31"),
            "green" => Some("32"),
            "yellow" => Some("33"),
            "blue" => Some("34"),
            "magenta" => Some("35"),
            "cyan" => Some("36"),
            _ => None,
        }) {
            codes.push(code);
        }

        if codes.is_empty() {
            "0".to_string()
        } else {
            codes.join(";")
        }
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
            "--vivid-theme-source" => {
                out.vivid_theme_source = Some(PathBuf::from(next_value(
                    &mut args,
                    "--vivid-theme-source",
                )?));
            }
            "--print-ls-colors" => out.print_ls_colors = true,
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
        "eza-vivid\n\nUsage:\n  eza-vivid generate [--out-dir DIR] [--eza-ref REF] [--source PATH] [--vivid-ref REF] [--vivid-source PATH] [--vivid-theme-source PATH] [--print-ls-colors]\n\nOptions:\n  --out-dir DIR              Output directory (default: generated)\n  --eza-ref REF              eza git ref to fetch\n  --source PATH              Local eza src/info/filetype.rs\n  --vivid-ref REF            vivid git ref to fetch\n  --vivid-source PATH        Local vivid config/filetypes.yml\n  --vivid-theme-source PATH  Local vivid themes/ansi.yml\n  --print-ls-colors          Print generated LS_COLORS to stdout"
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
    let vivid_theme = match args.vivid_theme_source {
        Some(path) => fs::read_to_string(&path)
            .map_err(|e| err(format!("failed to read {}: {e}", path.display())))?,
        None => fetch_vivid_ansi_theme(&args.vivid_ref)?,
    };

    let filetypes = parse_filetypes(&source)?;
    let eza_filetypes = render_vivid_filetypes(&filetypes);
    let combined_eza_filetypes =
        render_vivid_filetypes_skipping(&filetypes, &collect_vivid_filetype_keys(&vivid_filetypes));
    fs::create_dir_all(&args.out_dir)
        .map_err(|e| err(format!("failed to create {}: {e}", args.out_dir.display())))?;

    fs::write(
        args.out_dir.join("eza-adaptive.yml"),
        render_vivid_theme(&vivid_theme),
    )?;
    fs::write(args.out_dir.join("filetypes-eza.yml"), &eza_filetypes)?;
    fs::write(
        args.out_dir.join("filetypes-vivid-eza.yml"),
        render_combined_vivid_filetypes(&vivid_filetypes, &combined_eza_filetypes),
    )?;

    let ls_colors = render_ls_colors(&filetypes);
    fs::write(args.out_dir.join("LS_COLORS"), &ls_colors)?;

    if args.print_ls_colors {
        println!("{ls_colors}");
    }

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

fn fetch_vivid_ansi_theme(vivid_ref: &str) -> Result<String> {
    let url =
        format!("https://raw.githubusercontent.com/sharkdp/vivid/{vivid_ref}/themes/ansi.yml");

    fetch_with_command("curl", &["-fsSL", &url])
        .or_else(|_| fetch_with_command("wget", &["-qO-", &url]))
        .map_err(|e| {
            err(format!(
                "failed to fetch {url}; install curl/wget or pass --vivid-theme-source: {e}"
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

fn render_vivid_theme(vivid_ansi_theme: &str) -> String {
    let mut out = String::from(
        "# Generated by eza-vivid. Base is pinned vivid ansi.yml; eza section is appended.\n",
    );
    out.push_str(vivid_ansi_theme.trim_end());
    out.push_str("\n\neza:\n");
    for category in all_categories() {
        out.push_str("  ");
        out.push_str(category.theme_key());
        out.push_str(":\n");
        category.style().write_yaml(&mut out, "    ");
    }

    out
}

fn render_vivid_filetypes(filetypes: &EzaFileTypes) -> String {
    render_vivid_filetypes_skipping(filetypes, &BTreeSet::new())
}

fn render_vivid_filetypes_skipping(
    filetypes: &EzaFileTypes,
    skip_keys: &BTreeSet<String>,
) -> String {
    let mut grouped: BTreeMap<FileCategory, Vec<String>> = BTreeMap::new();
    for (name, category) in &filetypes.filenames {
        push_vivid_entry(&mut grouped, *category, name.to_string(), skip_keys);
    }
    for (ext, category) in &filetypes.extensions {
        push_vivid_entry(&mut grouped, *category, format!(".{ext}"), skip_keys);
    }
    for (pattern, category) in &filetypes.patterns {
        push_vivid_entry(&mut grouped, *category, pattern.to_string(), skip_keys);
    }

    let mut out = String::from("# Generated by eza-vivid from eza src/info/filetype.rs.\n\n");
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

fn push_vivid_entry(
    grouped: &mut BTreeMap<FileCategory, Vec<String>>,
    category: FileCategory,
    entry: String,
    skip_keys: &BTreeSet<String>,
) {
    if !skip_keys.contains(&normalize_vivid_entry(&entry)) {
        grouped.entry(category).or_default().push(entry);
    }
}

fn collect_vivid_filetype_keys(filetypes_yml: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for raw_line in filetypes_yml.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }

        if let Some(entry) = line.strip_prefix("- ") {
            keys.insert(normalize_vivid_entry(entry));
        }

        if let (Some(start), Some(end)) = (line.find('['), line.rfind(']')) {
            for entry in line[start + 1..end].split(',') {
                let entry = entry.trim();
                if !entry.is_empty() {
                    keys.insert(normalize_vivid_entry(entry));
                }
            }
        }
    }
    keys
}

fn normalize_vivid_entry(entry: &str) -> String {
    let entry = entry
        .trim()
        .trim_end_matches(',')
        .trim_matches('"')
        .trim_matches('\'');

    if entry.contains('*') {
        entry.to_string()
    } else {
        format!("*{entry}")
    }
}

fn render_combined_vivid_filetypes(vivid_filetypes: &str, eza_filetypes: &str) -> String {
    let mut out = String::from(
        "# Generated by eza-vivid. Includes pinned vivid defaults plus eza-derived additions.\n",
    );
    out.push_str(vivid_filetypes.trim_end());
    out.push_str("\n\n");
    out.push_str(eza_filetypes.trim_start());
    out
}

fn render_ls_colors(filetypes: &EzaFileTypes) -> String {
    let mut pairs: Vec<String> = core_ls_pairs()
        .into_iter()
        .map(|(key, style)| format!("{key}={}", style.ansi_codes()))
        .collect();

    for (name, category) in &filetypes.filenames {
        pairs.push(format!(
            "{}={}",
            escape_ls_key(name),
            category.style().ansi_codes()
        ));
    }
    for (ext, category) in &filetypes.extensions {
        pairs.push(format!(
            "*.{}={}",
            escape_ls_key(ext),
            category.style().ansi_codes()
        ));
    }
    for (pattern, category) in &filetypes.patterns {
        pairs.push(format!(
            "{}={}",
            escape_ls_key(pattern),
            category.style().ansi_codes()
        ));
    }

    pairs.join(":")
}

fn core_ls_pairs() -> Vec<(&'static str, StyleSpec)> {
    vec![
        ("fi", StyleSpec::new(None, &[])),
        ("di", StyleSpec::new(Some("blue"), &["bold"])),
        ("ln", StyleSpec::new(Some("cyan"), &[])),
        ("pi", StyleSpec::new(Some("yellow"), &[])),
        ("so", StyleSpec::new(Some("red"), &["bold"])),
        ("bd", StyleSpec::new(Some("yellow"), &["bold"])),
        ("cd", StyleSpec::new(Some("yellow"), &["bold"])),
        ("or", StyleSpec::new(Some("red"), &[])),
        ("mi", StyleSpec::new(Some("red"), &["bold"])),
        ("ex", StyleSpec::new(Some("green"), &["bold"])),
    ]
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

fn escape_ls_key(value: &str) -> String {
    value.replace(':', "\\:").replace('=', "\\=")
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
        assert_eq!(parsed.patterns["README*"], FileCategory::Build);
        assert_eq!(parsed.patterns["*~"], FileCategory::Temp);
    }

    #[test]
    fn serializes_styles_to_ansi_codes() {
        assert_eq!(FileCategory::Build.style().ansi_codes(), "1;4;33");
        assert_eq!(FileCategory::Temp.style().ansi_codes(), "2");
        assert_eq!(FileCategory::Image.style().ansi_codes(), "35");
    }

    #[test]
    fn renders_outputs() {
        let parsed = parse_filetypes(SAMPLE).unwrap();
        let ls_colors = render_ls_colors(&parsed);
        assert!(ls_colors.contains("di=1;34"));
        assert!(ls_colors.contains("Cargo.toml=1;4;33"));
        assert!(ls_colors.contains("*.png=35"));
        assert!(ls_colors.contains("README*=1;4;33"));
        assert!(ls_colors.contains("*~=2"));

        let database = render_vivid_filetypes(&parsed);
        assert!(database.contains("- \"Cargo.toml\""));
        assert!(database.contains("- \".rs\""));
        assert!(database.contains("- \"README*\""));
    }

    #[test]
    fn filters_entries_already_present_in_vivid_database() {
        let parsed = parse_filetypes(SAMPLE).unwrap();
        let vivid_keys = collect_vivid_filetype_keys(
            r#"
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
        );

        assert!(vivid_keys.contains("*.rs"));
        assert!(vivid_keys.contains("*.class"));
        assert!(vivid_keys.contains("*Cargo.toml"));

        let database = render_vivid_filetypes_skipping(&parsed, &vivid_keys);
        assert!(!database.contains("- \"Cargo.toml\""));
        assert!(!database.contains("- \".rs\""));
        assert!(database.contains("- \"README*\""));
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
            "--vivid-theme-source".to_string(),
            "ansi.yml".to_string(),
            "--print-ls-colors".to_string(),
        ])
        .unwrap();

        assert_eq!(args.out_dir, PathBuf::from("out"));
        assert_eq!(args.eza_ref, "abc123");
        assert_eq!(args.vivid_ref, "def456");
        assert_eq!(args.source, Some(PathBuf::from("filetype.rs")));
        assert_eq!(args.vivid_source, Some(PathBuf::from("filetypes.yml")));
        assert_eq!(args.vivid_theme_source, Some(PathBuf::from("ansi.yml")));
        assert!(args.print_ls_colors);
    }
}
