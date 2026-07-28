---
id: PX8-F-CAP-41
title: "PX8 clause-(a) behavior blocker — closed buffer endpoint (start==capacity) must derive zero-effective ReadEof, not host-reject"
status: draft
owner: foundation
size: M
gate: none
depends_on: [NATIVE-HANDLE-CARRIER]
blocks: [PX8]
github: 41
origin: charter backlog (#41); RE-GROUNDED as a live PX8 clause-(a) blocker by architect verdict evt_163mfgjs7fkh8 (2026-07-23); RE-SCOPED spec-first by architect ruling evt_xnkrzjy1c8br (2026-07-23)
---

> ## ⭐⭐ 2026-07-27 — Phase 2 IS FRAMED, and it is NOT a separate WP
>
> **Phase 2 and [[NATIVE-HANDLE-CARRIER]] are ONE deliverable and close in ONE
> merge.** The frame is `docs/program/wp/NATIVE-HANDLE-CARRIER.md` (owner
> **Runtime**, size **M**), measured at `origin/main = 5404108a`.
>
> ⛔ **Do not frame or release Phase 2 separately.** The carrier fix is
> meaningless without the fixture it unblocks, and the fixture cannot land
> without the fix — the Architect ruled an honest partial inadmissible here
> because the candidate *regresses* an already-GREEN native row. ⇒ Flip **both**
> nodes on the same merge.
>
> ⭐ **The Phase 2 WIP is the frame's input**, and it needs no separate fold:
> `f0eb65ce` is the **parent** of the carrier's `c07e63c2`, so `c07e63c2` alone
> already carries the handle/admission impl. The text below says to "fold with
> `f0eb65ce`" — that is a measured premise error; there is nothing to fold.
>
> ⚠ Both refs are recorded as `preserved/*`. A handoff-gate "preserved at …"
> line is a claim, ⛔ not a resolved ref — resolve every name you write down.

> ## ⚠ STATUS CORRECTED `active` → `draft` — 2026-07-25 (Steward, tracker honesty)
>
> **Nothing is building this, and its `depends_on` is unmet** (NATIVE-HANDLE-CARRIER (draft)). The tracker's own
> legend defines `draft` = *not framed / **deps unmet***, `ready` = *deps met,
> unassigned*, `active` = ***a team is building***. So `active` was a false claim
> that a seat held this node, and it polluted the releasable-frontier read that the
> next sequencing pass depends on.
>
> ⛔ **Blocked-ness is DERIVED, never spelled in `status`.** `gen-progress.sh`
> computes the frontier as *`ready` **and** every `depends_on` merged*, and lists
> blockers separately. ⇒ Do not invent a "blocked" status; fix `depends_on` and let
> the generator say so.
>
> ⇒ **Flip to `ready` only when every `depends_on` entry is `merged`.**

## ⚠ RE-SCOPED spec-first 2026-07-23 (two-phase, like [[PX8-SPAN-PROV]])

**The "bounded S-sized prelude fix" premise is FALSE on `origin/main@cbf6a298`.**
On intake, foundation-implementer (evt_7gy73496fwn1p) found — and the Architect
ruled (evt_xnkrzjy1c8br) — that **capacity is not observable in checked Ken**:
`BufferWindow` carries only caller-forgeable `(start, length)`, `Resource Buffer`
is opaque, and `readAt` receives no acquisition capacity. So the checked path
**cannot distinguish `start == capacity` from `start < capacity` or `start >
capacity`**; a caller-supplied capacity would be forgeable (unsound bypass) and a
host primitive would violate the locked no-host-visit postcondition. The RED seed
records this exact missing observation (`seed-buffer-io.md:602-607`).

**Sound mechanism (Architect) — the same unforgeable-acquisition-bound idiom just
merged as `PrivateBufferSpan` in [[PX8-SPAN-PROV]]:** bind capacity to the
**acquisition**, not the request. A constructor-private checked handle
`data BufferHandle = PrivateBufferHandle (Resource Buffer) Int` where
`withBuffer capacity` is the **sole constructor** (stores the just-allocated
capacity); private constructor/projections out of the public name map;
`withBuffer` passes the handle to the body; `readAt`/`writeAt`/`spanBytes` accept
it; release + private host ops project the underlying exact `Resource Buffer`;
`BufferWindow` stays the public raw request descriptor. Host resource token + ABI
unchanged; unforgeable by checked user code. Derived `readAt` admission then:
validate host-width/nonneg `fileOffset`; validate nonneg `start`/`length` +
`start <= capacity`; `effective = min(length, capacity - start)`; derived
`ReadEof`/no-primitive iff `effective == 0`; else `PrivateFsReadAt` on the
effective range (start 8/len 4 cap 8 ⇒ EOF/no-host; start 9 ⇒ `InvalidBounds`;
invalid offset ⇒ error). Keep host `checked_buffer_range` as defense in depth.

**Because this changes the checked buffer-handle surface + all buffer consumers,
it is two-phase:**
- **✅ Phase 1 (spec-first, SPEC ENCLAVE) — MERGED @ `origin/main = 8ebe370a`
  (PR #915, 2026-07-23).** Folded the capacity-carrying `BufferHandle` into §38 +
  the `Resource Buffer` signatures + prelude API; locked the ordered admission
  algorithm + the four absolute `seed-buffer-io.md` rows (RED-until-Phase-2). CV
  independent fold + Architect soundness APPROVE, exact `60cecd3b`. Enclave retros
  requested (thr_220eqm77azw9v).
- **⛔ Phase 2 (impl, FOUNDATION) — BLOCKED 2026-07-23 on [[NATIVE-HANDLE-CARRIER]].**
  Kicked off `8ebe370a` (root `evt_4ea3p6r302xq3`); foundation-implementer completed
  the checked handle + admission + fixtures and hit a **native-lowering hard-stop**
  (evt_563ss8821n7f): the sealed `BufferHandle` does not lower on the native path
  (`Driver(MissingClosureMetadata …)`, pre-Cranelift, pre-erasure — *distinct* from
  RT-NATIVE-FNSPLIT's VReg wall). Architect means/representation ruling
  `evt_2zkjr68y1sdgf`: representation stands; fix the compiler layer; do **not** land
  as an honest partial (it *regresses* an already-GREEN pre-existing 2-bracket native
  row). Interp half is confirmed GREEN on all four rows at `f0eb65ce`. **WIP preserved
  durably on origin** as `wp/PX8-F-CAP-41-p2-buffer-handle @ f0eb65ce`. Foundation
  re-sequenced onto [[NATIVE-HANDLE-CARRIER]]; on that fix, **fold with `f0eb65ce` and
  run the full two-engine oracle** — Phase 2 then lands complete. The (superseded)
  re-sized
  derivation + consumers in `crates/` (elaborator prelude: `PrivateBufferHandle`
  repr + `withBuffer` mint + migrate `readAt`/`writeAt`/`spanBytes`/`freeze`/
  `writeAll` to consume the handle + the derived-`readAt` capacity admission;
  ken-host keeps `checked_buffer_range` as defense in depth, private ops project
  the raw `Resource Buffer`). Oracle = flip the four RED seed rows GREEN (both
  engines). **Sized M.** Foundation cuts a fresh branch off `8ebe370a`.

**No longer merely deferred backlog — the Architect's PX8 closure-property
verdict (`evt_163mfgjs7fkh8`) identifies this as a *live clause-(a) behavior
blocker*.** PX8 cannot close while it is red.

## The defect (Architect-grounded, exact anchors)

LOCKED §38 says `0 <= start <= capacity`; a **positive raw request at
`start == capacity`** has zero effective length and the **derived wrapper must
return without invoking the positive primitive** — a zero-effective `ReadEof`
path with no host visit (`spec/30-surface/38-ffi-io.md:404-408`). Current
`readAt` bypasses the primitive **only when the caller's raw length is zero**
(`crates/ken-elaborator/src/prelude.rs:1977-1986`). A **positive** raw length at
the closed endpoint therefore reaches host dispatch, where `checked_buffer_range`
rejects every `effective == 0` as `InvalidBounds`
(`crates/ken-host/src/effect_v1.rs:1736-1750`).

⇒ The reified checked value is `Err InvalidBounds` where the locked contract
requires the derived zero-effective `ReadEof`. The conformance record states this
explicitly and remains **RED** (`conformance/behavioral/buffer-io/
seed-buffer-io.md:598-614`) — that RED row is the acceptance oracle.

## Scope note / queuing gate

The fix site is bounded (`prelude.rs:1977-1986`: broaden the primitive-bypass
condition from raw-length-zero to effective-length-zero at the closed endpoint,
routing to the derived `ReadEof`), with a LOCKED spec anchor and a RED
conformance row as the absolute oracle ⇒ **conformance/CV in the review lane.**

⚠ **History:** the operator's queuing gate was RELEASED (operator "the plan sgtm"
2026-07-23) and this was kicked to the Foundation ring as Track 2 of two impl
tracks — but **on intake it re-scoped spec-first** (see the banner above): §38
locked the *behavior contract* (derived `ReadEof`/no-host at the closed endpoint)
but the tree has **no capacity representation** to drive that admission, so
Phase 1 must fold the capacity-carrying `BufferHandle` into the spec + API before
any impl. Current status: **Phase 1 MERGED @ `8ebe370a` (PR #915); Phase 2
(Foundation impl) now ACTIVE off that lock.**

Track 1 [[RT-NATIVE-FNSPLIT]] (Runtime) continues unaffected. Sibling clause-(a)
evidence gap: [[PX8-WROTE-ABS]] (A2 — still needs the operator's normative scope
call); clause-(b) gap: [[PX8-SPAN-PROV]] (✅ merged, the idiom this reuses); root:
[[PX8]].
