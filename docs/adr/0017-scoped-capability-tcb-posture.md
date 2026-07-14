# ADR 0017 — Scoped-capability model: TCB posture and the confinement trust boundary (I-5)

- **Status:** **Accepted** — the component design, the attenuation lattice, the
  enforcement mechanism, and the TCB verdict were designed by the Architect and
  grounded against the landed tree; the Steward accepted the verdict
  (2026-07-14). **No operator decision is required: I-5 is realizable at zero
  kernel-TCB**, so no new kernel rule and no new trusted primitive go to the
  operator. This ADR fixes the *design and the honesty boundary* before the I-5
  build WP is framed; the build itself lands the Rust.
- **Date:** 2026-07-14 (design round I-5; Architect design pass, Steward-accepted
  same day).
- **Deciders:** the Architect (component design + TCB verdict, grounded against
  `crates/ken-kernel`, `crates/ken-elaborator/src/capabilities.rs`, and
  `crates/ken-interp/src/eval.rs`); the Steward (accepted, framing the build WP
  off this artifact). The operator is **not** a decider here — establishing that
  is one of this ADR's findings.
- **Relates to:** ADR 0009 (capability supply strategy), ADR 0004 (Security
  Tier 1 / information flow), ADR 0007 (security policy as code), the Program I
  contract `docs/program/ken-cli-program-i-contract.md` §3.2 (the scoped-model
  gate this ADR designs the *how* for), spec `60-security/62-authority.md`
  (the authority lattice and attenuation bound), and the I-4 reshape
  (Ken-callable `attenuate` deleted, semantic runner attenuation preserved —
  the machinery I-5 is built on). `docs/PRINCIPLES.md` (honesty about the
  boundary — the reason §3 of this ADR exists).

## Context

FS write/delete currently ship **functional but honestly over-privileged**:
coarse authority (`ANone`/`APartial`/`AFull`) confines *nothing to a path* —
`authorizes(cap, path)` (`eval.rs:2859`) receives the path and ignores it. Until
a scoped model lands, Ken cannot advertise least-privilege confinement. The
Program I contract §3.2 fixes *what* the capability must carry and the
driver must enforce (operation rights, directory scope, symlink policy,
narrowing-only attenuation, TOCTOU-safe `openat` enforcement); this ADR
fixes *how*, and — more importantly — **states precisely what is and is
not guaranteed**, so a green
suite is never misread as a kernel proof of confinement.

The contract asserted I-5 is "zero TCB" and located the attenuation machinery in
"the kernel `discharge_attenuation`." **The verdict below was required to be
grounded against the landed code rather than inherited from that phrasing** —
this is the exact failure I-4 cost six hours on (a spec that was internally
coherent, faithful, and *unbuildable* because nobody asked "can this be written
against the primitive it sits on?"). Grounding it in the design lane, before the
build, is cheap; discovering it mid-build is not.

## Decision

### 1. TCB verdict — zero **kernel**-TCB, and the contract's phrasing corrected

**I-5 is realizable at zero kernel-TCB.** No `ken-kernel` file changes; no new
trusted primitive; no new kernel rule. Grounded against the landed tree:

1. **The entire authority lattice and attenuation machinery lives in the
   untrusted elaborator**, not the kernel: `Authority`, `authority_meet`,
   `attenuate`, `AttenuationObligation`, and `discharge_attenuation` are all in
   `crates/ken-elaborator/src/capabilities.rs:38-189`.
2. **`crates/ken-kernel` is innocent of the lattice.** A grep over the kernel
   crate for authority / capability / attenuation returns only unrelated
   test-comment hits ("capability" in the K4/K5/K7 *proof* sense). No production
   kernel code models authority, rights, or scope.
3. **`discharge_attenuation` (`capabilities.rs:159-189`) does not encode the
   lattice into the kernel.** It `declare_postulate`s *fresh opaque constants*
   for the authority value, builds `Eq(auth_type, child, bound)`, and discharges
   with `Refl(child)`. The kernel checks exactly one thing: are `child` and
   `bound` the *same opaque constant*? The **elaborator** decides same-vs-distinct
   by its own Rust comparison (`obl.child_authority == obl.bound`, line 174).
   The kernel is a **generic reflexivity oracle over opaque postulates; it is
   lattice-agnostic.**
4. **Therefore widening `w` from a scalar `Authority` to a `(rights × scope)`
   meet changes only elaborator Rust** — the `Authority` type,
   `authority_meet`, `authority_flows_to`, and the `==`/`⊑` in
   `AttenuationObligation`. The kernel term `discharge_attenuation` emits is
   **unchanged in kind** — still `Eq(opaque, opaque)` + `Refl`, discharged iff
   the elaborator computed `child = bound`. The kernel never learns the lattice
   got richer. Program I §3.2 point 4 ("reuse `discharge_attenuation`… extend its
   `w` from a scalar to a (rights × scope) meet") is realizable verbatim with
   zero kernel touch.
5. **Runtime enforcement is trusted runtime Rust, not the kernel.** The scope
   check lives in `authorizes` + the FS driver in `ken-interp`. `authorizes`
   (`eval.rs:2859`) already receives the path argument and ignores it — the seam
   is pre-cut — and its own doc (AC8) already pins it: *"trusted Rust,
   conformance-netted, NOT kernel-backed, zero `declare_postulate`/Obligation
   emission."* The `discharge_attenuation` reference in `eval.rs` is a comment;
   attenuation discharge never runs at runtime.

**Contract-phrasing correction (recorded so the next reader does not re-derive
it):** the Program I contract calls `discharge_attenuation` "kernel machinery."
That is imprecise. The function **lives in the untrusted elaborator** and merely
*calls* pre-existing, general-purpose kernel APIs (`declare_postulate`,
`attempt_with_cert`, `Term::Eq`, `Term::Refl`) that already exist for all
elaboration. Using them with a richer elaborator-side lattice adds no kernel
rule. The distinction between "I-5 is a build WP" and "I-5 is an operator
decision" turns on this: it is the former.

### 2. The trust boundary, unhedged

I-5 **grows the trusted runtime driver.** The scope / symlink / `openat`
enforcement is new *runtime-trusted* Rust in `ken-interp`, in the **same trust
class as the existing FS driver and today's `authorizes`** (already
trusted-not-kernel-backed). That is the normal build lane, netted by conformance
discriminators — **not** a kernel or logical-TCB expansion. The load-bearing
distinction: **"trusted-runtime-driver code grows" ≠ "kernel TCB / new
primitive."** Only the latter is the operator's call, and I-5 needs none of it.

**And the part that must not be softened:** the kernel's `Refl` re-check of the
attenuation bound is **degenerate** — it mirrors the elaborator's own decision
via wired-up opaque postulates. So the real net for the attenuation bound is the
**elaborator's `meet`/`⊑` Rust plus the runtime `authorizes`**, *not* an
independent kernel proof. I-5 inherits that posture and **must not claim a
stronger kernel guarantee than exists.**

Stated plainly, what is and is not guaranteed:

- **Guaranteed:** the surface type discipline (`Cap` is opaque and unforgeable
  from Ken; the sole entry is the untrusted runner mint at the program boundary,
  bounded by the declared authority); the coarse static gate (write requires
  `AFull` monomorphically); and — *once the driver is built and its
  discriminators pass* — that FS operations are confined to the granted subtree,
  rights, and symlink policy at runtime.
- **NOT guaranteed by the kernel:** least-privilege **path confinement is a
  security property netted by trusted Rust and conformance discriminators, not
  by kernel-checked proof.** A green suite means the driver's discriminators
  passed — it does **not** mean "the kernel proves confinement." This is
  defensible (it is the same trust class as every FS driver, which the kernel
  neither does nor should model), but it is exactly the kind of boundary
  `docs/PRINCIPLES.md` demands we be honest about, and exactly the kind that
  decays into an over-claim once three WPs have cited it secondhand. This ADR
  exists so that never happens.

### 3. Two load-bearing design rulings

**(a) `scope` stays in the opaque `Cap` VALUE — it is not lifted into the
surface type index.** The surface type stays `Cap : Auth -> Type0` (coarse). The
type system already cannot destruct the opaque `Cap`, so a richer *value* is
invisible to it → **zero surface-type change, zero kernel change**, and the
static write⇒`AFull` gate is preserved. Rights × scope confinement is the
**runtime** refinement `authorizes` enforces.

- *Deferred alternative, named so it is not re-litigated mid-build:*
  **path-indexed dependent capabilities** that lift scope into the type index.
  That would buy *static* scope-checking, but it is a real surface + elaborator
  change (and a much larger one). It is a **future milestone option**, explicitly
  out of scope for I-5.

**(b) `root` is a directory HANDLE, not a path prefix.** This is the TOCTOU-safe
representation. The runner mints the handle when it mints the cap (the same mint
boundary as authority today), and every op resolves relative to it. A stored
path-prefix string, re-resolved per op, is exactly the racy and escapable
representation that point 5 of the contract forbids.

### 4. Data shape, lattice, and enforcement

**Data shape** — extend the Rust `capabilities::Cap` struct with a scope field
carried as runtime payload inside the opaque value:

```
struct Cap { authority_val: Authority,  // KEEP: coarse Auth the surface type is over
             effect: EffectName,         // KEEP
             scope: FsScope }            // NEW: least-privilege refinement
struct FsScope { rights: RightSet,       // bitset {Read,Write,Create,Delete,Enumerate,Metadata}
                 root: DirHandle,         // granted subtree as an openat-anchored handle, NOT a path string
                 symlink: SymlinkPolicy }
enum SymlinkPolicy { NoFollow,           // default, most restrictive
                     FollowWithinScope }
```

**Attenuation lattice** — the scalar lattice generalizes to the product
`RightSet × ScopeReach × SymlinkPolicy` with componentwise meet:

- **rights:** meet = set intersection (∩); narrowing drops rights.
- **scope:** meet = reachable-path-set intersection; for subtree scopes,
  `meet(A, B) = A` if `A ⊆ B`, `B` if `B ⊆ A`, else `∅`. The child's reachable
  set is always ⊆ the parent's.
- **symlink:** meet toward `NoFollow` in `NoFollow ⊑ FollowWithinScope`.

**Monotone-narrowing law** the obligation expresses (generalizing
`authority c' ⊑ authority c ⊓ w`):
`rights(c') ⊆ rights(c) ∩ rights(w) ∧ reach(c') ⊆ reach(c) ∩ reach(w) ∧
symlink(c') ⊑ symlink(c) ⊓ symlink(w)`. Canonical `attenuate` sets
`c' = c ⊓ w` exactly (⊑-refl discharges). **Never widening** — no join/amplify
path exists, exactly as the scalar lattice has no `strengthen`
(`capabilities.rs:54-56`). Discharge stays kernel-unchanged per §1.4.

**Enforcement point — what `authorizes` becomes.** Today it is
`authorizes(cap, _path, required, op) -> bool`. I-5 fills the path argument and
returns the resolved handle so that *check and use are the same fd*:
`authorizes(cap, op, target) -> Result<DirRelHandle, CapabilityDenied>`,
checking in order, all fail-closed:

1. decode `EvalVal::Cap` (non-`Cap` → deny, as today);
2. **rights:** `op.required_right() ∈ scope.rights` else `RightNotHeld`;
3. **coarse authority** (defense-in-depth: write still requires `AFull`);
4. **scope via `openat`:** resolve `target` component-by-component *relative to
   `scope.root`* — never by string concatenation/normalization. `..` that would
   ascend above the root → `ScopeEscape`. A symlink component: under `NoFollow`
   → `SymlinkDenied`; under `FollowWithinScope` → resolve and re-verify still
   beneath the root, else `ScopeEscape`. On Linux this is
   `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS)` where available, with an
   `openat(O_NOFOLLOW)` per-component fallback otherwise;
5. success returns the directory-relative fd the driver performs the op on.

**The check-and-use-share-the-fd property — the thing that actually makes it
race-safe.** The driver arm consumes the fd `authorizes` produced under scope
constraints; it **never receives a path string to re-open.** There is no
re-resolution window between check and use, so the classic TOCTOU race
(check a path, then reopen it after an adversary swaps a component) cannot
occur. Via the injectable host-handler interface (contract §4.2) the in-memory
VFS models the same beneath-root / no-escape semantics for deterministic tests.

**Fail-closed on named variants** — extend the runtime `CapabilityDenied` value
with one reason per failure class (exhaustive, no catch-all success):
`RightNotHeld { op, held_rights }`; `ScopeEscape {}` (`..`-ascent, absolute-path
target, or cross-symlink escape); `SymlinkDenied {}`; `AuthorityInsufficient`
(the existing coarse gate). A non-`Cap`/malformed op value denies **before any
syscall** (the existing fail-closed posture); an unknown op stays the loud
`_ => UnknownEffect`. **Every denial precedes the host syscall** — the existing
`host.fs_trace().is_empty()` "denial before syscall" discipline from the I-4 §B
tests carries forward.

### 5. Discriminators the build WP must prove

Each is a **non-degenerate pair** — a deny-case *and* an accept-case on the same
shape (per COORDINATION §7). A path-string implementation passes a lone
deny-case; the *pair* is what discriminates, because the wrong implementation
inverts both.

- **Symlink escape:** cap scoped to `dir1/`, with a symlink `dir1/link → /etc`
  (or `→ ../dir2`). An op via `dir1/link/x` is **DENIED** (`SymlinkDenied` /
  `ScopeEscape`) while a real `dir1/x` is **ACCEPTED**.
- **`..` traversal:** cap scoped to `dir1/sub/`. `dir1/sub/../secret` is
  **DENIED** (`ScopeEscape`) while `dir1/sub/ok` is **ACCEPTED**. (An
  unnormalized string-prefix check passes the escape — the classic real-world
  bug this pair catches.)
- **Absolute-path target** under a subtree cap → **DENIED** (`ScopeEscape`).
- **Right absent:** a read-only cap on a write op is **DENIED** (`RightNotHeld`)
  while the same cap on a read is **ACCEPTED**.
- **Attenuation monotonicity:** narrowing rights `{Read,Write} → {Read}` and
  scope `dir1/ → dir1/sub/` **discharges**; a deviant child claiming
  `{Read,Write}` from a `{Read}` parent, or scope `dir1/ → dir1/../dir2/`, is
  **undischargeable** (Unknown).
- **TOCTOU (structural, not a flaky timing test):** prove *by construction* that
  the op consumes the check's fd, not a re-resolved string; a VFS test where a
  rename of a path component between check and use does not change which inode
  the op hits (the fd is pinned).

### 6. Guardrails honored

No kernel rule and no new trusted primitive (§1 confirms none is needed). The
I-4 wrapper surface and its Option (ii) are **not** reopened — attenuation stays
runner/elaborator-side, there is no Ken-callable `attenuate`, and write stays
`AFull`-gated and monomorphic. CLI grants and OS sandboxing remain out of scope
— this ADR is about what the driver enforces on a *minted* cap, not how the
operator grants one.

## Consequences

- I-5 proceeds as a **Runtime build WP**, not an operator decision. The Steward
  frames it citing this ADR; Runtime builds from a citable artifact rather than
  a quoted design note.
- Ken's least-privilege path confinement will be, and must be described as, a
  property **netted by trusted Rust and conformance discriminators** — the same
  trust class as the FS driver — and explicitly **not** kernel-proved. Every
  future reader of `authorizes`, and every WP/QA/conformance seed that cites
  confinement, inherits that boundary from here.
- The semantic `attenuate` / `AttenuationObligation` preserved through the I-4
  reshape is the foundation this model extends: generalize the scalar `w` to the
  product meet, discharge unchanged. Had it been grep-and-nuked with the dead
  Ken-callable wrapper, I-5 would be rebuilding the authority lattice from
  scratch. The I-4 hard-stop discipline paid a direct dividend here.
