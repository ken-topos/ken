# WP L3-strings-surface — derived string surface over `List Char` (slice 2/2)

**Owning team:** Language (elaborator-layer derived defs)
**Branch:** `wp/L3-strings-surface`  ·  **Base:** `origin/main@f50be22`
**Size:** M  ·  **Risk:** low (zero-TCB-delta; all recursion shapes have landed
precedent)
**Depends on (all merged):** slice 1 `L3-strings-roundtrip` (`f50be225` —
`string_to_list_char`/`list_char_to_string` are real); `elim_List` + `elim_Nat`
(L2, landed); `Ord Char` / `DecEq Char` (lawful-classes-lane, landed).
**Gate:** G-surface (Language ring → Architect soundness + CV conformance → CI).

> **Status of "current implementation state" claims in this frame:** perishable.
> Every "X is landed / X does not exist" line was `git grep`-verified against
> `origin/main@f50be22` at authoring; the elaboration must **re-verify against
> the landed code at pickup**, not trust this line. The *objective + acceptance*
> below is the durable contract.

## 1. Objective (one line)

Deliver Ken's derived **string surface** — `concat`, `slice`, `charAt`, `eq`,
`compare` over `String` — by (a) building the **minimal `List` combinator
floor** those ops need as termination-checked recursive derived defs over landed
eliminators, and (b) deriving the 5 string ops on top, routing through the
now-real `string_to_list_char` / `list_char_to_string`. **Zero new native prims,
zero `trusted_base()` delta.**

## 2. Settled inputs — fixed, do not reopen

These are decided. A build model must **execute** them, not relitigate them.

1. **Approach = (A) derive, not native prims.** Architect ruling
   `evt_4k1yqah3yvpds` (thread `thr_5y9aya3y6vawh`), confirming the derivation
   table `evt_66g17exdhd767`. The 6 combinators are **termination-checked
   recursive derived defs** lowering via `declare_recursive_group` + `sct_check`
   + `declare_def`. **Do not** add native interp prims for these (they are
   trivially structural folds — a native prim would grow the tested-not-trusted
   reduction surface for no benefit; a subsume-don't-proliferate violation).
2. **Scope = minimal floor + 5 string ops, one WP.** The full
   `L3-strings-collections` surface — `Array`/`Map`/`Set`, combinators-with-
   laws-as-propositions, the verified `sort` — is a **separate WP**. Do not
   expand into it here.
3. **`eq`/`compare` are codepoint-wise** (ADR 0010, `docs/adr/0010-lawful-deceq-
   requires-canonical-carrier.md`). `String` is **canonical** w.r.t. `List Char`
   (slice 1 made `s2l`/`l2s` a real round-trip bijection over scalars), so
   `DecEq String` / `Ord String` are **soundly deliverable** codepoint-wise.
   **NFC-normalization equality is OUT of scope** and must **never** be built as
   a `DecEq`/`Ord` here: NFC-eq identifies distinct codepoint sequences, so over
   the codepoint carrier it is a **non-canonical** equality → a lawful `DecEq`
   for it would inhabit `Bottom` (ADR 0010). If NFC-eq is ever wanted it is an
   `Eq` instance in a later WP, not this one.
4. **Char element ops come from the landed classes.** `eq` rides `eqChar` /
   `DecEq Char`; `compare` rides `Ord Char`. Both landed on
   lawful-classes-lane — **reuse them**, do not re-derive a Char comparison.

## 3. The 6-combinator floor — mandated shape

Build these as generic derived defs over `List a` / `Nat` (each ends in a
**concrete** recursion, not a survey). The landed **precedent** for each exact
recursion shape is in `crates/ken-elaborator/tests/l3a_acceptance.rs`
(elaborates + SCT-passes on `main` today — the de-risking reference, cited per
Architect's capability confirm):

| Combinator | Signature | Recursion shape | Landed precedent |
|---|---|---|---|
| `list_append` | `List a → List a → List a` | single-list structural, self-call on `Cons` tail | `map` (simpler than) |
| `nth` | `Nat → List a → Option a` | index + list structural | `map` |
| `take` | `Nat → List a → List a` | Nat-fuel, out-of-range → `Nil` | `unfoldUpTo` |
| `drop` | `Nat → List a → List a` | Nat-fuel, out-of-range → `Nil` | `unfoldUpTo` |
| `list_eq` | `(a → a → Bool) → List a → List a → Bool` | two-list structural, nested `match` | `zip` |
| `list_compare` | `(a → a → Ordering) → List a → List a → Ordering` | two-list structural + threaded element compare | `zip` / `insert` |

- **`list_append` — module-qualified name; do NOT shadow the Bytes `append`.**
  A `Bytes`-domain `append` (FS-effect) is already registered
  (`crates/ken-elaborator/src/bytes.rs`). The `List` one must be a **distinct
  name** (`list_append`, or module-qualified) so the implementer does not shadow
  the Bytes op. (Architect brief-condition 2.) Apply the same hygiene to any
  other clash you hit.
- **`Ordering`** — reuse whatever comparison-result type `Ord Char` already
  yields on `main` (grep `Ord Char` / the `compare` method's codomain in
  `crates/ken-elaborator/src/classes.rs` / `packages/lawful-classes/`); do not
  introduce a second one.

## 4. The 5 string derivations — mandated bodies

Derive exactly per the Architect's table (`evt_66g17exdhd767`), through the
now-real `s2l = string_to_list_char` / `l2s = list_char_to_string`:

- `concat a b   = l2s (list_append (s2l a) (s2l b))`
- `slice i j s  = l2s (take (sub j i) (drop i (s2l s)))`  — clamps by
  construction (out-of-range `take`/`drop` → `Nil`); `sub` is the landed
  saturating/`Nat` subtraction (grep the landed name; if `j < i`, `sub j i = 0`
  → empty slice, not underflow)
- `charAt i s   = nth i (s2l s) : Option Char`  — `None` when out of range
- `eq a b       = list_eq eqChar (s2l a) (s2l b)`
- `compare a b  = list_compare (Ord Char).compare (s2l a) (s2l b)`

## 5. Acceptance criteria — testable, discriminating

1. **AC1 — floor registered + total.** All 6 combinators elaborate as
   `declare_recursive_group` / `declare_def` members over the **real**
   `elim_List` / `elim_Nat` (a `Term::Elim`, not a bespoke reducer) and
   **SCT-pass**. **Producer-grep** the registration in
   `crates/ken-elaborator/src` — not hand-fed (the
   `conformance-hand-feeds-the-deliverable` net).
2. **AC2 — SCT sound-zone check (Architect brief-condition 1, soundness).** For
   **each** combinator, the recursive call is an **applied call on a strict
   subterm** of a matched argument (the tail of a `Cons`, the predecessor of a
   `Suc`). The frame does **not** lean on the SCT to bless *unapplied*
   self-reference or recursion-through-an-opaque-`Map`
   (`sct-unapplied-self-reference-over-accepts` — the SCT over-accepts there).
   None of the 6 need that shape; the AC is a cheap per-combinator confirmation
   that the recursion is genuinely decreasing.
3. **AC3 — 5 string ops derived + correct.** Each of `concat`/`slice`/`charAt`/
   `eq`/`compare` elaborates and **reduces to the correct value** on a corpus
   (through the real `s2l`/`l2s`): non-ASCII + multi-byte codepoints included
   (reuse slice-1's boundary corpus). `charAt` returns `None` on out-of-range
   and empty; `slice` clamps (including `j < i → ""`).
4. **AC4 — `eq`/`compare` codepoint-wise, discriminating PAIR (ADR 0010).** A
   **non-degenerate pair** (COORDINATION §7): `eq` **accepts** two equal
   codepoint sequences **and rejects** a differing pair (incl. a same-length,
   single-codepoint-differing pair); `compare` gives correct lexicographic order
   on an ordered triple (`"a" < "ab" < "b"`). Assert the **result value**, not
   "it type-checks." NFC-normalization eq is **absent** — a canonically-
   equivalent-but-codepoint-distinct pair (e.g. NFC vs NFD of the same grapheme)
   must compare **unequal** here (pins that NFC-eq was not smuggled in).
5. **AC5 — zero TCB delta.** `git diff origin/main -- crates/ken-kernel/` is
   **empty**; `trusted_base()` unchanged; **grep no new `declare_primitive` /
   `declare_postulate` / `declare_opaque`** in the diff (the combinators are
   `declare_def` = checked, the string ops are derived). Same tested-not-trusted
   floor as slice 1 / conversions.
6. **AC6 — name hygiene (Architect brief-condition 2).** `list_append` does not
   collide with / shadow the Bytes `append` (FS-effect); both resolve to their
   intended op. Any other clash resolved the same way.
7. **AC7 — round-trip / totality sanity.** `concat`+`slice` compose sanely on
   the corpus (e.g. `slice 0 (len a) (concat a b) ≡ a` on scalar-clean inputs);
   `list_append` associativity on a small corpus; every combinator total (no
   `Neutral`/stuck on well-typed input).

## 6. Do-not-reopen guardrails

- **Approach A is settled** — no native prims for the combinators; no relitigate
  derive-vs-native.
- **Scope is floor + 5 ops** — no `Array`/`Map`/`Set`, no laws-as-propositions,
  no verified `sort` (separate WP).
- **Codepoint-wise only** — no NFC-normalization `DecEq`/`Ord` (ADR 0010: that
  would inhabit `Bottom` over the codepoint carrier; it is an `Eq` in a later
  WP).
- **Reuse landed element ops** — `eqChar`/`DecEq Char`/`Ord Char` are landed;
  do not re-derive Char comparison.
- **SCT sound zone** — do not use SCT to bless a disguised non-terminator.

## 7. Out of scope / tracked follow-ons

- **Full `ken-cli` REPL/driver end-to-end wiring** — the driver is stale
  (pre-Language-layer; it does not wire `console_ids` either — slice-1 CV note).
  This WP exercises the derived surface end-to-end through the **real
  elaboration + eval path in acceptance tests** (mirroring slice 1's test-setup
  wiring of `store.list_char_ids`), which is the load-bearing correctness
  evidence. Driver wiring stays a tracked follow-on, not this WP's job.
- **`conformance/surface/strings/` durable seed** — CV owns it as a follow-on
  (slice-1 note; the `surface/strings/*` case-ids already exist). Non-blocking;
  the AC4/AC7 corpus here promotes into it cleanly.
- **`L3-strings-collections` remainder** — `Array`/`Map`/`Set`, laws-as-props,
  verified `sort`, user-type `DecEq`/`Ord` instancing (L3b AC7) — later WP(s).
