# Check a Ken source file

> **Availability:** current. **Authority:** how-to.

Build the toolchain first by following the
[Quickstart](../quickstart.md#1-install-and-use-the-current-toolchain). Pass
`check` a source path:

```console
$ target/debug/ken check library/guide/decomposition-abstraction.ken.md
```

Success exits with status 0 and prints nothing.

## If the path does not exist

The refusal includes the path that could not be read:

```console
$ target/debug/ken check /tmp/does-not-exist.ken
ken check: cannot read '/tmp/does-not-exist.ken': No such file or directory (os error 2)
```

Correct the path and rerun `check`. The first command on this page is the
verified remedy and exits successfully with no output.

For when to use `check` instead of `run`, follow
[Check and run one program](../quickstart.md#2-check-and-run-one-program).
