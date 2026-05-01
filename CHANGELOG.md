# kosei-cli

## 2.0.0

### Major Changes

- Rewritten in Rust. The binary is still distributed via `npm install -D kosei-cli`.
- Config file format changed from `kosei.config.json` (JSON) to `kosei.yaml` (YAML).
- Added `kosei migrate` command to automatically convert `kosei.config.json` to `kosei.yaml`.
- Added `kosei init` command to scaffold a new `kosei.yaml` in a given directory.

## 1.1.1

### Patch Changes

- 64f835c: fix bug where successful switch is logged per number of files

## 1.1.0

### Minor Changes

- b549a08: Force replacement to use user defined regex flags, otherwise default to global flag
- b549a08: Log successful switches to the console
