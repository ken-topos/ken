# `ken native-build`

> **Availability:** current. **Authority:** derived reference.

## Synopsis

```text
ken native-build <file> <output-dir>
```

`native-build` checks an admitted Program I entrypoint and emits a native
artifact. `<output-dir>` is a required positional argument, not an option.

## Output and status

On success, stdout contains the artifact path and the command exits 0:

```console
$ target/debug/ken native-build library/guide/decomposition-abstraction.ken.md target/doc-w4-toolchain/native-ok
target/doc-w4-toolchain/native-ok/ken-starter
```

A pure source without `main` exits 1:

```console
$ target/debug/ken native-build catalog/packages/Core/Logic/Transport.ken.md target/doc-w4-toolchain/native-fail
ken native-build: program admission failed: MissingMain
```

For the build-and-run procedure and remedy, see
[Build a native artifact](../../how-to/build-a-native-artifact.md).

