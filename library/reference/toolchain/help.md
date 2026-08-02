# `ken help`

> **Availability:** partial. **Authority:** derived reference.

## Accepted forms

```text
ken help
ken --help
ken -h
```

All three forms print human-readable usage and the seven subcommands, then
exit 0. The output begins:

```console
$ target/debug/ken --help
ken 0.0.0 — verified topos-oriented language

Usage: ken <subcommand>

Subcommands:
  run <file>    Elaborate and run a Ken source file (Console IO)
  check <file>  Elaborate a Ken source file and verify its fences,
```

**Why partial:** the help text documents `fmt [--check]` but omits all four
accepted global option spellings: `--version`, `-V`, `--help`, and `-h`.
Those aliases work and are documented in this reference; the help omission is
a toolchain finding rather than a feature this page can repair.

