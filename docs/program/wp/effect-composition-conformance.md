# effect-composition — D5 conformance / acceptance plan (enclave deliverable)

**Author:** conformance-validator (CV). **Branch:**
`conformance-validator/effect-composition`, off `wp/effect-composition@aa45565`
(itself off `origin/main@43e97d02`, VAL2 16/0). **Frame:**
`docs/program/wp/effect-composition.md` (Steward). **Assignment:** spec-leader
`evt_3qmjknw1dntm7` — D5 drives **AC2, AC3, AC5, AC6, AC7**, plus the totality
**face** (AC4's formal cert is Architect's). AC1 (kernel-untouched) is
Architect's.

> **Reconcile note (both seams ruled; §3 collapsed from the two-route draft).**
> My first draft (`b721cfe`) wrote AC3 to **both** D3-seam routes because the
> subsume-vs-coexist call was open. It is now ruled:
> **D3↔D5 (Architect `evt_mgdfthndwx3w`): COEXIST** — `run_state` stays a
> kernel-re-checked pure fold; folding it into trusted-Rust `run_io` would be a
> TCB regression (rejected on soundness, not convenience). AC5's *"State becomes
> an instance"* is satisfied as a literal **instance** of D1's general
> `resp_sum`, **not** by running State through `run_io`.
> **D2 (spec-author `b4db413`): form (a)** — explicit named `injectL`/`injectR`
> sequenced with `bind`; **no new syntax → frame STOP not triggered.** §3–§6 are
> reconciled to these rulings, and this pass folds in the discriminators they
> surfaced (faithful-retag, `resp_sum`-reduces, authority-non-laundering,
> the optional synthetic-third peel probe).

This is the executable contract the Runtime build's tests must satisfy — **what
each conformance face asserts and why it discriminates**, every face traced to a
verdict/observable flip, no green-vs-green. It does **not** author the mechanism
(D1–D3 Architect, D2 surface spec-author); it grounds each face against the
**landed** `aa45565` substrate + the now-pinned D1/D2/D3 designs, not the
frame's perishable citations.

## 0. What D5 must pin

| Face | AC | Discriminates | Cert owner |
|---|---|---|---|
| No-hand-fed coproduct (acceptance) | AC7 | real surface producer vs. Rust-built tree | CV |
| Generality — structural peel | AC3 | effect-blind `InL`/`InR`-generic vs. effect-literal dispatch | CV |
| Generality — 2 distinct pairings | AC3 | general `resp_sum` at ≥2 `(g,h)` vs. one special-case | CV |
| `resp_sum` reduces on ctors | AC3/D1 | reducing `declare_def` vs. postulate | CV |
| Faithful retag (inject) | AC2/AC3 | `injectL`→`InL` tag-preserving vs. drop/reorder/mis-tag | CV |
| Composed program runs | AC2 | in-program `[Console]` print vs. CLI post-render | CV |
| Authority not laundered | AC2/soundness | peel reaches `authorizes` vs. composition bypass | CV + Architect |
| Totality (face) | AC4 | halts on multi-step continuation | CV face / Architect cert |
| State + FS unbroken | AC5 | instance-of vs. fork | CV |
| Asterisk retired | AC6 | residual claim matches landed capability | CV |
| Kernel untouched | AC1 | — | Architect |

## 1. Substrate grounding (fixed inputs — `aa45565` + pinned D1/D2/D3)

Landed substrate (read from the tree, not the frame):

- **`run_io` is the CLI's sole top-level driver** (`crates/ken-cli/src/main.rs`
  `run_file` → `run_io(…)`; `crates/ken-interp/src/eval.rs`): raw op-dispatch on
  two arms — Console `write_id` (`println!`) and FS `readfile_id`
  (`std::fs::read` **behind the `authorizes` gate**, the sole runtime FS net);
  `Ret r`→done, `Vis op k`→dispatch→`apply(k,resp)`, **`_ => UnknownEffect`**
  fail-closed. **Zero `Sum` awareness today** — a coproduct-wrapped op falls to
  the catch-all. That is the baseline D3 lifts.
- **`resp_sum` State-left-pinned** (`effects/state.rs` `declare_resp_sum`) —
  D1 abstracts the `resp_state s` first-summand to a parameter `rg`.
- **`run_state` is the only `Sum`-fold, CLI never invokes it** — a *pure*
  kernel-re-checked `declare_def` (the intermediate handler role).
- **`Sum a b = InL a | InR b`** (`declare_sum`) already exists, effect-agnostic.
- **The example** `examples/rosetta/read-file-lines/` — pure `[FS]` `main`, CLI
  renders the return value (the Option-3 asterisk); `expected` =
  `alpha\nbeta\ngamma\n`.

Pinned designs (Architect `evt_241dchcb5y6j8`/`evt_mgdfthndwx3w`, spec-author
`b4db413` §D2 / `36 §4.5.7`) — the build grounds these against landed code, but
the **contracts** my faces key to are:

- **D1** `resp_sum : (g h:Type)->(rg:g->Type)->(rh:h->Type)->Sum g h->Type`, a
  reducing `declare_def` (non-recursive `Term::Elim` over `Sum`) with
  **`resp_sum g h rg rh (InL o) ≡ rg o`**, `(InR o) ≡ rh o` **definitionally by
  ι**. *Re-derived from first principles:* `method_inl = λx. rg x` applied to
  `InL o`'s arg `o` ι-reduces to `rg o`. **Load-bearing:** inject type-checks
  **only** because this reduces (D2.2); a postulate breaks it.
- **D2** `injectL/injectR : ITree g rg a -> ITree (Sum g h) (resp_sum g h rg rh)
  a`, an `elim_ITree` tag-map rewriting each `Vis op k` to `Vis (InL op) …`
  (resp. `InR`), branching shape untouched. Surface: author writes
  `bind (injectL … c) (λx. injectR … c')` — ordinary applications, no new
  grammar. inject **materializes `InL`/`InR` in the op value** (D3 dispatches by
  peeling ctors off the runtime op — a type-only row view is unroutable).
- **D3** terminal `run_io` gains a **`Sum`-peel**: at each `Vis op k`,
  recursively strip `InL`/`InR` to the innermost base op, match its tag against
  the existing base table, `apply(k,resp)`, loop. **Effect-blind** (no
  `ConsoleOp`/`FSOp` literal in the peel). **`authorizes` stays unconditional
  inside the `ReadFile` arm**, reached only after the peel lands on
  `readfile_id` — the peel adds no gate-skipping path (Architect's hard build
  constraint).

## 2. AC7 — no hand-fed coproduct in the acceptance e2e

The **acceptance** e2e (AC2/AC3) must enter through **`ken-cli`'s `run_file`**
over a real `.ken`: `Sum`, `InL`/`InR`, and the `injectL`/`injectR` lift are
built by **elaborating surface Ken**, never hand-constructed in Rust and handed
to `run_io`. [[conformance-hand-feeds-the-deliverable]] — the trap the frame
names for D5, and the one that let the flip's `Result` field-order lie ship
latent (its driver tests never elaborated a real consumer).

**Producer-grep guard** (structural, asserts the *absence* of a hand-feed in the
acceptance file): scan for a hand-built coproduct at the interpreter entry —
`InL` / `InR` / `make_ctor(…inl…|…inr…)` / `EvalVal::Ctor { id: inl_id|inr_id` /
a bare `sum_id` / a hand-emitted `Vis (InL …)` / any `run_io(` on a
locally-constructed tree. **Allowed only** inside doc-comment methodology prose
(as the flip's AC3 allowed `EvalVal::Cap` to name-but-not-construct). **Required
producer:** the CLI binary / `run_file` over `examples/rosetta/<compose>/*.ken`,
the rosetta harness path.

**Scope, stated precisely.** AC7 bans hand-feeding a coproduct **as the
acceptance path** (standing in for the surface flow). A separately-labeled
white-box **mechanism unit test** MAY hand-construct a synthetic `Sum` tree to
probe `run_io`'s peel (§3, the synthetic-third probe) — it is a unit test of one
function, not an acceptance e2e, and is exempt by construction. The line: the
AC2/AC3 acceptance drives surface Ken; a peel-mechanism probe is white-box and
must be labeled as such.

## 3. AC3 — generality (COEXIST: structural primary + executable pairings)

**Vacuity risk (plainly).** A single FS+Console example is **green-vs-green**
between the general mechanism and a hardcoded `Sum ConsoleOp (FSOp a)`
special-case — both print the three lines. Generality is a **dimension the
FS+Console example is blind to** (K2c-series-2: a multi-dimensional net needs a
discriminating case *per* dimension). Under COEXIST, generality is guarded on
three legs:

**(1) Structural — the effect-blind peel (primary; parametric route).**
Grep asserts `run_io`'s `Sum`-peel carries **no** `ConsoleOp` / `FSOp` /
`write_id` / `readfile_id` literal and dispatches only on `InL`/`InR` structure
+ the innermost base tag. For a ~3-line effect-blind peel this grep is
**dispositive** (Architect: the right guard for the base-driver layer, not a
fallback), backed by (2)+(3)+AC7. Mirror on D1: `resp_sum` is
**quantified over `g h`** — no summand-family literal.

**(2) Executable — two distinct `(g,h)` pairings, same machinery.**
- **{FS, Console}** through `run_io`'s D3 peel (executable; **+ position-swap
  `Sum FSOp ConsoleOp` vs `Sum ConsoleOp FSOp`** witnesses order-independence /
  §4.5.4 commutativity — verdict-flip against a position-hardcoded dispatcher).
- **{State, Console}** via the author writing **in-source `runState s₀ (…)`**
  (existing §4.5.2 surface, no CLI `s₀` synthesis, no new syntax): `runState`
  peels State via the **D1-general `resp_sum`**, re-emits the base op, `run_io`
  runs the residual Console tree; the observable is the Console output. This
  exercises the **pure-handler** role's generality — a *different* coproduct
  mechanism from {FS,Console}'s terminal-driver peel, **both resting on the
  general `Sum`+`resp_sum` (D1)**. This is the executable ≥2-distinct-pairings
  discriminator (spec-author's §D2.8 "two distinct `(g,h)`").

**(3) `resp_sum` reduces on ctors (positive reduction face — guards the
postulate regression).** Assert `resp_sum g h rg rh (InL o)` **evaluates to**
`rg o` (not stuck) at a concrete instantiation — a positive, self-verifying
reduction case. If a build shipped `resp_sum` as a **postulate** (breaking
D2.2), inject would fail to type *and* the driver's response threading breaks;
this face + AC2 elaborating make the reduction observable.

**Honest residual (Architect-affirmed; stated in D5 and the example README).**
For `run_io`'s **own** dispatch, the only executable variation without a 3rd
base effect is FS+Console + swap — a set-{Console,FS}-either-order adversary
passes those green. That behavioral discriminator is **fundamentally
unavailable** at the base-driver layer without proliferating a 3rd base effect
(option 3, rejected). So `run_io`-dispatch generality rests **structurally** on
leg (1), which for a 3-line effect-blind peel is dispositive. Not an
over-claim: I do **not** assert an executable proof that `run_io` routes an
arbitrary base tag — only what legs (1)+(2)+(3) actually establish.

**Faithful-retag discriminator (D2 inject — spec-author §D2.8).** `injectL`
produces `InL`-tagged ops preserving the op and branching; `injectR` →`InR`.
Verdict-flip: a correct inject → tags match input → correct dispatch →
byte-exact output; an inject that **drops / reorders / mis-tags** (e.g.
`injectL` emits `InR`, or elides the op) → wrong dispatch / `UnknownEffect`.
Witnessed by the composed program producing the exact oracle **and** — as the
executable negative — a mis-tag mutation flipping it.

**Optional but recommended — synthetic-third peel probe (mechanism unit test,
AC7-exempt).** A white-box test driving `run_io`'s peel with a **synthetic
third op-family** (test-only handler stub) proves it routes an *arbitrary* base
tag — the ONE executable check that closes leg-(1)'s structural residual for the
base driver itself. Architect flagged it available-not-required; I **adopt it as
recommended** (it converts the run_io-dispatch residual from structural-only to
structural+executable-on-a-synthetic-third), scoped as a labeled mechanism probe
distinct from the AC7-governed acceptance e2e (§2).

## 4. AC2 — the composed program + the exact expected oracle

The re-authored example (`read-file-lines` re-worked, or a dedicated
`compose-fs-console` — filename is D4's choice, `(oracle: D4 example path)`)
**genuinely composes**, in surface Ken, form (a):

```
main = bind (injectL … (read_bytes …)) (\r. injectR … (print_line …))
```

— an `[FS]` read **and** an `[Console]` print in **one** type-checked
`ITree (Sum (FSOp a) ConsoleOp) (resp_sum …) …`, elaborated via `run_file`,
executed via `run_io`'s D3 peel.

- **Oracle:** `alpha\nbeta\ngamma\n` — **byte-identical** to today's `expected`,
  but now produced by genuine **in-program `[Console]` printing**, not the CLI's
  post-hoc render. Clean invariant: *same observable, real composition* — the
  flip is that the print now comes from inside the program.
- **Face keys the observable to STDOUT** (the composed program's printed lines),
  byte-exact via the rosetta harness (`expected` file, exact match) —
  **not** the CLI return-value render (the Option-3 path being retired).
- Oracle **value + semantics** pinned; the surface **spelling** is now D2-fixed
  (form (a), `injectL/injectR`+`bind`), so this is no longer a deferred token.

## 5. Totality face (AC4) + Authority-non-laundering + AC5 / AC6

**Totality face (AC4 — executable witness; formal fold-terminates cert =
Architect).** A composed program sequencing **multiple** ops through the
continuation — read, then print each of N lines (a fold over the continuation
resumption) — runs to completion and **halts** with correct output, witnessing
`run_io`'s loop making progress on each `Vis` and terminating at `Ret`. **Not a
value-flip** (a total language yields a value *iff* it halts) — a
termination/does-it-halt face beside Architect's structural cert (D1
non-recursive elim; D3 bounded Sum-peel + finite-tree descent).

**Authority-non-laundering (AC2/soundness — CV discriminating pair beside
Architect's cert).** Composition adds a *new* path to `readfile_id` (through the
`InL`/`InR` peel). The face verifies the peel does **not** bypass the sole FS
net: a composed FS+Console program with a **sufficient** cap (`Cap APartial`)
reads + prints (oracle); the **same** program with an **insufficient** cap
(`Cap ANone`) is **denied at `authorizes`** — `CapabilityDenied`, right-reason
(not `NotFound`, not a bind error) — exactly as in the flip's AC4, now **under
composition**. Verdict-flip on cap-sufficiency; proves the peel changes only
*how* `readfile_id` is reached, not *whether* the gate fires. (Architect
certifies the gate placement; this is its executable discriminator.)

**AC5 — State + FS as instances/unbroken.** `cargo test --workspace` green.
State becomes a literal **instance** `resp_sum (StateOp s) f (resp_state s)
RespF` of D1; `runState`/`get`/`put`'s `resp_sum`-apps update to the 4-arg form
(mechanical), State's structure unchanged — **not** run by `run_io`. Pin
(re-derive line numbers at build):

- State-effect: EFF6 direct-`[State]` conformance + `run_state`'s tests stay
  green through the mechanical 4-arg update; a face asserts State's
  observable is unchanged pre/post generalization.
- FS driver: `fs_read_file_lines_flip_e2e` +
  `read_bytes_untracked_is_type_error` + the `declared_fs_authority` tests.
- **Rosetta corpus stays 0-gap** — `read-file-lines` flips its *mechanism*
  (render → compose), `expected` byte-identical ⇒ count invariant (16/0), or
  17/0 if a dedicated example is added. The face keys to **0 gaps**, not the
  literal count.

**AC6 — asterisk retired (honesty gate, NOT "delete the asterisk").** Grep the
example `README.md` + `.ken` header. Retire the landed text —

> *"demonstrates **FS-read + pure-parse, NOT effect composition** … one
> type-checked `ITree` needs a coproduct (`Sum`) that `run_io` does not support
> today."*

— and **rewrite the deferral to what actually remains** (row-directed
auto-injection sugar `visits [FS,Console]` is deferred, per D2.3; `run_io`-own
dispatch generality rests on the structural argument + no 3rd base effect, per
§3's honest residual), **not** delete it into an over-claim. AC6 passes **iff**
the residual claim **matches the landed capability**
([[trust-level-prose-vs-locked-adr-crosscheck]] applied to the example's prose).

## 6. Cross-deliverable seams — resolved

- **SEAM-D3 (D5↔D3) — RESOLVED: COEXIST** (Architect `evt_mgdfthndwx3w`).
  `run_state` stays a kernel-re-checked pure fold (folding into `run_io` = TCB
  regression, rejected). AC3 primary = structural effect-blind peel;
  executable = {FS,Console}+swap (terminal driver) + {State,Console} in-source
  `runState` (pure handler) — two distinct `(g,h)` on the general `resp_sum`;
  residual guarded structurally + the recommended synthetic-third probe.
- **SEAM-D2 (D5↔D2) — RESOLVED: form (a)** (spec-author `b4db413`).
  `injectL/injectR`+`bind`, no new syntax (STOP not triggered). AC2's example
  and AC7's producer-grep key to `bind (injectL …) (λ. injectR …)`. **Hard
  D1↔D2 contract my §3 leg-(3) guards:** `resp_sum` must **reduce** per-tag to
  `rg o`/`rh o`, else inject fails to type — so the reducing-`declare_def` (not
  postulate) is a conformance-checked property, not just a design note.
- **SEAM-oracle — RESOLVED enough:** oracle value + semantics + surface form
  pinned; only the example **filename** is D4's choice (`(oracle: D4 example
  path)`).

## 7. Independence + gate note

This D5 plan is **my authored artifact** — it goes to **Architect's soundness
lane**, never self-reviewed (I author `/conformance` and cast the **Spec
fidelity** vote on the pieces I did **not** author — spec-author's D2, the
Architect's D1/D3 — never on my own D5). AC1 (kernel-untouched) and AC4 (the
totality formal cert) are **Architect's** to certify; the
authority-non-laundering gate placement is his cert, my face its
executable discriminator. On
elaboration-complete, spec-leader assembles and opens the merge Decision;
**both** the Architect's soundness vote and my Spec-fidelity vote must be
recorded, and the Decision `resolved`, before the Integrator merges
(COORDINATION §14). At vote time I reconcile every structural token — the
`resp_sum` reduction shape, the `injectL`→`InL` tag, the peel's
effect-blindness — against the
**landed** D1–D3 body, not this plan's drafting; the flip taught that the
cheapest catch is my own artifact's staleness at the fidelity vote.
