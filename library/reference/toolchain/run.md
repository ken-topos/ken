# `ken run`

> **Availability:** current. **Authority:** derived reference.

## Synopsis

```text
ken run <file> [-- <program-arguments...>]
```

`run` elaborates an executable Ken source, drives its Console IO, writes the
program's output to the process streams, and returns the program's exit status.

## Inputs and output

- `<file>` is a Ken source or literate `.ken.md` source with one admitted
  `main` entrypoint.
- Arguments after `--` become program arguments rather than CLI arguments.
- Successful program output is written directly to stdout and stderr.

## Observed statuses

```console
$ target/debug/ken run library/guide/decomposition-abstraction.ken.md
decomposition guide ok
```

This exits 0. A probe program returning `Failure 37` prints nothing and makes
`ken run` exit 37, establishing program-status propagation. A source without a
`main` exits 1:

```console
$ target/debug/ken run catalog/packages/Core/Logic/Transport.ken.md
ken run: missing entrypoint 'main' in 'catalog/packages/Core/Logic/Transport.ken.md'
```

For the executable-file procedure and remedy, see
[Run a Ken program](../../how-to/run-a-program.md).
