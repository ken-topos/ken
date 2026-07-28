---
id: RT-FNSPLIT-C3-ACTIVATION
title: "the opaque activation owner — one Rust representation authority in ken-runtime that constructs, publishes and tears down per-invocation boundary storage, with the deployment-supplied capacity profile and the one-argument public adapter seam"
status: merged
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-C1, RT-FNSPLIT-B2R, RT-FNSPLIT-B2V]
blocks: [RT-FNSPLIT-B2F]
github: 1181
origin: Architect corrected ruling evt_2yjg12pyqqjdv, bound to exact B2F checkpoint 7ce4198f, which supersedes the owner/capacity/scope parts of the arena ruling evt_1m082dp6xf0mw. Two premises of that ruling were falsified by measurement — runtime-implementer evt_5nc20xj7h9e31 (no production activation owner exists on either launcher path; ken-runtime is rlib-only so BoundaryArenaV1::publish is unreachable from the linked C stub) and Steward evt_53gz05jhsz2fp (the capacity numbers are caller-supplied parameters, so the "existing store/plan capacity authority" has no referent). "Fold it into S6/D6" and "capacity comes from the existing authority" are both retracted, and the former fail-closed clause is withdrawn because applying it would turn every linked executable from working to non-starting. The Architect directed the Steward by name to frame and register this node. Same shape as B2V (evt_28cnmxf6ncghn on B2F hard-stop #10), C1 (evt_7ay6s5s79awz8 on #11) and C2-SYNTH-ID (evt_xf4znbnb6vz9): one prerequisite inserted ahead of B2F rather than a widening inside it. Steward-filed; Steward owns the frame and the AC/control placement.
---

# `RT-FNSPLIT-C3-ACTIVATION`

> ## ✅ MERGED 2026-07-28 — PR #1181, `main 747addce → 49e62ed8`
>
> Exact `c63a5a9d`, tree `d8ffa43c`; landed tree `c799506b`. ⭐ `git diff
> c63a5a9d origin/main -- crates/` is **empty** — every code path landed
> byte-identical. Decision `dec_78ccs36yskgfk`, resolved APPROVE by the
> Architect. Runtime QA APPROVE `evt_dkpq53qmht7v`.
>
> ⚠ **One rejection preceded it.** `394c4afe` was blocked
> (`dec_aedje7f4vc9j`, Architect `evt_4rr5fhzhhqv5d`): the measurement ledger
> said both generated C starters had removed `KenNativeBigEntryV1`, but
> `object_linker_packaging.rs` still **emitted** it inside
> `process_starter_c_stub_for_authority`. ⛔ **A dead declaration is still a
> private C copy of the native-`Int` layout** — the exact shape `AC-5` bans —
> and build/link success does not discriminate it.
>
> ⭐⭐ **How it survived QA is the part worth keeping:** the `AC-5` residual was
> stated as an honest **partition** (*"removing two known copies doesn't prove a
> third was never added — review-enforced"*), and QA confirmed the partition was
> honestly stated. ⛔ **Nobody re-verified the factual half underneath it** —
> that the two known copies were actually gone. One wasn't.
>
> ⭐ `AC-4` closed as **six** generated-code fill-to-ceiling controls, each
> attributing its own `(scope, resource)`, plus **two** cells *measured*
> unreachable by the admitted relation with a guard that reddens if that relation
> is expanded. ⚠ The fill-to-ceiling step came from the control **failing**:
> `limit + 1` against an *empty* region is refused, so nothing is bumped and the
> attribution names nothing.
>
> ⛔ **Residual, unguarded and stated as such:** an *arbitrary additional* private
> C copy is **review-enforced**. The only mechanical repair available was a
> source-line census test, declined per the operator test policy that oracles
> assert behaviour, not source lines.
>
> ⇒ ⭐ **`RT-FNSPLIT-B2F` now waits on `RT-FNSPLIT-C2-SYNTH-ID` alone.**

`B2F`'s `S6`/`D6` switch-over needs the generated root and every unit to take
`(frame_ptr, services_ptr)`. Nothing in production can supply that second
argument: no launcher constructs or publishes a `BoundaryArenaV1`, and the
object-linked starter cannot be taught to — `ken-runtime` is rlib-only, so the
publishing authority is unreachable from the C stub it links.

This node creates the missing owner as **one Rust representation authority in
`ken-runtime`**, exported as an object-linkable static library behind an opaque
activation handle, together with the deployment-supplied capacity profile that
sizes it and the one-argument public adapter seam that lets `B2F` change the
internal convention without touching the public C representation.

⛔ `B2F` is **held** until this node is durable. ⚠ The hold is a **capability**
boundary — no per-function binder, no shared-root signature change, no `S6`/`D6`
reland — ⛔ **not a SHA**: `B2F`'s tip advances under the authorized independent
`AC` work, so any SHA written here would be stale on the next permitted commit.

Frame: `docs/program/wp/RT-FNSPLIT-C3-ACTIVATION.md`.

## Sequencing — first of Runtime's two prerequisites

Runtime sequenced **this node first, then `RT-FNSPLIT-C2-SYNTH-ID`**
(runtime-leader `evt_35karwwpdas3g`, 2026-07-28); the branch
`wp/RT-FNSPLIT-C3-ACTIVATION` was cut from exact `6187d147`.

⛔ **Scheduling, not dependency** — the two nodes are independent and both
remain `ready`. ⚠ `B2F` waits on **both**, so neither ordering shortens its
hold; this one is first because it is the larger unknown.
