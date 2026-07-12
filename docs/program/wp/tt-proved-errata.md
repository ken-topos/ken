# WP: `tt`→`Proved` cross-corpus errata (spec/ + conformance/)

**Owner:** Spec enclave (spec-author authors spec errata; conformance-validator
owns the conformance reconcile; spec-leader coordinates + casts).
**Size:** S–M. **Risk:** low (doc/fixture fidelity; zero kernel/code/TCB).
**Base:** `origin/main @ 42426bbb` (fetched ref).
**Kind:** doc-only reconcile — the required fast-follow of
`proof-vocab-completion` (merged `97c31b3f`, PR #531).

## Objective

`proof-vocab-completion` renamed the **surface** Top-introduction constant
`tt` → `Proved` and removed the surface `tt` binding (the kernel id is
unchanged). Two corpora this WP did **not** touch still spell the old surface
term and now contradict the landed surface (CV `evt_6yspgfbr70znj`):

- **spec/** — 70 surface `tt` references (normative surface now wrong).
- **conformance/** — 143 references, including a **now-broken fixture** that no
  longer elaborates.

Reconcile both to the landed surface so the normative spec and the conformance
corpus match what the elaborator actually accepts.

## Fixed inputs (settled — do not reopen)

- **The rename is operator-decided and landed.** `Proved` is the surface
  Top-intro constant, bound to the kernel's existing `tt_id`; `Top` is the
  surface Top type over `top_id`. Both kernel ids are **unchanged**
  (`prelude.rs`; CV-verified `insert("Proved", env.tt_id())`,
  `insert("Top", env.top_id())`). Do **not** relitigate `tt`-vs-`Proved`, and
  do **not** touch any kernel id or kernel-level constant.
- **The surface `tt` binding is gone.** A surface proof term written `tt` no
  longer elaborates — this is why `sound-ord-proved.ken` breaks.
- **Grounding:** CV inventory `evt_6yspgfbr70znj`; Architect impredicativity
  ruling `evt_4eantd6r4s8hh`; `proof-vocab-completion` @ `8c65aaed`.

## The load-bearing discrimination (surface vs kernel-id)

This is **not** a blind `s/tt/Proved/`. Migrate **only** occurrences of the
**surface** Top-intro constant; **preserve** every reference to the kernel's
internal `tt` id and any conceptual/explanatory prose that is *about the
mechanism* rather than *spelling a surface term the reader would type*.

- **Migrate → `Proved`:** surface proof terms and surface fences — `= tt`,
  `⇒ tt`, `\h. tt`, `Nil => tt`, `ordered_empty = tt`, and a surface constant
  declaration like `tt : Top` presented as "a prelude constant"
  (`16-observational.md:156-159`) — anything a user would *write in Ken source*.
- **Preserve `tt`:** text describing the **kernel** Top-intro id, the K5/K7
  observational-collapse machinery narrated at the kernel level, and prose that
  says "`tt` after the observational collapse" as mechanism exposition where
  renaming would misstate the kernel. When in doubt on a given line, the test
  is: *would this appear in Ken surface source?* → `Proved`; *is it kernel-level
  narration?* → keep, but ensure it doesn't imply a surface `tt` a user could
  type.
- **Out of scope (do not touch):** the legitimately-kept `Equal Bool True True`
  lines CV enumerated (`Dec (Equal Bool True True)`, the K7 `Inl … Proved`
  witnesses, hypothesis binders) — these are not `tt` refs and are correct.

## Scope / deliverables

**Lane A — spec errata (spec-author):**
- `spec/` surface Top-intro spellings → `Proved` per the discrimination above.
  CV-named sites: `16-observational.md:156-159`; `54-map-verified-laws`
  (`ordered_empty = tt`, `Nil => tt`, `Leaf => \h. tt`, the §2.3 `tt`-vs-`Refl`
  discipline); `55-lawful-functors §3.2`; `56-effectful-classes`; the
  `Refl`/`absurd`/`tt` surface sugar in `34-data-match` / `39-elaboration`.
  Sweep the full 70-ref set — the list is a floor, not a ceiling; grep to
  confirm none remain that denote the surface constant.
- `catalog/guide/README.md:26` — the "`tt` vs. `Refl`" prose → `Proved`.

**Lane B — conformance reconcile (conformance-validator, owned):**
- `conformance/challenge/C6-lawful-ord-vs-stub/sound-ord-proved.ken` — the
  surface proof terms (`match x { Lo ⇒ tt ; Hi ⇒ tt }`, line 18; and the `tt`
  discharge sites) → `Proved`, so the fixture **elaborates green again**.
- `conformance/kernel/observational/seed-k5-omega-fragment.md` — reconcile its
  surface `tt` per the discrimination (preserve kernel-level narration).
- Sweep the remaining conformance `tt` refs for any other surface proof term.

## Acceptance criteria (testable)

1. Every `spec/` occurrence denoting the **surface** Top-intro constant reads
   `Proved`; kernel-id / kernel-narration `tt` references are preserved and
   verified to not imply a user-writable surface `tt`.
2. `sound-ord-proved.ken` **elaborates green** at the enclave-run check (the
   load-bearing regression — CV runs it; it was the concrete breakage).
3. `seed-k5-omega-fragment` and any other surface conformance `tt` reconciled;
   no surface proof term spells `tt`.
4. `catalog/guide/README.md:26` updated.
5. No spec/conformance line contradicts the landed surface (`Proved`/`Top`).
6. **Doc/fixture-only** — zero `crates/`, `Cargo`, lockfile, or kernel-id
   change; `git diff --check` clean.

## Review / merge

Enclave-internal: spec-author authors Lane A, CV executes + verifies Lane B
(runs `sound-ord-proved.ken`), spec-leader coordinates and casts the Spec vote.
No build-team lane. Steward honesty gate (doc-only) + publisher on the merge
request. Sequenced as a fast-follow, so it may co-land in one enclave cycle.
