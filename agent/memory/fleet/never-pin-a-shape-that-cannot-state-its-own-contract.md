---
scope: fleet
audience: (see scope README) — anyone pinning a trait, a type, an effect op, or a spec claim
source: Architect synthesis, 2026-07-14 — the general form behind three consecutive saves
---

# Never pin a shape that cannot state its own contract

> **The question: "Every obligation this shape must carry — where does it get
> WRITTEN, and is that place inside the shape's own checked vocabulary, or a
> reach outside it?"**

Three consecutive expensive near-misses turned out to be **one failure**. A
**shape** is pinned — a trait, a type, an effect op, a spec claim — that must
**carry an obligation**. The obligation is **not false**, and **not expensive to
prove**. It is **unsayable in the shape as pinned**: there is **no term, type,
handle, or method *in the shape* where it can be written and checked.**

| WP | the shape | the obligation | where it had no home |
|---|---|---|---|
| **I-6** | `HostHandler` trait | "any host mints its own-identity cap" | trait declared no mint → the obligation lived on the **concrete** type; the generic path could not reach it |
| **Erratum** | the conversion relation | `byteLength "abc" ≡ 3` | `conv` has no `Op` arm → **the spec said it anyway, in prose the kernel cannot check** |
| **I-8** | `monotonic_now : {Clock} → Int` | "this read ≥ the last" | two `Int`s, **no handle relating them** — the law is unsayable |

**Same hole every time: the shape cannot state its own contract.**

## ★ Why it is invisible in a green diff — the load-bearing part

**Tests exercise VALUES. The gap is in the TYPE / ABI / RELATION surface.**

The values compile and the tests pass **because the missing thing was never a
value in the first place** — it was **a place to write a guarantee**. No
value-level test can reach it. **No amount of green touches it.**

**The gap surfaces only when someone tries to WRITE the obligation and finds
there is nowhere to put it.**

⇒ **The audit is not "run the suite."** It is:

> **"Try to write each obligation in the shape's own vocabulary, and see if your
> pen has anywhere to land."**

All three saves came from exactly that: someone tried to write *mint* from the
trait, *the equality* in conversion, *monotonicity* over two `Int`s — and found
**no home**.

## The escape hatches — every one is a door the gap arrives through

When the shape can't say it, the build **reaches outside**. Each reach is a
distinct failure mode, and **all of them mean the same thing**:

- a **comment** — PRINCIPLES **#14**, the original and narrowest form;
- a **per-consumer `Axiom`** — **unbounded TCB** (the monotonic-clock door;
  see [[primitive-ops-do-not-reduce-under-conversion]] and `PRINCIPLES #15`);
- a **new trusted primitive** — TCB growth;
- a **caller-fabricated value** that manufactures the missing binding (e.g. a
  cap divorced from the host that minted it);
- a **concrete-only method** the generic path silently cannot reach (see
  [[making-a-concrete-path-generic-enumerate-every-op-against-the-trait]]).

*The shape had no home for the obligation, so we put it somewhere nothing
checks, or somewhere that costs trust.*

## ⇒ This is PRINCIPLES #14, widened

**#14 already states the whole thing** — *"when a required fact has no
language-proper home, extend the language or file the gap, never enshrine it."*
The three saves are #14 applied to a **trait ABI**, a **conversion relation**,
and an **effect-op type** instead of a `--` comment. **The comment was only ever
the most obvious of the escape hatches.**

## The audit, actionable

For the shape being pinned:

1. **Enumerate its contract obligations.**
2. For **each**, **name the in-shape checkable home** — the exact term, type,
   handle, or method where it is written **and the kernel/elaborator checks it.**
3. Any obligation whose **only** discharge is a reach-outside is an
   **expressibility gap**. Then either:
   - **extend the shape to give it a home** (a handle, a session, a trait
     method, a kernel rule) — **and if that home grows the TCB, it is the
     OPERATOR's call, not the build's**; or
   - **descope the obligation honestly** (I-8 shipping wall-clock-only, with
     **no** ordering law, because a wall clock genuinely has none — the absence
     of a law is the *truthful* statement, not a gap).

**Never pin a shape that cannot state its own contract.**

## ★ And why this is structurally the DESIGN pass's catch

- The **implementer builds values** — they compile.
- **QA tests values** — they pass.
- **Only the design review asks whether the *shape* can express its *contract*** —
  a question about the **type/ABI/relation surface** that neither
  value-construction nor value-testing ever touches.

**That is not a coincidence about these three WPs; it is why the design-review
edge exists at all.** The expensive failures are expressibility failures
**precisely because expressibility is the one property that lives entirely above
the value layer** — where the only instrument that reaches it is someone trying
to write the guarantee down. **Point the audit there.**
