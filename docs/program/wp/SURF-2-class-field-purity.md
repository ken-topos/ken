# WP `SURF-2-class-field-purity` — purity keywords on class fields

- **Lineage:** SURF-1 follow-on. SURF-1 delivered `const`/`fn`/`proc` at the
  **definition** level; this WP extends the same checked static-purity
  classification to **class fields**, so an effect-polymorphic class operation
  (`proc traverse`) is expressible, classified, enforced on instances, and
  projected.
- **Trigger:** CAT-2 D3 (Traversable) is blocked — `56 §5.1` writes `proc
  traverse` as a class field, but `33`'s class-decl grammar admits only
  `field_name : type`, and the elaborator's class machinery carries no purity
  on fields. Grounded independently by language-implementer, language-leader,
  and the Architect (`evt_c4cz82tm7n5g`).
- **Owner (build):** Language team (owns surface/elaborator extensions, as with
  CAT-1's constructor-class work).
- **Spec elaboration:** Spec enclave (spec-leader → spec-author) — this WP
  changes **surface grammar** (`33`) and **elaboration semantics** (`39`), so it
  runs the full pipeline, not a build-only release.
- **Reviewer:** Architect (elaborator-design fidelity; AC4 sort-discriminant
  non-interference).
- **Size:** M. **Risk:** medium — surface + elaboration change, **but zero
  kernel delta** (see §3.G5).
- **Blocks:** CAT-2 D3. **Blocked by:** nothing (independent of KL-let-check;
  may proceed in parallel).

> **Perishable-state caveat.** Line numbers are `origin/main @ 7e0534b`;
> re-locate by shape at pickup.

## 0. Objective

Admit and **enforce** an optional purity keyword (`const`/`fn`/`proc`) on each
class field, mirroring the definition-level SURF-1 discipline. Concretely, make
this parse, elaborate, classify, and check:

```
class Traversable (f : Type → Type) {
  functor  : Functor f
  foldable : Foldable f
  proc traverse :
    (g : Type → Type) → Applicative g → (a b : Type) →
    (a → g b) → f a → g (f b)
}
```

so that `d.traverse` classifies `proc` at a use site, and an instance whose
`traverse` implementation is not `proc`-classifiable is **rejected**.

## 1. Fixed inputs (settled — do not reopen)

- **The `proc traverse` contract is REQUIRED, not relaxable** (`56 §5.1`,
  `§5.2`/AC6; SURF-1; operator directive 2026-07-05 via PRINCIPLES §13). The
  keyword is the checked static-purity signature; a plain `traverse` field would
  mislabel an effect-row-polymorphic operation as pure. The weaker encoding
  (`proc` only in instance bodies) is explicitly **rejected** as the answer
  (Steward ruling `evt_1j731652bqmnx`, Architect concurring).
- **`proc` here means effect-row-polymorphic** over the Applicative `g` (SURF-1
  row-polymorphism), consistent with `36 §1.6` and the `const`/`fn`/`proc` split
  (`33` lines 14–40). It is the same classification SURF-1 already computes for
  definitions — reuse that path, do not invent a second one.
- **Zero kernel delta.** A class is a right-nested Σ over field *types*
  (`33 §5.2`, `13 §3`); purity is surface/elaborator metadata **erased before the
  kernel**, exactly like definition-level purity. The kernel Σ, its sort, and
  `trusted_base()` are untouched.
- **Backward compatible.** An **unmarked** class field keeps today's behavior
  (no purity enforcement); this WP only *adds* the optional marker + its
  enforcement. Existing classes (`Functor`, `Foldable`, `Eq`, …) are unchanged
  and must stay green with no edits.

## 2. Mandated deliverable outline

### 2a. Spec (enclave) — reconcile grammar and elaboration to the usage

- **`30-surface/33-declarations.md`** — extend the class-decl grammar so a field
  may carry an optional leading purity keyword: `[const|fn|proc] field_name :
  type`. State that an unmarked field is unclassified (status quo) and that a
  marked field's keyword is a checked signature over the field's type/instance
  impl (cross-ref `33` lines 14–40 and `36 §1.6`). Note explicitly that the
  purity marker does **not** enter the AC4 sort discriminant (the record's
  Type/Ω sort is still kernel-computed over field *types* only, `33 §5.1`/AC4).
- **`30-surface/39-elaboration.md`** — specify class-field purity elaboration:
  where the keyword is parsed, how it is carried on `ClassInfo`, how an
  instance's field value is checked against the declared purity (the SURF-1
  `proc` path), and how `.field` projection surfaces the field purity so a use
  site classifies correctly. Reconcile `56 §5.1`'s `proc traverse` as now
  grammatical.
- Keep the change **minimal and additive**; do not restate SURF-1's
  definition-level rules — reference them.

### 2b. Build (Language) — thread purity through the five sites

Each ends in a concrete edit; grounded on `origin/main @ 7e0534b`:

1. **Parser** — `parse_class_decl` field loop (`parser.rs:524-529`): accept an
   optional leading `DefKeyword` (`const`/`fn`/`proc`, tokens per
   `parser.rs:82`/`150`) before `field_name : type`. Absent ⇒ `None`.
2. **AST** — `Decl::ClassDecl.fields` (`ast.rs:161`) carries the optional
   keyword per field (e.g. `Vec<(Option<DefKeyword>, String, Type)>` or a small
   field struct); update every constructor/consumer.
3. **ClassInfo** — add a `field_purities: Vec<Option<DefKeyword>>` parallel to
   `field_names` (`classes.rs:27-50`), populated in `elab_class_decl`
   (`elab.rs:2155`). Do **not** alter `field_types` or the Σ / sort computation.
4. **Instance-field check** — when checking an instance's field value
   (`elab.rs`, the instance Σ-Intro re-check, ~`2181+`): if the class field is
   `proc`, route the instance impl through the SURF-1 `check_surface_purity`
   `proc` path so a non-`proc`-classifiable impl is rejected; `fn`/`const`
   fields enforce their arity/purity likewise; unmarked fields unchanged.
5. **Projection** — `.field` projection (`RExpr::RProj`, `elab.rs:~1285`):
   surface the field's purity so `d.traverse` classifies `proc` at the use site
   (carry the `ClassInfo::field_purities` entry into the projected result's
   purity classification).

## 3. Acceptance criteria (all required)

- **G1 — parse+elaborate.** The `Traversable` class above (with `proc traverse`)
  parses and elaborates; add a focused test.
- **G2 — use-site classification.** `d.traverse …` (local dict or
  `(Traversable_instance_List).traverse …`, per the Architect-approved
  projection forms — no known-bad local-let) classifies `proc`.
- **G3 — instance enforcement discriminator.** An instance whose `traverse`
  implementation is **not** `proc`-classifiable is **rejected** for the right
  reason (assert the specific purity/classification error, not bare
  `is_err()`); a correct `proc` instance is accepted. This proves the contract
  is enforced, not decorative.
- **G4 — backward compatibility.** Every existing class (unmarked fields) parses
  and elaborates unchanged; the full existing suite is green with **no** edits to
  those classes.
- **G5 — zero kernel delta.** `crates/ken-kernel` and `Cargo.lock` diffs empty;
  `trusted_base()` byte-unchanged; the class Σ-of-field-types and its
  kernel-computed sort are identical to pre-WP for the unmarked classes.
- **G6 — workspace green.** `cargo test --workspace` passes.
- **G7 — unblocks D3 (the point of the WP).** After this lands, CAT-2 D3
  (Traversable) builds on top: the real `proc traverse` class + `List`/`Option`
  instances with zero `Axiom`, and the three `56 §5.3` coherence laws
  (identity, naturality **proved**, composition) — that is the *separate* CAT-2
  D3 slice resumed from `wp/CAT-2-build @ 8d2fb4d`, not part of this WP.

## 4. Guardrails (do-not)

- No kernel change; purity is erased-before-kernel surface metadata.
- Do **not** conflate the purity marker with the AC4 Type/Ω sort discriminant —
  sort stays computed over field *types* only.
- Do **not** force existing classes to annotate fields — the marker is optional;
  unmarked = status quo.
- Do **not** invent a second purity-classification path — reuse SURF-1's
  `check_surface_purity`/`proc` machinery.
- Do **not** smuggle any of this into the held CAT-2 D3 branch; it lands first on
  its own reviewed WP, then D3 resumes on top.
