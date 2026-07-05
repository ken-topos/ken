---
scope: build/implementers
audience: (see scope README)
source: private memory `eval-store-resync-recurring-trap`
---

# A scratch eval harness's stale store is often the bug, not the interpreter

On `wp/VAL2-rosetta-pangram` (2026-07-03) I hit this SAME trap twice in one
session: a scratch `ken_elaborator`/`ken_interp` test harness produced
`EvalVal::Neutral` (or Ctors full of `Neutral`/`CtorPending` holes) for a
computation that should have reduced to a concrete value — each time I spent
real time suspecting a genuine interpreter/algorithm bug before tracing it back
to my own harness.

**Root cause both times:** each `elaborate_decl` call registers any NEW
numeric/string literals it introduces into `env.num_values`, but
`store.num_values` is only a SNAPSHOT taken when the `EvalStore` was built (or
last resynced). If you call `elaborate_decl` again AFTER building the store
(e.g. to declare a fresh probe view referencing a new literal), that literal's
id is missing from `store.num_values`, so it evaluates to `Neutral` — a
completely silent, no-error failure mode that looks exactly like "the interp is
stuck" rather than "my test harness is stale." The analogous trap exists for
`store.list_char_ids`: if it's never set (or the harness is built before
`String`-op-touching decls exist), `string_to_list_char`/`list_char_to_string`
degrade to `Neutral` too (`ken-interp/src/eval.rs:1032-1040`, by design — "never
silently wrong," but silent to a test author who doesn't know to check).

**How to apply:** the moment a scratch probe returns `Neutral`/
`CtorPending`/garbage instead of the expected value, check the harness's OWN
store-population discipline BEFORE suspecting the interpreter or the `.ken`
source:
1. Is `store.num_values` resynced (`env.num_values` re-scanned into
   `store.num_values`, using `.entry(...).or_insert_with(...)` so it's
   idempotent) after EVERY `elaborate_decl` call that might introduce a fresh
   literal — not just once at store-creation time?
2. Is `store.list_char_ids` actually set (`Some(ListCharIds{nil_id, cons_id})`
   from `env.prelude_env`) if the code under test touches `String`/`List Char`
   conversions at all?

The established, correct pattern lives in
`crates/ken-elaborator/tests/l3_strings_roundtrip_acceptance.rs`'s `eval_view`
helper — copy it wholesale into any new scratch harness rather than re-deriving
a leaner version, since the leaner version is exactly what drops the resync
step. This also burned real `ken-cli` production time: the identical
missing-resync shape (never wired at all, not just stale) was `ken-cli`'s actual
`list_char_ids`-unwired bug (VAL2 finding #7) — the SAME class of omission, just
in shipped code instead of a scratch test.
