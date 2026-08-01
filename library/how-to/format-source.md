# Format a Ken source file

> **Availability:** current. **Authority:** how-to.

Build the toolchain first by following the
[Quickstart](../quickstart.md#1-install-and-use-the-current-toolchain). To try
the formatter without changing a tracked file, copy a known non-canonical
input to `/tmp`:

```console
$ cp conformance/challenge/C1-deceq-noncanonical/unsound-deceq-decimal.ken /tmp/ken-fmt-demo.ken
$ target/debug/ken fmt --check /tmp/ken-fmt-demo.ken
ken fmt --check: non-canonical: /tmp/ken-fmt-demo.ken
```

Run `fmt` without `--check`, then verify the result:

```console
$ target/debug/ken fmt /tmp/ken-fmt-demo.ken
$ target/debug/ken fmt --check /tmp/ken-fmt-demo.ken
```

Both remedy commands exit with status 0 and print nothing. Apply the same pair
to a path you own: first format it, then use `--check` to confirm that no
further rewrite is needed.

For the shorter check-only workflow, see
[Format it](../quickstart.md#3-format-it).
