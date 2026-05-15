# vieza

Generate vivid theme/database files that mimic eza filename colors, then use vivid to produce `LS_COLORS`.

## Install

From source:

```sh
cargo install --git https://github.com/vufly/vieza
```

After a release is published to crates.io:

```sh
cargo install vieza
```

## Usage

```sh
cargo run -- generate
```

Outputs:

- `generated/vieza.yml`
- `generated/vieza-filetypes.yml`
- `generated/filetypes.yml`
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
cargo run --quiet -- generate && export LS_COLORS="$(tr -d '\n' < generated/LS_COLORS)"
```

Fish:

```fish
cargo run --quiet -- generate; and set -gx LS_COLORS (string trim < generated/LS_COLORS)
```

Nushell:

```nu
cargo run --quiet -- generate; if $env.LAST_EXIT_CODE == 0 { $env.LS_COLORS = (open generated/LS_COLORS | str trim) }
```

PowerShell:

```powershell
cargo run --quiet -- generate; if ($LASTEXITCODE -eq 0) { $env:LS_COLORS = (Get-Content -Raw generated/LS_COLORS).Trim() }
```

Regenerate `LS_COLORS` manually with vivid:

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

Generate files and set from vivid output in one command:

Bash, Zsh, Ksh:

```sh
cargo run --quiet -- generate && export LS_COLORS="$(vivid -d generated/filetypes.yml generate generated/vieza.yml)"
```

Fish:

```fish
cargo run --quiet -- generate; and set -gx LS_COLORS (vivid -d generated/filetypes.yml generate generated/vieza.yml | string trim)
```

Nushell:

```nu
cargo run --quiet -- generate; if $env.LAST_EXIT_CODE == 0 { $env.LS_COLORS = (vivid -d generated/filetypes.yml generate generated/vieza.yml | str trim) }
```

PowerShell:

```powershell
cargo run --quiet -- generate; if ($LASTEXITCODE -eq 0) { $env:LS_COLORS = (vivid -d generated/filetypes.yml generate generated/vieza.yml).Trim() }
```

`filetypes.yml` is intended for vivid. It contains vivid's default `config/filetypes.yml` plus eza-derived filename, extension, and pattern entries. `vieza-filetypes.yml` contains only the eza-derived additions.

`vieza.yml` is generated directly from eza-like styles with adaptive ANSI colors. It also includes vivid-compatible stock categories so the combined filetypes database works with vivid.

Use local eza source instead of fetching:

```sh
cargo run -- generate --source /path/to/eza/src/info/filetype.rs
```

Use local vivid filetypes source instead of fetching:

```sh
cargo run -- generate --vivid-source /path/to/vivid/config/filetypes.yml
```

Generate from specific upstream refs. Defaults use latest known release tags for reproducible output:

- eza: `v0.23.4`
- vivid: `v0.11.1`

```sh
cargo run -- generate --eza-ref main
cargo run -- generate --vivid-ref master
```

## Notes

- `LS_COLORS` cannot represent all eza UI colors. This project targets filename/filetype coloring only.
- Colors are adaptive ANSI (`ansi:red`, `ansi:blue`, etc.), not fixed RGB.
- `AGENTS.md` records maintenance instructions for future upstream ref updates.

## Maintenance

Default upstream refs use release tags for reproducibility. Logic changes are only needed when eza or vivid changes source/schema semantics, not for every extension or filename added under existing categories. See `AGENTS.md` before bumping upstream refs.

## License

MIT. See `LICENSE`.
