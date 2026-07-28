---
id: PX8-WROTE-ABS
title: "PX8 clause-(a) evidence gap — interpreter capped-short Wrote lacks an absolute oracle; PR-C error identities unreached"
status: merged
owner: verify
size: S
gate: none
depends_on: []
blocks: [PX8]
github: null
origin: architect PX8 closure-property verdict evt_163mfgjs7fkh8 (2026-07-23); Steward-filed (agents cannot create tracked work per COORDINATION §2)
---

> ## ✅ MERGED 2026-07-28 — PR #1142, `origin/main = 45647b51`
>
> **Candidate `d5a938c496bbc758d0361711693504bcb673195f`** (tree
> `42d7109dafcf6aeef913ce56762113fd491ade74`), Decision `dec_7tvjg6e79dnwm`
> (Architect APPROVE, resolved `2026-07-28T00:12:28Z`). Blob-verified on `main`:
> `crates/ken-interp/src/eval.rs = 57041e577a18e6c5065c85722b06f97725346e10`.
> One path, +188 lines, full CI green.
>
> **What landed:** the capped-short `Wrote` absolute oracle at the **component
> boundary** — a test-local `HostEffectBackendV1` short write → the real
> `dispatch_host_op_v1` (which validates and mints the private
> `TransferCountV1(2,4)`) → the existing `reify_host_reply_v1` → the LOCKED
> `§38.1.7.2` literal `remaining = 2`. ⛔ Production unchanged: no seam, no public
> constructor, no `cfg(test)` production hook, no relaxed visibility.
>
> **Both wrong shortcuts are now discriminated** (`AC-1‴`): `effective := count`
> fails the new short test while capped-full stays green; the raw-request-length
> substitution fails both. The `ReadSome` arm at `:5303` is untouched and its
> capped-short read test stays green under both.
>
> ⭐ **This closes `A2a` and it is `PX8`'s first blocker to discharge.**
> ⚠ `A2b` remains as [[PX8-ERRID-SCOPE]], now blocked behind
> [[PX8-ERRID-ALLOC]] → [[RT-NATIVE-FNSPLIT]].
>
> ### ⭐⭐ It also settled a question the size wall had opened
>
> `d5a938c4` passed the **same** `rt_parity_native` job that reddened PR #1141
> (`PX8-ERRID-ALLOC`) on Cranelift `Code for function is too large`. ⇒ **the
> code-size ceiling is NOT general** — it is specific to native lowering growth
> from the added resource-error alternative. This WP adds no native alternatives.

## ⭐⭐ SCOPED AND FRAMED 2026-07-27 — A2a only; A2b split to `PX8-ERRID-SCOPE`

**Frame:** `docs/program/wp/PX8-WROTE-ABS.md`, inputs pinned by blob at
`origin/main = 12a5ef4f`. **Owner Verify** — `BUDGET-EFF`, the WP that authored
this oracle family and whose in-source comment states the discipline, was
`owner: verify`.

⭐ **This is the only unblocked node on the Linux ABI I critical path.** `PX8`
gates 15 of that program's 19 nodes; `PX8` needs this, [[PX8-ERRID-SCOPE]], and
[[PX8-F-CAP-41]]. The other two are blocked — `PX8-F-CAP-41` behind
[[NATIVE-HANDLE-CARRIER]] ← [[RT-NATIVE-FNSPLIT]], and `PX8-ERRID-SCOPE` behind
a normative call. **This one depends on nothing and sat unowned and unframed.**

⛔ **A2b below is NOT in the framed scope.** It needs the Architect's route-2
normative call before it can be sized, and is filed as [[PX8-ERRID-SCOPE]].

> ## ⚖️ RULED TWICE 2026-07-27 — and the second ruling REVERSES the first
>
> ⚠ **This node previously said the cell might be *inexpressible*, a full
> deliverable in the class of [[CONF-FMT8-LEVELTOK]] / [[CONF-SEC4-REFL-PAIR]].
> ⛔ That is now FALSE.** Sequence, so the reversal is legible:
>
> 1. **Steward `evt_1grq3fcfkz4yy`** — the frame's capacity-vs-installed-window
>    dichotomy was false in **both** branches. Ruled `D3`'s inexpressibility on a
>    four-site census: capacity-backed `effective`, live-window admission,
>    `InterpreterHostBackend` not overriding `fs_resource_write_at`, and
>    `TransferCountV1::new` being `pub(crate)`.
> 2. ⛔ **Architect `evt_5h884g6xhtts3` REVERSES it.** Rows 3 and 4 are true
>    facts with a **false consequence**: the seam is the **`pub trait
>    HostEffectBackendV1`** (`effect_v1.rs:1214`), not the one concrete backend
>    that declines to override it — `dispatch_host_op_v1` calls it through the
>    trait at `:1801-1803` — and **ken-host mints the `TransferCountV1` itself**
>    at `:1811`, so the constructor's visibility never mattered.
>
> ⇒ ⭐ **The census proves only that no end-to-end regular-file fixture exists —
> a reachability limit, not a semantic absence.** LOCKED `§38.1.7.2` admits
> `0 < n <= effective` including a short write.
>
> ✅ **`D1` is available at the COMPONENT boundary** and is the required
> discharge: a test-local `HostEffectBackendV1` returning a short write → the
> real `dispatch_host_op_v1` → the real minted count → the existing
> `reify_host_reply_v1` → assert `remaining = 2`. ⛔ Production unchanged; ⛔ a
> comment-only deliverable does **not** discharge `PX8`.
>
> ⭐ Same component shape native already uses. Requiring an interpreter *OS*
> fixture while accepting native's *component* evidence was an accidental
> asymmetry — and that asymmetry is what read as inexpressibility.

Surfaced by the **Architect's PX8 closure-property verdict** (`evt_163mfgjs7fkh8`)
— clause (a) *absolute-not-differential* evidence is **not** discharged for two
value populations on the positioned/partial IO path. This is a clause-(a)
**evidence** gap (the source formulae are presently right; they are not
*asserted absolutely*), distinct from the clause-(a) **behavior** gap in
[[PX8-F-CAP-41]] and the clause-(b) provenance gap in [[PX8-SPAN-PROV]].

## The gap (Architect-grounded, exact anchors)

### A2a — interpreter capped-short `Wrote` has no absolute oracle
Interpreter `ReadSome` has capped-full **and** capped-short absolute assertions;
native has capped-full and capped-short for both `ReadSome` and `Wrote`.
**Interpreter `Wrote` has only capped-full** (`crates/ken-interp/src/eval.rs:6274-6379`).
Its distinct reifier arm is `eval.rs:4981-4997`, and the wrong shortcut
`effective := count` is **green when full** because both yield `remaining == 0`.
There is no interpreter capped-short `Wrote` assertion corresponding to native's
load-bearing `raw 8 / effective 4 / count 2 / remaining 2` case
(`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/effects.rs:425-455`).
The closure condition requires this value asserted **absolutely** against LOCKED
`spec/30-surface/38-ffi-io.md`, and it is not.

### A2b — several PR-C error identities have no independent reaching evidence
`MalformedResource`, `InvalidBounds`, allocation-failure-distinct-from-`BufferLimit`,
unsupported-nonblocking posture, and host-I/O-failure-distinct-from-`Interrupted`
have **no independent reaching evidence** (`conformance/behavioral/buffer-io/
seed-buffer-io.md:619-645`). These are values reified by the positioned/partial
path, so the universal absolute-evidence claim of clause (a) cannot be made yet.

## Disposition / open question

Two admissible closure routes (Architect's verdict):
1. **Add the evidence** — the interpreter capped-short `Wrote` absolute oracle
   (mirroring native's `effects.rs:425-455` case) + independent reaching tests
   for the five error identities, each asserted absolutely against §38.
2. **Narrow the root property normatively** — if some error rows are out of the
   intended positioned/partial closure scope, the *current universal text of the
   PX8 property includes them*, so narrowing is a **spec/normative decision**
   (spec enclave + operator), not a silent scope trim.

⇒ Needs a scoping call (which error rows are in-scope for PX8 closure) before
sizing. The `Wrote` oracle (A2a) is a bounded test addition and is the
shovel-ready core; A2b's error-row set may split by the normative call.
Fix site crate: `ken-interp` (oracle) + conformance (`seed-buffer-io.md`) → **CV
in the review lane.**
