# WP L3 — strings & the lawful collections core

**Owner:** Team Language (L-stream). **Branch:** `wp/L3-strings-collections`
(cut
from `origin/main`). **Stream / gate:** L-stream → **G6**; **unblocks T3**
(test/property framework). **Depends on:** K1 (primitives); L1 (`Char`/numbers);
L2 (`data`/`match` — `List` is inductive data) — all **merged**. **Spec
source:**
`spec/30-surface/37-strings-collections.md` (+ `34` data, `35 §2` Char, `41`
values, `33 §5` classes, `36 §4` space, `42 §2` Lazy).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `37` to team-ready rigor +
> conformance before Team Language builds. **Perishable:** `String`/collections
> ride the **landed** `41` content-addressed value model + L2's `data` machinery
> —
> pin against the code, not this line.

## 1. Objective (one line)

Deliver a **curated, lawful collections core**: UTF-8 `String`, `List`/`Array`/
`Map`/`Set`/`Option`/`Result` (all immutable + persistent), the core combinators
with laws, and structural equality/ordering — **inductive and total, with NO
coinduction** (infinitude is served by generators / fuel-bounded `Lazy` / the
behavioral seam, never a coinductive type).

## 2. Settled inputs — FIXED, do not reopen

Per `37`:

1. **`String` is a UTF-8 primitive (§1, `14 §5`/`41`)** — immutable, **content-
   addressed** (equal strings share storage, **O(1) equality**), a **packed byte
   buffer** — **NOT `List Char`** (that's a separate convertible view). The API
   **distinguishes byte-length from char-length** (avoid the UTF-8 traps);
   indexing by code point or explicit byte view; raw bytes are `Bytes` (L6). No
   new kernel rule.
2. **Core collections are immutable + persistent (§2)** — `List a` (the
   inductive
   `data List`, L2 — pattern-matchable, canonical for proofs), `Array a`
   (contiguous, O(1) index, **structural-sharing** persistence), `Map k v` /
   `Set a` (`DecEq`/`Ord`-keyed), `Option`/`Result` (L2 sums). Updates **return
   new values sharing structure**; mutation only in a `space` (`36 §4`). `List`
   is
   inductive; `Array`/`Map`/`Set` are abstract types with **proven operations +
   laws as propositions**.
3. **Iteration is recursion + combinators, NOT a kernel protocol (§3)** —
   structural recursion/`match` is primitive; `map`/`filter`/`fold`/`zip` are
   ordinary **stdlib `view`s with laws** (functor/fold laws as propositions);
   comprehensions/`for` are sugar.
4. **★ NO coinduction / NO productivity checker (§3, `OQ-coinduction` DECIDED —
   deferred).** Ken's core is **inductive and total**. Infinitude is served
   three
   ways, all available now: **Generators** (`view … visits [Yield]`, finite-step
   effectful producer — rides L5), **`Lazy a` streams** (opt-in `Lazy` thunk,
   `42 §2`, with a **fuel/depth bound** — `take n`, finite-by-construction), and
   **the behavioral seam** (a `space`/actor with a total per-message handler). A
   `Stream`/`Iterator` is a **library type over these idioms**, never a language
   primitive. This is durable (the dual of SCT / the declined `OQ-temporal`).
5. **Equality is structural + content-addressed (§4, `41 §4`)** — the runtime
   default; `DecEq` (`33 §5`) makes it constraint-usable; `Eq` (observational,
   `15`) is the propositional version. **`Ord` is a lawful class** (total-order
   propositions provable); **sortedness is a refinement** (`34 §5`), provable.
6. **L-classes coupling (flag, don't resolve):** the collection **types** +
   structural equality ship in L3; full **`DecEq`/`Ord` instancing by user
   type**
   depends on **L-classes** (`33 §5`/`39`). Pin that boundary (the L1-numerics
   precedent: built-in now, user-instancing L-classes-gated).

## 3. Mandated deliverable outline (each ends in an implementable choice)

Deliver in the surface/elaborator + prelude (lowering to the landed `41`):

1. **`String`** — UTF-8 primitive on the landed `41` content-addressed buffer;
   `Char` (`35 §2`); byte/char-view ops (length-byte vs length-char distinct);
   the `String ↔ Bytes` (L6) + `String ↔ List Char` convertible views.
2. **The collection types** — `List` (L2 `data`), `Array` (persistent, O(1)
   index, structural sharing), `Map`/`Set` (`DecEq`/`Ord`-keyed), reusing
   `Option`/`Result` (L2). Pin each lowering + the persistence/sharing.
3. **Combinators + laws** — `map`/`filter`/`fold`/`zip` as prelude `view`s; the
   functor/fold/`Map`-lookup laws as **propositions** (no new kernel rule).
4. **No-coinduction infinitude** — at least the **generator** (`visits [Yield]`)
   *or* the **fuel-bounded `Lazy`** idiom (`take n`); **assert no coinductive
   type
   / productivity checker exists** (structural absence).
5. **Equality + ordering** — structural content-addressed equality;
   `DecEq`/`Ord`
   as lawful classes (built-in instances now, user-instancing L-classes-gated);
   the verified **`sort : List a → { xs | isSorted xs }`** (the canonical
   example).

## 4. Testable acceptance criteria

- **AC1 (`String` UTF-8 primitive)** A `String` is content-addressed (equal
  strings O(1)-equal) and **byte-length ≠ char-length** on a multi-byte string
  (structural — assert both views, not "compiles"); not `List Char`.
- **AC2 (persistent collections)** `List` pattern-matches (L2 `elim`); an
  `Array`/
  `Map` **update returns a new value sharing the unchanged structure** (assert
  the
  sharing / O(1)-comparability, not just a correct result).
- **AC3 (lawful combinators)** A functor/fold law (e.g. `map id ≡ id`, or a
  `Map`
  lookup/insert law) is **provable as a proposition** (structural on the emitted
  obligation).
- **AC4 (NO coinduction — the headline guardrail, structural)** Assert **no
  coinductive type / productivity checker** in the kernel/surface (the
  grep-for-forbidden-construct absence), AND a working **generator** (`visits
  [Yield]`) or **fuel-bounded `Lazy` `take n`** produces a finite prefix —
  infinitude without a coinductive value.
- **AC5 (structural equality + `DecEq`)** Structurally-equal collections are
  O(1)-comparable (content-addressed); a `Map` requires `DecEq`/`Ord` on its key
  (reject a non-`DecEq` key — verdict flip).
- **AC6 (the verified example)** `sort` produces `{ xs | isSorted xs }` — the
  sortedness **refinement obligation** is emitted + dischargeable (structural,
  per
  the untrusted-layer lesson; the canonical verification example).
- **Conformance:** `conformance/surface/collections/` — UTF-8 byte/char-length
  edge cases, persistent-update **sharing**, the verified `sort`, the
  no-coinduction
  absence. **QA gate:** **producer-grep** the `String`/collection
  **registration**
  in `ken-elaborator/src/` before counting green (new-surface WP); the laws/sort
  route **real** obligation emission, not synthetic.

## 5. Do-not-reopen guardrails

- **`String` is a primitive, content-addressed, not `List Char`** (§2.1) — no
  new
  kernel rule.
- **Collections immutable + persistent** (§2.2) — updates share structure;
  mutation only in a `space`.
- **★ NO coinduction / NO productivity checker** (§2.4) — infinitude via
  generators / fuel-`Lazy` / the seam; **assert the absence**. Durable.
- **Combinators + laws are stdlib propositions** (§2.3) — no new kernel rule.
- **L-classes boundary** (§2.6) — built-in `DecEq`/`Ord` now; user-instancing
  L-classes-gated (flag, don't resolve).

## 6. Sequencing notes

- L3 **unblocks T3** (test/property framework needs the collections) and feeds
  **L8** (the full lawful stdlib).
- **⚠ Resolver-scope limitation (from L2-build — verify at pickup):**
  `elaborate_decl` runs a **fresh resolver scope per call**, so **lowercase
  globals don't cross declarations** (only uppercase `ConId` cross-reference).
  If
  L3's combinators/laws need **inter-declaration lowercase references**, the
  resolver must thread globals through first — **flag it early** (a possible
  blocker / a small resolver-enhancement sub-WP), don't discover it mid-build.
- Standard §2c: frame → spec-leader elaborates `37` + conformance → merge
  (Architect + conformance-validator) → Team Language compacted, then kicked
  off.
