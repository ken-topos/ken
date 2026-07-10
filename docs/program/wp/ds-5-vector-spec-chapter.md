# DS-5 · Length-indexed `Vector` — spec chapter (prerequisite to the package)

**Owned by the Steward** (frame); **home: Spec enclave** (spec-leader routing;
**Architect** owns the dependent-type design rulings; spec-author transcribes;
CV grounds conformance implications). Data-section item of
`wp/catalog-data-structures-program.md`, run as the **parallel Spec track** in
the operator's autonomous window (2026-07-10) alongside Foundation's DS-7 build
(program-doc `P3`: independent tracks parallelize when they don't contend — a
spec chapter and a catalog build do not).

## Why this is a spec chapter, not a package kickoff

`Vec n a` is **absent even from the spec** (`grep` finds no `Vector` anywhere in
`spec/`). It is a genuine spec-gap, not merely unimplemented. Per the program
doc, DS-5 is **spec-gated**: the chapter (spec-leader/spec-author + Architect)
lands first, then the package follows on Foundation. This WP is **the chapter
only** — no package, no `.ken` catalog entry.

## Goal

A new `spec/50-stdlib/` chapter (suggested `60-length-indexed-vectors.md`;
enclave picks the final number) specifying **length-indexed vectors** — the
canonical dependent-types showcase, high teaching value:

- The former `Vec : Nat → Type → Type` (an **indexed** inductive family — the
  length is an index that constructors refine, not a uniform parameter).
- Constructors `vnil : Vec Zero a` and `vcons : (n : Nat) → a → Vec n a → Vec
  (Succ n) a`.
- **Total `head : (n : Nat) → a → Vec (Succ n) a → a`** — the length index rules
  the empty case out *at the type level*, so `head` is total with no `Option`,
  no partiality. This is the showcase: the index makes a partial operation total.
- A safe **`index`/`lookup`** — the enclave decides the bounds discipline: a
  `Fin n` bounded-index type (spec-mentioned in `10-kernel/18a-primitive-registry`,
  not built), or a `Nat` index carrying an `IsTrue (lt i n)`/`Vec`-length proof.
  Pick the one that is *buildable on the landed elaborator* and states the
  totality cleanly; if `Fin` is the right answer it may itself need a small
  companion spec, name that dependency rather than assume it.
- **`zip : (n : Nat) → Vec n a → Vec n b → Vec n (Pair a b)`** — the length
  index guarantees the two vectors align, so `zip` is total and loses no
  elements (contrast the `List` `zip`, which must truncate). A second showcase.
- The laws worth pinning (e.g. `index`/`vcons` computation, `zip`/`map`
  naturality, a length/`toList` bridge) — the enclave scopes which are in-chapter
  vs deferred; keep the first chapter focused on the totality showcase.

## The one axis that MUST be ground-truthed first (buildability)

**Does the landed surface elaborate a true indexed inductive family?** A
*parameterized* `data` decl works today (`data Dec (P : Omega) : Type0`,
`EmptyDec.ken.md:66`) and a GADT-style `where` form exists in the surface
(`data EmptyAttempt : Type where { }`, `EmptyDec.ken.md:275`). But `Vec` needs
constructors that **specify a refined result index** (`vcons` targets `Vec (Succ
n) a`, `vnil` targets `Vec Zero a`) — a genuine index, not a uniform parameter.
Whether `crates/ken-elaborator/src/data.rs` accepts and correctly checks such a
family (result-type index refinement, the elimination motive over the index) is
**the load-bearing question** and must be settled against the landed code before
the chapter commits to a surface — do not assume it from the parameterized case
(`Equal`, the one landed indexed family, is **prelude-built in Rust**, not
surface `data`, so it is *not* evidence the surface path works). Ground every
axis the design touches (elaboration of the family decl, the dependent
eliminator/motive over the index, `head`'s index-refined domain, `zip`'s
two-index alignment) — a 2-line build-probe of a minimal `Vec`/`vcons`/`vnil` is
worth more than a from-one-axis ruling.

**If the surface indexed-family path does NOT elaborate today, that is a
first-class finding, not a blocker to route around silently:** the chapter still
specifies `Vec` (the spec is the design), and names the elaborator gap as a
prerequisite (a `data.rs` extension WP, re-forked to Steward → Kernel/Ergo).
Say so plainly per the honesty principle; do not spec a surface the elaborator
can't build and imply it's ready.

## Boundary / constraints

- **Spec chapter only** — no package, no catalog `.ken`. The Foundation package
  build (`Vec` entry) is a **follow-on** WP once the chapter lands.
- **`Ω`/Type discipline + zero-Axiom intent.** `Vec` is a real inductive with an
  eliminator, so its operations should be genuinely kernel-proved (the `List`/
  `Nat` posture), not postulated — flag any spot where the design would force an
  `Axiom` as a finding.
- **Reflect-don't-extend (#6), subsume-don't-proliferate (#7).** Reuse `Nat`
  (landed, inductive), the existing `data` machinery, and — if a bounded index
  is needed — prefer subsuming it into one clean `Fin`/proof story over
  proliferating ad-hoc bounds. A new *kernel* capability is permitted this run
  (boundary rules) **only if it is the right fix** and routed through the full
  Kernel ring + Architect gate; the chapter's job is to say clearly whether one
  is needed, not to pre-authorize it.
- **Wrap at 80; Mermaid for any diagrams; PRINCIPLES-charter reasoning** where
  the design has a fork the existing spec doesn't settle.

## Gate

Enclave build-out: Architect design rulings on the dependent-type forks (indexed
family surface, bounds discipline, totality statements) → spec-author transcribes
into the chapter → spec-leader coherence + CV conformance-implication grounding →
Architect fidelity gate on the committed chapter → `git_request` to Steward
(doc-only unless it touches conformance fixtures). Log every design call that
affects the language surface/elaboration/functionality in the handback so the
Steward records it for the operator. Escalate to me on any kernel-move fork
(the indexed-family elaborator gap is the likely one).
