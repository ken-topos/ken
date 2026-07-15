# SUB-2 — Retire the cached-`Nat` carriers

**Owner:** Team Runtime · **Size:** M · **Depends on:** SUB-1 (merged,
`origin/main @ 6e415f23`) · **Does NOT depend on SUB-1b**

> **Verify every "current state" claim in this frame against the landed code, not
> against this line.** Frames go stale; the tree does not.

---

## 1. Objective

**Delete the caller-fabricated length obligations from the byte-parsing path.**

Two carriers in the catalog make a caller *supply* a `Nat` length **and a proof
that the length is correct**, because — before SUB-1 — the length of a `Bytes`
could not be *computed* in Ken. SUB-1 landed `bytes_nat_length`, an ordinary
structural fold. **The cache and its proof obligation are now redundant.**
Retire them.

## 2. The disease, at both sites

**Site 1 — `catalog/packages/Capability/Parsing/Cursor.ken.md`.** `ArgBytes` is a class
whose *third field is a proof the caller must fabricate*:

```ken
fn ArgByteLength (bs : Bytes) (n : Nat) : Prop =
  Equal Int (bytes_length bs) (cursor_nat_to_int n)      -- relates the OPAQUE Int primitive to a Nat

class ArgBytes {
  arg_bytes_field        : Bytes;
  arg_length_field       : Nat;                            -- the CACHE
  arg_length_valid_field : ArgByteLength arg_bytes_field arg_length_field   -- the CALLER'S BURDEN
}
```

**Site 2 — `catalog/packages/Capability/Parsing/Parsing.ken.md`.** The same
shape, plus this, which is the tell:

```ken
fn byte_unit_zero_int (unit : Bytes) : Int = bytes_length unit - bytes_length unit
```

**That is `0 : Int`, spelled as "the length of a thing minus itself."** It is a
*runtime-computed zero*, written that way because the opaque primitive does not
reduce in conversion. **Nobody would write this if the length were structural.**
It is a workaround fossil, and it should not survive this WP.

**Why it existed:** `bytes_length : Bytes → Int` is a `PrimReduction::Op`. It
**computes at runtime and is opaque to kernel conversion.** So a *proposition*
about a byte length could not be *proved* — only *assumed*, by the caller, at
every construction site.

## 3. The fix — move to the structural view, and the Int primitives fall out

SUB-1 gives `bytes_to_list : Bytes → List UInt8` and its round-trip laws.
`Collections` already has **`length`, `take`, `drop`** — all total, structural,
`Nat`-indexed, kernel-checked.

**So express the whole path on the `List UInt8` view:**

| Today (opaque, Int, partial) | After SUB-2 (structural, Nat, total) |
|---|---|
| `bytes_length bs : Int` | `bytes_nat_length bs : Nat` *(landed by SUB-1)* |
| `bytes_slice bs i j` — partial, `bytes_at.bounds` obligation | `list_to_bytes (take n (drop m (bytes_to_list bs)))` — **total** |
| `bytes_at bs i` — partial, `SafeOption` | `nth (bytes_to_list bs) i` — **total, returns `Option`** |
| `ArgByteLength bs n` — caller must prove | **deleted.** Nothing is cached, so nothing needs validating. |

**`ArgBytes` collapses to plain `Bytes`.** `arg_length arg` becomes
`bytes_nat_length bs` — **computed, not asserted.** The class, its cache field,
its validity field, and `ArgByteLength` all **go away.**

**Bonus, and worth stating:** `bytes_at` / `bytes_slice` carry **runtime
partiality obligations** (`bytes_at.bounds`, `erasure.rs:1169-1179`). The
structural replacements are **total**. This WP therefore retires a class of
partiality obligation as a side effect, not just a cache.

## 4. ⛔ THE TRAP — read this before you write a line

**You will be tempted to keep `bytes_slice` (which takes `Int` offsets) and just
feed it a converted structural length.** The moment you do, you need:

```ken
bytes_length_agrees : (bs : Bytes) → Equal Int (bytes_length bs) (cursor_nat_to_int (bytes_nat_length bs))
```

**That equation is NOT PROVABLE.** `bytes_length` is a `PrimReduction::Op` —
opaque to conversion — so `Refl` cannot discharge it and no structural argument
reaches it. **It would have to be a new postulate: a new permanent
`trusted_base()` entry.**

**⛔ THAT IS NOT AUTHORIZED, AND IT IS NOT THE POINT OF THIS WP.** SUB-2 exists to
*remove* fabricated obligations, not to trade a per-caller one for a global one.

**The escape is to go structural ALL THE WAY**: if slicing and indexing also move
to the `List UInt8` view, **the opaque `bytes_length` / `bytes_slice` / `bytes_at`
never appear in the consumer path at all**, and the bridge is never needed.
**Zero new trust.** That is the design, and it is the whole design.

**If you find you cannot avoid the bridge — STOP AND REPORT.** Do not add the
postulate. It means this frame's §3 is wrong and I want to know that, not have it
papered over. (This is the same wall that stopped CC8: *an opaque primitive
computes at runtime but cannot be reasoned about in conversion.* Whether we ever
pay the definitional cost for that family is **Pat's open decision** — do not
pre-empt it here.)

## 5. Acceptance criteria

1. **`ArgByteLength`, `arg_length_field`, `arg_length_valid_field`, and the
   `ArgBytes` class are GONE** from `Capability/Parsing/Cursor.ken.md`; consumers take plain
   `Bytes`. Grep proves absence **in the extracted Ken**, not the prose.
2. **`byte_unit_zero_int` is gone**, along with the `Capability/Parsing` length
   relation. No function computes a constant by subtracting a value from itself.
3. **Zero `trusted_base()` delta.** Assert it with a set-equality test, the same
   fail-closed shape SUB-1 used. **No new postulate, no new primitive, no
   `Axiom`.**
4. **The opaque Int byte primitives (`bytes_length`, `bytes_slice`, `bytes_at`)
   do not appear in the extracted Ken of the touched packages.** Assert on
   `extract_ken_md(...).source` — **not** on the raw literate file. *(You fixed
   that exact oracle twice today. Do not rebuild the trap.)*
5. **ArgParse still parses.** The existing ArgParse/Process acceptance suites are
   green, unchanged in intent — this is a refactor of the *carrier*, not of the
   *behavior*.
6. **Partiality obligations retired:** the `bytes_at.bounds` obligation is no
   longer reachable from the touched packages.

## 6. Do-not-reopen guardrails

- **⛔ NO new postulate / primitive / `Axiom`.** (See §4. Stop and report.)
- **⛔ Do NOT touch the byte-COMPARISON path.** `argparse_byte_matches_char`
  (`eq_int (uint8_to_int actual) (charToInt expected)`) stays exactly as it is.
  Making that *lawful* is **SUB-1b's** job, not yours, and the two must not
  collide. **SUB-2 is spine-only: length, slice, index. No element equality.**
- **⛔ Do NOT re-mint a cached-length carrier** anywhere, in any package, under
  any name. If a consumer wants a length, it **computes** one.
- **⛔ No kernel, `spec/`, or `conformance/` change.** If you think you need one,
  stop and report.

## 7. Why this is ready NOW (and why it is not blocked on SUB-1b)

**Length, slice, and index are SPINE operations.** A fold walks the list's
structure and never inspects an element. **SUB-1 delivered the spine, and the
spine is all this WP touches.**

SUB-1b exists because *key comparison is an ELEMENT operation*, and elements are
still opaque. **SUB-2 needs no element.** So it is unblocked by SUB-1 alone, and
Language can run it immediately after SUB-1b without waiting for anything else.

*This is the same question that caught CC8, asked in the other direction and
answered the other way: **does my consumer touch the spine, or an element?***

## 8. Downstream

Retiring these carriers unblocks **`Path.Posix`** (which cannot be written
against a cached-length carrier), and `Path.Posix` in turn unblocks **CC9**.
