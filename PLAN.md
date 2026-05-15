# eza-vivid Plan

## Goal

Generate an `LS_COLORS` setup that mimics eza filename coloring while staying usable across dark and light terminal themes.

## Decisions

- Output both a vivid theme/database pair and raw `LS_COLORS`.
- Prefer adaptive ANSI colors over fixed RGB/256 colors.
- Match eza filename and extension colors, not eza UI columns.
- Keep project as standalone Rust CLI in this directory.
- Pin upstream eza source by default so generated output is reproducible.

## Source Of Truth

- `src/theme/default_theme.rs` defines eza default file kind and file category styles.
- `src/info/filetype.rs` defines eza filename and extension category matching.
- `LS_COLORS` can represent file kinds plus glob/extension mappings.
- vivid can keep color/style names readable and generate shell-ready `LS_COLORS`.

## Mapping Strategy

- Core file kinds map directly to LS_COLORS keys:
  - `di`, `ex`, `ln`, `pi`, `so`, `bd`, `cd`, `or`, `fi`
- eza file categories map to vivid groups:
  - image, video, music, lossless, crypto, document, compressed, temp, compiled, build, source
- eza filename mappings become exact filename patterns in vivid filetypes database.
- eza extension mappings become `.<ext>` entries in vivid filetypes database.

## Adaptive Palette

Use terminal ANSI palette via vivid `ansi:*` values:

- directory: blue bold
- executable: green bold
- symlink: cyan
- pipe/special: yellow
- block/char device: yellow bold
- socket/broken link: red bold or red
- image/video: magenta, video bold
- music/lossless: cyan, lossless bold
- crypto/document: green, crypto bold
- compressed: red
- temp: faint
- compiled/source/build: yellow, source/build bold, build underline

## CLI Shape

- `eza-vivid generate`: write generated files.
- `--out-dir <dir>`: output directory, default `generated`.
- `--eza-ref <ref>`: upstream eza ref, default `eed27ed05e74542af5852aed40e3dbff87d69c43`.
- `--source <path>`: parse local eza `filetype.rs` instead of fetching.
- `--vivid-source <path>`: use local vivid `config/filetypes.yml` instead of fetching.
- `--vivid-theme-source <path>`: use local vivid `themes/ansi.yml` instead of fetching.
- `--print-ls-colors`: print raw `LS_COLORS` to stdout.

## Outputs

- `generated/eza-adaptive.yml`: vivid ANSI theme plus eza-derived style section.
- `generated/filetypes-eza.yml`: eza-derived vivid filetype additions only.
- `generated/filetypes-vivid-eza.yml`: vivid default database plus eza-derived additions.
- `generated/LS_COLORS`: direct shell-ready output.

## Verification

- Unit tests parse representative eza filename and extension mappings.
- Unit tests verify style serialization for core categories.
- `cargo fmt` and `cargo test` must pass.

## Later Work

- Add `EZA_COLORS` output for full eza UI column matching.
- Add sample preview directory and screenshot helper.
- Add dotfiles/chezmoi integration if wanted.
