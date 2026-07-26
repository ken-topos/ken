---
id: RT-SYMLINK-LANE
title: "SymlinkPolicy is honoured by the interpreter lane and unreachable in the native lane — FollowWithinScope has no native behaviour"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: []
blocks: []
github: null
origin: surfaced during ABI-R1 (2026-07-26) when two opposite universal prose claims about symlink enforcement were BOTH blocked. Node filed by the Steward on the operator's standing directive that discovered work becomes a tracked node rather than a prose aside. Agents cannot create tracked work (COORDINATION §2).
---

> ## ▶ THIS NODE EXISTS BECAUSE A PROSE WP COULD NOT STATE THE TRUTH IN ONE SENTENCE
>
> `ABI-R1` corrected the `Capability/Filesystem` security-boundary paragraph. It
> took **three candidates**, and the two that were blocked were blocked for
> **opposite** universal claims about the same mechanism:
>
> | candidate | claimed | blocked by |
> |---|---|---|
> | `0c8b77fc` | the resolver **enforces** the scope's `SymlinkPolicy` | Architect |
> | `f93a81bd` | resolution **does not consult** it | QA |
>
> ⭐ **Neither universal is true, and that is the finding.** The landed prose
> therefore stops at the true common statement — `SymlinkPolicy` is a **carried,
> per-scope, two-state** mechanism — and says nothing about enforcement in either
> direction. ⛔ **That is an honest floor, not a resolution.** The divergence
> underneath it is a **code** question, which is this node.

## The measurement — full-stream, at `origin/main` 2026-07-26

⛔ **Stated with the pipe named, because the first version of this measurement was
wrong.** A `git grep … | head -20` reported *"no production consumer branches on
the policy"* and the window cut the deciding line off the bottom. Re-measured with
no truncation; counts are `grep -c` over the whole stream, tests excluded.

**Six production reads of a scope's `.symlink` field:**

```
crates/ken-elaborator/src/capabilities.rs:159,160,176,177   meet + diagnostic
crates/ken-interp/src/eval.rs:4040                          -> fs_resolve(..., scope.symlink)
crates/ken-verify/src/scenario.rs:185                        scenario plumbing
```

**Exactly four sites BRANCH on the value, all in the interpreter:**

```
crates/ken-interp/src/eval.rs:2608  if symlink == NoFollow
crates/ken-interp/src/eval.rs:2631  if symlink == NoFollow
crates/ken-interp/src/eval.rs:3356  && symlink == NoFollow
crates/ken-interp/src/eval.rs:3371  if symlink == NoFollow
```

⇒ **The interpreter and virtual lanes genuinely honour the policy.**

### ⛔ And the native lane's shape is NOT "ignores it" — it is "never receives it"

This is the part a one-line summary gets wrong in both directions:

- `ken-host/src/abi_v1.rs:324` `reject_symlink` rejects **unconditionally** — it
  calls `readlink_at` and returns `SymlinkDenied` on success. **It takes no policy
  argument at all**, so there is no branch to write.
- `resolve_fs_root_spec_v1_with_lookup` (`ken-host/src/lib.rs:580`) **takes**
  `symlink: SymlinkPolicy` and **stores** it into the returned `FsScope`
  (`:681`, `:692`) without ever branching on it.
- Every native/host construction site passes the **literal**
  `SymlinkPolicy::NoFollow` — `native_effect_v1.rs:200`,
  `object_linker_packaging.rs:3057`, and ~14 sites in `ken-host` itself.

⚠ **So the native lane is not violating the policy — it is hardcoding the strict
value and then enforcing that strictly.** The result is *safe*, and that is exactly
why nothing has ever failed. **The gap is expressiveness, not permissiveness.**

### ★ The consequence, stated as a property rather than a complaint

`crates/ken-elaborator/src/capabilities.rs:105` defaults a capability to
`SymlinkPolicy::FollowWithinScope`, and `symlink_meet`/`symlink_flows_to`
(`:111`–`:119`) give the two-state lattice real elaboration-time semantics. So:

⛔ **`FollowWithinScope` is expressible in the capability language, carries through
the elaborator's meet, survives into the runtime scope — and then has NO native
behaviour distinct from `NoFollow`.** A program can be granted it, the type system
will track it, and on the native lane it changes nothing.

⇒ ★ **The shape is a value that is inhabited in the authority and uninhabited in
one lane's behaviour.** A two-state policy where one lane implements one state is
not a policy in that lane; it is a constant with a parameter attached.

## ▶ THE QUESTION FOR THE ARCHITECT — this node is NOT a proposed fix

⛔ **Do not read the framing above as prescribing a direction.** There are at least
three coherent answers and they are not small:

1. **Close the native lane over the policy** — thread it into `reject_symlink` and
   the resolver so `FollowWithinScope` means something natively. ⚠ This *widens*
   what native programs may do, which is a security-boundary change, not a repair.
2. **Narrow the authority to what every lane implements** — drop
   `FollowWithinScope`, make `SymlinkPolicy` a single state (or delete it), and
   accept that the interpreter's extra capability disappears. ⚠ This *removes*
   expressiveness the interpreter lane already has and the elaborator already
   tracks.
3. **Declare the divergence intended and NAME it** — the policy is honoured where a
   sandboxed resolver can honour it safely, and the native lane is deliberately
   maximally strict. ⚠ Then the *catalog prose must say so*, and today it cannot,
   because a lane-conditional security claim needs a documented lane taxonomy that
   does not exist yet.

⇒ **The Architect's call, and it needs the `RT-NATIVE-FNSPLIT` context** — option 1
touches the native effect surface that FNSPLIT is actively restructuring.

⚠ **SEQUENCING: this is NOT releasable now.** The fleet is single-threaded on
`RT-NATIVE-FNSPLIT` (operator, 2026-07-25, settled — do not re-ask). This node
exists so the finding is **durable and tracked** rather than living in a merged
WP's retro, and so the question reaches the Architect **before** anyone writes
lane-universal prose about symlinks again.

## ⭐ What this node already bought, before anything is decided

The `ABI-R1` retros converged on the generalization, from three seats:

- **Implementer:** *before authoring a universal prose clause, record its
  provenance as **measured here** or **inherited**.* An inherited universal needs a
  producer/consumer closure sweep across **every** production lane; if that sweep
  is not required by the deliverable, **omit the universal.**
- **QA:** *when the shared invariant is only "carried/expressible", stop there* —
  do not infer an enforcement behaviour from one lane's resolver.
- **Steward (me):** the overclaim that caused the second block came from a
  **truncated** probe, and the untruncated version **still** could not answer,
  because the consuming code tests `== NoFollow` and treats follow as the
  fall-through — so the variant never appears textually. **A grep for a spelling is
  not a measurement of a property.**

⇒ Recorded in `agent/memory/fleet/` as
`a-probe-truncated-before-the-grep-is-not-a-measurement`.
