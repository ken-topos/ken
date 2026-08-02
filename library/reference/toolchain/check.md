# `ken check`

> **Availability:** current. **Authority:** derived reference.

## Synopsis

```text
ken check <file>
```

`check` elaborates a Ken source and verifies its literate fences without
driving IO.

## Output and status

A successful check exits 0 with no output:

```console
$ target/debug/ken check library/guide/decomposition-abstraction.ken.md
```

A path that cannot be read exits 1 and names the path:

```console
$ target/debug/ken check target/doc-w4-toolchain/does-not-exist.ken
ken check: cannot read 'target/doc-w4-toolchain/does-not-exist.ken': No such file or directory (os error 2)
```

For the checking procedure and remedy, see
[Check a Ken source file](../../how-to/check-a-source.md).

