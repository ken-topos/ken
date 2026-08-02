# `ken repl`

> **Availability:** current. **Authority:** derived reference.

## Synopsis

```text
ken repl
```

`repl` starts the interactive Little Prover loop. It reads declarations,
expressions, and colon commands until `:quit` or end of input.

## Output and status

A session that immediately quits prints the banner, prompt, and farewell, then
exits 0:

```console
$ printf ':quit\n' | target/debug/ken repl
ken 0.0.0 repl — :help for commands, :quit to exit
kernel 0.0.0
ken> bye
```

Evaluation and elaboration errors are reported inside the session while the
loop remains active. For session commands and a verified replacement after an
unresolved name, see [Use the REPL](../../how-to/use-the-repl.md).

