# Build a native artifact

> **Availability:** current. **Authority:** how-to.

Build the toolchain first by following the
[Quickstart](../quickstart.md#1-install-and-use-the-current-toolchain). Give
`native-build` an executable source file and an output directory:

```console
$ target/debug/ken native-build library/guide/decomposition-abstraction.ken.md target/howto-native
target/howto-native/ken-starter
$ target/howto-native/ken-starter
decomposition guide ok
```

## If program admission reports `MissingMain`

A pure library entry has no executable entrypoint:

```console
$ target/debug/ken native-build catalog/packages/Core/Logic/Transport.ken.md target/howto-native
ken native-build: program admission failed: MissingMain
```

Choose a source file with an executable `main`, then rerun `native-build`. The
first command on this page is the verified remedy and prints the created
executable's path.

For the runnable guide used here, see
[Check and run one program](../quickstart.md#2-check-and-run-one-program).
