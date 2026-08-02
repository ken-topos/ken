# `ken repl`

> **Availability:** current. **Authority:** derived reference.

## Synopsis

```text
ken repl
```

`repl` starts the interactive Little Prover loop. The observed `:quit` command
ends the session.

## Output and status

A session that immediately quits prints the banner, prompt, and farewell, then
exits 0:

```console
$ printf ':quit\n' | target/debug/ken repl
ken 0.0.0 repl — :help for commands, :quit to exit
kernel 0.0.0
ken> bye
```

For session commands and a verified replacement after an unresolved name, see
[Use the REPL](../../how-to/use-the-repl.md).
