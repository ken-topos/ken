# `ken version`

> **Availability:** current. **Authority:** derived reference.

## Accepted forms

```text
ken version
ken --version
ken -V
```

All three forms print the Ken version, kernel version, and interpreter
identity, then exit 0:

```console
$ target/debug/ken --version
ken 0.0.0 — verified topos-oriented language
kernel 0.0.0
ken reference interpreter (X1)
```

The output does not identify a compilation target. `--version` and `-V` are
accepted global option aliases, but the current `ken help` text omits them.

