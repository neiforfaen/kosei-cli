# kosei

Regex-based environment switcher for any config file. Zero external dependencies — 1.6KB gzipped.

## Overview

It applies named sets of regex replacements across your config files. Instead of manually changing values in different config files, you define your environments once in `kosei.config.json` and switch between them with a single command.

Works on any text file — `.env`, JSON, YAML, TOML, XML, or anything else.

https://github.com/user-attachments/assets/826b198f-fcd3-4848-b0b4-d33ae9bcb3dd

## Installation

```sh
npm install -D kosei-cli
# or
pnpm add -D kosei-cli
```

## Quick start

Create a `kosei.config.json` at the root of your project:

```json
{
  "environments": {
    "dev": {
      "replacements": [
        {
          "files": [".env"],
          "regex": "/API_URL=.*/",
          "value": "API_URL=https://api.dev.example.com"
        }
      ]
    },
    "staging": {
      "replacements": [
        {
          "files": [".env"],
          "regex": "/API_URL=.*/",
          "value": "API_URL=https://api.staging.example.com"
        }
      ]
    }
  }
}
```

Then switch environments:

```sh
kosei switch dev
kosei switch staging
```

Or create a script in `package.json`:

```json
{
  "scripts": {
    "switch:dev": "kosei switch dev",
    "switch:staging": "kosei switch staging"
  }
}
```

## Config reference

### `kosei.config.json`

```json
{
  "environments": {
    "<name>": {
      "description": "optional human-readable label",
      "replacements": [
        {
          "files": ["relative/path/to/file", "another/file"],
          "regex": "/pattern/flags",
          "value": "replacement string"
        }
      ]
    }
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `environments` | object | yes | Map of environment names to their config |
| `environments.<name>.description` | string | no | Human-readable label for the environment |
| `environments.<name>.replacements` | array | yes | One or more replacement rules to apply |
| `replacements[].files` | string[] | yes | Paths to target files, relative to the config |
| `replacements[].regex` | string | yes | A regex literal as a string: `/pattern/flags` |
| `replacements[].value` | string | yes | The string to replace each match with |

### Regex format

The `regex` field must be written as a regex literal string — pattern wrapped in `/` with optional flags:

```
"/pattern/"
"/pattern/i"
"/pattern/gm"
```

Supported flags: `g`, `i`, `m`, `s`, `u`, `y`.

Capture group references (`$1`, `$2`, etc.) work in `value` as they do with JavaScript's `String.replace()`.

## Commands

### `kosei switch <env>`

Applies all replacements for the named environment.

```sh
kosei switch production
```

**Options:**

| Flag | Description |
|---|---|
| `--dry-run` | Preview changes without writing to disk |

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

kosei walks up the directory tree from `process.cwd()` until it finds a `kosei.config.json`. This means you can run it from any subdirectory of your project. Ideally the config file should live at the root of your project. If running in a monorepo setup, this makes it easier to setup different configs for different packages with a single file.

## Example: multiple files, multiple replacements

```json
{
  "environments": {
    "production": {
      "description": "Live production environment",
      "replacements": [
        {
          "files": [".env"],
          "regex": "/NODE_ENV=.*/",
          "value": "NODE_ENV=production"
        },
        {
          "files": [".env"],
          "regex": "/API_URL=.*/",
          "value": "API_URL=https://api.example.com"
        },
        {
          "files": ["config/database.yml"],
          "regex": "/host: .*/",
          "value": "host: db.example.com"
        }
      ]
    }
  }
}
```
