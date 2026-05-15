# Maintenance Instructions

## Upstream Refs

Default upstream refs should point to release tags, not moving branches:

- eza: use latest stable release tag, e.g. `v0.23.4`
- vivid: use latest stable release tag, e.g. `v0.11.1`

Users can still opt into moving refs with `--eza-ref main` or `--vivid-ref master`.

## When Code Changes Are Needed

Do not assume every upstream release needs logic changes. Code changes are needed when:

- eza changes `src/info/filetype.rs` away from `phf_map! { "ext" => FileType::Category }` syntax.
- eza adds/removes `FileType` enum variants.
- eza changes default filename/filekind styles in `src/theme/default_theme.rs`.
- eza changes dynamic matching in `FileType::get_file_type`, such as the current `readme*`, `*~`, and `#*#` rules.
- vivid changes `config/filetypes.yml` schema.
- vivid changes theme YAML schema or `vivid generate` CLI flags.
- vivid adds top-level categories not covered by `render_vivid_theme`.

Code changes usually are not needed when:

- eza adds exact filenames under existing categories.
- eza adds extensions under existing categories.
- vivid adds filetype entries under category paths already covered by `vieza.yml`.

## Updating Defaults

Before bumping `DEFAULT_EZA_REF` or `DEFAULT_VIVID_REF`:

1. Check latest upstream release tags with `gh release view --repo eza-community/eza` and `gh release view --repo sharkdp/vivid`.
2. Update refs in `src/main.rs`.
3. Run `cargo run --quiet -- generate`.
4. Verify `vivid -d generated/filetypes.yml generate generated/vieza.yml` succeeds.
5. Run `cargo fmt`, `cargo test`, and `cargo check`.
6. Inspect generated `LS_COLORS` for key rules:
   - `di=1;34`
   - `*README.md=1;4;33`
   - source files like `*.rs=1;33`

## Known Limitations

- `LS_COLORS` cannot fully encode eza's prefix rule `readme*`, so `vieza` hardcodes common README filename variants.
- `LS_COLORS` cannot fully encode eza's `starts_with('#') && ends_with('#')` temp-file rule, so `vieza` uses best-effort suffix matching.
- `LS_COLORS` cannot encode eza's context-sensitive compiled-artifact detection based on sibling source files.
