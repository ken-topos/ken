# Read a file line by line

Read a text file and process it line by line.

Reference: <https://rosettacode.org/wiki/Read_a_file_line_by_line>

## Status

**PASS.** `main` declares `Cap APartial` on its own signature (the manifest
IS the signature); `ken run` reads that declaration, mints exactly that
authority (never full, never ambient), and binds it before running the
program. `read_bytes` carries the cap into a real `[FS]` effect
computation, gated by the runtime `authorizes` check
(`ken-interp/src/eval.rs`) before any syscall. Reads the checked-in
hermetic fixture `conformance/fs/fixtures/three-lines.txt`, splits it into
lines (`str::lines()` terminator semantics — a trailing `\n` yields no
trailing empty line), and prints each line via `[Console]` from *inside*
the program.

**Genuine effect composition (`effect-composition` D1–D4 — retires the
prior honesty asterisk).** `main` performs the `[FS]` read *and* the
`[Console]` printing in **one** type-checked
`ITree (Sum (FSOp APartial) ConsoleOp) (resp_sum ...) (Result IOError Unit)`,
built at the surface via `injectL`/`injectR` (the general `g ↪ Sum g h` /
`h ↪ Sum g h` inclusions) sequenced with the ordinary homogeneous `bind` —
no hand-fed coproduct anywhere. `run_io`'s coproduct-aware terminal driver
strips the `InL`/`InR` tags and dispatches both effects through the same
loop. What remains deferred (the honest residual, not a gap in this
example): the row-directed auto-injection sugar (`visits [FS, Console]`
inserting `injectL`/`injectR` automatically) is not yet landed — this
example uses the explicit, `injectL`/`injectR`-named door, the floor under
that future sugar. On a denied/insufficient capability or a missing file,
`main` does **not** print (fail-closed) and returns `Err e`, which
`ken-cli` still surfaces as a non-zero exit with the exact `IOError`
variant named on stderr — never a false success.
