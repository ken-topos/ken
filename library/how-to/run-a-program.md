# Run a Ken program

> **Availability:** current. **Authority:** how-to.

Build the toolchain first by following the
[Quickstart](../quickstart.md#1-install-and-use-the-current-toolchain). Then run
a source file that declares an executable `main`:

```console
$ target/debug/ken run library/guide/decomposition-abstraction.ken.md
decomposition guide ok
```

## If the file has no `main`

Running a pure library entry reproduces this refusal:

```console
$ target/debug/ken run catalog/packages/Core/Logic/Transport.ken.md
ken run: missing entrypoint 'main' in 'catalog/packages/Core/Logic/Transport.ken.md'
```

Choose a file with an executable entrypoint. The guide used above has one in
its final fence; run that file again and expect `decomposition guide ok`.

For the difference between checking and driving IO, follow
[Check and run one program](../quickstart.md#2-check-and-run-one-program).
