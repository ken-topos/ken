# CC6b · `System.Path.Posix` — the held half of CC6, now unblocked

**Owner:** Team Runtime · **Size:** M · **Branch:** `wp/cc6b-path-posix`
**Base:** `origin/main @ 2c184550` · **Gate:** Runtime QA (+ Architect
terminal **only if trust moves — it must not**)

**Blocked until SUB-1b merges. Do not start before the Steward's GO.**

**⚠ Ownership note — this is a DELIBERATE break in continuity, not a mistake.**
**CC6a was Foundation's**, so CC6b would ordinarily be theirs too. **It is
Runtime's, for two reasons:** (1) Foundation is queued on **CC8**, which unblocks
on the same SUB-1b merge — assigning both to one ring would serialize the two
WPs that are finally able to run in parallel; (2) **CC6b's dominant skill is
SUB-2's**, not CC6a's — total structural folds over `List UInt8`, keeping the
opaque byte primitives out of the consumer path, and consuming a lawful `DecEq`
instead of a bare `Bool` fold. **Runtime just did exactly that, and wrote the
ledger in §1.** CC6b is a **greenfield package** (no `Path` code exists to
inherit), so there is little CC6a context to lose.

## 0. Why this WP exists now, and not four hours ago

CC6 was `Process.Arguments` + `System.Exit` + `System.Path.Posix`. The first two
shipped as **CC6a**. `Path.Posix` was **held**, and CC6a's frame said exactly
why:

> *"Splitting a path on `/` requires iterating a `Bytes`. `bytes_at`/
> `bytes_length` traffic in **opaque `Int`**, and there is **no `Int → Nat`
> anywhere** — so a terminating structural fold over a bare `Bytes` **cannot be
> written**. The only route that builds is the **cached-`Nat` carrier**… the
> decision is **with the operator**."*

**That decision was made, and both halves of it have landed:**

- **SUB-1** gave `Bytes` a **structural view** — `bytes_to_list : Bytes → List
  UInt8` and `list_to_bytes` (emitted in `crates/ken-elaborator/src/bytes.rs`,
  **not** a `.ken` declaration — grep the prelude, not the catalog). **A
  terminating structural fold over a `Bytes` is now an ordinary total function.**
- **SUB-2** *retired* every cached-`Nat` carrier the old substrate forced
  (`ArgBytes`, `ArgByteLength`, `Source`'s cache, `byte_unit_zero_int`), **net
  −231 lines.**
- **SUB-1b** (in flight) gives `UInt8` a **lawful `DecEq`** — `eq` + `sound` +
  `complete` — from one audited retraction postulate.

**⇒ The hold's premise is dead. Build it.**

## 1. ★ The obligation ledger — read this before you design anything

I ran the **consumer-edge ledger** on this WP (your own SUB-2 carry —
`evt_qseekjq9wjjp`) and it changed my mind mid-frame. **Here is the honest
partition, and it is not the one I first wrote in the tracker.**

| Path.Posix operation | Kind | Needs |
|---|---|---|
| iterate the bytes at all (terminating, total) | **spine** | **SUB-1** ✅ landed |
| **split on `/`** | **ELEMENT** — you compare each byte to `/` | **SUB-1b** |
| drop `.` segments | **ELEMENT** — is this segment equal to `"."`? | **SUB-1b** |
| resolve `..` against its parent | **ELEMENT** | **SUB-1b** |
| join / render segments back to `Bytes` | **spine** | **SUB-1** ✅ |
| byte-preservation law (segment bytes unchanged) | **spine** | structural `Equal` — `Refl` |

**★ My tracker said the split was spine-only. It is not.** A fold that walks a
list *and asks a question about each element* is **element-wise**, no matter how
spine-shaped its recursion looks. **I corrected that by trying to write the
obligation instead of trusting my own summary of it** — the same instrument that
caught CC8, aimed at my own note.

**The consequence, stated precisely, because it is subtle and it is the whole
reason this WP was gated:**

> **Every element operation here BUILDS TODAY.** The decision procedure has always
> existed — `eq_int (uint8_to_int b) (charToInt c)` is exactly what
> `argparse_byte_matches_char` does on `main` right now. **You could write all of
> `Path.Posix` this afternoon and it would run.**
>
> **What you could not do is PROVE anything about it.** A bare `Bool` fold gives
> you a decision with **no `sound`/`complete`**, so **no law that quantifies over
> segment equality can be discharged.** And *every* CC package carries laws.
>
> **⇒ The gap was never the decision procedure. It was the LAW.** That is what
> SUB-1b supplies, and it is why this WP waits for it.

## 2. Fixed inputs — settled, grounded at `origin/main @ 2c184550`

**Treat every anchor as perishable. If a fixed input is FALSE, say so with exact
tree anchors and ESCALATE — do not build around it.** *(That clause has now
caught a bad pin of mine six times. It is the most valuable line in every frame
I write. Use it.)*

1. **The structural view is prelude-emitted, not catalog-declared.**
   `bytes_to_list` / `list_to_bytes` live in
   `crates/ken-elaborator/src/bytes.rs:226-241`. **Do not re-declare them.**
   `bytes_nat_length` **is** catalog Ken
   (`Data/Collections/Collections.ken.md:1343`).
2. **The List toolbox you have** (`Data/Collections/Collections.ken.md`):
   `list_append:515` · `nth:521` · `take:531` · `drop:541` · `map:577` ·
   `filter:583` · `mem:593` · `length:603` · `reverse:692` · `concat_map:791` ·
   `foldl:821` · `list_eq:1251` · `list_compare:1269`.
   **⚠ There is NO `foldr`, and NO `split`/`span`/`break`.** You will **write the
   splitter yourself** — `foldl` + `list_append` + `reverse` is sufficient, and
   it is ordinary total structural Ken. **Do not add a primitive for it.**
3. **`instance DecEq (List a) where DecEq a` ALREADY EXISTS**
   (`Core/LawfulClasses.ken.md:2022`). **So the moment SUB-1b lands `DecEq
   UInt8`, you get `DecEq (List UInt8)` — i.e. lawful segment equality — FOR
   FREE, by instance resolution.** Do not hand-roll it. Do not re-derive it.
4. **`DecEq` is** (`Core/LawfulClasses.ken.md:74`) — `eq : a → a → Bool` ·
   `sound : IsTrue (eq x y) → Equal a x y` ·
   `complete : Equal a x y → IsTrue (eq x y)`.
   **`sound`/`complete` are what your laws consume.** An instance whose
   `sound`/`complete` are opaque would merely *relocate* the trust — SUB-1b's
   are genuine proofs transported from the kernel's `DecEq Int` certificate.
5. **Greenfield.** There is **no** `Path`/`Posix` anything in `catalog/` today.

## 3. Mandated deliverable — the design is made; execute it

### 3.1 The representation — **parse once into structure; do NOT re-parse per op**

```ken
-- a segment is a nonempty run of bytes containing no '/'
-- a path is an absoluteness flag + its segments
record Path {
  path_absolute : Bool;
  path_segments : List (List UInt8)
}
```

**Pinned, do not relitigate:** `Path` is **structured**, not a `Bytes` blob that
each operation re-splits. Parse at the boundary (`Bytes → Path`), render at the
boundary (`Path → Bytes`), and let every operation in between be **ordinary total
structure-walking**. *This is the same move SUB-2 made, and for the same reason:
keep the opaque representation out of the interior.*

### 3.2 The surface

- **`path_parse : Bytes → Path`** — absolute iff the first byte is `/`. Split on
  `/`. **Empty segments are dropped at parse** (so `a//b` and `a/b/` and `a/b`
  all parse to the same segment list; the absoluteness flag carries the leading
  `/`). **Byte-preserving: segment bytes are passed through UNTOUCHED — no
  decoding, no normalization of case, no `String` anywhere.**
- **`path_render : Path → Bytes`** — the inverse at the boundary.
- **`path_normalize : Path → Path`** — **LEXICAL** normalization only (below).
- **`path_join`**, **`path_parent`**, **`path_is_absolute`** — the ordinary views.

### 3.3 Lexical normalization — the exact rules, pinned

1. Drop every `.` segment.
2. A `..` segment **cancels the preceding segment**, unless that preceding
   segment is itself `..`.
3. On an **absolute** path, a leading `..` that has nothing to cancel is
   **dropped** (POSIX: `/..` is `/`).
4. On a **relative** path, a leading `..` that has nothing to cancel is
   **KEPT** (`../a` cannot be simplified).

### 3.4 ⛔ THE HONESTY GUARDRAIL — this is the point of the package

**`path_normalize` is LEXICAL. It does NOT claim filesystem canonicalization.**
It does not resolve symlinks, does not touch the filesystem, does not know
whether any segment exists. **On a filesystem with symlinks, lexical `a/../b` is
NOT necessarily the same location as `b` — and this package must never imply that
it is.**

**Say so in the package's own prose, in the doc comment on `path_normalize`, in
the words "lexical, not canonical."** This is `docs/PRINCIPLES.md`'s
honesty-about-the-boundary clause, and it is **an acceptance criterion, not a
nicety** — a path library that quietly implies canonicalization is exactly the
kind of latent overclaim we just spent a whole thread purging from the verifier
prose.

## 4. Acceptance criteria — every one testable

- **AC1 — byte-preservation.** For any `Bytes`, every byte of every surviving
  segment is **identical** to its input byte. Prove it structurally (`Equal (List
  UInt8) …`), and test it on **invalid UTF-8** — a byte sequence that is not
  valid text must survive a parse/render round-trip **unchanged**. *This is the
  "POSIX bytes, not `String`" guarantee and it is the package's reason to exist.*
- **AC2 — normalization is IDEMPOTENT.** `normalize (normalize p) = normalize p`
  using native kernel `Eq`. **A proof, not a test.** *(This is the law that
  needs `DecEq (List UInt8)`'s
  `sound`/`complete`; if you find you cannot discharge it, STOP AND REPORT —
  that means §1's ledger is wrong and I want to know.)*
- **AC3 — no `.` survives normalization**, and **no `..` survives on an absolute
  path.** Proofs.
- **AC4 — round-trip on valid paths:** `path_valid p = True → parse
  (render p) = p`, with the result stated using native kernel `Eq`, where
  validity means every segment is nonempty and contains no slash byte. Also
  prove the unconditional parser-image corollary `parse (render (parse raw)) =
  parse raw` using native kernel `Eq`. Normalization is not the validity
  precondition: a valid path containing `..` still round-trips, while the raw
  public `MkPath` carrier can also express invalid empty or slash-bearing
  segments that deliberately fail `path_valid`.
- **AC5 — `..` cancellation is correct at the edges:** `/..` → `/`; `../a` keeps
  its `..`; `a/../../b` → `../b`; `../..` → `../..`.
- **AC6 — ZERO `trusted_base()` delta.** No postulate, no primitive, no `Axiom`.
  Assert it with a **fail-closed set-equality** test, same shape SUB-1/SUB-2 used.
- **AC7 — the opaque byte primitives NEVER appear in the consumer path.**
  `bytes_length`, `bytes_slice`, `bytes_at` must be **absent** from this package's
  extracted Ken. You go structural all the way, exactly as in SUB-2.

## 5. ⛔ Do-not guardrails

- **⛔ Do NOT re-mint a cached-`Nat` carrier.** You just deleted four of them. A
  consumer that wants a length **computes** one (`bytes_nat_length` / `length`).
  If you feel you need a cached length for `Path` — **STOP AND REPORT.** *That is
  how a fifth carrier gets quietly born.*
- **⛔ Do NOT bridge back to opaque `Int`.** No `bytes_length`/`bytes_slice`/
  `bytes_at` in the consumer path; no `Equal Int …` obligation over an opaque
  `Op`. It is not provable and it would need a postulate. **Not authorized.**
- **⛔ Do NOT hand-roll segment equality.** Fixed input 3: `DecEq (List a)`
  already exists and SUB-1b gives you `DecEq UInt8`. A hand-rolled `Bool` fold
  would **build and never prove** — that is the exact trap this WP waited to
  avoid.
- **⛔ Do NOT decode to `String`.** Ever. Not for comparison, not for display.
- **⛔ No kernel / `spec/` / `conformance/` change.** Tell me if you touch `spec/`
  or `conformance/` and I add CV to the gate.
- **When you write the zero-`Axiom` gate: EXTRACT the literate Ken first, then
  assert on DECLARATIONS.** You repaired that oracle yourself as I-8's Step 0 and
  proved it again in SUB-2. **A grep SELECTS candidates; it never DECIDES** — and
  note that **this very frame names every forbidden token**, so a raw scan of the
  tree will hit it. **Do not let my prohibition text fail your own WP.**
