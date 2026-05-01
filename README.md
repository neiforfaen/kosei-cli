# kosei

Regex-based environment switcher for any config file.

**v2.0 — Rust rewrite.** For migration instructions from v1.x (TypeScript), see [Upgrading from v1.x](#upgrading-from-v1x) below.

## Overview

Define named sets of regex replacements across your config files once in `kosei.yaml`, then switch between environments with a single command.

Works on any text file — `.env`, JSON, YAML, TOML, XML, or anything else.

https://github.com/user-attachments/assets/826b198f-fcd3-4848-b0b4-d33ae9bcb3dd

## Installation

### pnpm (recommended)

```sh
pnpm install -D kosei-cli
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/neiforfaen/kosei-cli/releases).

### From source

```sh
cargo build --release
```

## Quick start

Create a `kosei.yaml` at the root of your project:

```yaml
environments:
  dev:
    replacements:
      - files:
          - .env
        regex: /API_URL=.*/
        value: API_URL=https://api.dev.example.com

  staging:
    replacements:
      - files:
          - .env
        regex: /API_URL=.*/
        value: API_URL=https://api.staging.example.com
```

Then switch environments:

```sh
kosei switch dev
kosei switch staging
```

## Config reference

### `kosei.yaml`

```yaml
environments:
  <name>:
    description: optional human-readable label
    replacements:
      - files:
          - relative/path/to/file
          - another/file
        regex: /pattern/flags
        value: replacement string
```

| Field                              | Type     | Required | Description                                                                                 |
| ---------------------------------- | -------- | -------- | ------------------------------------------------------------------------------------------- |
| `environments`                     | object   | yes      | Map of environment names to their config                                                    |
| `environments.<name>.description`  | string   | no       | Human-readable label for the environment                                                    |
| `environments.<name>.replacements` | array    | yes      | One or more replacement rules to apply                                                      |
| `replacements[].files`             | string[] | yes      | Literal file paths, relative to config file. Paths are matched exactly (no glob expansion). |
| `replacements[].regex`             | string   | yes      | A regex literal as a string: `/pattern/flags`                                               |
| `replacements[].value`             | string   | yes      | The string to replace each match with                                                       |

### Regex format

The `regex` field must be written as a regex literal string — pattern wrapped in `/` with optional flags:

```
/pattern/
/pattern/i
/pattern/gm
```

**Supported flags:**

- `i` — case-insensitive matching
- `m` — multiline mode (^ and $ match line boundaries)
- `s` — dot matches newlines
- `g`, `u`, `y` — accepted for backward compatibility with v1.x configs, but have no effect in v2.0

Capture group references (`$1`, `$2`, etc.) work in `value`.

## Commands

### `kosei init [path]`

Scaffolds a new `kosei.yaml` in the given directory (defaults to the current directory):

```sh
kosei init
kosei init path/to/project
```

Errors if a `kosei.yaml` already exists at that location.

### `kosei migrate`

Converts a legacy `kosei.config.json` to `kosei.yaml`:

```sh
kosei migrate
```

Walks up the directory tree from the current working directory to find `kosei.config.json`, converts it to `kosei.yaml`, and deletes the original JSON file.

### `kosei switch <env>`

Applies all replacements for the named environment.

```sh
kosei switch production
```

**Options:**

| Flag              | Description                             |
| ----------------- | --------------------------------------- |
| `-d`, `--dry-run` | Preview changes without writing to disk |

### `--dry-run`

Shows a line-level diff of what would change, without modifying any files:

```sh
kosei switch staging --dry-run
```

```
.env:
  - API_URL=https://api.example.com
  + API_URL=https://staging.api.example.com
```

## Config file resolution

kosei walks up the directory tree from the current working directory until it finds a `kosei.yaml`. This means you can run it from any subdirectory of your project.

## Example: multiple files, multiple replacements

```yaml
environments:
  production:
    description: Live production environment
    replacements:
      - files:
          - .env
        regex: /NODE_ENV=.*/
        value: NODE_ENV=production

      - files:
          - .env
        regex: /API_URL=.*/
        value: API_URL=https://api.example.com

      - files:
          - config/database.yml
        regex: /host: .*/
        value: "host: db.example.com"
```

## Upgrading from v1.x

kosei v2.0 is a complete rewrite in Rust with the same command-line interface and behavior. **All functionality is backwards compatible**, but the config file format has changed.

### Config file format

**v1.x used JSON** (`kosei.config.json`):

```json
{
  "environments": {
    "dev": {
      "replacements": [
        {
          "files": [".env"],
          "regex": "/API_URL=.*/",
          "value": "API_URL=http://localhost"
        }
      ]
    }
  }
}
```

**v2.0 uses YAML** (`kosei.yaml`):

```yaml
environments:
  dev:
    replacements:
      - files:
          - .env
        regex: /API_URL=.*/
        value: API_URL=http://localhost
```

### Migration steps

#### Automated

1. **Run the migration command:**

   ```sh
   kosei migrate
   ```

   This will:
   - Find `kosei.config.json` in your project (walking up the directory tree)
   - Convert it to `kosei.yaml` with the new format
   - Delete the original `kosei.config.json`

2. **Review changes:**
   - Check the generated `kosei.yaml` for correctness
   - Run `kosei switch <environment> --dry-run` to preview changes

#### Manual

1. **Rename and convert config file:**
   - Rename `kosei.config.json` → `kosei.yaml`
   - Convert JSON to YAML (see example above)

2. **Update regex syntax (if needed):**
   - v1.x: `"regex": "/pattern/flags"` (string)
   - v2.0: `regex: /pattern/flags` (unquoted, but quoted works too)
   - **Important:** Literal braces must be escaped: `/{}/` → `/\{\}/` (to match literal `{}`, `{`, or `}`)

3. **Verify and test:**

   ```sh
   # Preview changes without modifying files
   kosei switch <environment> --dry-run

   # Apply when ready
   kosei switch <environment>
   ```

### What's the same

- All regex patterns work identically
- Capture group references (`$1`, `$2`, etc.) work the same way
- File paths are relative to the config file location
- Config file discovery (walks up directory tree) is unchanged
- `--dry-run` flag behavior is identical
- All flags (`i`, `m`, `s`, `g`, `u`, `y`) are supported

### What's improved

- **Faster:** Rewritten in Rust for near-instant execution
- **Better errors:** Validation errors identify the environment name and replacement index where the problem occurred
- **Safer:** No partial edits — all replacements succeed or all fail
- **Easier to read:** YAML format is more human-friendly than JSON
