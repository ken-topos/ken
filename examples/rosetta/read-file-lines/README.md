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
trailing empty line), and returns the line list as a total
`Result IOError (List String)`.

**Honesty boundary.** This example demonstrates **FS-read + pure-parse,
NOT effect composition.** `main`'s `[FS]` computation contains no `[Console]`
printing — the returned line list is rendered by the CLI (`ken-cli`'s
`run_file`) *after* the program's `ITree` finishes running, not from
within the Ken program itself. A single Ken program driving BOTH an `[FS]`
effect and `[Console]` printing in one type-checked `ITree` needs a
coproduct (`Sum`) that `run_io` does not support today — that is a
separate, deferred effect-composition frontier (Steward-tracked), not
demonstrated here. Failure (a denied/insufficient capability, or a missing
file) surfaces as a non-zero exit with the exact `IOError` variant named on
stderr — never a false success.
