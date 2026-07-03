# CLI-string-wiring — `ken run` never wires `store.list_char_ids` (String ops → Neutral)

**Steward frame → Team Runtime. Small mechanical cross-lane fix — same shape,
same lane, same light gate as `console-harvest-fix`.** VAL2 finding #7. Owner:
**runtime-leader → runtime-implementer → runtime-qa.** Gate: **light** —
Runtime QA + CI, Integrator merges. No spec/Architect/CV (no `/spec`, no
kernel/`trusted_base` surface — CLI driver wiring only). Findings → **Steward**.

## Why
`crates/ken-cli/src/main.rs`'s `run_file` builds an `EvalStore` but never sets
`store.list_char_ids` — it stays `EvalStore::new()`'s default `None`. By design,
`string_to_list_char`/`list_char_to_string` degrade to `Neutral` when
`list_char_ids` is unset (`ken-interp/src/eval.rs:1032-1040` — "never silently
wrong"). So **any `ken run` on a program using String ops** produces `Neutral`
instead of the string: `palindrome` through the real CLI fails `unhandled
effect: Ctor { id: g87, args: [Neutral] }` instead of printing `"PASS"`.

Not caught by `console-harvest-fix` because `hello-world` (the only example run
through the fixed CLI so far) touches no String ops. In-process verification
(the implementer's harness, `list_char_ids` wired manually) already confirms
`palindrome`/`closures`/`merge-sort` produce **correct values** — this is purely
a `ken run`-path wiring gap, not a correctness issue in the examples or
`packages/collections`.

## Settled inputs — DO NOT REOPEN
- **Lane = Runtime.** `ken-cli`'s `run_file` is Runtime's file (the
  `console-harvest-fix` precedent). Value semantics are correct; the driver
  just under-wires the store.
- **The mechanical fix** (verify against landed code — *perishable*): in
  `run_file`, populate `store.list_char_ids` exactly as every acceptance test
  does, mirroring the `num_values` population already right above it:
  ```rust
  store.list_char_ids = Some(ken_interp::eval::ListCharIds {
      nil_id:  elab_env.prelude_env.nil_id,
      cons_id: elab_env.prelude_env.cons_id,
  });
  ```
- **Soundness-inert.** Kernel/`trusted_base` untouched; `/spec` unchanged. This
  makes the CLI driver match what the interpreter already expects — no behavior
  change to any *correctly-wired* path.

## Consider (Runtime's engineering call) — stop the class, don't just patch #7
This is the **second** `run_file` wiring gap VAL2 has surfaced (console IDs →
`console-harvest-fix`; now `list_char_ids`). Both are the same shape: `run_file`
constructs an `EvalStore` field-by-field and silently omits a field every test
wires manually. **Runtime's call** whether to (a) just add the 3 lines, or (b)
factor the store-setup that tests and `run_file` share into **one** helper so a
future store field can't be wired in tests but forgotten in the CLI driver
(subsume-don't-proliferate, `docs/PRINCIPLES.md`). Either is in-lane; (b) is the
durable fix if the shared setup is clean to factor. Not mandated — your judgment.

## Acceptance criteria
- **AC1 — Kernel untouched.** `git diff origin/main -- crates/ken-kernel/`
  empty; `trusted_base()` unchanged; the fix is `ken-cli` only (plus a
  test-shared helper if you take option (b)).
- **AC2 — `ken run` on a String-op program prints the string.** `palindrome`
  (or a minimal String-op `.ken`) through the real `ken run` CLI produces the
  correct printed value, not `Neutral`/`unhandled effect`. A regression test
  guards it (drive the CLI/`run_file` path, not just in-process eval — the whole
  point is the gap only shows at the CLI boundary).
- **AC3 — No regression.** `hello-world` and the existing CLI/console path stay
  green; `cargo test --workspace` green.

## Gate & sequencing
- **Gate:** light — Runtime QA + CI; Integrator merges. No spec/Architect/CV.
- **Lane:** Team Runtime (owns `ken-cli`). Branch off `origin/main`.
- **Sequencing — NON-URGENT, leader's call.** VAL2 is **not** fully blocked
  (`hello-world` verifies; String-op examples are confirmed correct in-process;
  `gcd` is separately parked on RTP1). This fix touches `ken-cli/main.rs`, **not**
  RTP1's `ken-interp/eval.rs`, so it can interleave without disturbing RTP1 D2 —
  but that's the leader's sequencing judgment; do not derail D2 for it. On land →
  the String-op VAL2 examples verify end-to-end through `ken run`.
