---
id: PX8-SPAN-PROV
title: "PX8 clause-(b) gap — BufferSpan carries no originating-buffer identity; freeze accepts a same-shape span from a different buffer"
status: merged
owner: spec-enclave
size: M
gate: none
depends_on: []
blocks: []
github: 914
origin: architect PX8 closure-property verdict evt_163mfgjs7fkh8 (2026-07-23); Steward-filed (agents cannot create tracked work per COORDINATION §2)
---

## ✅✅ CLOSED — PX8-SPAN-PROV COMPLETE (both phases merged, 2026-07-23)

Both phases landed; the clause-(b) same-buffer provenance gap is discharged.
- **Phase 1** (spec §38 + conformance) merged @ `origin/main b64ad9f3` (PR #913).
- **Phase 2** (route-1 crate implementation) merged @ `origin/main =
  cbf6a298d33c650baf33c08c25557de2e6089fd4` (PR #914, squash of `b5dbf176`),
  content-verified byte-identical to the reviewed tree; main push-CI green (run
  30008017125). 5 merge Decisions (4 rejected rounds — each caught a real
  distinct gap: interp-matrix completeness ×3, then a CI-red stale-test scope
  gap — no repeats); Architect (soundness) + CV (spec/conformance) both APPROVE
  on the final exact SHA (`dec_1ds1vb4y5anj7`).
- **Landing shape:** operator-approved option (a) — proven mechanism lands with
  honest-partial native rows (SP-A-write/SP-B/SP-C native =
  `BLOCKED-ON-NATIVE-REACHABILITY`, interpreter GREEN), native cells flip on the
  named follow-up [[RT-NATIVE-FNSPLIT]].
- **Retros in:** runtime-qa `evt_526a6kxka4vsx`, runtime-implementer
  `evt_1s2bdq7k3hrh3`, runtime-leader `evt_6xc71w2axea3g`. Adversary deep-hunt
  `evt_4eb0tx3bjpsy9` — no findings ("cleanest large change this session").
- **PX8 root:** clause-(b) discharged; clause-(a) [[PX8-F-CAP-41]] +
  [[PX8-WROTE-ABS]] remain, so [[PX8]] stays open.

---

Surfaced by the **Architect's PX8 closure-property verdict** (`evt_163mfgjs7fkh8`)
— the numeric request/effective/span-length/count co-indexing IS closed
(host mints span+count from one `(start, effective, read)`; `TransferCountV1`
inseparably carries `(transferred, effective_request)`; SPAN-SEAL + SEAL-2 close
checked-source carrier producers). But the property's explicit **same-buffer**
portion is **not** discharged.

## The gap (Architect-grounded, exact anchors)

`BufferSpanV1` contains only `{ start, length }` (`crates/ken-host/src/effect_v1.rs:2044-2048`);
the checked `BufferSpan` likewise carries only start + structural length. **No
buffer identity travels in the span or is checked at `freeze`.** `BufferFreeze`
resolves the caller-supplied target buffer and validates the numbers only against
*that target's* current initialized window (`effect_v1.rs:655-666,1582-1596`).

**Counterexample that both engines share (so interp==native cannot detect it):**
initialize buffers A and B with the **same** live window; obtain `span_a` from A;
call `freeze B span_a`. The numeric check accepts and returns B's bytes — but
`span_a` is **not the current span of B**, contrary to §38's "current live
subrange" / "validates a current span" contract
(`spec/30-surface/38-ffi-io.md:374-388`) and PX8's same-buffer clause. Producer
closure (SPAN-SEAL/SEAL-2) cannot prove **consumer provenance**.

⚠ RT-ESCAPE's R2 oracle does **not** close this: it uses a length-6 span from A
against capacity-2 B, so ordinary bounds alone reject it
(`crates/ken-cli/tests/rt_escape_second_resource_native.rs:352-360,633-655`). The
missing discriminator is the **same-shape** two-buffer case.

## ✅ ARCHITECT MECHANISM RULING (evt_4bb5hr3n38pr1, 2026-07-23) — size M

**Route (1): origin-bound private `BufferSpan`.** Route (2) (no-representation
relation at `freeze`) is **impossible** for the same-shape replay — after
excluding the caller-supplied buffer, `span_a`/`span_b` present identical
consumer inputs, so any deterministic check accepts/rejects both; an identity
side-table isn't copy-stable / engine-agnostic; a copy-surviving nonce *is* a
hidden span field = route (1).

- **Representation:** `data BufferSpan = PrivateBufferSpan (Resource Buffer) Int Nat`
  — field 0 constructor-private = the **exact originating `ResourceTokenV1`**
  (slot+generation) copied from the live buffer operand of the successful
  `readAt`. Use the token (not `ResourceTraceIdentityV1`, not a new nonce): Ken
  can't mint it, generation binds acquisition lifetime (close/reuse can't alias),
  both engines already have the operand at reification, no second identity system.
  **All public projections/signatures unchanged; origin not projectable.** Every
  reconstruction (esp. `write_all_advance_span`) preserves it byte-for-byte,
  changing only start/budget.
- **Reification — bind from the REQUEST SEAT, NOT the wire.** Do NOT put the live
  token in `CanonicalReplyV1::ReadProgress`, the effect trace, or `BufferSpanV1`.
  interp: retain `ResourceInputsV1::FileBuffer.buffer` for `FsReadAt` → pass into
  `reify_host_reply_v1`; native: retain the lowered `buffer` operand in
  `lower_process_host_effect` → field 0. Both constructor sites already have the
  operand (native `lowering/core.rs:5161`; interp `eval.rs:4966`).
- **Admission — protect BOTH consumers** (`freeze`/`spanBytes` AND `writeAt`→
  `writeAll`; writeAt currently slices by the foreign span's numbers, same
  replay). Pass the origin token through the private effect op with named fields:
  freeze `{target_buffer, span_origin}`; write `{file, target_buffer,
  span_origin}`; keep read `{file, buffer}` (split read/write native ABI structs
  rather than add a meaningless read origin). Canonical host dispatcher compares
  `target_buffer == span_origin` on the **full opaque token** BEFORE slicing/
  backend I/O; only then the existing live-kind/window validation. Mismatch =
  fail-visible, **zero backend calls**. Both engines route the SAME host check
  (compiler-only/interp-only is insufficient).
- **§38 / Spec / CV — pulls the path, narrowly.** §38 must state: a span is bound
  to the exact acquisition that produced it; `freeze`+`writeAt` reject a
  foreign-acquisition span even when capacity/start/length/live-window match; and
  the exact `ResourceError` identity+precedence. Architect *recommends the
  existing `InvalidBounds`* (no new surface sum member) — but that identity is
  **Spec-enclave normative authority**, not this ruling's to create. **Merge
  Decision needs Architect + Spec/CV votes** (spec/conformance + crate travel
  together, or spec-first).
- **Acceptance (each engine, absolute):** A,B same capacity + same numeric live
  window, distinct bytes; `freeze B span_a` rejects (locked error, no B bytes);
  `freeze B span_b` succeeds (B's bytes); `writeAt … B span_a` rejects before
  backend write; `writeAt … B span_b` succeeds; close/realloc no-revive via slot
  reuse. **Mutations:** delete token-equality → both foreign tests fail; reduce
  to start/length → fails on same-shape pair; swap the reifier's captured request
  token → caught; enumerate every `PrivateBufferSpan` creation supplies origin +
  every subspan preserves it. **Retain SEAL-2's producer-closure** (private
  constructor stays hidden; no new checked-source producer).
- **Size M.** Fence: `ken-elaborator` prelude/constructor arity, interp reifier +
  private-effect args, runtime/native lowering, host resource-input + native
  request ABI, cross-backend absolute tests, §38, conformance. **HARD-STOP +
  re-size if impl finds it must put provenance in `CanonicalReplyV1`/effect-wire,
  expose a new public projection, or add a new surface error constructor** (none
  required by the grounded seams).

## ✅ Phase 1 MERGED (2026-07-23) — §38 + conformance locked; WP STAYS ACTIVE

**Phase 1 (spec + conformance) landed at `origin/main = b64ad9f3`** (PR #913,
squash of `b90b184f`), content-verified byte-identical, publisher post-merge
currency green. Enclave reviewed on exact SHA: Architect (`evt_7xz3wzhkdtxzg`) +
CV (`evt_3f6vqn8729f1f`), Decision `dec_37ghmhf3yme1v` resolved. Retros in
(spec-author `evt_6xq84h6k7071e`, CV `evt_6f7nf00jvm2wm`, spec-leader coord).
Scope: 3 files (`38-ffi-io.md` + `seed-buffer-io.md` + `conformance/README.md`),
154+/3−, no crates. **No hard-stop fired** — locked existing `InvalidBounds`, no
new constructor.

§38 now locks: exact-acquisition binding; foreign-acquisition rejection on both
`freeze` and `writeAt` even under matched capacity/start/length/window; observable
precedence (host-width `InvalidOffset` → span-validity `InvalidBounds` → byte
exposure/backend); non-revival across close/reacquire; remainder-span
preservation; `Closed` for the matching closed acquisition. CV's absolute
two-buffer oracle rows (SP-A same-shape discriminator, SP-B precedence, SP-C
slot-reuse) are the RED oracle Phase 2 turns green.

**⛔ This WP is NOT closed.** Phase 2 (the route-1 crate implementation against
this locked text) is owed. WP stays `active`.

## ▶▶ Phase 2 ACTIVE — kicked to Runtime (2026-07-23)

**Phase 2 (route-1 crate impl) kicked to the Runtime ring** — the single impl
track for the away window (Steward pick, operator's Runtime lean; the hard part
is native lowering + host ABI + canonical-dispatcher admission). Kickoff
`evt_4kdc8ma3fk6hw` (new root, code_share → runtime-leader), delivered/Working;
Runtime ring handoff-gate-compacted to ctx 0% @ `b64ad9f3` first. Base =
`origin/main = b64ad9f3`; branch `wp/PX8-SPAN-PROV-buffer-span-provenance-p2`.
Brief = Phase-2 section of the WP file (AC-5..AC-9 + Fence + 3 hard-stops). Crate
fence: `ken-elaborator` + `ken-interp` + `ken-runtime` + `ken-host`. Review lane =
Runtime ring + Architect (soundness) + CV (conformance rows go green); merge
Decision carries Architect + CV. At P2 `git_request`: NOT --doc-only, re-grep
ledger on branch diff, **notify @adversary on the P2 crate merge**. Watch for a
hard-stop mention (provenance→`CanonicalReplyV1` / new public projection / new
surface error constructor) ⇒ re-size. Phase 1 merge-commit push CI (`b64ad9f3`)
also confirmed green (run 29991063848).

## ⚖ Phase 2 SCOPE RE-SCOPE (2026-07-23) — option (a), land mechanism + honest partial native rows

Phase-2 impl is mechanism-complete and mutation-proven (Architect + CV + QA all
confirm route-1 faithful; **no hard-stop**). Closing CV's conformance-coverage
block, the ring hit a **pre-existing native-backend wall**: a native end-to-end
write/precedence/slot-reuse discriminator needs **4 nested resource brackets**, and
any 4-bracket program hits Cranelift `Code for function is too large` in
`build_native_program` (measured repro). So the **native** SP-A-write / SP-B / SP-C
oracle matrices cannot run; CV ruled (option c) they cannot flip GREEN as locked.

**Steward scope ruling (`evt_7c160ej3bwz4`): option (a) — resize and land the sound
mechanism**, do not hold it behind the backend fix. The clause-(b) provenance
property is genuinely enforced on **both** engines (shared engine-agnostic
dispatcher; SP-A-freeze proves the full native reject path end-to-end); the native
gap is a separable, general backend limitation. Re-scoped AC-7 (CV owns wording):
SP-A-freeze GREEN both engines; SP-A-write / SP-B / SP-C = **honest partial**
(interpreter GREEN + native **BLOCKED-ON-NATIVE-REACHABILITY**, pointing to
[[RT-NATIVE-FNSPLIT]]). In-fence closure now: implementer adds the distinct-token
native ABI/lowering discriminator (closes CV's 5-of-6-field seam) + completes the
interpreter e2e half. Native SP matrices flip GREEN later on
[[RT-NATIVE-FNSPLIT]]'s landed capability (a conformance-only follow-up fold).

⚠ **This landing has a partial native oracle** — a scope precedent kin to the
operator-parked [[PX8-F-CAP-41]]/[[PX8-WROTE-ABS]] calls. Steward takes the
operator's **landing-shape confirmation** on return (~11:30 UTC 2026-07-23) before
publishing Phase 2. The fresh merge Decision carries Architect (soundness) + CV
(honest-partial-row fold).

## Sequencing (Steward) — spec-first

This is cross-cutting (enclave §38/CV + build crates). Correct order: **enclave
locks §38 (binding + foreign rejection + `InvalidBounds` identity) + CV frames the
absolute conformance rows FIRST**, then a build team implements against locked
text. Deepest of the 3 PX8 gaps but now fully specified. Sibling of
[[PX8-WROTE-ABS]] (clause-a evidence) and [[PX8-F-CAP-41]] (clause-a behavior);
root [[PX8]].
