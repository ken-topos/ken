---
scope: steward
audience: (see scope README) — applies to anyone authoring a WP frame
source: CC3 kickoff, 2026-07-14 (two frame bugs caught pre-edit by the implementer)
---

# A frame that pins a shape must ground the SUBSTRATE that shape needs

When a frame pins an **abstraction over landed types**, grounding the *names*
(the types exist, the functions exist) is **not enough**. Two obligations must
be grounded *before* the pin, or the frame is unbuildable and the ring stalls
on day one. CC3 shipped with **both** bugs — same root cause, caught pre-edit
by the implementer only because the frame told it to escalate false fixed
inputs.

## 1. Dependency DIRECTION — an abstraction module must not depend on its clients

I pinned `byte_cursor_ops` (an instance over CAT-5's `Source`) *inside* the new
`Parsing.Cursor` abstraction module, **and** ordered `Parsing.Cursor` before
`Capability.Parsing` (CAT-5) in the load order — because CAT-5 was to be
refactored to *consume* the abstraction. That is a **cycle**: Cursor → CAT-5 →
Decoder → Cursor. No reordering saves it.

**Rule: the instance lives with the CARRIER it is over, never with the
abstraction.** The abstraction module holds the record/dictionary + laws and
takes **no** dependency on any client. Each client package supplies its own
instance value. The generic module must also define its **own result/error
carriers, parameterized** — the moment it reaches for a client's concrete type
(a `Span`, a `ParseResult`), the cycle comes straight back.

The tell I ignored: I wanted **cosmetic symmetry** ("both instances live in the
Cursor module"). Cosmetic symmetry is what created the cycle.

## 2. Substrate OBLIGATIONS — can the primitive actually PRODUCE the type you pinned?

I pinned `CursorOps.remaining : c → Nat` (a `Nat`, because fuel/termination
needs a structural type) over a carrier of raw `List Bytes`. But
`bytes_length : Bytes → Int`, **`Int` is opaque, and there is no `Int → Nat`
bridge in the prelude.** So the carrier cannot compute its own `remaining` —
the pinned field is **unproducible**. (`list_length` gives only the *arg
count*, which would make seeded fuel unsound for byte-wise advance.)

**Rule: for every field you pin at a structural type (`Nat`, `List`, …), check
the landed primitive can PRODUCE it.** Opaque primitives (`Int`, `Bytes`,
`String`) are **constructible but not destructible** — see
[[opaque-primitive-constructible-not-destructible-format-gap]]. A length,
index, or size read *out* of one is exactly the hop that does not exist.

**The landed idiom (reuse it, don't invent one):** CAT-5's `Source` is a
**proof-carrying cached-`Nat`-length wrapper** over opaque `Bytes` — it caches
`source_length_field : Nat` and carries
`source_length_valid_field : SourceLength unit bytes n`, the proof that the
cached `Nat` agrees with `bytes_length : Int`. It never converts; it **carries
and proves**. The `Nat` is supplied **at construction**, at the boundary — which
confines the opaque-`Int` debt out of every verified law (the same confinement
CC2 ruled).

**The wrong fix is to mint `int_to_nat`** — that is +1 trusted primitive and the
same deferred opaque-`Int` substrate gap CC2 named honestly (`show_int`). A
substrate gap must be **escalated to the operator**, never dissolved inside a
build WP.

## What actually saved it

Not my grounding — **the frame's own escape hatch**: *"treat every anchor as
perishable; if a fixed input turns out false against the landed code, say so and
escalate — do not quietly build around it."* The implementer held edits (branch
clean) and posted the exact tree anchors. **Keep that clause in every frame.** A
T1-authored frame is still wrong sometimes, and it is the only thing standing
between a wrong pin and a ring that silently builds the wrong thing.

**Route these correctly:** module layering / instance homing / which-landed-idiom
is **scope → Steward**, not soundness → Architect (COORDINATION §4). Both bugs
were routed to the Architect by the ring; I ruled them myself and told him to
stand down, keeping his context on the critical-path review he actually owned.

Sibling of [[sizing-a-subsume-fix-enumerate-every-piece]] and
[[brief-settled-input-enabler-must-be-probed]] — a settled input that names an
*enabler* must be probed before it is pinned.
