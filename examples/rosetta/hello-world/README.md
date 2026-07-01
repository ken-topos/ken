# Hello world

Print "Hello, World!" to standard output.

Reference: <https://rosettacode.org/wiki/Hello_world/Text>

## Status

**BLOCKED — two surface gaps.**

## GAP: string literals not in expression grammar

`Token::Str` (double-quoted strings) is lexed but only parsed inside `foreign`
declarations (for symbol/library names). The `parse_atom_expr` function in
`parser.rs` has no `Token::Str` branch; `"Hello, World!"` as a surface
expression produces a parse error.

Required fix: add `Token::Str(s) => Ok(Expr::EStr(s, span))` to
`parse_atom_expr` and wire the resulting AST node through resolve and elab to
`Term::Const` of a `String`-typed postulate or to `EvalVal::Str`.

## GAP: no runtime print / I/O surface

There is no `print`, `println`, or equivalent function accessible in a `.ken`
program. The Console effect exists as an ITree mock in the elaborator/interp
tests, but has no surface-visible name. Even once string literals land, there is
no way to produce console output.

Required fix: expose a `print_line : String -> IO Unit` (or equivalent) as
either a `foreign` postulate wired to the ITree Console effect, or a named
primitive registered in the prelude.

## Current placeholder

`view main : Nat = Zero` — elaborates and returns `Zero`; confirms the
elaborator path is reachable. Replace when both gaps above are closed.

## Intended program

```ken
foreign print_line : String -> IO Unit = "puts" "libc" [Console]
view main : IO Unit = print_line "Hello, World!"
```
