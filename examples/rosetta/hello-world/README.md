# Hello world

Print "Hello, World!" to standard output.

Reference: <https://rosettacode.org/wiki/Hello_world/Text>

## Status

**PARTIALLY implemented — string literal elaborates; I/O blocked by one gap.**

## GAP: string literals — FIXED (`37 §2.1`, VAL1-surface)

`Token::Str` (double-quoted strings) is now parsed in `parse_atom_expr`
and wired through resolve/elab/eval. `"Hello, World!"` elaborates to type
`String` and evaluates to `EvalVal::Str("Hello, World!")`. The current
`hello-world.ken` uses the literal directly:

```ken
view main : String = "Hello, World!"
```

## GAP: no runtime print / I/O surface

There is no `print_line` or equivalent function accessible in a `.ken` program.
The Console effect exists as an ITree construct in the elaborator/interp tests,
but has no surface-visible name.

Required fix: expose `print_line : String -> IO Unit` as a `foreign` postulate
wired to the ITree Console effect. This is tracked as a separate gap and is
pending `wp/VAL1-console-exec` (runtime-leader's work).

## Intended program (once GAP-io-surface is closed)

```ken
foreign print_line : String -> IO Unit = "puts" "libc" [Console]
view main : IO Unit = print_line "Hello, World!"
```
