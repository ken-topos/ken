---
scope: fleet
audience: (see scope README) — anyone changing a spec-pinned/normative API, anyone
  reviewing such a change, and anyone writing the frame that asks for it
source: AX-2, 2026-07-14 — the candidate replaced the frozen
  `trusted_base() -> Vec<GlobalId>` with a new type, kept every behavior, passed
  QA, and was caught only at the Architect's terminal gate
---

# A frozen API is frozen at its TYPE, not at its behavior

**"All the old operations still work" is NOT signature compatibility.** When a spec
pins a signature as normative/stable, the **type** is the contract. A change that
preserves every observable behavior and still alters the type **has broken the
pin** — and it will sail through a behavioral review.

## What happened

`spec/10-kernel/18 §4` pins:

```rust
GlobalEnv::trusted_base(&self) -> Vec<GlobalId>
```

AX-2 needed the trust base to become **readable** (postulates carry names). The
candidate changed the return type to a new `TrustedBase`,
**carefully preserving `len`, `contains`, and owned-ID iteration**
so that every existing assertion in the suite still passed.

**QA approved it, and QA's reasoning was correct as far as it went:** *"historical
ID iteration/contains/len semantics remain available, while names are readable."*
**Every word of that is true.** The suite was green.

**The Architect blocked it.** Preserving the *operations* is not preserving the
*type*:

- downstream **type annotations** (`let v: Vec<GlobalId> = env.trusted_base()`) and
  ordinary `Vec` operations **break**;
- `TrustedBase` **equality now includes labels**, so `==` means something new;
- its owned iteration has a **different element view** from `entries()`.

**The close required no design change at all:** keep `trusted_base() -> Vec<GlobalId>`
exactly, and read each label through the **already-required** `Decl::Opaque.name`
via `env.lookup(id)` — *which is precisely what the landed §18 contract already
says*. Any convenience projection must be **additive under a distinct name**, never
an implicit replacement of a frozen method.

## ★ Why a green suite is structurally blind to this

**Because the same diff migrates every in-repo caller.** After the change, nothing
in the workspace still asks for a `Vec<GlobalId>` — so nothing can fail. The
breakage lands on:

- **future** code written against the spec'd signature,
- **out-of-repo** consumers,
- **the spec itself**, which now describes a method that does not exist.

**None of those are in the test suite, and none of them ever will be.** This is the
same family as the expressibility gap: **the defect lives in the TYPE surface,
where no value-level test looks.** *(See
[[never-pin-a-shape-that-cannot-state-its-own-contract]] — tests exercise VALUES;
this break is in the shape.)*

## The rule

> **Before changing any spec-pinned API: diff the SIGNATURE against the spec text,
> character by character. "Behaviorally compatible" is not an answer to "is the
> type the one the spec pins?"**
>
> **If you need to add capability to a frozen method: ADD A METHOD.** A frozen
> surface grows **additively, under a new name**, or it is re-pinned by an explicit
> spec change. It is **never** widened in place and called compatible.

**For reviewers:** when a diff touches a normatively-pinned symbol, **the green
suite is not evidence.** Go read the spec line and compare the type.

## ★ And the frame's wording invited it — the Steward's error

My AX-1 frame said the trust base must become readable *"**without churning existing
`trusted_base_delta` assertions**."* That is an **assertion-level** requirement, and
the implementer **satisfied it exactly** — by keeping the operations and changing
the type. **I asked for behavioral compatibility and got it; the spec wanted type
compatibility.**

**⇒ When you frame a change to a frozen surface, state the bar as the TYPE, not as
the tests:** *"the signature `f(&self) -> Vec<T>` is pinned by `spec/… §N` and must
be byte-identical after this change"* — **not** *"don't break the existing
assertions."* **The second sentence is a strictly weaker requirement, and a
conscientious implementer will meet it and still break the contract.**
