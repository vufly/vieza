# vieza

Generate [vivid](https://github.com/sharkdp/vivid) theme/database files that mimic [eza](https://github.com/eza-community/eza) filename colors, then use vivid to produce `LS_COLORS`.

## Requirements

- `vivid` must be installed and available in `PATH` because `vieza generate` delegates final `LS_COLORS` rendering to vivid.
- `curl` or `wget` is needed when generating from upstream refs. Use `--source` and `--vivid-source` to generate from local files instead.

## TL;DR

If you only want the latest generated `LS_COLORS`, download it from the latest GitHub release and export it for the current shell.

Bash, Zsh, Ksh:

```sh
export LS_COLORS="$(curl -fsSL https://github.com/vufly/vieza/releases/latest/download/LS_COLORS)"
```

Fish:

```fish
set -gx LS_COLORS (curl -fsSL https://github.com/vufly/vieza/releases/latest/download/LS_COLORS | string trim)
```

Nushell:

```nu
$env.LS_COLORS = (http get https://github.com/vufly/vieza/releases/latest/download/LS_COLORS | str trim)
```

PowerShell:

```powershell
$env:LS_COLORS = (Invoke-WebRequest -UseBasicParsing https://github.com/vufly/vieza/releases/latest/download/LS_COLORS).Content.Trim()
```

Tcsh/Csh:

```csh
setenv LS_COLORS "`curl -fsSL https://github.com/vufly/vieza/releases/latest/download/LS_COLORS`"
```

This does not install `vieza` or `vivid`. It only uses the prebuilt release asset.

## Install

From crates.io:

```sh
cargo install vieza
```

From GitHub source:

```sh
cargo install --git https://github.com/vufly/vieza
```

Or download a binary archive from the GitHub release assets.

## Usage

Installed CLI:

```sh
vieza generate
```

Source checkout:

```sh
git clone https://github.com/vufly/vieza.git
cd vieza
cargo run -- generate
```

Both commands write the same outputs:

- `generated/vieza.yml`
- `generated/vieza-filetypes.yml`
- `generated/filetypes.yml`
- `generated/LS_COLORS`

Examples below use `vieza generate`. If you are working from a source checkout without installing, use `cargo run -- generate` instead.

## Release Assets

GitHub releases publish these assets:

1. `vieza.zip`: all generated files in one archive.
2. `LS_COLORS`: generated `LS_COLORS` output.
3. `filetypes.yml`: vivid defaults plus eza-derived filetypes.
4. `vieza-filetypes.yml`: eza-derived filetypes only.
5. `vieza.yml`: vivid theme that mimics eza filename colors.
6. `vieza-*.tar.gz` / `vieza-*.zip`: Linux, macOS, and Windows binary archives for x86_64 and ARM64 where supported.

## Shell Setup

### 1. Generate Files

Generate or refresh files in `generated/`:

```sh
vieza generate
```

### 2. Export Generated LS_COLORS

Use this when `generated/LS_COLORS` already exists.

Bash, Zsh, Ksh:

```sh
export LS_COLORS="$(tr -d '\n' < generated/LS_COLORS)"
```

Fish:

```fish
set -gx LS_COLORS (string trim < generated/LS_COLORS)
```

Nushell:

```nu
$env.LS_COLORS = (open generated/LS_COLORS | str trim)
```

PowerShell:

```powershell
$env:LS_COLORS = (Get-Content -Raw generated/LS_COLORS).Trim()
```

Tcsh/Csh:

```csh
setenv LS_COLORS "`cat generated/LS_COLORS`"
```

### 3. Generate And Export

Use this when you want one command to refresh files and update the current shell.

Bash, Zsh, Ksh:

```sh
vieza generate && export LS_COLORS="$(tr -d '\n' < generated/LS_COLORS)"
```

Fish:

```fish
vieza generate; and set -gx LS_COLORS (string trim < generated/LS_COLORS)
```

Nushell:

```nu
vieza generate; if $env.LAST_EXIT_CODE == 0 { $env.LS_COLORS = (open generated/LS_COLORS | str trim) }
```

PowerShell:

```powershell
vieza generate; if ($LASTEXITCODE -eq 0) { $env:LS_COLORS = (Get-Content -Raw generated/LS_COLORS).Trim() }
```

### 4. Export From Vivid Directly

Use this when you edited `generated/filetypes.yml` or `generated/vieza.yml` manually and only want vivid to render `LS_COLORS`.

Bash, Zsh, Ksh:

```sh
export LS_COLORS="$(vivid -d generated/filetypes.yml generate generated/vieza.yml)"
```

Fish:

```fish
set -gx LS_COLORS (vivid -d generated/filetypes.yml generate generated/vieza.yml | string trim)
```

Nushell:

```nu
$env.LS_COLORS = (vivid -d generated/filetypes.yml generate generated/vieza.yml | str trim)
```

PowerShell:

```powershell
$env:LS_COLORS = (vivid -d generated/filetypes.yml generate generated/vieza.yml).Trim()
```

Tcsh/Csh:

```csh
setenv LS_COLORS "`vivid -d generated/filetypes.yml generate generated/vieza.yml`"
```

### 5. Generate And Export From Vivid Directly

This skips reading `generated/LS_COLORS` and exports vivid's stdout instead.

Bash, Zsh, Ksh:

```sh
vieza generate && export LS_COLORS="$(vivid -d generated/filetypes.yml generate generated/vieza.yml)"
```

Fish:

```fish
vieza generate; and set -gx LS_COLORS (vivid -d generated/filetypes.yml generate generated/vieza.yml | string trim)
```

Nushell:

```nu
vieza generate; if $env.LAST_EXIT_CODE == 0 { $env.LS_COLORS = (vivid -d generated/filetypes.yml generate generated/vieza.yml | str trim) }
```

PowerShell:

```powershell
vieza generate; if ($LASTEXITCODE -eq 0) { $env:LS_COLORS = (vivid -d generated/filetypes.yml generate generated/vieza.yml).Trim() }
```

`filetypes.yml` is intended for vivid. It contains vivid's default `config/filetypes.yml` plus eza-derived filename, extension, and pattern entries. `vieza-filetypes.yml` contains only the eza-derived additions.

`vieza.yml` is generated directly from eza-like styles with adaptive ANSI colors. It also includes vivid-compatible stock categories so the combined filetypes database works with vivid.

Use local eza source instead of fetching:

```sh
vieza generate --source /path/to/eza/src/info/filetype.rs
```

Use local vivid filetypes source instead of fetching:

```sh
vieza generate --vivid-source /path/to/vivid/config/filetypes.yml
```

Generate from specific upstream refs. Defaults use latest known release tags for reproducible output:

- eza: `v0.23.4`
- vivid: `v0.11.1`

```sh
vieza generate --eza-ref main
vieza generate --vivid-ref master
```

## Notes

- `LS_COLORS` cannot represent all eza UI colors. This project targets filename/filetype coloring only.
- Colors are adaptive ANSI (`ansi:red`, `ansi:blue`, etc.), not fixed RGB.
- `AGENTS.md` records maintenance instructions for future upstream ref updates.

## Maintenance

Default upstream refs use release tags for reproducibility. Logic changes are only needed when eza or vivid changes source/schema semantics, not for every extension or filename added under existing categories. See `AGENTS.md` before bumping upstream refs.

## License

MIT. See `LICENSE`.
