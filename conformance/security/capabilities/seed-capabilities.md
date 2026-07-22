# Capabilities & authority conformance — seed cases (Sec2)

Format: `../../README.md`. These pin the **authority discipline** of
`spec/60-security/62-authority.md` (Sec2, elaborated impl-ready): no ambient
authority, capabilities as static/visible/least tokens, **monotone-downward
attenuation**, transitive revocation plus its bounded OS-operation behavior,
statically-known audit points, and authority+flow composition. They sit beside
the Sec1 IFC seed
(`../ifc/seed-ifc.md`), with which it **composes** (AC6), and **supersede**
the two `authority/` placeholders in `../seed-security.md` (`attenuation-cannot-
amplify` → AC3, `no-ambient-authority` → AC1; retired there in this WP — see
"Placeholder absorption").

Grounding (landed `§`-bodies + landed code on this branch, content-reconciled —
not the plan): `62 §1`–`§9`/`§H` (no-ambient, the authority lattice, attenuation
as an emitted but direction-degenerate refinement obligation, revocation
lineage/admission/projection/settlement contract, audit, compose, trust-boundary
table); `36 §2.5`/`§3`/`§3.1` (the
capability-passing
translation: `Cap E` is a value parameter via ordinary Π, minted by a handler;
the cross-workstream contract "capability `Cap E` → a value parameter (Π) → read
by Sec2 authority/attenuation `62`"); `36 §1.4` (the EFFECT-ESCAPE check — a no-
row `view` is inert); `34 §5`/`21 §2` (refinement = carrier + emitted obligation
— the attenuation encoding); `16 §1`/`61 §2.1` (the `Ω`-valued `⊑` order the
`Authority` lattice rides); `61 §3.1` (`L-SINK`, the flow half of AC6); `61 §4`
(declassify is capability-gated/audited); `25 §3`/`63` (`trusted_base_delta`).
**Landed code pinned against:** `CapParam { name, effect }` + `cap_set`
(`crates/ken-elaborator/src/effects/algebra.rs`), `cap_params` on `EffectSig`
(`effects/infer.rs`); `DeclassifyCap { from, to }` with `is_valid` (`to ⊑ from`
∧ strict), `check_declassify`, `check_declassify_in_delta`
(`crates/ken-elaborator/src/ifc.rs`) — the **declassification special case** of
the monotone-downward, validity-checked, delta-audited capability Sec2
generalizes.

**I-4 §C RESHAPE boundary.** This seed's attenuation cases name the semantic
runner/host operation realized by `capabilities.rs::attenuate`; they do **not**
place `attenuate` in Ken's source environment. The program-facing I-4 wrappers
only consume opaque capabilities minted from the parsed program header:
`readFile` is authority-polymorphic and `writeFile` requires `Cap AFull`. Ken
source exposes neither an opaque-`Cap` constructor nor any capability-producing
attenuation function. Raw `attenuate` and `revoke` are trusted host management
actions and both remain unbound in Ken. Sections C and D therefore observe
host-managed capability values and their real authority checks; a Ken source
management call is not their fixture.

## Reading these cases — the Sec2-specific disciplines

**Capabilities are REAL Π values, but attenuation's bound is trusted-Rust +
conformance-netted (`62 §3.1`/`§H`, ADR-0017 §2).** A `Cap E` is a real Π
parameter (`36 §2.5`), so **cap presence** has a genuine independent kernel net:
a missing-capability `perform` denotes to an unbound variable and is rejected.
That does not make the attenuation bound independently kernel-proved. For
`authority c' ⊑ authority c ⊓ w`, `discharge_attenuation` emits an `Eq` + `Refl`
obligation over fresh opaque postulates, but the elaborator's Rust comparison
chooses whether `child` and `bound` name the same postulate. The kernel never
computes the meet or sees the authority lattice; its discharge mirrors the
elaborator's decision. The real net for the bound is the elaborator's `meet`/`⊑`
computation plus the non-degenerate {C1, C2} conformance pair. This gives the
bound the same trust class as declassify: trusted-by-typing and conformance-
netted, not an independent kernel proof. These cases assert **accept/reject of
the elaboration** and the emitted obligation structurally — never a synthetic
`is_weaker` predicate over literals (that guards nothing).

**The order-dual orientation — the `[Sec2-dual]` discipline (`62 §3.2`; the
`[Sec1-dual]` trap-class).** `⊑` on `Authority` is a **direction**: *more
authority is higher*, `attenuate` moves **down**, and a sink demands *at least*
its authority (`a ⊑ authority c`). Getting `⊑` **backwards** (the bound or the
sufficiency check as `⊒`) **silently inverts** attenuate-weakens into
attenuate-strengthens — privilege escalation. **The kernel obligation alone does
NOT net it:** the canonical witness `authority c' = authority c ⊓ w` discharges
**both** orientations by `⊑-refl` (the bound is **direction-degenerate at the
meet**). So the orientation is held **only** by the **non-degenerate
distinguishing pair** {C1, C2} on **strict** authorities (`authority c ⊓ w ⊏
authority c`): a weaker cap **accepts** at a weak sink **while** it **rejects**
at a sink demanding the parent's full authority. A single accept case is
green-vs-green and nets nothing — the same lesson as the taint-axis orientation
pair and the cast-direction non-degenerate-endpoints rule.

**Route REAL `Cap` values through the REAL `authority`-`⊑` check (the QA gate,
`62 §9`).** The AC3 pair must use real runner/host capability values and the
real authority order with non-degenerate authorities; never a boolean flag or a
Ken-callable constructor. A degenerate (meet-equal) instance collapses the
direction and passes green-vs-green under a flipped order.

**No amplification is an ABSENCE — assert no operation exists (`62 §3.2`).**
"Downward-only" rests on three facts: the trusted-Rust, conformance-netted
semantic bound (above),
the **enumerated absence** of any Ken-callable capability producer
(`attenuate`/`revoke`/`strengthen`/`amplify`/public-`Cap` constructor), and
**unforgeability** (`62 §2.2`: `Cap E` is opaque). The semantic host operation
still satisfies the `⊑` bound at `w = ⊤`; that does not expose it to Ken source.

**Runtime contract vs mechanism (`62 §4`/`§5`/`§H`).** ABI-REVOKE makes
lineage, admission, the two `Revoked` projections, and settlement normative for
the current implicit-root OS-operation face. Section D pins those observations
as `(oracle)` cases that are **RED UNTIL ABI-REVOKE**. It does not select the
runtime representation or claim general spaces, cross-space revocation,
transport, or distributed isolation. Audit-record emission remains separately
deferred to runtime/`Ward`; its static boundary set is already fixed.

**Tags.** `(soundness)` = a real authority commitment that must never regress
(AC1, AC3, AC6, and the contracts of AC4/AC5). `(oracle)` = confirm against
Ken's reference elaborator/runtime once the owning build lands, and
(defer-spelling-not-concept) the literal source spellings `Cap_FS`/`using` stay
`OQ-syntax`-deferred. Raw `attenuate` and `revoke` are semantic host-operation
names that must remain unbound in Ken, not deferred Ken tokens. Cases pin the
**value-set + invariants** (a typed
unforgeable token; the `⊑`-bounded semantic derivation; the audited boundary),
not a capability-producing surface spelling; deferred runtime mechanisms remain
`(oracle)` likewise.

---

## A. No ambient authority (AC1) — the structural precondition

### security/capabilities/world-action-without-capability-rejected
- spec: `62 §1`, `36 §2.5`/`§7.3`
- given: a world-action (`write_at p d`, a `visits [FS]` op) in a `view` that
  declares **no** `Cap_FS` parameter (and/or no `FS` row)
- expect: **rejects** — the `perform_FS` denotes to an unbound `Cap_FS`
  reference; a missing-capability error (kernel-ill-typed; the elaborator
  surfaces the source-located diagnostic, `36 §7.3` class 2)
- why: (soundness) AC1, no ambient authority. **Kernel-backed flip:** the *same*
  body with the `Cap_FS` parameter present **accepts** — right=accept (cap +
  row) / wrong=reject (no cap), on the real Π-binding discriminator, not a
  synthetic gate. A no-row/no-cap `view` is provably inert (`ITree 𝟘 ≅ B`,
  `36 §2.4`).

### security/capabilities/no-row-view-is-inert
- spec: `62 §1`, `36 §1.4`/`§2.4`
- given: a `view classify (x) : Tag` with **no** effect row, whose body attempts
  any effect
- expect: **rejects** — EFFECT-ESCAPE (`ρ_inf ⊄ ρ_decl = ∅`), naming the
  escaping effect + a witness perform/call
- why: (soundness) inert-by-type precondition every authority claim rests on.
  Flip: declaring the row (and holding the cap) accepts; the bare `view` cannot
  perform. (Distinct from A1: here the **row** is absent, not just the cap — the
  two halves of "no ambient".)

## B. Least by default (AC2)

### security/capabilities/uses-unpassed-capability-rejected
- spec: `62 §2`, `36 §2.5` (`cap_params`)
- given: a `view` whose body uses a `Cap_Net` it was **not** passed (no
  `Cap_Net` in `cap_params`, no enclosing handler providing it)
- expect: **rejects** — default authority is `∅`; the capability is an unbound
  reference
- why: (soundness) AC2, least by default (PoLA). Flip: passing `Cap_Net` (adding
  the `CapParam`) **accepts** the same body — right=accept / wrong=reject on
  whether the cap is in scope. A function holds **exactly** the caps it is
  passed, never ambient ones.

## C. Attenuation — monotone-downward (AC3, THE headline)

> The order-dual distinguishing pair is **{C1, C2}** on the **same** attenuated
> cap, strict authorities (`authority c ⊓ w ⊏ authority c`): verdict **flips**
> on the sink's demand. C3 pins the emitted-but-degenerate discharge mechanism;
> C4 pins the **absence**.

### security/capabilities/attenuated-cap-at-weak-sink-accepts
- spec: `62 §3`/`§3.1`
- given: the runner/host derives `c_tmp` from `c` and window `dir "/tmp"` via
  its semantic attenuation operation (so `authority c_tmp = authority c ⊓
  "/tmp" ⊏ authority c`), then uses it at a sink demanding only
  `authority c_tmp`'s scope (`a_weak ⊑ authority c_tmp`)
- expect: **accepts** — the child's reduced demand is met (`a_weak ⊑ authority
  c_tmp`)
- why: (soundness) the **accept** half of the order-dual pair. **Necessary but
  degenerate alone** — green under *both* `⊑` orientations (the meet-witness
  satisfies `a_weak ⊑ authority c_tmp` and its reverse), so this case **cannot**
  net the orientation by itself. Pairs with C2.

### security/capabilities/attenuated-cap-at-strong-sink-rejects
- spec: `62 §3.2`/`§3`/`§7`
- given: **same** `c_tmp` (`authority c_tmp ⊏ authority c`, strict) used at a
  sink demanding the **parent's full** authority `authority c` (e.g.
  `write_at c_tmp (path "/etc/passwd")`)
- expect: **rejects** — sufficiency needs `authority c ⊑ authority c_tmp`, false
  by strictness; the weakened cap is **insufficient**
- why: (soundness) **THE net.** This is where the order-dual orientation lives:
  under a **backwards** `⊑` (sufficiency `authority c_tmp ⊑ authority c`) this
  would **wrongly accept** — a weakened cap passing a strong sink (privilege
  escalation). The pair C1↔C2 flips green↔red on exactly the orientation bug; a
  single accept case (C1) is green-vs-green. **Non-degenerate** authorities are
  required (a meet-equal `c_tmp` collapses it). Real `Cap` through the real
  `authority`-`⊑` check (QA gate), never a synthetic flag.

### security/capabilities/attenuate-bound-discharge-mirrors-elaborator
- spec: `62 §3.1`/`§H`, `34 §5`, `21 §2`, `23 §1`, `18 §4`
- given: the runner/host semantic `capabilities.rs::attenuate(c, w)` derives a
  child and its **emitted obligation**; observe the discharge core for
  `authority c' ⊑ authority c ⊓ w`. No Ken source expression constructs `c'`.
- expect: the elaborator **emits an obligation** for
  `authority c' ⊑ authority c ⊓ w` (`22 §2.1`) as `Eq(child, bound)` over fresh
  opaque postulates. For the canonical child, the elaborator chooses the same
  postulate for both sides and `Refl` discharges. For a too-strong child it
  chooses distinct postulates, so `Refl` yields `Unknown` and the obligation is
  **undischargeable**. The kernel never computes the meet or `⊑` relation.
- why: (soundness) **trust-boundary assertion**, not an independent kernel net.
  The emitted `Eq` + `Refl` mechanism is real, but its postulate identities are
  chosen by the elaborator's Rust comparison. Therefore the discharge mirrors
  the elaborator's decision and is direction-degenerate; the elaborator's
  `meet`/`⊑` computation plus the non-degenerate {C1, C2} pair nets the bound.

### security/capabilities/no-amplifying-operation-exists
- spec: `62 §3.2`/`§2.2`
- given: a holder of `c : Cap E` seeking a `c'' : Cap E` with `authority c'' ⊐
  authority c` — enumerate Ken's source environment for `attenuate`/`revoke`/
  `strengthen`/`amplify`/a public `Cap` constructor, while separately applying
  the runner/host semantic attenuation at `w = ⊤`
- expect: **no Ken-callable capability producer exists** and `Cap E` exposes no
  constructor (`62 §2.2`). The separate semantic host derivation still yields
  `authority c' ⊑ authority c` and cannot exceed the parent even at `w = ⊤`.
- why: (soundness) **assert absence** — "downward-only" is the conjunction of
  the trusted-Rust + conformance-netted semantic bound ({C1, C2, C3}), this
  enumerated Ken-surface absence, and unforgeability (`62 §2.2`). The kernel
  cannot witness a missing source
  operation, so the **absence** is the guard. (Absorbs `../seed-security.md`'s
  `attenuation-cannot-amplify` — a child cannot exceed `w`.)

## D. Revocation — lineage and bounded OS-operation behavior (AC4)

Every case in this section is `(oracle)` and **RED UNTIL ABI-REVOKE**. Fixtures
use a trusted host-management harness for raw attenuation/revocation and expose
only later ordinary capability-consuming operations to Ken. They observe the
logical admission boundary without requiring a validity-cell, forwarder,
generation, or other representation. They make no general-space, cross-space,
transport, or distributed claim.

### security/capabilities/revocation-lineage-is-selective-and-transitive
- spec: `62 §4`/`§4.3`/`§H`
- given: the host mints parent `p`; attenuates siblings `a` and `b` from `p`;
  attenuates grandchild `g` from `a`; copies `a`; and acquires then duplicates a
  resource under `g`. All identities are opaque to Ken. Revoke `a`, then issue
  otherwise-valid operations through every capability/resource. In a second
  arm, start from the same live tree and revoke `p`.
- expect: after revoking `a`, `a`, its copy, `g`, and both resource tokens are
  denied as revoked on their next operation, while `p` and sibling `b` remain
  live and their controls reach the backend. After revoking `p`, both sibling
  subtrees are revoked. Copying never forks revocation identity, and consuming
  a resource token never bypasses its sponsoring lineage.
- why: (soundness) the descendant, parent, sibling, copy, and resource controls
  distinguish exact subtree closure from parent-only, leaf-only, global, or
  capability-table-only invalidation. No identity inspection is a Ken fixture.

### security/capabilities/revoked-path-operation-is-distinct-fileerror
- spec: `62 §4.1`/`§4.2`, `38 §1.3.1`
- given: use the same `readFile APartial cap path` request with a readable file,
  a well-formed/live/sufficient grant, and an empty capture-host trace. The live
  arm runs unchanged. In the negative arm the host revokes the grant after the
  request is formed but before admission. Separate neighbouring controls keep
  the grant live and trigger the existing insufficient/malformed denial or a
  real host I/O failure.
- expect: the live arm returns the bytes and records one backend read. The
  revoked arm returns exactly
  `Err (MkFileError OpReadFile (Some path) Revoked)` and records no backend
  operation. `Revoked` is an `IOError` cause beside `CapabilityDenied`: it is
  not `CapabilityDenied`, a malformed/stale capability identity, or any host
  I/O error. Each neighbouring control retains its existing non-revocation
  identity and never reports `Revoked`.
- why: (soundness) this is the path/capability projection discriminator. A
  generic denial or empty trace alone cannot pass: the exact constructor and
  the live/neighbor controls make every ruled collapse observable.

### security/capabilities/revoked-resource-operation-is-distinct-resourceerror
- spec: `62 §4.1`/`§4.2`, `38 §1.3.1`/`§1.7.2`
- given: acquire a live, sufficiently-righted resource under a live grant and
  issue one ordinary resource operation. Repeat after the host revokes that
  grant before admission. Controlled neighbours instead use a released token,
  a never-minted token, insufficient rights, a live wrong-kind token, or a
  backend forced to return a known host I/O error.
- expect: the live control reaches the backend. The revoked arm returns exactly
  nullary `ResourceError.Revoked` and has no backend operation. The five
  neighbours respectively remain `Closed`, `MalformedResource`,
  `RightNotHeld`, `ResourceKindMismatch`, and `ResourceHostIO <known-error>`;
  stale generation remains the lifetime identity `Closed`. None becomes
  `Revoked`, and revocation is never wrapped as `ResourceHostIO Revoked` or
  `ResourceHostIO CapabilityDenied`.
- why: (soundness) this binds the second type-local projection without inventing
  a shared sum. Holding the operation well formed, live, and sufficiently
  righted in the revoked arm avoids unspecified precedence among invalid inputs.

### security/capabilities/revoke-admission-race-preserves-real-settlement
- spec: `62 §4.2`/`§4.3`, ADR 0021 settlement discipline
- given: a controlled host scheduler runs two arms of the same resource write.
  The revoke-wins arm pauses before admission, revokes the sponsor, then resumes.
  The admission-wins arm pauses immediately after live-ancestry admission and
  before the backend write, revokes the sponsor, then lets the backend return a
  known result and commit a known byte change. Attempt one new operation through
  the same lineage while the admitted operation drains. Configure settlement
  once for close success and once for a known `ReleaseFailed` outcome.
- expect: revoke-wins returns `ResourceError.Revoked` with no backend trace.
  Admission-wins returns the backend's real result, records exactly one backend
  write/known side effect, and is never rewritten to `Revoked`; the new operation
  is denied as `ResourceError.Revoked` without another backend call. The owned
  resource is not closed while admitted work remains, then records exactly one
  close success or one configured `ReleaseFailed`. Settlement failure never
  reopens authority.
- why: (soundness) the two controlled schedules flip at admission, the normative
  linearization point. The result, side-effect trace, new-admission trace, and
  exactly-once terminal settlement jointly reject rollback, cancellation,
  premature close, result rewriting, and post-failure reopening.

## E. Audit points statically known (AC5)

### security/capabilities/unaudited-boundary-effect-is-impossible
- spec: `62 §5`, `36 §1.4`/`§3.1`
- given: a trust-boundary effect (`space` / FFI / declassify / delegation)
  attempted **without** a declared audit point (no row declaring it)
- expect: **rejects** — an un-audited boundary effect is impossible: you cannot
  perform an effect the row did not declare (`36 §1.4`), and the boundary set =
  the `Vis` nodes the type declares (`36 §3.1`)
- why: (soundness, static face) AC5 — the audit points are **statically known**
  because they are the type's `Vis` nodes. The runtime record **emission**
  (serialization / tamper-evidence) is the deferred face (`(oracle)`,
  runtime/`Ward`). Flip: a boundary effect *with* its declared row/point is
  performable-and-audited; without it, impossible.

### security/capabilities/declassify-every-use-audited-and-in-delta
- spec: `62 §5`, `61 §4`, `25 §3`, `63`; landed `check_declassify_in_delta`
- given: a package performing an authorised `declassify` (a capability whose
  every use is an audit point)
- expect: the declassification authority appears in `trusted_base_delta`
  (`check_declassify_in_delta` true); **rejects as an honesty violation** if the
  package downgrades but **omits** the authority from the delta
- why: (soundness) declassify is the capability whose every use is audited
  (`62 §5`); the delta-completeness backstop is the sole net for a hidden
  downgrade (the same guard as Sec1 B3, here owned by `62`). Flip on
  presence/absence in the delta. (Reuses the landed Sec1 `ifc.rs` check — Sec2
  states it as the audit tie, not a new mechanism.)

## F. Authority + flow compose (AC6)

### security/capabilities/net-write-needs-capability-and-clearance
- spec: `62 §6`, `61 §3.1` (`L-SINK`)
- given: `send c s msg` to `s : Socket Public`, exercised three ways — (i) `c :
  Cap_Net` present **and** `msg : Bytes @ Public`; (ii) `c` present **but** `msg
  : Bytes @ Secret`; (iii) `msg @ Public` **but** no `Cap_Net`
- expect: (i) **accepts** (both concessions); (ii) **rejects** — `Secret ⊔ pc ⋢
  Public`, IFC-FLOW error **despite** holding `Cap_Net`; (iii) **rejects** — a
  missing-capability error **despite** a clean flow
- why: (soundness) AC6 — authority and flow are **independent** concessions and
  **both** are required; dropping **either** rejects. Authority does not buy
  clearance, clearance does not buy authority. **Two flips** on one sink
  shape: the cap discriminator (kernel-backed, `62 §1`) and the flow
  discriminator (erased-label/trusted, `61 §H`) — composes Sec1 + Sec2, neither
  green-vs-green.

---

## G. I-5 least-privilege FS confinement — RED-UNTIL-I-5

These cases transcribe ADR-0017 §3–§5/§4a and the I-5 frame. **Every case in
this section is RED-UNTIL-I-5**: its reachability precondition is the integrated
I-5 build — the product scope carried by a real runner-minted opaque `Cap`, the
handle-only `HostHandler` seam, and the inode-keyed capture VFS. No case may be
made green by calling a helper with a fabricated verdict, by bypassing
`fs_dispatch`, or by handing an already-resolved success to the operation.

The harness drives the real capability gate and resolver from an FS operation.
On a denial it matches the exact `CapabilityDenied` variant and asserts
`host.fs_trace().is_empty()`: the denied operation never reaches a host FS
effect. On the paired acceptance it asserts both the returned value/state and
the exact handle-based host operation. Resolver observations may be recorded
separately, but never count as the denied operation having run.

For the real host, the positive mechanism is an `openat2` walk with
`RESOLVE_BENEATH` and the selected symlink policy, or a component-wise `openat`
fallback with `O_NOFOLLOW`; the operation consumes the resulting fd. A
normalized-string check wrapped in a handle-shaped type does not satisfy these
oracles.

This is a trusted-runtime-driver property, conformance-netted and **not
kernel-proved**. The pairs below are the security net. A case that cannot fail
against a naive path-string implementation is not an acceptable substitute.

### security/capabilities/i5-handle-only-seam-replaces-byte-path-bypass
**RED-UNTIL-I-5**

- spec: ADR-0017 §4a(i)–(iii); I-5 AC0
- given: inspect the landed `HostHandler` trait and the route reachable from
  `fs_dispatch`
- expect: the positive half finds an owned resolved-handle type,
  `fs_resolve`, and handle-only `fs_read_at`/`fs_write_at`/creation operations.
  The negative half finds **none** of the old byte-path operate entries
  `fs_read`/`fs_write`/`fs_append`/`fs_metadata`/`fs_read_directory`/
  `fs_create_directory`/`fs_remove_file`/`fs_remove_directory`/`fs_rename`
  taking unresolved `&[u8]`. `fs_dispatch` threads the minted handle verbatim;
  no operate call also receives a path.
- why: this is a two-sided structural discriminator: the replacement API must
  be present **and** the bypass API absent. Keeping both makes every runtime
  pair below avoidable by a wrong op-id or future call, so a green value test
  would not establish confinement.

### security/capabilities/i5-dotdot-beneath-root-pair
**RED-UNTIL-I-5**

- spec: ADR-0017 §4.4/§5; I-5 AC1–AC3
- given: one real `AFull` FS cap rooted at `dir1/sub`, with `secret` reachable
  only from the parent and `ok` a file beneath the root; issue the same read
  shape with public targets `dir1/sub/../secret` and `dir1/sub/ok` (root-relative
  component lists `../secret` and `ok`)
- expect: `../secret` **denies** as exactly `ScopeEscape` and leaves
  `host.fs_trace()` empty; `ok` **accepts** and reads the beneath-root node via
  its resolved handle
- why: deny+accept on one cap and operation catches both permissive
  unnormalized-prefix checking (which wrongly accepts `../secret`) and blanket
  traversal rejection (which wrongly rejects `ok`). The target is resolved
  component-by-component relative to the cap root, never normalized and
  reopened as a string.

### security/capabilities/i5-symlink-policy-pairs
**RED-UNTIL-I-5**

- spec: ADR-0017 §3/§4.4/§4a(iv)/§5; I-5 AC1–AC3
- given: an inode-keyed VFS under root `dir1` containing real file `x`, an
  in-scope symlink `inside -> x`, and escape symlink `link -> /etc`; exercise
  reads through the real resolver with each settled policy
- expect: under `NoFollow`, `link/x` **denies** as exactly `SymlinkDenied` with
  an empty operation trace while direct `x` **accepts**. Under
  `FollowWithinScope`, `link/x` **denies** as exactly `ScopeEscape` with an
  empty operation trace while `inside` **accepts** and reads the same node as
  direct `x`.
- why: both policy orientations have a same-root accept arm. A resolver that
  rejects every symlink fails `inside`; one that follows strings without
  re-verifying beneath-root fails `link/x`; a VFS without a real `Symlink`
  node cannot reach either distinction.

### security/capabilities/i5-absolute-target-pair
**RED-UNTIL-I-5**

- spec: ADR-0017 §4.4/§5; I-5 AC1–AC3
- given: one subtree cap rooted at `dir1` and the same existing leaf `x`, read
  once as absolute target `/dir1/x` and once as relative target `x`
- expect: `/dir1/x` **denies** as exactly `ScopeEscape` with an empty operation
  trace; `x` **accepts** through the cap's root handle
- why: the verdict flips only on absolute-vs-relative addressing. Ignoring the
  root for an absolute string wrongly accepts the first arm; rejecting all
  targets fails the second.

### security/capabilities/i5-right-not-held-pair
**RED-UNTIL-I-5**

- spec: ADR-0017 §3/§4.2/§5; I-5 AC1–AC3/AC5
- given: one real runner-minted `Cap AFull` whose product scope holds only
  `{Read}`, rooted at `dir1`, and one existing file `x`; issue a write and a
  read through the same cap and target
- expect: write **denies** as exactly `RightNotHeld` before resolution or host
  effect and leaves `host.fs_trace()` empty; read **accepts** and records the
  handle-only read of `x`
- why: the coarse `AFull` index is deliberately held constant, so the pair
  isolates the runtime right set rather than the unchanged static write gate.
  An implementation that checks only coarse authority wrongly accepts the
  write; one that rejects the cap wholesale fails the read.

### security/capabilities/i5-product-attenuation-orientation-pair
**RED-UNTIL-I-5**

- spec: ADR-0017 §1.3–§1.4/§4/§5; `62 §3.1`/§H; I-5 AC3
- given: real runner/host caps and the real product-meet attenuation path.
  First derive `{Read}` at `dir1/sub` from a `{Read,Write}` cap at `dir1` and
  discharge its emitted obligation. Then construct the two non-degenerate
  deviant obligations independently: `{Read,Write}` from a `{Read}` parent,
  and scope `dir1/../dir2` from parent scope `dir1`.
- expect: the canonical rights+scope narrowing **discharges**. Each single-axis
  widening is **undischargeable** (`Unknown`), with the other axis held fixed.
  No Ken-callable attenuation or opaque-`Cap` construction is introduced.
- why: the accept/reject orientations isolate both components of the product
  order and catch a backwards `⊑` or a join disguised as meet. As in C3, the
  emitted `Eq` + `Refl` mirrors the elaborator's Rust comparison; the kernel
  does not compute rights, reachability, or their meet. The product
  implementation plus this non-degenerate pair is the net.

### security/capabilities/i5-inode-pinned-structural-toctou-pair
**RED-UNTIL-I-5**

- spec: ADR-0017 §3(b)/§4a(i)–(ii)/§4a(iv)/§5; I-5 AC3-TOCTOU
- given: the capture VFS stores nodes by identity. Initially
  `(dir1, sub) -> node 3` and `(node 3, file) -> node 7`; replacement directory
  node 4 contains `(node 4, file) -> node 8`. Drive one write through the real
  dispatcher. Its deterministic host hook runs **after** `fs_resolve` returns
  `Existing(handle 7)` but before operate: it renames entry `(dir1, sub)` to
  `(dir1, other)` and installs `(dir1, sub) -> node 4`. Run the same write once
  without the hook as the control orientation.
- expect: both orientations accept and `fs_write_at` consumes the resolved
  handle for **node 7**. With the intervening rename, node 7 changes and node 8
  remains byte-identical; the trace contains one resolve followed by
  `WriteAt(node 7)` and **no second resolve**. Without the rename, the same
  handle and node are used.
- why: the renamed arm fails a naive path-string check-then-reopen: re-resolving
  `dir1/sub/file` hits replacement node 8, not pinned node 7. The control arm
  prevents a fixture that merely makes all writes fail. This is structural,
  deterministic, and only expressible because directory entries and node
  identity are separate; a path-keyed VFS or a timing race is not an equivalent
  oracle.

### I-5 reachability and anti-tautology sweep

- Each value case enters through `fs_dispatch` with a real runner-minted opaque
  cap; no test invokes `authorizes`, `fs_resolve`, or `fs_*_at` as its verdict
  producer.
- Every deny arm matches its exact named variant and an empty operation trace;
  every accept arm asserts the concrete returned bytes or inode state and the
  matching handle operation.
- The `..`, absolute, right, and symlink cases each share root/cap/target shape
  across their flip. The product-meet case changes one lattice axis at a time.
- The TOCTOU hook mutates only directory entries after a real resolution. It
  neither supplies the resolved handle nor chooses the write target, and node 8
  is asserted unchanged, so the test cannot pass by observing a hand-fed node.
- AC0's absence sweep is mandatory: a green secure route does not compensate
  for a surviving byte-path bypass.
- These cases assert trusted-runtime behavior only. No case name, expectation,
  comment, or diagnostic calls confinement kernel-backed or kernel-proved.

---

## Coverage map (AC → cases)

- **AC1** no ambient → A1 (`world-action-without-capability-rejected`),
  A2 (`no-row-view-is-inert`).
- **AC2** least by default → B1 (`uses-unpassed-capability-rejected`).
- **AC3** attenuation monotone-downward (THE headline) → **C1↔C2** (the order-
  dual non-degenerate pair), C3
  (`attenuate-bound-discharge-mirrors-elaborator`, emitted-but-degenerate
  mechanism), C4 (`no-amplifying-operation-exists`, the absence).
- **AC4** revocation lineage + bounded OS-operation behavior → D1 (selective
  transitive closure and resource provenance), D2/D3 (the two exact local
  `Revoked` projections and forbidden collapses), D4 (admission and settlement).
- **AC5** audit points static → E1 (`unaudited-boundary-effect-is-impossible`),
  E2 (`declassify-every-use-audited-and-in-delta`).
- **AC6** authority + flow compose → F1 (`net-write-needs-capability-and-
  clearance`).
- **I-5 AC0** handle-only seam / no byte-path bypass → G0
  (`i5-handle-only-seam-replaces-byte-path-bypass`).
- **I-5 AC1–AC3** named pre-operation denials + non-degenerate accept arms →
  G1 (`i5-dotdot-beneath-root-pair`), G2 (`i5-symlink-policy-pairs`), G3
  (`i5-absolute-target-pair`), G4 (`i5-right-not-held-pair`).
- **I-5 AC3 product meet** → G5
  (`i5-product-attenuation-orientation-pair`).
- **I-5 AC3-TOCTOU** → G6
  (`i5-inode-pinned-structural-toctou-pair`).
- **I-5 AC5** unchanged monomorphic write gate → G4 holds `Cap AFull`
  constant and discriminates only the runtime right set.
- **I-5 AC7** honest boundary → the section method + reachability sweep: trusted
  Rust + conformance net, never kernel-proved confinement.

## Cross-case consistency sweep (pre-handoff gate)

- **No-ambient / least class {A1, A2, B1}** — agree: every world-action needs an
  explicit `Cap E` **and** a declared row; default authority is `∅`; a no-cap or
  no-row `view` is inert. Each a clean verdict flip (cap/row present → accept;
  absent → reject), kernel-backed (real Π binding), none green-vs-green.
- **Order-dual orientation {C1, C2}** — the load-bearing pair: the **same**
  attenuated cap **accepts** at a weak sink **while** it **rejects** at a sink
  demanding the parent's full authority, on **non-degenerate** authorities
  (`authority c ⊓ w ⊏ authority c`). A flipped `⊑` inverts **both** verdicts, so
  the pair (not a single case) holds the orientation. The `[Sec2-dual]` net,
  the same shape as `[Sec1-dual]` integrity/`@ct` (`../ct/`) and the cast-
  direction non-degenerate-endpoints rule.
- **Two faces of the bound {C2, C3}** — agree without merging: C3 pins that an
  `Eq` + `Refl` obligation is emitted and that a too-strong child becomes
  undischargeable, while also pinning that the elaborator chooses the postulate
  identities. Therefore the discharge is **not an independent kernel proof** of
  `authority c' ⊑ authority c ⊓ w`; it mirrors trusted Rust. C2 supplies the
  non-degenerate conformance net for the orientation that C3 cannot distinguish
  at the meet. Treating C3 as a stronger kernel net, or C2 as redundant, is the
  bug this pair forecloses (`62 §3.2`/`§H`, ADR-0017 §2).
- **Semantic operation vs Ken surface {C1–C4, D1–D4}** — attenuation and
  revocation use the trusted host harness and real opaque values. C4 and the I-4
  surface case independently require both raw management names and every `Cap`
  producer to remain absent from Ken. A source wrapper fails this class.
- **Bounded runtime contract {D1–D4}** — all cases agree on copied identity,
  selective descendant closure, resource provenance, two distinct public
  projections, admission, and settlement. They are RED until ABI-REVOKE, but the
  behavior is normative now. They neither choose the runtime mechanism nor
  extend the claim to general/cross-space/distributed realization.
- **Audit split {E1, E2}** — the static boundary set and delta-completeness are
  fixed; runtime audit-record emission/tamper-evidence remains `(oracle)` and
  deferred to runtime/`Ward`, named rather than omitted.
- **Compose class {F1}** — both concessions independent and required; the cap
  discriminator (kernel-backed) and the flow discriminator (trusted-by-typing)
  each flip on the same sink shape — composes Sec1 + Sec2.
- **I-5 path class {G1, G2, G3}** — all use root-relative resolution and pair a
  named denial with a real beneath-root acceptance. Together they distinguish
  lexical `..`, absolute addressing, `NoFollow`, and follow-within-scope; no
  blanket allow or blanket deny satisfies the class.
- **I-5 authority class {G4, G5}** — G4 holds coarse `AFull` constant while the
  runtime right flips; G5 holds one product axis fixed while the other narrows
  or widens. The static write gate, runtime rights, and attenuation order are
  therefore observed separately rather than collapsed into one green result.
- **I-5 seam/identity class {G0, G6}** — G0 removes every unresolved-byte-path
  operation; G6 proves why by changing the directory entry after resolution
  and requiring the operation to retain node 7. A handle-shaped path string or
  a path-keyed VFS fails G6, while a second legacy route fails G0.

## Placeholder absorption (reconcile note)

This seed supersedes the two `authority/` cases in `../seed-security.md`:
`attenuation-cannot-amplify` → **C4** (+ the C1↔C2 pair sharpening "cannot
amplify" into the order-dual orientation net); `no-ambient-authority` → **A1**
(+ A2 splitting the cap and row halves). Both retired from `seed-security.md`
in this WP to avoid a stale-sibling contradiction; the `supply-chain/` and
`trust/` placeholders there are Sec3+ and untouched.

## Build-sequencing note

AC1–AC3 + AC5-static + AC6 exercise the landed capability-passing,
authority-check, and refinement-obligation machinery (`36 §2.5`, `34 §5`/
`21 §2`). Section C reaches `capabilities.rs::attenuate` and its discharge
directly at the semantic host boundary; it deliberately has no Ken-callable
attenuation fixture. Section D is the **RED UNTIL ABI-REVOKE** runtime contract;
Runtime may choose its mechanism, but must satisfy D1–D4 before the bounded
OS-operation face is conformant. General space realization remains open. Audit
record emission (E1) remains deferred to runtime/`Ward`. The authority lattice
`⊑` rides `Ω`
(`16 §1`, level 0 for finite carriers); the semantic refinement `{c' | …}`
lands at `level(Cap E) = ℓ_op` (predicative, same as the carrier, `21 §2`/
`62 §9`). Literal source spelling `Cap_FS` stays `OQ-syntax`-deferred; raw
`attenuate` and `revoke` remain non-Ken-visible management actions. Cases pin
value-sets + invariants, never a Ken-callable management token. Sec2 **unblocks
B4** (the agentic boundary = Sec1 + Sec2
envelope) and contributes to **G-Sec**. Section G is a land-together companion
to Runtime I-5: it remains RED until that integrated build reaches the cases and
must not be published through a standalone conformance route.
