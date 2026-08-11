---
name: migrate
description: Replaces hand-copied installs of the gauntlet-loop skill with the plugin, keeping each one's enabled or disabled state. Use when the user has copied this skill into projects by hand and wants it managed by the plugin instead, or asks to clean up duplicate gauntlet-loop skills.
disable-model-invocation: true
license: CC-BY-4.0
---

# Migrate hand-copied installs

The migration tool ships with this plugin and is already on your PATH. Pick the
build for this machine:

| Platform | Command |
|---|---|
| macOS, Apple silicon | `gauntlet-loop-migrate-macos-arm64` |
| macOS, Intel | `gauntlet-loop-migrate-macos-x64` |
| Linux, x86_64 | `gauntlet-loop-migrate-linux-x64` |
| Linux, arm64 | `gauntlet-loop-migrate-linux-arm64` |
| Windows | `gauntlet-loop-migrate-windows-x64.exe` |

Run `uname -sm` to determine which.

## 1. Report

Run it with no arguments. It changes nothing.

Relay the output and explain what it found:

- `[ok]` entries will be replaced by a plugin install at the same scope, keeping
  their current enabled or disabled state
- `[skip]` entries contain files this project does not ship, so they are left
  alone for the user to review
- if it found nothing, `--write` installs the plugin anyway

Ask whether to proceed. Wait for an answer.

## 2. Apply

```
<binary> --write
```

Add `--scope project` if nothing was found and the user wants the plugin in the
current project rather than across their account.

Report what changed. The skill is invoked as `/gauntlet-loop:gauntlet-loop`
afterwards, and takes effect in a new session or after `/reload-plugins`.

`--json` emits the findings as structured data if you need to reason about them
rather than relay them.
