# Use the REPL

> **Availability:** current. **Authority:** how-to.

Build the toolchain first by following the
[Quickstart](../quickstart.md#1-install-and-use-the-current-toolchain), then
start the interactive loop:

```console
$ target/debug/ken repl
ken 0.0.0 repl — :help for commands, :quit to exit
kernel 0.0.0
ken>
```

## If a name is unresolved

Entering a name that is not defined in the session produces an error but keeps
the session open:

```console
ken> Missing
  error: unresolved type 'Missing' at 0-7
ken>
```

Use names already in scope, or define a declaration before referring to it.
This verified replacement evaluates an existing primitive application:

```console
ken> :eval add_int 2 3
  5 : g6
ken> :quit
bye
```

For the surface forms to use in declarations, consult the checked
[surface reference](../guide/surface-reference.ken.md) and return here for the
interactive commands.
