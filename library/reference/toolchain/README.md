# Toolchain command reference

> **Availability:** current. **Authority:** derived reference.

This is the lookup reference for the current `ken` command surface. It records
commands, inputs, output, options, and observed process statuses. For an
ordered task or a repair after a refusal, follow the linked how-to instead.

| Command | Reference | Task guide |
|---|---|---|
| `run` | [Run](run.md) | [Run a Ken program](../../how-to/run-a-program.md) |
| `check` | [Check](check.md) | [Check a Ken source file](../../how-to/check-a-source.md) |
| `native-build` | [Native build](native-build.md) | [Build a native artifact](../../how-to/build-a-native-artifact.md) |
| `fmt` | [Format](fmt.md) | [Format a Ken source file](../../how-to/format-source.md) |
| `repl` | [REPL](repl.md) | [Use the REPL](../../how-to/use-the-repl.md) |
| `version` | [Version](version.md) | — |
| `help` | [Help](help.md) | — |

The option surface has five accepted spellings across three options:
`fmt --check`, `--version` / `-V`, and `--help` / `-h`. The
`native-build <output-dir>` value is positional, not an option.

