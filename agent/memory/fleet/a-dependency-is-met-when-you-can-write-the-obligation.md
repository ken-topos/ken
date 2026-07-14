---
scope: fleet
audience: (see scope README) — anyone declaring a WP "unblocked" because its dependency merged
source: SUB-1 → CC8, 2026-07-14 — the unblock that wasn't
---

# A dependency is MET when you can WRITE the obligation — not when the WP it points at merged

**SUB-1 merged.** It gave `Bytes` a structural view: `bytes_to_list : Bytes →
List UInt8`, plus round-trip postulates. **CC8 was "unblocked" — it needed
`DecEq Bytes` for an environment lookup, and SUB-1 was the WP that made `Bytes`
reasonable-about.** Three review lanes approved SUB-1. CI was green. Every test
passed. **Every one of those facts was correct.**

**CC8 was not unblocked.**

```
DecEq Bytes  needs  DecEq (List UInt8)        ✓ landed (a generic instance)
             needs  bytes_to_list injectivity  ✓ derivable from SUB-1, zero trust
             needs  DecEq UInt8                ✗ UNWRITABLE
                    UInt8 is PrimReduction::OpaqueType — you cannot case on it.
```

**SUB-1 moved the wall from `Bytes` to `UInt8`. It did not remove it.**

## ★ Why nobody saw it: the bridge delivered the SPINE, and the consumer needed an ELEMENT

**SUB-1's own consumer was `bytes_nat_length = length ∘ bytes_to_list`.** A fold
computes over the **spine** of the list — **it never looks at an element.** It
worked perfectly, proved its laws, and passed every test.

**A key comparison is an ELEMENT operation.**

**So the WP's own acceptance criteria could not have caught this**, and no
reviewer was negligent: *SUB-1 did exactly what SUB-1 claimed, and its tests
exercised exactly what it built.* **The gap was in the reach we ASSERTED for it
on a neighbouring WP's behalf.**

## The check that does catch it — and it takes ninety seconds

**Before you tell a team it is unblocked, TRY TO WRITE THE OBLIGATION the
dependent actually needs, in the vocabulary the dependency actually delivers.**
Follow the chain link by link, and **grep each link at the emission**:

- `DecEq (List a)` — **grep**: landed generic instance ✓
- injectivity of `bytes_to_list` — **derive it on paper**: falls out of the
  retraction by cong-rewrite ✓
- `DecEq UInt8` — **grep**: no instance, no `eq_uint8` primitive, no `DecEqCert`,
  no injectivity law for `uint8_to_int` **anywhere** in `crates/`, `catalog/`, or
  `spec/`. ✗ **STOP.**

**The pen had nowhere to land. That is the answer, and it arrives before a team
burns a day discovering it — and before the only escape left to them is an
`Axiom`, which is the exact disease the whole line of work exists to cure.**

## The general form

**"Merged" is a fact about a branch. "Unblocked" is a claim about
EXPRESSIBILITY.** They are not the same fact, and **the second one is the one you
are actually asserting when you kick a WP.**

**A bridge to a structure gives you the structure's SPINE. It does not
automatically give you its ELEMENTS.** If the elements are themselves opaque, an
element-wise obligation (equality, ordering, comparison, decoding) is **still
unwritable** — and every test that only folds, sums, lengths, or traverses will
be **green**.

**Ask: does my consumer touch the spine, or an element?**

This is [[never-pin-a-shape-that-cannot-state-its-own-contract]] applied at the
**WP-sequencing** layer rather than the design layer, and it is caught by the same
instrument: **try to write the guarantee down and see if there is anywhere to put
it.** Sibling of [[kernel-backed-claim-grep-the-emission-not-the-name]] — *grep
the emission, not the name* — because "SUB-1 makes `Bytes` reasonable-about" is a
**name**, and `DecEq UInt8` is the **emission**.
