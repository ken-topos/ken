# WP `proof-vocab-completion` — enforce the proof vocabulary + name its primitives (`fn`⊥Ω, `tt`→`Proved`, surface `Top`)

**Owner:** Language team (elaborator gate + prelude source-of-truth + the
catalog migration, **one atomic branch**) · **Foundation** advisory on the
catalog Ω-valued-`fn`/`tt` inventory (it mapped these in the pedagogic
rewrite) · **Spec enclave** for the §16/§33/§51 rule + naming transcription
and conformance · **Architect** surface + fidelity review. **Size:** M–L ·
**Risk:** low (zero-soundness surface: a surface gate + behaviour-preserving
relabel/rename + one bounded, per-site-verified type adoption). **Depends
on:** `proof-vocabulary` change **A** — **MERGED `origin/main @ c755837f`**,
so this is releasable. **Absorbs** the held `fn-lemma-partition` WP and
`proof-vocabulary`'s deferred `_ind`→recursive-`lemma` collapse.

## Objective

Complete and *enforce* the proof vocabulary, and give its two primitives
readable names — bundled into **one catalog pass** for credit economy:

1. **`fn`⊥Ω partition (enforce).** `fn`/`const` must **not** classify at Ω;
   `lemma`/`proof` are the only Ω-valued definitions. Makes the erasure boundary
   a surface invariant you read off the keyword.
2. **`tt` → `Proved` (rename).** Rename the surface name of `Top`'s sole
   inhabitant from the inscrutable `tt` to **`Proved`** (operator-decided:
   past-tense finality, QED-like — rejected `Trivial`/`Proven`). Purely the
   user-facing name.
3. **Surface `Top` (expose + bounded adoption).** Expose `Top : Ω₀` as a
   nameable prelude global (today it exists in the kernel but is unnameable), and
   adopt it where a catalog definition's sole purpose is a trivial-truth token
   currently spelled `Equal Bool True True`.

All three touch the same catalog `.ken.md` files + `prelude.rs`; doing them
in one migration pass touches each site once (e.g.
`const mul_zero_r : IsTrue … = tt` becomes
`lemma mul_zero_r : IsTrue … = Proved` in a single edit).

## Settled design — FIXED inputs

Part 1 grounds on Architect ruling **evt_6eyh27x6p7h0x** (endorse; the
former `fn-lemma-partition` frame). Parts 2–3 are operator-decided
(`tt`→`Proved`; expose `Top`, which has real utility — the catalog's `Equal
Bool True True` stand-ins). Do not relitigate; cite these.

**(1) The `fn`⊥Ω partition is CLEAN — exception-free.** Grounded in the kernel
sort model: `Term::Type(Level)` and `Term::Omega(Level)` are **distinct universe
constructors** (`crates/ken-kernel/src/term.rs:230/237`; Ω ∉ `Type l`). The only
sort variable is `Level`, never the Type-vs-Ω distinction, so every `fn`/`const`
result type classifies at a determinate Type-or-Ω sort at definition time. The
check is `ensure_NOT_omega` = "result sort is `Term::Type(_)`, not
`Term::Omega(_)`" — decidable, no legitimate Ω-valued `fn` exceptions
(HK/universe-poly `fn`s are always `Type l`). `const` partitions identically;
`def`/`prop` are orthogonal (they name types / Ω-prop families, not proof terms).

**(2) Zero-soundness surface (whole WP).** `fn`/`const` and `lemma`/`proof` both
bottom out in the kernel `declare_def` (no keyword tag); erasure partitions by
the **Ω sort**, not the keyword (`erasure.rs`, keyword-free). So `ensure_NOT_omega`
is a pure surface gate, symmetric to the existing `ensure_omega_type`
(`elab.rs:5396`). Nothing downstream keys on `fn`-vs-`lemma`.

**(3) `tt`→`Proved` is a SURFACE rename — zero kernel/TCB touch.** `tt` is a
kernel constant (`env.rs` `tt_id`, K5 `tt : Top`) surfaced at `prelude.rs:463`.
Rename **only the surface-exposed name** to `Proved` (the string the prelude binds
+ every `.ken.md`/spec/conformance use). **Leave the kernel-internal identifier
untouched** — the elaborator maps the surface `Proved` to the same kernel
constant. Zero kernel diff, so it stays off the soundness lane.

**(4) `Top` exposure mirrors `Bottom`.** `Bottom : Ω₀` is already surfaced
(`prelude.rs:475`, needed to spell `Not`). Surface `Top : Ω₀` the same way — a
prelude global addition, no kernel change. Its inhabitant is `Proved` (post-part-2).

**(5) Migrate-then-enforce — atomic.** The Part-1 gate must **never** land before
the Ω-`fn`→`lemma` migration, or it rejects the existing tree. Single branch;
the migration edits precede the `ensure_NOT_omega` enforcement so `origin/main` is
never in a rejecting state. Migration scope for Part 1 = **ALL** Ω-result
`fn`/`const`, catalog **and** the Rust-emitted prelude (grep the emission, not
just `.ken.md`); verify none sits in a mixed `fn`↔`proof` cycle (today's are
self-recursive/homogeneous — a guard to assert, not a blocker).

## Scope — one atomic branch, four workstreams

**A. Elaborator gate (Part 1).** Add `ensure_NOT_omega` on the `fn`/`const`
result sort, symmetric to `ensure_omega_type` (`elab.rs:5396`): reject a
`fn`/`const` whose result classifies at `Term::Omega(_)`, diagnostic naming the
rule ("`fn`/`const` compute; use `lemma`/`proof` for an Ω-valued definition").

**B. Prelude source-of-truth (`prelude.rs`).** (i) Rebind the surface name of the
`tt` constant to **`Proved`** (Part 2). (ii) Add **`Top`** to the surfaced globals
(Part 3), mirroring `Bottom` at :475. No kernel edit.

**C. Catalog migration — one pass over the 13 `.ken.md` files.**
- **Part 1:** every Ω-valued `fn`/`const` → `lemma`/`proof`. This **subsumes** the
  `_ind`+thin-wrapper idiom → single recursive `lemma` (those `_ind` fns are
  exactly the Ω-valued fns the gate rejects; A makes the recursive `lemma`
  expressible). Preserve name + signature + proof term per site.
- **Part 2:** every surface `tt` → `Proved` (~245 uses; largely mechanical).
- **Part 3 (BOUNDED, per-site verified):** replace `Equal Bool True True`
  trivial-truth **stand-ins** with `Top` **only where that is the definition's
  sole purpose** (the ~8 sites: Collections, Parsing, EmptyDec, LawfulClasses …).
  This is a genuine type change (inhabitant `Refl` → `Proved`), so each site's
  proof term is updated and re-checked. **Not** a blanket `Equal Bool True True`
  sweep — only clean trivial-truth tokens.

**D. Spec + conformance (Spec enclave).** Transcribe: §33 — `fn`/`const` result
must not classify at Ω (`lemma`/`proof` are the only Ω-valued term defs); §16 —
`Top`'s inhabitant's surface name is `Proved`, and `Top` is a nameable global;
§51 — any catalog-facing naming. Update conformance for the `tt`→`Proved` rename
and the gate. Spec edits (state the rules), **not** a soundness review.

## Out of scope (keep fail-closed narrow)

- **Mixed `fn`↔`proof` mutual cycles** — an Ω-`fn` in a mixed cycle can't
  migrate to `lemma` until `proof-vocabulary`'s deferred mixed-cycle support
  lands. Assert the catalog + prelude contain none today.
- Renaming the **kernel-internal** `tt` identifier (surface-only rename here).
- Blanket `Equal Bool True True` → `Top` (only sole-purpose trivial-truth tokens).
- `def`/`prop` (orthogonal). No change.

## Acceptance criteria (testable)

1. **Gate rejects (Part 1).** A `fn`/`const` with an Ω-result (e.g.
   `fn f (x:Nat) : Equal Bool (leq_nat x x) True = …`) is rejected with the
   vocabulary diagnostic — assert the **specific** error variant, not `is_err`.
2. **Gate admits.** A proof-relevant `fn` whose result is `Type` (`total_leq_nat
   : Or …`, any `Dec`/`Sigma`-valued `fn`) still elaborates green; `const` arm
   symmetric.
3. **Migration complete & green.** Full catalog + prelude bootstrap elaborate
   green **with the gate active**; no Ω-valued `fn`/`const` remains (catalog grep
   **and** prelude-emission grep clean) — migrate-then-enforce held in the branch.
4. **`_ind` collapse.** The `proof-vocabulary` `_ind`+wrapper pairs become single
   recursive `lemma`s, each preserving the public law's name + signature + proof.
5. **`tt`→`Proved` (Part 2).** No user-facing `tt` remains in catalog/spec/
   conformance; `Proved : Top` elaborates as the trivial-truth token; **kernel
   untouched** (internal `tt_id` unchanged; `git diff` shows no `ken-kernel/**`).
6. **`Top` surfaced + adopted (Part 3).** `Top : Ω₀` resolves as a nameable
   global; each migrated stand-in site elaborates with `Top` inhabited by
   `Proved`; a conformance pair pins `Top`/`Proved`.
7. **Zero TCB.** `trusted_base_delta` unchanged; `git diff --name-only` shows
   **no** `crates/ken-kernel/**` and no `Cargo.*`; scope = elaborator + prelude +
   catalog + spec + conformance only.
8. **Mixed-cycle guard.** A test or explicit audit note asserts no migrated def
   sits in a mixed `fn`↔`proof` cycle.

## Do-not-reopen guardrails

- The partition is **exception-free** — no "allow Ω-valued `fn` for X" hatch.
- The whole WP is **zero-soundness surface** — do not route through a kernel/Spec
  **soundness** gate; it is `ensure_*` + relabel/rename + one bounded type
  adoption + spec-rule transcription.
- **Migrate-then-enforce is atomic** — the gate never lands in a state that
  rejects the tree.
- **`Proved`** is the chosen name (operator; not `tt`/`Proven`/`Trivial`); the
  **kernel-internal `tt` constant name is untouched**.
- Mixed `fn`↔`proof` cycles and blanket-`Equal`→`Top` stay OUT.

## Review lanes

- **Gate + rename + migration:** Architect **surface + fidelity** review (gate is
  surface; Ω-`fn`→`lemma` and `tt`→`Proved` are behaviour-preserving relabels/
  renames of already-checked terms; the bounded `Top` adoption is the one part
  needing **per-site** fidelity — each stand-in's type + proof term re-checked) +
  a conformance/fidelity pair. **No** kernel/Spec soundness gate.
- **Spec rules:** enclave transcribes §16/§33/§51. Note the enclave's spec-author
  is now `gpt-5.6-sol(high)`; the Opus Architect (independent spec re-derivation)
  and the Opus CV (code↔spec) remain the backstops — the exact defense-in-depth
  the swap was chosen for.

## Notes

Capstone of the proof-vocabulary initiative: with A landed and this in place,
`fn` provably means "computes", `lemma`/`proof` mean "proof-irrelevant" (enforced
surface invariant), and the vocabulary's two primitives read as `Proved : Top`
instead of `tt`. Bundled per operator (2026-07-11) for one catalog pass /
Handoff-Gate / review cycle. Owner is Language (single atomic branch: gate +
prelude + catalog) to preserve migrate-then-enforce and avoid a cross-team
branch handoff; Foundation advises on the Ω-`fn`/`tt` inventory it already
mapped. Sequenced by the operator; timing is the operator's call.
