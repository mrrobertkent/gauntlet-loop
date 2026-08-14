# Changelog

## 1.1.0

The bar can now be the project's own spec, for work that has nothing to copy.

- New bars table entry, and a section on judging against a spec instead of an outside product.
- A spec variant of the prompt template, plus a worked example.
- A resolver script, so a project that keeps its specs in one place can point at that directory once in `.claude/gauntlet-loop.conf`. The config file belongs to the project, so plugin updates never touch it.
- The skill asks for the bar when it does not have one. It never searches the project for it.

The migration tool's source moved to `skills/migrate/scripts/`, beside the skill that uses it. The compiled binaries stay in `bin/`, which is what puts them on the Bash tool's `PATH`.

## 1.0.0

Initial plugin release.
