# Analysis Digest — for the Ken language spec author

This digest synthesizes two source documents about the **Yon** topos-oriented
programming language and distills usable design ideas for **Ken**, a clean-room
descendant. The two sources:

1. **"Yon Programming Language Analysis.md"** — a long Kagi/AI research
   conversation written almost entirely from public web docs (`yon-lang.org`,
   an LLVM Discourse post, Hacker News). It is a strong *idea map* but
   factually unreliable about the actual system. Its central thesis — that
   "every value is an f64" is a damaging premature optimization — is its most
   sustained argument.
2. **"01-reality-check.md"** — a later five-pass source audit of the `1.1-dev`
   tree (dated 2026-06-21) that **refutes** the f64 thesis and corrects many
   wire/kernel/feature claims with `file:line` evidence.

**Reading rule for the spec author:** Treat the analysis as a source of
*design directions* and the reality-check as the *facts*. Where the analysis
states a "precise technical commitment," it is usually a reconstruction from
docs, not from code — flagged below. Ken is its own design; nothing here is a
mandate, but the corrections must not be re-imported as truth.

A note on provenance: the reality-check is explicit that its `file:line`
citations are commentary about AGPLv3 prototype code and must not be copied or
closely paraphrased into Ken. Carry the *understanding* forward; write Ken from
specs and tests.

---

## 1. The verification thesis

### (a) What the analysis proposed
A three-level hierarchy of static assurance:

| Level | Mechanism | Languages | What you get |
|---|---|---|---|
| **L0** | Tests (empirical) | Python, Ruby | "Works on these inputs" |
| **L1** | Static types (safety) | Rust, TypeScript, Haskell | "Well-typed terms don't go wrong" (Milner) |
| **L2** | Dependent types / proof | Lean 4, F*, Coq, Agda | "Term satisfies this proposition" |

The analysis placed Yon at **L1** but called it "the most *proof-adjacent* L1
language" because its semantic substrate is a topos (a proof-relevant
structure). It argued the gap from L1→L2 is "conceptually short but in practice
requires a completely different surface language, interaction model, and
compiler architecture." Market gap targeted: **verification for agentic code
generation** — agents write code that compiles but is wrong (the L2 problem);
existing strict languages (Rust/TS) only catch the L1 problem. The analysis's
distinctive framing: current target languages are "human interface languages,"
and the verification problem is the central obstacle to trustworthy agentic
output.

### (b) What the reality-check corrected
The L1-vs-L2 framing is correct **for the surface programmer** — you cannot
today attach a postcondition, refinement, or correctness proposition to your
own function; `proposition` is a runtime three-valued (Heyting/Ω) computation
decided by place visibility, *evaluated, never proven*. There is no tactic
language and no refinement types on arrows.

**But the kernel is partway to L2, not pure L1** — the analysis was written
before the dependent kernel existed and is "materially out of date." The
reality-check's single biggest reframing: *the L1→L2 task is "plumbing an
already-dependent kernel to the surface plus generalizing existing prover
backends," not building a proof system from nothing.*

### (c) Distilled idea for Ken
Adopt the L0/L1/L2 hierarchy as the organizing spine of Ken. The product
thesis ("verified language for agentic development") is sound and is the core
to build around. Design Ken from day one so the kernel's dependent/proof
capability is **reachable from the surface** — do not repeat Yon's split where
a powerful kernel sits under a surface that cannot express propositions. The
agentic-diagnostics angle (section 8) is the differentiator.

---

## 2. Core type theory (the trusted kernel)

### (a) What the analysis proposed
The analysis said little concrete about a kernel calculus — it worked from docs
that described "elementary topoi, Heyting algebras, directed type theory" and
HoTT-style universes (`Type`, `Type_0`, `Type_1`, …). It noted Yon 1.0 had
"type parameters on functions and HoTT-style universes" and a `place P [over X]`
syntax it read as a **slice/fiber (dependent type)** construction = display
maps. It speculated Yon lacked dependent *products* (Π, the type of sections of
a fibered place) while having dependent *sums* (Σ, the total space) implicitly.
It described the topos's full internal type theory as the **predicate
fibration** (predicates → Ω) plus the **type fibration** (display maps), linked
by **comprehension** (predicate→type, a pullback of `true`) and **image
factorization** (type→predicate), citing the "Native Type Theory" account and
the adjunction triple Σ_f ⊣ f* ⊣ Π_f.

### (b) What the reality-check corrected
The kernel had moved far beyond what the analysis saw. Confirmed present and
working in `1.1-dev`:

- **Genuine dependent Π** (dependent functions).
- **Endpoint-aware identity type `Id`** with real compile-time path equality.
- **`J` / path-induction eliminator** (test green).
- **SCT-certified δ-conversion**: size-change termination (SCT) gates
  definitional unfolding — i.e., conversion checking unfolds definitions only
  when SCT certifies termination. The "δ-debt" roadmap item is *closed* on this
  branch.
- **Yoneda mechanically verified** by kernel oracles (the analysis's "no
  mechanical verification" describes the pre-1.1 state).

The reality-check does not enumerate Σ/universe predicativity details, but
establishes the kernel is dependent, has Id/J, and does real definitional
equality (conversion) with a termination-gated δ rule. The de Bruijn-criterion
intent (a small trusted checker that re-checks proofs) is implied by the oracle
suite (`test_j_tarski`, `test_path_core 15/15`, `test_yoneda_typed 5/5`,
`test_yoneda_lemma 11/11`, etc.).

### (c) Distilled idea for Ken
Specify a small trusted kernel with: terms, types, a universe hierarchy
(decide and *state* predicativity/cumulativity — open decision, §10),
dependent **Π**, dependent **Σ** (pairs), an **identity/path type `Id`** with a
**`J`** eliminator, and **inductive types** with reducing eliminators. Specify
the **typing judgment** and the **conversion (definitional-equality) judgment**
separately; make conversion's δ-rule (definition unfolding) gated by a
termination criterion (SCT is the proven-workable choice). Choose an
**evaluation strategy** — NbE (normalization by evaluation) is the natural fit
for a dependent kernel and is an open decision to confirm (§10). Keep the
checker small enough to satisfy a de Bruijn criterion: the trusted core only
*checks* fully-elaborated terms.

---

## 3. Cubical / HoTT layer

### (a) What the analysis proposed
Largely implicit. The analysis treated content-addressing as "extensionality
made physical" and the Yoneda lemma as a proof technique, and mentioned
"directed type theory" (Riehl–Shulman) as a possible foundation giving directed
paths-as-proofs. It did not describe a working cubical machinery — it argued
only that the topos substrate *could* support L2.

### (b) What the reality-check corrected
A **computing cubical core** already exists in `1.1-dev` (this is the most
striking correction to the "aspirational only" reading):

- **the interval** and **transport**;
- **comp** (composition) and **hcomp** (homogeneous composition);
- **Glue types** with **univalence-as-computation** (the reality-check notes the
  surface even "computes surface univalence through Glue");
- **higher inductive types (HITs)** with *reducing* eliminators;
- **`isEquiv`** with coherence checked on real terms.

Oracle suite observed green: `test_glue 19/19`, `test_isequiv 11/11`,
`test_path_core 15/15`, `test_hit_compute 5/5`. Separately, surface cleanup work
gave "genuine surface hcomp/comp" via face/partial notation lowering to
`Ast.HComp`/`Ast.Comp`.

So per the reality-check: **claimed-working, not aspirational** — interval,
transport, comp/hcomp, Glue, univalence-via-Glue, HITs, isEquiv all compute.

### (c) Distilled idea for Ken
A cubical core is a proven-feasible foundation for the equality story Ken needs
(path equality, transport, univalence). Specify the interval, path types,
`transport`, `comp`/`hcomp`, `Glue`, univalence, and HITs as kernel features
with *computing* (reducing) eliminators, and `isEquiv`/equivalences as the
bridge for univalence. This is the route that makes "extensionality" a theorem
rather than a runtime-heap accident. Whether Ken commits to *full* cubical type
theory vs. a lighter HoTT-with-Id core is an open decision (§10).

---

## 4. Topos / internal-logic layer

### (a) What the analysis proposed
This is the richest seam in the analysis. Core claims:

- A **subobject classifier Ω** is the truth-value object; in a topos Ω is a
  **Heyting algebra** (intuitionistic), Boolean only in special cases.
- `unknown` is a **first-class citizen** of the logic — the Heyting negation
  element, *not* classical `false`, and crucially **`¬¬φ ≠ φ`** in general.
- **Propositions = morphisms into Ω** (predicates); **comprehension** =
  `{x:A | φ(x)}` is the pullback of `true` along φ; quantifiers ∃_f/∀_f are
  left/right adjoints to pullback (substitution).
- **Yoneda lemma** as the identity principle: "a thing is what you can observe
  of it" — content-addressing makes this "physical."
- **Typeclasses-from-Ω**: a "typeclass" is a subobject of the universe (a prop
  `Hashable(T): Bool` quantified by a `where` clause that restricts to the
  subobject Ω classifies) — "the most category-theoretically faithful account
  of typeclasses possible."
- Presheaf/sheaf semantics: places are objects, a place's interface is its
  **presheaf of observations** (its arrows), "no typeclasses; a place's arrows
  *are* its interface."
- The internal logic surfaces to the programmer through `unknown`, `is`/`is
  not` guards, and (aspirationally) refinement predicates.

### (b) What the reality-check corrected
- `proposition` is real but is a **runtime three-valued (Ω) computation decided
  by place visibility** — evaluated, never proven. There is no way to *prove* a
  proposition about your own function at the surface.
- The structured-error model exists: `error E subcontains Base` subobject
  places + tri-valued logic + `is`/`is not` guards (note: the implemented
  keyword is `subcontains`, NOT `extends`, despite some doc/example filenames).
- **Yoneda is mechanically verified** by kernel oracles (not merely
  philosophical).
- Law/property verification exists via `law`/`verify` + an MLIR
  `AlgebraVerifier`.

### (c) Distilled idea for Ken
Keep the topos internal logic as Ken's logical foundation, but **surface it as a
real proof logic**, not only a runtime tri-valued evaluator. Specifically:
- Ω as a Heyting algebra with first-class `unknown` (intuitionistic; `¬¬φ ≠ φ`).
- Propositions-as-(morphisms-into-Ω); comprehension/refinement subtypes
  `{x:A | φ(x)}`.
- The **typeclasses-as-subobjects-of-the-universe** idea is genuinely elegant
  and worth adopting as Ken's constraint/trait mechanism (a `where φ(T)` that
  restricts to the Ω-classified subobject) — note Yon does NOT have open
  user-defined typeclasses, so this is greenfield.
- Yoneda/identity as a verified principle, not decoration.
The reality-check's caution: do not conflate "runtime Ω evaluation" with
"machine-checked proof." Ken must offer the latter at the surface.

---

## 5. Runtime & evaluation

### (a) What the analysis proposed
- A **content-addressed heap**: same content ⇒ same slot ⇒ O(1) structural
  equality by a single comparison; deduplication is global.
- An interpreter/runtime where **every value is an f64 handle** into this heap
  (the central thesis — see §6).
- **No GC** (heap append-only, "slots stable for life of heap, heap for life of
  process"); reclamation only via the ephemeral `spawn`/`promote` model
  (munmap an ephemeral arena).
- The **"slot ceiling"**: 196,560 slots/heap = the **kissing number of the
  Leech lattice Λ₂₄** (count of minimal/type-2 vectors); 256 heaps/chain
  (8-bit id, 24-bit slot); ~50.3M distinct contents/process; ~64 MB arena/heap;
  ~17–18 GB content/process. The analysis treated this as a hard capacity bound
  and connected it (speculatively) to Co₀/Conway-group canonicalization and
  Golay-code error correction. It read **heap addressing as Leech-lattice
  geometry** ("Co₀-orbit canonicalization collapses equivalent heap contents
  under Leech-lattice symmetries").
- "Loud refusal over silent degradation" at limits.
- No surface runtime introspection; proposed a coalgebraic `witness` mechanism
  to expose heap stats extensionally.

### (b) What the reality-check corrected
- **Value representation is heterogeneous typed lowering, NOT uniform f64** —
  see §6. Scalars are unboxed SSA values, not heap traffic.
- **Heap addressing is FNV-1a hashing + memcmp, NOT lattice geometry.** The
  Leech quantizer is never on the allocation path; slot ids are a monotonic
  counter. The lattice has **three distinct, code-separate roles**:
  1. **Golay(24,12,8)** error-correction (VoyagerList);
  2. the **kissing number as a fixed-size bitmap/MPHF domain** (the "XSet" —
     196,560 bits ≈ 3073 words);
  3. **Co₀/M24 orbit canonicalization** (separate `leech_orbits` /
     `yon_curtis_canon` code, not the allocator).
- The slot/heap/process numbers are **confirmed** (196,560 slots/heap, 256
  heaps/chain, ~50.3M ceiling, 64 MB arena, ~17–18 GB/process), and "loud
  refusal at limits" is a real design principle. Auto-chaining + global dedup
  via a process-wide `(root,hash64)→HeapRef` index is real.
- **No automatic GC/compaction**, but manual reclamation exists
  (`clear`/`reset`/`strip_trim` via `madvise(MADV_DONTNEED)`). Dedup means real
  burn is "one slot per *distinct* value," not per occurrence.
- **C-level introspection exists** (`yon_rt_heap_occupancy`, checkpoint
  snapshots) but **no surface primitive** exposes it. (So the analysis's
  `witness` proposal targets a real gap.)
- Termination: the runtime's relevant termination machinery for the *kernel* is
  **SCT for δ-conversion** (§2), which the analysis never mentioned.
- The Co₀ orbit cardinalities (98280/8386560/8292375) cited in docs are **not**
  present as source constants — an unverified doc claim.

### (c) Distilled idea for Ken
- Keep the **content-addressed heap with global dedup** — it is genuinely
  valuable (O(1) structural equality, automatic sharing). But make it a
  **storage substrate for compound/identity-bearing values**, not the universe
  of all values.
- **Do NOT make the heap addressing depend on Leech-lattice geometry.** Use a
  conventional hash (FNV-1a-style) + memcmp. If Ken wants the Leech/Golay/Co₀
  machinery at all, scope it to the *three separate, optional* roles the
  reality-check identified (error-correcting lists; a fixed-size set/bitmap
  domain; orbit canonicalization) — never the allocation hot path.
- **Reconsider the capacity bound.** 196,560 is an aesthetic choice tied to Λ₂₄,
  not a necessity (a 24-bit slot field holds ~16M). Ken can keep a "loud refusal
  over silent degradation" *philosophy* while choosing a capacity bound on
  engineering grounds (encoding width) rather than lattice numerology. This is
  an open decision (§10).
- Provide **surface heap introspection** as a first-class, extensional-safe
  facility (process-level stats: slots used, dedup rate, arena bytes, Merkle
  root — never per-value identity/provenance, which would break
  extensionality).
- Specify **SCT (size-change termination)** as the termination criterion gating
  definitional unfolding in conversion.

---

## 6. Numerics & primitive types — THE BIG CORRECTION

### (a) What the analysis proposed (and got wrong)
The analysis's central, most-developed thesis: **"f64-uniform representation is
the most consequential premature optimization in the language."** It argued:
- every Yon value is an f64 handle (NaN-boxing-style), so the runtime never
  type-dispatches;
- this wastes the GPR register file (everything in XMM), corrupts `Int` above
  2^53, blocks strings on the wire, forces every distinct scalar to consume a
  heap slot, and is the root cause of ~20 downstream problems (wire narrowness,
  serialization, FFI, sum types, networking, debugging, GC);
- the fix is **typed handles**: scalars become immediate machine values
  (`f64`→XMM, `Int`→i64 GPR, `Bool`→i1, `Handle`→ptr/i32, `Array`→memref) and
  only compound structures live on the content-addressed heap.
It supported this with the 2026 Watt "type tag checking" paper and argued the
MLIR pipeline *already* re-types everything, so the uniform-f64 layer serves no
purpose and is "erased by the first lowering pass."

### (b) What the reality-check refuted
**REFUTED — the premise is backwards.** The OCaml frontend emits
**heterogeneously typed** MLIR from the start; the type converter maps each
surface type to a conventional ABI type:

| Surface type | MLIR/LLVM type |
|---|---|
| `number` / `money` | `f64` |
| `boolean` | `i1` |
| `proposition` (Ω, tri-valued) | `i8` |
| `section` (place instance / handle) | `i64` = `(heap_id:u32<<32) \| slot` |
| `probe` (closure) | `struct{i32,i32}` |
| `heyt_int` | `struct{i64,i64}` |

Scalars are unboxed SSA values (`arith.addf`, `arith.cmpf`, `arith.andi`), never
heap traffic. Booleans use i1 GPRs, handles use i64 GPRs; only floats use XMM.
Raw pointers exist (`!llvm.ptr`). **There is no uniform-f64 stratum that a pass
decodes away.**

**The one grain of truth:** when a `section` *handle* crosses a function or
Space boundary it is shuttled as an f64 via `sitofp` of its i64 slot index — a
**wire convention** ("handles travel as f64"), plain integer-in-double, **not**
NaN-boxing, and *not* "every value is an f64."

**What actually survives of the f64 argument:** there is **no distinct
fixed-width `Int` or decimal type**. The single surface numeric type is
`number`, lowered to `f64`, so integer-valued computation silently loses
precision above 2^53. This is a **missing-type problem, narrow and contained**
— not a representation-ontology problem.

> The reality-check's explicit consequence: the analysis's largest proposed
> workstream — "abandon f64, adopt typed handles, redesign runtime/wire/heap" —
> **should not be undertaken.** It targets a problem that does not exist.
> Replace it with a small, contained work package: **add `Int` (and optionally
> `Decimal`) as first-class types.**

### (c) Distilled idea for Ken
- **Ken has `Int` from day one** as a first-class fixed-width (or arbitrary-
  precision) integer type — do NOT fold integers into a float type. This is the
  one real defect the analysis correctly identified.
- Ken should have a clear primitive-type story: `Int` (decide fixed 64-bit vs.
  arbitrary precision — open decision, §10), `Bool`, a real `Float`/`f64`,
  optionally `Decimal` for money, and distinct `Handle`/heap-reference and
  pointer types. Use heterogeneous typed lowering (the prototype already does
  this correctly).
- **Reals/floats:** keep IEEE `f64` as a *numeric* type, honestly named, not as
  a universal value carrier. Be explicit that ℝ does not embed faithfully into
  f64 (the analysis's one fair caveat).
- **Do not** build Ken around the false "uniform f64" ontology, and do not
  inherit the wire-as-4×f64 framing (§7) as a fundamental constraint.

---

## 7. Surface language

### (a) What the analysis proposed
Reconstructed surface vocabulary (from docs — treat as approximate):
- **`place`** = product type / struct (an object of the topos); declarations
  `place P { field: T, ... }`, with `place P [over X]` for a fibered/slice
  (dependent) construction.
- **`view`** / **`move`** / **reduction** = arrows (morphisms); a view is a pure
  function `A → B`. "Behaviours" are arrows.
- **`Space`** = the mutation/process escape hatch (mutable cells with identity;
  isolated processes; the 1.0 mutation mechanism). `becomes` = surface sugar for
  cell update; a mutable variable *is* a cell.
- No sum types (analysis flagged this as the top gap), no typeclasses, no effect
  system, no `match`/`case`, no module system, no Result/Option (the `0.0`
  handle convention for failure).
The analysis then *proposed* many extensions (all greenfield ideas, not Yon
features): coproduct/sum places (`Result`/`Option`); Kleisli/effect arrows
(`effect view f(...): F<B>`, an effect signature `place AWS {...}`, or lighter
`effects [http, auth, clock]` annotations); linear/affine arrows (`linear move
next_page(token): PageResult`); graded resources (semaphores
`Semaphore[N] → Semaphore[N-1]`); natural-transformation protocol adapters /
"protocol views"; closure/slice places carrying config; a `Bytes` place;
`foreign` declarations for FFI with `linkage`/`library`/`pure`/`impure`; a
`witness` co-observation for runtime stats; `posix_process` with `on signal`
coalgebraic handlers and `boundary` annotations; delimited continuations via
`shift`/`reset`; refinement-typed view arguments
(`view f(list: List<f64> | ∀e ∈ list. e > 0)`); and `prove:`/`assume:` REPL
forms. Sample syntax appears throughout the source conversation.

### (b) What the reality-check corrected
Much of what the analysis called "missing" **already exists** in `1.1-dev`:
- **Module/import system**: file import, qualified import with alias,
  cross-package `from Space` RPC import.
- **Package manager + build**: `pkg/yon-pkg`, git-based, `yon.toml`/`yon.lock`
  with pinned commits.
- **Effect system**: `visits`, statically checked, transitively inferred (NOT
  the analysis's hypothetical Kleisli scheme — a real, simpler effect-tracking
  mechanism). NB: `requires` is *capability tokens only*, not refinement.
- **Hindley-Milner polymorphism + dependent types** both present.
- **Full tooling**: formatter `yonfmt`, linter `yon_lint`, doc-gen `yon_doc`,
  LSP `yon_lsp` + nvim/vscode clients.
- **TCP sockets** (`Wire.*_net`); **serialization** (`yon_rt_serialize/
  deserialize` + Merkle); structured error model (`subcontains` subobject places
  + tri-valued logic + `is`/`is not`); **33 stdlib modules** (not 20).

Genuinely missing (analysis correct): FFI (external symbols are a fixed
prefix-gated allowlist, no general C/BLAS); `Bytes`/binary I/O (File is
text-only); `Result`/`Option`/`Either`; linear/affine types; delimited
continuations/coroutines; `Stream.flatMap`/`bind`; crypto beyond FNV-1a;
HTTP/TLS; `match`/`case` + exhaustiveness; **open user-defined typeclasses**;
interactive debugger; runtime reflection.

Partial/stubbed: **sum types** (`A | B` *parses*, a `variant` AST node exists,
but lowers to an opaque base type with **no constructor/eliminator** — the
single most important thing to *finish*); `pushout` expression (lowers to
`0.0`); `with multishot` (parsed, reaches an MLIR attribute, but the reducer
ignores it — no real multi-resume; effect handlers are tail-resumptive in-place
substitution, not reified continuations); `requires` capabilities (runtime
FNV-1a gate only, static check not wired).

Surface-keyword corrections worth carrying: the real op is **`Land.reach`** (not
`Magma.reachable`); the implemented keyword is **`subcontains`** (not
`extends`); real `Magma` ops are `empty/gen/is_commutative/is_associative/
identity/closure_size/word_push/normal_form/from_catalog` (the analysis's
`subsetsum`/`knapsack`/`solve P`/"P=NP era" Magma op list is **retracted
material**).

### (c) Distilled idea for Ken
- Ken needs **sum types / coproducts (`Result`, `Option`, `Either`) as
  first-class, with constructors and eliminators** — the prototype's
  parsed-but-opaque path is the prime cautionary tale: *finish this in Ken from
  the start.* Add `match`/`case` with exhaustiveness checking (also missing).
- Keep core surface vocabulary clean-room but informed by the prototype:
  product types, arrows/behaviours as the function notion, an effect system
  (the prototype's `visits`-style statically-checked, transitively-inferred
  effects are simpler and more proven than the analysis's Kleisli proposal —
  prefer that shape), and a state/mutation escape hatch analogous to `Space`.
- The analysis's **proposed** extensions are idea fodder, not requirements.
  High-value, spirit-aligned: refinement-typed arrow arguments (the route to
  L2 at the surface); typeclasses-as-subobjects (open user-defined typeclasses
  are genuinely missing); `Bytes`; a principled FFI surface. Lower-priority /
  research: linear/affine arrows, delimited continuations, the whole
  coalgebraic `Space`-enrichment program (below).
- The analysis's **coalgebraic layer** (Store-comonad cells/lenses, process
  coalgebras + bisimulation, profunctor wires, co-Heyting boundaries for
  signals/concurrency/continuations) is intellectually rich but the
  reality-check classed it **research track, not core** — "partly subsumed by
  `visits` + Space." Treat it as optional future research for Ken, not spec
  baseline.

---

## 8. Proof automation & diagnostics

### (a) What the analysis proposed
The analysis's strongest technical contribution. Key claims:

- **SMT solvers (Z3, cvc5) are classical; the topos logic is intuitionistic.**
  You *cannot* naively encode Ω-valued propositions into Z3, because Z3 will use
  excluded middle / double-negation elimination and "prove" things false in the
  topos. F*'s direct-classical-encoding approach is unsound for a genuinely
  non-Boolean Ω.
- **A fragment classifier** splitting each proof obligation into:
  - **Fragment D (decidable)** — `φ ∨ ¬φ` holds (scalar/handle equality,
    arithmetic comparisons, Boolean predicates, finite membership). Classical =
    intuitionistic here → **direct Z3 encoding**.
  - **Fragment N / FO (¬¬-stable / first-order intuitionistic)** — use a sound
    translation to classical FOL, then Z3.
  - **Fragment I / HO (full/higher-order intuitionistic)** — no SMT; manual
    Lean/Agda-style tactics (+ Itauto-style native intuitionistic SAT for the
    propositional skeleton).
- **The Kripke-embedding recommendation** (the analysis's headline idea): rather
  than the SMTCoq "solve classically, then re-check the proof certificate
  intuitionistically" pattern, **express Kripke semantics as a classical
  first-order theory** so that `φ` is intuitionistically valid iff `φ#` is
  classically valid. Then Z3 solves `φ#` directly and the result is sound *by
  the translation's correctness* — no per-certificate re-checking. The
  justification: **Yon's topos semantics ARE Kripke semantics** (a place is a
  "world," the slice category is the accessibility relation), so the embedding
  is the language's native meaning, not an encoding trick. Cost: the embedding
  adds a `World` sort and +1 arity to every predicate, slowing Z3; reserve it
  for Fragment FO, with Fragment D using direct encoding.
- It catalogued alternatives: native intuitionistic solvers (**Itauto**,
  **intuit**), **Herbrand constructivization** (classical proof → intuitionistic
  proof via expansion proofs), the **Kripke-as-classical-FOL embedding**, and
  proof-reconstruction systems (**SMTCoq**, Sledgehammer, Lean-SMT). It argued
  the "classical oracle + intuitionistic recheck" pattern is *expedient* (the
  CDCL SAT engine is the performance bottleneck and is inherently classical) but
  **not the natural structure** — for decidable predicates classical and
  intuitionistic coincide, and for first-order goals the Kripke embedding moves
  the problem to formula level (no recheck).
- **Proof-failure diagnostics** (the agentic differentiator), four mechanisms
  inherent to the topos structure:
  1. **Kripke countermodels** as diagnostic witnesses — a finite model showing
     *why* a proof fails and *what would fix it*; richer than a classical
     counterexample because it distinguishes "false" from "unknown."
  2. **Typed holes with provenance** (the Hazel "total error localization and
     recovery" model) — an unprovable obligation becomes a typed hole; the
     program still type-checks and runs, producing `unknown`, so downstream
     consequences are visible.
  3. **Subobject decomposition** — the Heyting **three-region map**: proved-true
     (`S_φ`), proved-false (`¬S_φ`), and **unknown** (`¬¬S_φ \ S_φ`), precisely
     characterizing the gap.
  4. **Slice-category contextualization** — "your claim fails in the base topos
     but holds in slice `Place/Y`; here is the bridge obligation."
  It proposed emitting these as machine-parseable JSON for agents
  (`countermodel`, `subobject_decomposition`, `hole`,
  `slice_contextualization`, `suggested_actions`).

### (b) What the reality-check corrected
- **Z3 AND Coq backends already exist** in `1.1-dev` (`naturality_smtcheck.ml`,
  `naturality_coqcheck.ml`, gated by `YON_F2D`/`YON_F3`) — but **only** for
  naturality of natural transformations with single-variable Real-arithmetic
  bodies. So the prover is real but narrow; the task is to **generalize existing
  backends**, not build from scratch.
- The reality-check explicitly **adopts** the Kripke-embedding idea as "the
  strongest technical idea; now directly applicable" and the proof-failure
  diagnostics (countermodels, holes, three-region Heyting) as "sound; high
  value for agents — the agentic differentiator."

### (c) Distilled idea for Ken
This whole section is **high-priority, adopt as core** for Ken:
- A **fragment classifier** (D / FO / HO) that routes obligations to the
  cheapest sound method.
- A **Kripke-semantics-as-classical-FOL embedding** as the primary sound bridge
  to a classical SMT solver (Z3/cvc5), with direct encoding for the decidable
  fragment and manual/tactic + native-intuitionistic fallback for the
  higher-order fragment. Specify the embedding's soundness obligation.
- **Generalize beyond the narrow naturality/Real-arithmetic domain** the
  prototype's backends handle.
- **Proof-failure diagnostics** as a first-class, machine-parseable output:
  Kripke countermodels, Hazel-style typed holes with provenance (program runs
  with `unknown` for open obligations), the Heyting three-region decomposition,
  and slice contextualization — plus `suggested_actions` for agents. This is the
  feature that most differentiates Ken for agentic use.

---

## 9. Tooling, modules, packaging

### (a) What the analysis proposed
The analysis declared modules, package management, build system, FFI, test
framework, LSP/formatter/linter, REPL/incremental compilation, and structured
serialization all **absent** — "the true Layer 0 prerequisite." It proposed a
GHCi-style REPL (compiled modules + JIT-interpreted expressions via LLVM ORC,
incremental type environment, ephemeral `spawn` Spaces for exploratory eval),
and floated content-addressed packages (Merkle-based dedup across processes) and
"version compatibility as homotopy."

### (b) What the reality-check corrected
Most of these **already exist** (analysis outdated):
- **Module/import system** (file + qualified-with-alias + cross-package
  `from Space` RPC import).
- **Package manager + git-based build** (`pkg/yon-pkg`, `yon.toml`/`yon.lock`
  with pinned commits).
- **Serialization** (`yon_rt_serialize/deserialize` + Merkle).
- **Full tooling**: `yonfmt`, `yon_lint`, `yon_doc`, `yon_lsp` (+ nvim/vscode
  clients).
- **Content-addressed storage** is real (global dedup index) and a genuine
  feature.
Still genuinely missing: FFI (general), `Bytes`/binary I/O, an interactive
debugger, runtime reflection, a REPL.

### (c) Distilled idea for Ken
Ken should ship: a module/import system, a package manager with lockfiles
(content-addressed packages are a natural fit given the heap — keep this idea),
build tooling, formatter/linter/doc-gen/LSP, and a general FFI. **Content-
addressed storage is a marketable feature** (deterministic builds, dedup,
Merkle verification) — keep it. A REPL is worth specifying as the *primary UI
for the proof system* (the analysis's "Little Prover/Little Topologist"
pedagogy — `prove:`/`assume:` forms turning proof failures into conversations is
a strong onboarding story; the reality-check tags REPL/pedagogy "adopt in
ergonomics phase"). The hardest REPL piece is incremental type-checking
(medium, not novel).

---

## 10. Concrete open design decisions / forks

Each item below is a genuine fork with materially different futures that a Ken
spec author must resolve or escalate.

1. **Integer type.** `Int` fixed-width 64-bit (matches ABI, fast, but overflow
   semantics to define) vs. arbitrary-precision `Int` from day one (safer for a
   verified language; perf/representation cost). Plus: is `Decimal` (money) a
   core type? The reality-check flags the missing `Int`/`Decimal` as *the* real
   numeric gap.

2. **Kernel evaluation strategy.** NbE vs. another reduction strategy for the
   dependent kernel / conversion checker. (NbE is the natural fit but unstated.)

3. **Universe discipline.** Predicative vs. impredicative; cumulative vs.
   non-cumulative; explicit `Type_i` levels vs. typical-ambiguity/universe
   polymorphism. (Analysis mentions HoTT-style `Type_0/Type_1/…` but no
   commitment.)

4. **Cubical commitment.** Full cubical type theory (interval, comp/hcomp, Glue,
   computing univalence, HITs — all proven feasible in the prototype) vs. a
   lighter HoTT-with-`Id`/`J` core. Trade-off: full cubical buys computational
   univalence and HITs at the cost of a larger kernel.

5. **Heap capacity bound.** Keep a Λ₂₄-derived 196,560 slot ceiling (aesthetic,
   ties to optional Co₀/Golay machinery) vs. choose a capacity bound on
   engineering grounds (24-bit field ≈ 16M; wider encoding for billions). The
   "loud refusal" philosophy can survive either choice. The reality-check shows
   the lattice is NOT load-bearing for addressing, so this is genuinely free.

6. **Whether to include Leech/Golay/Co₀ machinery at all**, and if so in which
   of the three *separate* roles (error-correcting lists; fixed-size set/bitmap
   domain; orbit canonicalization). None is required for the core language.

7. **Scalars on the heap vs. immediate.** The prototype already keeps scalars as
   unboxed SSA values (the analysis's worry was unfounded), but Ken must still
   *specify* which values are content-addressed (compound/identity-bearing) vs.
   immediate, and the equality story for each (hash-equality for handles vs.
   native comparison for scalars).

8. **Effect system shape.** Adopt a `visits`-style statically-checked,
   transitively-inferred effect system (proven in prototype) vs. the analysis's
   richer Kleisli/monadic effect arrows vs. algebraic-effects-with-handlers.
   Related: are capabilities (`requires`) static or runtime? (Prototype's are
   runtime-only.)

9. **Continuations / multishot.** Tail-resumptive handlers only (prototype's
   real behavior) vs. genuine reified/multishot continuations (parsed-only in
   prototype). Research-grade.

10. **Sum-type semantics.** How constructors/eliminators, exhaustiveness, and
    GADT-like indexing work — greenfield, since the prototype's path is opaque.

11. **Surface proof interface.** Refinement types on arrows (`requires`/`ensures`
    as real propositions) vs. a separate tactic/proof language vs. both. The
    prototype has neither at the surface.

12. **SMT integration strategy.** Kripke-embedding (analysis's recommendation;
    reality-check concurs) vs. SMTCoq-style certificate re-checking vs. Herbrand
    constructivization — and which solver(s) (Z3 vs. cvc5 vs. both; keep Coq
    backend?).

13. **Cross-process wire.** The prototype has a real DTO/64KB-frame structured
    stream that already carries strings + nested sections; the *real* gaps are
    LIST/MAP wire tags (reserved, unimplemented) and structured `promote`
    (scalar-only today). Ken must decide the wire data model — and must NOT
    inherit the false "4×f64 ceiling."

14. **Concurrency / `spawn` model.** `spawn` is plain `fork()` (COW), not an
    isolated-arena/munmap design; transport is POSIX shared memory
    (`shm_open`+`mmap`), so isolation is logical (copy-on-receive), not
    shared-memory-free. Ken must choose its concurrency/isolation model
    deliberately.

15. **Coalgebraic-`Space` research program.** Whether to pursue the analysis's
    Store-comonad cells, process coalgebras + bisimulation, profunctor wires,
    and co-Heyting boundaries (signals, sync, continuations) at all — flagged
    research-track, not core.

16. **Surface runtime introspection.** Expose process-level heap stats /
    Merkle-root `witness` (extensional-safe) — yes/no and exact surface.

---

## 11. Reality-check corrections summary (do NOT repeat these web-doc misconceptions)

Bullet list of every analysis claim the reality-check found WRONG or overstated,
with the corrected fact:

- **"Every value is a uniform f64 (NaN-boxing); it's the central premature
  optimization."** → **REFUTED.** Heterogeneous typed lowering from the start
  (`f64`/`i1`/`i8`/`i64`/`struct`); scalars are unboxed SSA values, never heap
  traffic. The *only* truth: a `section` handle travels across boundaries as an
  f64 via `sitofp` of its i64 slot — a wire convention, plain integer-in-double,
  not NaN-boxing.
- **"Integer precision is an f64 representation flaw."** → Partly: the real
  issue is a **missing `Int` type** (the single surface numeric `number` →
  `f64`), narrow and contained — not a representation ontology.
- **"Cross-Space wire is 4×f64 = 32 bytes; strings/sections can't cross; it's
  the fundamental ceiling."** → **REFUTED.** Two transports: an ~190-byte RPC
  mailbox with **up to 8 f64 args** + reply, and a **DTO/64KB structured frame
  stream** that serializes scalars, **strings** (`[len][bytes]`), and **nested
  sections** by value. Strings/structs already cross. Real gaps: LIST/MAP wire
  tags (reserved, unimplemented) and structured `promote` (scalar-only).
- **"Actor model with no shared memory."** → **FALSE.** Transport is POSIX
  shared memory (`shm_open`+`mmap(MAP_SHARED)`, process-shared mutex/cond);
  isolation is logical (copy-on-receive), not absence of shared pages.
- **"`spawn` is an isolated arena reclaimed via munmap."** → It is plain
  `fork()` (COW). (Crash-respawn on a virgin channel with advanced epoch IS
  real and correct.)
- **"Yon sits at L1 with no dependent types; a prover must be built from
  scratch."** → **OUTDATED.** Kernel has genuine dependent **Π**, identity type
  **`Id`** with compile-time path equality, **`J`** induction, a **computing
  cubical core** (transport, comp/hcomp, Glue, univalence-as-computation, HITs,
  `isEquiv`), and **SCT-certified δ-conversion**. L1 is only the *surface*
  status over an L2-capable kernel.
- **"No mechanical verification of Yoneda."** → **OUTDATED.** Yoneda is
  mechanically verified by kernel oracles (`test_yoneda_typed`,
  `test_yoneda_lemma`).
- **"No prover exists."** → **WRONG.** Z3 *and* Coq backends are already wired
  (gated by `YON_F2D`/`YON_F3`) — but only for naturality of natural
  transformations with single-variable Real-arithmetic bodies (generalize, not
  build).
- **"Heap addressing uses Leech-lattice geometry / Co₀-orbit canonicalization on
  the allocation path."** → **WRONG.** Addressing is FNV-1a + memcmp; slot ids
  are a monotonic counter; the Leech quantizer is never on the allocation path.
  The lattice has three *separate*, code-distinct roles: Golay(24,12,8)
  error-correction (VoyagerList), the kissing number as a fixed-size bitmap/MPHF
  domain (XSet), and Co₀/M24 orbit canonicalization (separate code).
- **"No module/import system, no package manager, no build."** → **WRONG.** Full
  import system (file, qualified+alias, cross-package `from Space` RPC) + git
  package manager (`yon.toml`/`yon.lock`, pinned commits).
- **"No effect system."** → **WRONG.** `visits` effects, statically checked,
  transitively inferred. (But `requires` is capability tokens only, and its
  static check is not wired — runtime FNV-1a gate only.)
- **"No serialization."** → **WRONG.** `yon_rt_serialize/deserialize` + Merkle
  exist.
- **"No tooling (LSP/formatter/linter)."** → **WRONG.** `yonfmt`, `yon_lint`,
  `yon_doc`, `yon_lsp` (+ nvim/vscode clients) all exist.
- **"No TCP/networking."** → **WRONG.** TCP sockets exist (`Wire.*_net`).
- **"~20 stdlib modules."** → **33 modules.**
- **"Magma supports `subsetsum`/`knapsack`/`solve P`, a 'P=NP era' op set."** →
  **RETRACTED material** per the prototype's own MANIFEST. Real Magma ops:
  `empty/gen/is_commutative/is_associative/identity/closure_size/word_push/
  normal_form/from_catalog`.
- **"The reachability op is `Magma.reachable`."** → The real op is
  **`Land.reach`** (a doc error in the prototype's own book).
- **"The subtyping/error keyword is `extends`."** → The implemented keyword is
  **`subcontains`** (some doc/example *filenames* wrongly say `extends`; example
  *bodies* are correct).
- **"Sum types are simply absent."** → They **parse** (a `variant` AST node
  exists) but lower to an **opaque base type with no constructor/eliminator** —
  partial/stubbed, needs finishing. (Likewise `pushout` lowers to `0.0`;
  `with multishot` parses but the reducer ignores it; effect handlers are
  tail-resumptive in-place substitution, not reified continuations.)
- **"`xleech_coord`/`yon_xcoord_to_int24` is a stub returning -1; the δ-debt is
  open."** → **STALE.** `yon_xcoord_to_int24` is fully implemented and the
  δ-conversion debt is closed on `1.1-dev`.
- **"The Co₀ orbit cardinalities (98280/8386560/8292375) are established."** →
  **Not backed by source constants** — an unverified doc claim the prototype's
  own roadmap flags for confirmation.
- **"No runtime introspection at all."** → C-level introspection exists
  (`yon_rt_heap_occupancy`, checkpoint snapshots); only the *surface* lacks a
  primitive (real gap).
- **Confirmed (NOT corrections, for completeness):** 196,560 slots/heap, 256
  heaps/chain, ~50.3M slot ceiling, 64 MB arena/heap, ~17–18 GB content/process,
  global auto-chaining + dedup, and "loud refusal at limits" are all real. No
  automatic GC, but manual reclamation exists (`clear`/`reset`/`strip_trim` via
  `madvise(MADV_DONTNEED)`). Genuinely missing (analysis correct): general FFI,
  `Bytes`/binary I/O, `Result`/`Option`/`Either`, linear/affine types, delimited
  continuations, `Stream.flatMap`/`bind`, crypto beyond FNV-1a, HTTP/TLS,
  `match`/`case`+exhaustiveness, open user-defined typeclasses, interactive
  debugger, runtime reflection, REPL.
