# `ken fmt`

> **Availability:** current. **Authority:** derived reference.

## Synopsis

```text
ken fmt [--check] <paths...>
```

Without an option, `fmt` rewrites each path to canonical form. With
`--check`, it performs no rewrite and reports whether every path is already
canonical.

## Output and status

- A successful rewrite exits 0 with no output.
- `--check` on canonical input exits 0 with no output.
- `--check` on non-canonical input exits 1 and names the path:

```console
$ target/debug/ken fmt --check target/doc-w4-toolchain/format-me.ken
ken fmt --check: non-canonical: target/doc-w4-toolchain/format-me.ken
```

For the safe copy, rewrite, and verification procedure, see
[Format a Ken source file](../../how-to/format-source.md).
