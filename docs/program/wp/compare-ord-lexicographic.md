# WP compare-ord-lexicographic — lawful 3-way `compare` + lexicographic `Ord`

**Owner:** Foundation team. **Steward-framed** (2026-07-11). Base:
`origin/main` (re-verify `file:line` at pickup — catalog moves). **Outer-ring
catalog proof:** no `crates/**/src`, no `ken-kernel`, no `Cargo.lock`, no `/spec`,
no `/conformance`. **@architect fidelity gate** on the lawful proofs (spec `51`
`Ord` laws char-for-char + the `Axiom`/`Refl`-paper grep). Uses the **landed
`case_eq` modifier** (`match e eqn: h`, `main @ b7c0de73`) for dependent dispatch
on the `Bool`/`OrdResult` comparisons — no new capability needed.

## Settled inputs — pin, do NOT reopen

- **`class Ord` is `leq`-based and stays that way** (spec `51 §2.3`,
  `LawfulClasses.ken.md:95`): fields `leq : a → a → Bool` + `refl`/`antisym`/
  `trans`/`total`. Spec `51 §2.3` explicitly rules the 3-way `cmp`/`OrdResult`
  form **"a derivable convenience"**, `leq` the primitive. **Do not add a
  `compare` field to `class Ord`; do not make the class `compare`-primary; do not
  relitigate `leq`-vs-`compare` primacy** — that is a *decided* fork. `compare`
  is a **derived** operation in this WP.
- **`OrdResult = Lt | Eq | Gt`** already exists (`Collections.ken.md:74`,
  inlined in `OrdNat.ken.md:161`). Reuse it — never a second name.
- **`instance Ord Nat` landed** (`OrdNat.ken.md:144`) with `compare` built from
  `leq_nat` (`:181`) — the shape brick 1 generalizes. `Ord Int`/`Bool`/`Char`
  landed in `LawfulClasses.ken.md`.

## Goal — three bricks, in order

1. **Lawful generic `compare` — UNBUNDLED raw-`leq` core + thin `Ord` wrappers
   (Architect ruling `evt_7v4argg2kp0b`, 2026-07-11 — the sanctioned shape;
   supersedes the earlier dict-projection / `case_eq`-for-soundness reading).**
   State the generic `compare_raw` **and its soundness lemmas over a raw
   `leq : a → a → Bool` parameter + explicit law arguments** (`antisym_law`/
   `trans_law`/… as ordinary `fn` parameters, never `d.leq`/dict projections):
   `compare x y = Eq` when `leq x y ∧ leq y x`, `Lt` when `leq x y ∧ ¬leq y x`,
   else `Gt` (reflecting `OrdNat.ken.md:181`). Public `Ord`-derived forms are
   **thin wrappers** (`compare_with a d := compare_raw a (d.leq)`,
   `compare_eq_sound a d … := compare_eq_sound_raw a (d.leq) (d.antisym) …`) —
   instantiation is pure application/δ. This is the catalog's established
   explicit-comparator idiom (`list_eq`/`list_compare`/`sort` already take raw
   `eqf`/`cmp`/`leq`); unbundling the **laws** the same way the **operations**
   already are is the coherent completion, and it routes around BOTH the
   `.field`-in-declared-type **parser** gap and the documented **K6 `conv_struct`
   Eq-operand-congruence** gap (`LawfulClasses.ken.md:689–723`) — every hypothesis
   type and supplied law shares the *literally identical* raw `leq` term, so no
   Eq-operand congruence is ever required. **Soundness lemmas use explicit
   fresh-`Bool` `J`** — `bool_dichotomy (leq x y)` + nested `bool_dichotomy
   (leq y x)`, motive spelling `compare_raw`'s reduced form, `antisym_law x y hxy
   hyx` in the `True/True` arm, `absurd` on transported `Bottom` (the landed
   deceq-List technique, `LawfulClasses.ken.md:558–589`, one dispatch deeper).
   `case_eq` (`match … eqn: h`) is used **only inside `compare_raw`'s own
   definition** (scrutinee occurs syntactically in its own return-type dispatch —
   the sugar's home turf), **NOT** in the soundness lemmas (transparent-wrapper
   case the modifier can't reach; see forward note in the judgment log).
2. **Rework Collections onto `compare` — RESOLVED acyclic (Architect ruling
   `evt_4p2683wvtwwcc`, 2026-07-11): no cycle, no relocation, absorbed into brick
   3.** The earlier cycle worry was the wrong reading: `list_compare`
   (`Collections.ken.md:753`) is **already raw-`cmp`-parameterized**
   (`cmp : a → a → OrdResult`), exactly as `list_eq` takes a raw `eqf` and never
   calls `DecEq`. So the canonical routing happens at the **instance layer** —
   `Ord (List a)` builds its element-comparator from the canonical `compare` and
   **threads it into the unchanged `list_compare`** (single source of truth, **zero
   Collections→Lawful edge**). **Collections' bodies are NOT edited** — a Collections
   body calling the Lawful `compare` is the one forbidden shape. `String.compare`
   keeps `compare_char` (`:766`) local; the canonical-String equivalence lemma is
   **out of scope** (unneeded without lawful `Ord String` — the DecEq-Char caveat).
   Brick 2 is therefore **not a separate follow-on** — its substance is the
   instance-threading in brick 3.
3. **Lexicographic `Ord (Pair a b)` and `Ord (List a)`.** The flagship: real
   `instance Ord (Pair a b)` and `instance Ord (List a)` (lexicographic order
   from the component/element `Ord` dictionaries), each with **all four laws**
   (`refl`/`antisym`/`trans`/`total`) proved as real terms. Pair: lexicographic
   `leq` over the two component dicts. List: recursive lexicographic `leq` with a
   structural induction hypothesis. The law proofs dispatch on the component
   comparisons via `case_eq` (`OrdResult`/`Bool`) + the component `Ord` laws —
   the sibling construction to the landed lexicographic `DecEq (Pair)/(List)`
   (`LawfulClasses.ken.md`, `deceq-structural-liftings`).

## Design seams — Architect fidelity gate (not a pre-shape; flag at pickup)

1. **Lexicographic-law provability (the real proof content).** `antisym`/`trans`/
   `total` over the *recursive* lex order on `List` are the hard lemmas. Confirm
   the landed capabilities (the `case_eq` modifier on `OrdResult`/`Bool`, the
   component `Ord` law fields, a structural IH) **suffice**. If a specific law
   hits a genuine capability gap, **stop and flag it to the Steward** (size-defer
   honestly, as DS-8 did — never `Axiom`/`Refl`-paper a law to force it closed).
2. **Placement + Track B collision (Steward-routed).** The new `Ord (Pair)/(List)`
   instances sit naturally beside the landed `DecEq (Pair)/(List)` in
   `LawfulClasses.ken.md`. The **case_eq-adoption WP** (Ergo) also edits
   `LawfulClasses` (the deceq List proof's internal dispatch) — a *different
   region*, so hunks should not overlap, but both touch the file. **The Steward
   routes merge order** (expect adoption to land first; rebase the `Ord` hunk if
   so). If Foundation prefers, the `Ord` instances may go in a **new sibling
   file** with a pinned load order — Architect/leader call. Flag any real
   conflict to the Steward; do not resolve cross-WP.
3. **`compare`↔`leq` coherence.** Fix the exact soundness statements brick 1
   carries (what `compare x y = Eq`/`Lt`/`Gt` each entail vs `leq`/`Equal`) so
   brick 3's lex proofs can consume them.

## Scope

- Catalog proof code in `LawfulClasses.ken.md` (or a pinned new sibling file per
  seam 2) + `Collections.ken.md` (brick 2 rework). Proof-only + one derived op.
- Acceptance tests beside the landed `Ord`/`DecEq` suites: the `compare`
  soundness lemmas and the `Ord (Pair)/(List)` law fields **elaborate +
  kernel-check** at the general statement; targeted builds (`-p <crate> <test>`),
  full-suite green in CI at merge.

### Out of scope

- Any `class Ord` change (settled `leq`-primary — see pinned inputs).
- Any kernel/elaborator/spec/conformance change (`trusted_base()` delta empty).
- `Ord String`/`Ord`-for-opaque-carriers as *lawful instances* (the `DecEq Char`
  canonical-carrier caveat, ADR 0010, still applies — `String`/`Char` comparison
  stays tested-not-trusted `fn`s, not lawful `Ord` instances, this WP).

## Acceptance criteria

- **AC1 — lawful generic `compare` (unbundled).** `compare_raw` (over raw `leq`
  + explicit law args) + its soundness lemmas (`= Eq → Equal`, `Lt`/`Gt` vs `leq`)
  elaborate + kernel-check as **real explicit-`J` proof terms** (fresh-`Bool`
  motives never inferred, law args applied on neutral `leq` comparisons sharing
  literal terms, `absurd` on genuinely-transported `Bottom`); the public `Ord`
  `compare`/soundness forms are thin δ-wrappers. No `Axiom`/`Refl`-paper on any
  general statement. (Architect `evt_7v4argg2kp0b`.)
- **AC2 — canonical `compare` single-sourced, acyclic (instance-threaded).** The
  canonical `compare` (in LawfulClasses) is the single source of truth; `Ord
  (List a)` threads it into the **unchanged** raw-`cmp` `list_compare` — no
  Collections body edit, no cycle. `list_compare`/`String.compare` stay
  comparator-parameterized (`compare_char` local). (Architect `evt_4p2683wvtwwcc`.)
- **AC3 — lexicographic `Ord` instances.** `instance Ord (Pair a b)` and
  `instance Ord (List a)` with **all four laws** proved as real terms (the
  Architect greps the tangled code for `Axiom`/`Refl`-paper). Any law that hits a
  real capability gap is **size-deferred + flagged**, never papered.
- **AC4 — fidelity.** The law statements match spec `51`'s `Ord` laws
  char-for-char (`refl`/`antisym`/`trans`/`total`); `OrdResult` reused, not
  renamed.
- **AC5 — zero trust/build delta.** No `Axiom`/`postulate`/`Decl::Opaque`/`sorry`;
  `trusted_base()` delta empty; no `crates/**/src`/`ken-kernel`/`Cargo.lock`/
  `/spec`/`/conformance` touch. Grep-confirmed. Workspace-green in CI at merge.

## Gate

Foundation ring (foundation-leader → foundation-implementer → foundation-qa) →
**@architect fidelity gate** (spec `51` `Ord` laws char-for-char on the law
statements; the tangled-code `Axiom`/`Refl`-paper grep; lex-law-provability seam
1) → `git_request` to Steward → **CI-gated** merge (real catalog code +
acceptance tests, not doc-only). Outer-ring, no soundness urgency. Own the retro
(terra harness readout). **No WP-token identifiers in production/tangled source.**
Re-verify `file:line` cites at pickup.
