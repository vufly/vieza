# eza-vivid

Generate vivid and raw `LS_COLORS` output that mimics eza filename colors while using the terminal ANSI palette for dark/light adaptation.

## Usage

```sh
cargo run -- generate
```

Outputs:

- `generated/eza-adaptive.yml`
- `generated/filetypes-eza.yml`
- `generated/filetypes-vivid-eza.yml`
- `generated/LS_COLORS`

## Shell Setup

Generate files first:

```sh
cargo run -- generate
```

Set `LS_COLORS` from `generated/LS_COLORS` for current shell session:

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

Generate and set in one command:

Bash, Zsh, Ksh:

```sh
export LS_COLORS="$(cargo run --quiet -- generate --print-ls-colors)"
```

Fish:

```fish
set -gx LS_COLORS (cargo run --quiet -- generate --print-ls-colors | string trim)
```

Nushell:

```nu
$env.LS_COLORS = (cargo run --quiet -- generate --print-ls-colors | str trim)
```

PowerShell:

```powershell
$env:LS_COLORS = (cargo run --quiet -- generate --print-ls-colors).Trim()
```

Use vivid instead of raw `generated/LS_COLORS`:

Bash, Zsh, Ksh:

```sh
export LS_COLORS="$(vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml)"
```

Fish:

```fish
set -gx LS_COLORS (vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml | string trim)
```

Nushell:

```nu
$env.LS_COLORS = (vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml | str trim)
```

PowerShell:

```powershell
$env:LS_COLORS = (vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml).Trim()
```

Tcsh/Csh:

```csh
setenv LS_COLORS "`vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml`"
```

Generate files and set with vivid in one command:

Bash, Zsh, Ksh:

```sh
cargo run --quiet -- generate && export LS_COLORS="$(vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml)"
```

Fish:

```fish
cargo run --quiet -- generate; and set -gx LS_COLORS (vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml | string trim)
```

Nushell:

```nu
cargo run --quiet -- generate; if $env.LAST_EXIT_CODE == 0 { $env.LS_COLORS = (vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml | str trim) }
```

PowerShell:

```powershell
cargo run --quiet -- generate; if ($LASTEXITCODE -eq 0) { $env:LS_COLORS = (vivid -d generated/filetypes-vivid-eza.yml generate generated/eza-adaptive.yml).Trim() }
```

`filetypes-vivid-eza.yml` is intended for vivid. It contains vivid's default `config/filetypes.yml` plus eza-derived filename, extension, and pattern entries. `filetypes-eza.yml` contains only the eza-derived additions.

`eza-adaptive.yml` is based on vivid's `themes/ansi.yml` with an extra eza section. This keeps all stock vivid nested categories valid while adding eza-derived file groups.

Use local eza source instead of fetching:

```sh
cargo run -- generate --source /path/to/eza/src/info/filetype.rs
```

Use local vivid filetypes source instead of fetching:

```sh
cargo run -- generate --vivid-source /path/to/vivid/config/filetypes.yml
```

Use local vivid ANSI theme source instead of fetching:

```sh
cargo run -- generate --vivid-theme-source /path/to/vivid/themes/ansi.yml
```

Generate from a specific eza ref. Default is pinned to `eed27ed05e74542af5852aed40e3dbff87d69c43` for reproducible output.

```sh
cargo run -- generate --eza-ref main
```

## Notes

- `LS_COLORS` cannot represent all eza UI colors. This project targets filename/filetype coloring only.
- Colors are adaptive ANSI (`ansi:red`, `ansi:blue`, etc.), not fixed RGB.
- `PLAN.md` records implementation direction and later work.
