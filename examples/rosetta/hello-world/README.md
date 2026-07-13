# Hello world

Print "Hello, World!" to standard output.

Reference: <https://rosettacode.org/wiki/Hello_world/Text>

## Status

**Working, end to end.** `ken run examples/rosetta/hello-world/hello-world.ken`
prints `Hello, World!` and exits 0 — confirmed after
`wp/console-harvest-fix` (`6701b29`) landed the Console-ID harvest fix in
`ken-cli` (the two gaps below are both closed).

## History (both gaps now closed)

- **String literals** (`37 §2.1`, VAL1-surface): `Token::Str` parses and
  elaborates to `String`/`EvalVal::Str`.
- **`print_line : String -> IO Unit`**: exposed as a derived prelude `view`
  (`ken-elaborator/src/prelude.rs`), reducing to
  `Vis Unit (Write s) (\_. Ret Unit MkUnit)`. `ken run` now correctly
  harvests the Console IDs (`wp/console-harvest-fix`) and drives the
  resulting `ITree` through `run_io`, printing to real stdout.

```ken
proc main (_input : ProcessInput) (_caps : ProgramCaps)
  : HostIO ExitCode visits [Console] =
  host_program (print_line "Hello, World!")
```
