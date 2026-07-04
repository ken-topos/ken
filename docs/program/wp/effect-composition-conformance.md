# effect-composition — D5 conformance / acceptance plan (enclave deliverable)

**Author:** conformance-validator (CV). **Branch:**
`conformance-validator/effect-composition`, off `wp/effect-composition@aa45565`
(itself off `origin/main@43e97d02`, VAL2 16/0). **Frame:**
`docs/program/wp/effect-composition.md` (Steward). **Assignment:** spec-leader
`evt_3qmjknw1dntm7` — D5 drives **AC2, AC3, AC5, AC6, AC7**, plus the totality
**face** (AC4's formal cert is Architect's). AC1 (kernel-untouched) is
Architect's.

This is the executable contract the Runtime build's tests must satisfy. It
pins **what each conformance face asserts and why it discriminates** — every
face traced to a verdict/observable flip, no green-vs-green. It does **not**
author the mechanism (D1–D3 Architect, D2 surface spec-author); it grounds
each face against the **landed** `aa45565` substrate, not the frame's
perishable citations.

## 0. What D5 must pin

| Face | AC | Discriminates | Owner of the cert |
|---|---|---|---|
| No-hand-fed coproduct | AC7 | real surface producer vs. Rust-built tree | CV |
| Generality | AC3 | general mechanism vs. `Sum ConsoleOp (FSOp a)` special-case | CV (**load-bearing**) |
| Composed program runs | AC2 | in-program `[Console]` print vs. CLI post-render | CV |
| Totality (face) | AC4 | halts on multi-step continuation | CV face / Architect cert |
| State + FS unbroken | AC5 | subsume vs. fork | CV |
| Asterisk retired | AC6 | residual claim matches landed capability | CV |
| Kernel untouched | AC1 | — | Architect |

## 1. Substrate grounding (fixed inputs — `aa45565`)

Read from the landed tree, not the frame:

- **`run_io` is the CLI's sole top-level driver** (`crates/ken-cli/src/main.rs`
  `run_file` → `run_io(tree, &console_ids, fs_ids, …)`;
  `crates/ken-interp/src/eval.rs`). It dispatches **raw** op-tags on exactly two
  arms: Console `write_id` (`println!`) and FS `readfile_id` (`std::fs::read`
  behind the `authorizes` gate). `Ret r` → done; `Vis op k` → dispatch then
  `apply(k, resp)`; **`_ => UnknownEffect`** — exhaustive, fail-closed, no
  catch-all syscall. **Zero `Sum`/`InL`/`InR` awareness today:** a
  coproduct-wrapped op (`InL (Write s)`) never matches `write_id`/`readfile_id`
  → falls to `_ => UnknownEffect`. That is the fail-closed baseline this WP
  lifts.
- **`resp_sum` is State-left-pinned** (`effects/state.rs` `declare_resp_sum`):
  `Sum (StateOp s) f` — `resp_state` for `InL`, the passed-in `RespF` for `InR`.
  D1 must generalize to `Sum g h` given each summand's response family.
- **`run_state` is the only `Sum`-fold** (`effects/state.rs`
  `declare_run_state`): it interprets State (the `InL` summand) and passes the
  other summand **un-run**. **The CLI never invokes it.** ⇒ **today only
  Console + FS are top-level-runnable base effects.** (Load-bearing for AC3 —
  see §3, §6 SEAM-D3.)
- **`Sum a b = InL a | InR b`** (`effects/state.rs` `declare_sum`) — general,
  effect-agnostic, already exists. The gap is not the type; it is a general
  response family (D1), a lift into it (D2), and a coproduct-aware driver (D3).
- **`print_line s` → `Vis (Write s) (\_. Ret MkUnit)`** (`eval.rs`
  `build_print_line_tree`); Console `Write` carries a `Str`.
- **The current example** `examples/rosetta/read-file-lines/` — `main : (cap :
  Cap APartial) -> FS APartial (Result IOError (List String))` performs a pure
  `[FS]` read and returns the parsed lines; the **CLI renders the return
  value** (the Option-3 honesty asterisk). `expected` = `alpha\nbeta\ngamma\n`.

## 2. AC7 — the no-hand-fed-coproduct guard (drive the real producer)

The e2e must enter through **`ken-cli`'s `run_file`** reading a real `.ken`
example: the coproduct tree (`Sum`, `InL`/`InR`, the lift/`inject`) is built by
**elaborating surface Ken**, never hand-constructed in Rust and handed to
`run_io`. This is [[conformance-hand-feeds-the-deliverable]] — the exact trap
the frame names for D5, and the one that let the flip's `Result` field-order
lie ship latent (its driver tests never elaborated a real consumer).

**Producer-grep guard** (structural, no value-flip — asserts the *absence* of a
hand-feed): scan the e2e test file(s) for a hand-built coproduct at the
interpreter entry —

- `InL` / `InR` / `make_ctor(…inl…)` / `make_ctor(…inr…)` /
  `EvalVal::Ctor { id: inl_id` / `EvalVal::Ctor { id: inr_id` / a bare
  `sum_id` / any `run_io(` call on a locally-constructed tree.
- **Allowed:** these tokens may appear **only** inside doc-comment methodology
  prose (as the flip's AC3 allowed `EvalVal::Cap` to name-but-not-construct) —
  never as constructed values reaching `run_io`.
- **Required producer:** the CLI binary / `run_file` over
  `examples/rosetta/<compose>/*.ken`, exactly the rosetta harness path.

If the interpreter consumes a Rust-built `Sum` tree, the e2e attests the
*interpreter*, not the *surface→execution* path AC2/AC3 claim.

## 3. AC3 — the generality discriminator (the load-bearing net)

**Vacuity risk (state it plainly).** A single FS+Console example is
**green-vs-green** between the general mechanism and a hardcoded `Sum ConsoleOp
(FSOp a)` special-case — both print the three lines. Generality is a
**dimension the FS+Console example is blind to** (K2c-series-2: a case
discriminating on one dimension is vacuous on another; a multi-dimensional net
needs a discriminating case *per* dimension). AC3 exists precisely because
*subsume-don't-proliferate* is *why the flip deferred the bolt-on*; the net must
force the general dispatch, not a second special-case.

The discriminator's **executability is contracted by D3** (§6 SEAM-D3,
[[discriminator-negative-arm-must-be-expressible-and-reaching]]). Written to
both routes; collapses to the chosen one on Architect's ruling:

### Route (b) — executable second pairing (preferred; needs a 2nd runnable pair)

A **second distinct effect pairing** through the **same** machinery, such that a
`Sum ConsoleOp (FSOp a)`-hardcoded `run` **fails** it:

- **verdict-flip:** hardcoded-`run` → `UnknownEffect` / wrong dispatch on the
  new pair; general-`run` → correct byte-exact output. Opposite observables —
  not green-vs-green.
- Concretely, if D3 **subsumes `run_state`** (SEAM-D3 route 1): a `[State,
  Console]` program (mutate a counter through the composed tree, print it) or
  `[FS, State]`. The point is a pairing whose op-ids the flip's special-case
  never baked in — proving the machinery dispatches on `InL`/`InR` **structure
  + inner tag**, not on specific op-ids.

### Route (a) — structural, if D3 "coexists" (only Console+FS runnable)

The general `run` / `resp_sum` / `inject` are **quantified over the summand
families `g h`**:

- **structural grep:** **no** `ConsoleOp` / `FSOp` / `write_id` / `readfile_id`
  literal in the general combinator's type or dispatch shape; it peels
  `InL`/`InR` and recurses to each summand's own handler, selected
  structurally.
- **plus** a position-swap `Sum FSOp ConsoleOp` instantiation — the only
  executable variation available under this route.
- **Honesty residual (mandatory flag).** Route (a) alone does **not**
  discriminate a *set-hardcoded* adversarial special-case (one matching
  {Console,FS} in *either* order) — the position-swap passes it green-vs-green.
  I state this residual in the plan and in the example README, and name what
  closes it: route (b)'s genuinely-distinct pairing, which needs D3's
  subsumption. Over-claiming route (a) as full generality would be exactly the
  trust-level over-claim my lane guards against.

**Either route:** the AC3 face asserts opposite observables under the precise
"hardcoded to one pairing" bug — never both-succeed.

## 4. AC2 — the composed program + the exact expected oracle

The re-authored example (`read-file-lines` re-worked, or a dedicated
`compose-fs-console` — the filename is D4's choice, `(oracle: D4 example
path)`) **genuinely composes**: `main` performs an `[FS]` read **and**
`[Console]`-prints each parsed line, in **one** type-checked `ITree (Sum …) …`,
elaborated from surface Ken and executed via `run_file` → `run_io`
(D3-generalized).

- **Oracle:** `alpha\nbeta\ngamma\n` — **byte-identical** to today's `expected`,
  but now produced by genuine **in-program `[Console]` printing**, not the
  CLI's post-hoc render. Clean invariant: *same observable, real composition* —
  the flip is precisely that the print now comes from inside the program.
- **Face keys the observable to STDOUT** (the lines printed by the composed
  program), asserted byte-exact via the rosetta harness (`expected` file, exact
  match). It is **not** the CLI's return-value render — that is the Option-3
  path being retired.
- I pin the oracle **value + semantics**; the surface **spelling** of the
  composition is spec-author's D2 (`(oracle: D2 surface form)`, §6 SEAM-D2) —
  not over-frozen here (T1 don't-pin-tighter-than-the-spec-locks).

## 5. Totality face (AC4) + AC5 / AC6

**Totality face (AC4 — executable witness; formal fold-terminates cert =
Architect).** A composed program sequencing **multiple** effect ops through the
continuation — read, then print each of N lines (a fold over the continuation
resumption) — runs to completion and **halts** with the correct output. This
witnesses `run_io`'s loop making progress on each `Vis` and terminating at
`Ret` — the recursive/continuation-resumption case the frame names as the
stress. **Not a value-flip** (a total language always yields a value *if* it
halts); it is a **termination/does-it-halt** face — I flag that the structural
proof the coproduct fold terminates is **AC4, Architect's**; this is the
executable companion.

**AC5 — State + FS subsumed/unbroken.** `cargo test --workspace` green. Pin the
specific modules that MUST stay green, and re-derive them at build (line
numbers perishable):

- State-effect: the EFF6 direct-`[State]` conformance + `run_state`'s own tests
  (`effects/state.rs` + interp). If D3 **subsumes** `run_state` (SEAM-D3 route
  1), State's EFF6 must pass **through** the generalized path unbroken —
  *subsume, don't break*; I add a face that State's observable is unchanged
  pre/post generalization.
- FS driver: `fs_read_file_lines_flip_e2e` +
  `read_bytes_untracked_is_type_error` + the `declared_fs_authority` tests.
- **Rosetta corpus stays 16/0** — `read-file-lines` flips its *mechanism*
  (Option-3-render → genuine-compose) but its `expected` oracle stays
  byte-identical, so the count is invariant. (If a dedicated new example is
  added instead, the corpus grows to 17/0 with `read-file-lines` unchanged —
  D4's call; the face keys to *0 gaps*, not the literal count.)

**AC6 — honesty asterisk retired (my honesty gate, NOT "delete the asterisk").**
The face greps the example's `README.md` + `.ken` header. Retire the exact
landed text —

> *"This example demonstrates **FS-read + pure-parse, NOT effect
> composition** … A single Ken program driving BOTH an `[FS]` effect and
> `[Console]` printing in one type-checked `ITree` needs a coproduct (`Sum`)
> that `run_io` does not support today."*

— and **rewrite the deferral to what actually remains** (e.g. effect-row
polymorphism, >2 effects via nested `Sum`, or whatever D1–D3 leave open),
**not** delete it into an over-claim. AC6 passes iff the residual claim
**matches the landed capability** — if route (a) shipped, the README must say
generality rests on the structural argument + the open 2nd-distinct-pairing, not
claim proven-general. This is [[trust-level-prose-vs-locked-adr-crosscheck]]
applied to the example's own prose.

## 6. Cross-deliverable seams

- **SEAM-D3 (D5↔D3) — OPEN, routed to Architect (`evt_7p7n62qwdmm41`),
  load-bearing.** AC3's executable-vs-structural face depends on whether D3's
  general `run` **subsumes `run_state`** (→ executable 2nd pairing, route b),
  merely **coexists** (→ structural-only + honest residual, route a), or adds a
  **3rd top-level effect**. §3 is written to both; I collapse it to the chosen
  route and reconcile on his ruling. Surfaced at plan time so D3 can reach the
  executable route if it chooses — the SEAM-A analog from the flip.
- **SEAM-D2 (D5↔D2) — surface form.** How an author writes both effects in one
  `main` (per-op lift/`inject`, combined-effect `view`, effect-row
  elaboration) is spec-author's D2. AC2's example + AC7's producer-grep key to
  whatever D2 lands; I lock the **observable** (byte-exact stdout) + the
  **producer path** (real elaboration), not the surface **spelling**. If D2
  forks to new surface syntax → **Steward→operator** (frame STOP); the example
  waits on the resolution.
- **SEAM-oracle.** The example filename/path is D4's choice; oracle value +
  semantics pinned, path tagged deferred.

## 7. Independence + gate note

This D5 plan is **my authored artifact** — it goes to **Architect's soundness
lane**, never self-reviewed (I author `/conformance` and cast the **Spec
fidelity** vote on the pieces I did **not** author — spec-author's D2, the
Architect's D1/D3 — never on my own D5). AC1 (kernel-untouched) and AC4 (the
totality formal cert) are **Architect's** to certify; my faces are the
conformance/executable witnesses beside them. On elaboration-complete,
spec-leader assembles and opens the merge Decision; **both** the Architect's
soundness vote and my Spec-fidelity vote must be recorded, and the Decision
`resolved`, before the Integrator merges (COORDINATION §14). Reconcile any
structural token against the **landed** D1–D3 body at vote time, not this
plan's drafting — the flip taught that the cheapest catch is my own artifact's
staleness at the fidelity vote.
