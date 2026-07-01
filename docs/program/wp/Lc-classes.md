# Lc — Typeclasses/constraints (classes-as-subobjects-of-the-universe)

**Owner (build):** Team Language. **Deps:** K1, V0 (both **met**). **Stream /
gate:** WS-L. **Unblocks:** L3b (`Map`/`Set` need `DecEq`/`Ord`), L8 (curated
lawful stdlib), `Ord`-based sorting, prover-facing **lawful** instances
(`Monoid`/`Ord` law proofs), and `derive`.

> **Steward frame** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails. The enclave elaborates `spec/30-surface/33 §5` +
> `spec/30-surface/39 §5` to **team-ready, algorithm-level** rigor and authors
> conformance **before** Team Language builds (§2c). This is the *frame*, not the
> spec. **No new kernel feature:** classes-as-subobjects reuses existing Σ/record
> (`10-kernel/13 §3`), strict-prop Ω (`10-kernel/16 §1`), and SCT
> (`10-kernel/17 §4`) machinery — if the build finds itself adding a kernel
> rule/judgment/"class" former, it has **mis-scoped**.

## 1. Objective

Deliver Ken's constraint mechanism: **typeclasses as subobjects of the
universe** (`33 §5`). A `class` elaborates to a **record** (a Σ of operations
*and their law propositions*); an `instance` is a **value** of that record
(including proofs of the laws); a constraint `where C A` is an **implicit
instance argument** the elaborator discharges by **instance search** (proof
search for subobject membership, `39 §5`). Coherence is governed by **ADR 0008**
(Accepted).

## 2. Settled decisions — FIXED, do not reopen (`OQ-classes` DECIDED, ADR 0008)

- **Two class kinds, split by where the dictionary lives:**
  - **Property classes** (Ω-valued: `Decidable p`, `IsHom f`). Proof-irrelevant
    (`16 §1`) ⇒ any two instances **definitionally equal** ⇒ **coherence is
    free** (the kernel guarantees it; *no resolver convention applies*). A
    dividend of the strict-prop Ω (ADR 0005).
  - **Structure classes** (`Type`-valued, computational dictionary: `DecEq`,
    `Monoid`, `Ord`). Many can exist on one carrier, so coherence is a **resolver
    convention**: **one canonical instance per (class, head-type)** in implicit
    search; **orphan instances = hard error** (an instance MUST be declared with
    its class or its head-type); **no overlapping instances**; **ambiguity = a
    compile error naming both candidates**, never a silent pick; **search
    terminates** (structural-decrease on the instance graph, the SCT family,
    `17 §4`).
- **Named instances are first-class values, passed explicitly.** An instance *is*
  a record value; you may define `byLength : Ord String` and pass it
  (`sortBy byLength xs`). *Implicit* search stays canonical + predictable;
  *explicit* passing is unrestricted. The resolver picks only one canonical thing
  silently; you may deliberately use any value. (The dependent-types escape hatch
  Haskell lacks — no `newtype` gymnastics.)
- **`derive (DecEq, Show)`** — an elaborator-generated structural instance for a
  `data`/`record`. **Generation is UNTRUSTED**: the generated instance is a
  *candidate the kernel re-checks*, never a trusted insertion.
- **Elaboration shapes:** `class` → a **record type** (right-nested Σ with
  definitional η, `13 §3`), whose **sort** (Ω vs `Type`) determines the kind; an
  `instance` → a **record value** including the **law proofs** (`law`/`verify`,
  `20-verification/21 §3`); `where C A` → an **implicit Π** over the class record,
  resolved by instance search.

Do **not** relitigate canonical-vs-local, orphan rules, overlap, the
named-value escape, or `derive`-is-untrusted. The only sanctioned reopening is
ADR 0008's **"Revisit if"** (a workflow needing *implicit* selection among
several structure dictionaries) — that is a **Steward escalation**, not a
build-time choice.

## 3. Deliverable outline (enclave elaborates `33 §5` + `39 §5` → team-ready)

Each item must land a concrete, implementable choice — not a survey.

1. **Class-declaration elaboration** — `class C (A : Type) { ops…; laws… }` → the
   record type. Pin the exact desugaring of the class body to Σ, how law
   propositions sit as fields, and how the record's **sort** (Ω vs `Type`)
   classifies the class **property vs structure** (the discriminant that governs
   whether the resolver convention applies at all).
2. **Instance-declaration elaboration** — `instance C T { … }` → a record value
   carrying the law proofs. Pin the **orphan check at declaration**: the instance
   must mention its class **or** its head-type `T`, checked **per-module** so
   canonicity is decidable and cannot be accidentally broken.
3. **Constraint elaboration** — `where C A` → implicit-instance-argument
   insertion at use sites and its discharge (`39 §5`). Pin how the implicit is
   inserted and threaded.
4. **Instance search (the algorithm)** — for **structure** classes: resolve THE
   canonical instance per (class, head-type); deterministic; overlap forbidden;
   two viable candidates → error naming both; **termination via SCT
   structural-decrease** on the instance graph (`17 §4`). For **property**
   classes: resolve to ANY instance (all Ω-equal). Pin the search procedure **and
   the termination metric**.
5. **Named-instance explicit passing** — a `named : C T` value passed explicitly
   bypasses search. Pin the surface + elaboration for explicit dictionary
   passing, and that it does **not** perturb implicit canonicity.
6. **`derive`** — which classes are derivable (structural: `DecEq`, `Show`, …),
   the generation procedure, and the **generation-is-a-candidate** discipline
   (kernel re-checks; a bad generated instance fails the kernel, not a soft
   check).
7. **Error reporting** — orphan-at-declaration, ambiguity-naming-both,
   no-instance-found, search-non-termination: each a **T1-protocol** diagnostic
   with source spans (`25-protocol`, the agent contract).

## 4. Acceptance criteria (discriminating — each must **flip**; non-degenerate pairs)

- **AC1 — coherence (structure).** The same (class, head-type) resolves to the
  **same** canonical instance at two distinct sites (program-wide stability —
  the property the law-carrying prover relies on).
- **AC2 — orphan reject (PAIR).** An instance declared **with** its class or
  head-type is **accepted**, **while** an otherwise-identical **orphan** instance
  (declared with neither) is **rejected at declaration**. Differ only in
  declaration locus.
- **AC3 — overlap/ambiguity (PAIR).** A single canonical instance **resolves**,
  **while** two overlapping instances for the same (class, head-type) produce a
  **compile error naming both** — never a silent pick.
- **AC4 — property vs structure (PAIR).** A **property** (Ω-valued) class with
  **two** instances resolves **cleanly** (all definitionally equal, no
  ambiguity), **while** a **structure** (`Type`-valued) class with two instances
  on the same head-type **errors**. The discriminant is the class's **sort** —
  not a flag.
- **AC5 — named explicit escape (PAIR).** A **non-canonical** `byLength : Ord
  String` passed **explicitly** is used at the call site, **while** *implicit*
  search at the same type still selects the **canonical** `Ord String`. Explicit
  and implicit paths are distinct.
- **AC6 — search termination (PAIR).** A well-founded instance chain **resolves**,
  **while** a non-decreasing / cyclic instance graph is **rejected** by the SCT
  bound (`17 §4`) — not a hang.
- **AC7 — `derive` is kernel-checked (soundness).** `derive DecEq` for a `data`
  type yields an instance the **kernel re-checks**; a deliberately-malformed
  generated instance is **caught by the kernel**, not admitted by a trusted
  insertion. *(Producer-grep: the derive path must emit its candidate through the
  real `declare_def`/kernel-check, never a trusted `env`-insert.)*
- **AC8 — lawful instance usable by the prover (verification).** a `Monoid`
  instance carries **assoc/unit proofs** as real record fields; the prover can
  **cite** them (the law field is a genuine kernel proof, not a stub).

## 5. Do-not-reopen guardrails

- **Coherence is ADR 0008 (Accepted).** No canonical-vs-local relitigation, no
  orphan/overlap loosening, no ambient local instances. Implicit-selection-among-
  several is the ADR "Revisit if" → **Steward escalation**.
- **No new kernel feature.** Reuse Σ/record (`13 §3`), Ω (`16 §1`), SCT
  (`17 §4`). A new kernel rule/judgment ⇒ mis-scoped (**subsume-don't-
  proliferate**).
- **`derive` stays untrusted-kernel-checked** — a generated instance is a
  candidate, never trusted.
- **Reuse the existing SCT termination** (`17 §4`) for instance search — do not
  invent a second termination checker.

## 6. Producer-grep gate (HIGH-signal — verify the real producer, not the test)

Every AC drives the **real** producers in `ken-elaborator` (class/instance
record desugaring, the orphan check, the instance-search procedure) and
`ken-kernel` (the re-check of `derive`d instances via the real `declare_def`
path, the SCT bound). **Never** a synthetic "class" literal, a hand-fed
dictionary, or a trusted `env`-insert standing in for kernel re-check. Grep the
producer, not the seed — a `derive` AC that inserts a ready-made dictionary and
re-checks a *downstream* consumer is green-vs-green.

## 7. Process (standard §2c)

- **Independence.** spec-author elaborates `33 §5` + `39 §5` → **`/spec` only**;
  conformance-validator authors `conformance/surface/classes/` → **`/conformance`
  only**.
- **3-reviewer merge Decision:** **Architect** soundness (coherence is
  **soundness-adjacent** per ADR 0008 — a lemma about "the `Monoid A`" unsoundly
  combined with data from a *different* `Monoid A` is the failure the canonicity
  convention prevents; the Architect gates that the enforcement is real);
  **conformance-validator** the Spec vote on `/spec`; **spec-author** Fidelity on
  `/conformance`. Real `@mentions` to actor_ids; thread under the kickoff.
- **Then Team Language builds** from the elaborated brief on `main`.
