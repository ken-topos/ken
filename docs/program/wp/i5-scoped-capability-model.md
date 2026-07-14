# WP I-5 — Scoped capability model (least-privilege FS)

Make FS write/delete **least-privilege**. Today they are functional but
**honestly over-privileged**: coarse authority confines *nothing to a path* —
a write-granted program can write anywhere the process can. I-5 is the **gating
fast-follow** that closes that, and until it lands Ken must not advertise
confinement.

**Program I, I-5.** Owner: **Runtime**. Reviewer: **Architect** (terminal;
he authored the design) + **CV** (conformance seeds). Size: **XL**.
Base: `origin/main @ dff76dc7`. Branch: `wp/i5-scoped-capability-model`.

## Step 0 — widen the host-handler seam

ADR-0017 §4a supersedes the original frame's false assumption that the landed
host-handler seam could already carry a resolved handle. The seam widening and
its scoped consumer land together; an unused ABI cannot establish confinement.

1. A resolved handle carries only an owned OS descriptor or a virtual node id,
   never path bytes. Dispatch threads it verbatim and cannot derive a path from
   it. Operate methods accept only handles.
2. Resolution walks relative to the capability's root handle and returns either
   a pinned existing handle or a pinned parent handle plus one validated leaf.
   Operations consume that result without reopening the original path.
3. The byte-path `HostHandler::fs_*(&[u8])` operate surface is replaced, not
   retained alongside the handle API. A surviving path entry point is a bypass.
4. `CaptureHost` is re-keyed from paths to inode-style node identity plus
   directory entries, and `VirtualFsNode` gains symlinks. Renaming an entry does
   not change an already-resolved node handle; this makes the TOCTOU
   discriminator falsifiable rather than simulated.

Only after this step lands does the rights × scope × symlink model below use the
widened seam.

## ★ THE DESIGN IS ALREADY WRITTEN — BUILD FROM THE ADR, NOT FROM THIS FRAME

**`docs/adr/0017-scoped-capability-tcb-posture.md`** (merged, `origin/main`) is
the **normative design**: data shape, attenuation lattice, enforcement point,
fail-closed variants, and discriminators. **Read it first and build from it.**
This frame only **scopes, sequences, and sets acceptance** — where the ADR and
this frame disagree, **the ADR wins** and you tell me.

That is deliberate. The Architect grounded the mechanism against the landed
tree; I am not going to paraphrase it into a second, drifting copy.

## Fixed inputs (settled — do NOT reopen)

**Treat every anchor as perishable — verify against the landed code at pickup.
If a fixed input is FALSE, say so with exact tree anchors and ESCALATE; do not
build around it.** (This clause has caught a bad pin of mine three times in the
last day. It is not decoration.)

1. **★ TCB VERDICT: ZERO KERNEL-TCB — SETTLED, do not re-derive** (ADR-0017 §1).
   The authority lattice + attenuation machinery live in the **untrusted
   elaborator** (`capabilities.rs`), **not the kernel**. The kernel is a
   **generic reflexivity oracle over opaque postulates and is lattice-agnostic**.
   Widening the attenuation weight from scalar `Authority` to a
   `(rights × scope)` meet is **elaborator-Rust-only**. **No kernel rule, no new
   trusted primitive, no postulate.** (The Program-I contract's phrase "kernel
   `discharge_attenuation` machinery" is **imprecise** — ADR-0017 corrects it.
   Do not be misled by the contract on this point.)
   **If you find yourself needing to touch `crates/ken-kernel/` or mint a
   primitive: STOP and escalate to me.** That would falsify the verdict and it
   becomes an **operator** decision — never a build-WP workaround.

2. **★ THE HONESTY CONSTRAINT — this is load-bearing and it binds your TESTS AND
   YOUR PROSE** (ADR-0017 §2). The kernel's `Refl` re-check of the attenuation
   bound is **degenerate** — it mirrors the elaborator's own decision and is
   **NOT an independent kernel proof**. So least-privilege confinement is a
   security property **netted by trusted Rust + conformance discriminators, NOT
   by kernel-checked proof.**
   - **I-5 GROWS THE TRUSTED RUNTIME DRIVER.** That is the normal build lane
     (same trust class as the landed FS driver), **not** a kernel/logical-TCB
     expansion — but it must be **stated, never implied away**.
   - **Do NOT write a comment, doc line, error message, or test name that claims
     or implies the kernel proves confinement.** A green suite must not read as
     a kernel guarantee. If you catch existing prose over-claiming, fix it.
   - **The discriminators ARE the net.** Weak discriminators here are not a test
     gap — they are a **security** gap.

3. **`scope` lives in the opaque `Cap` VALUE — NOT lifted into the surface type
   index** (ADR-0017 §3). The surface type stays `Cap : Auth -> Type0`. This
   preserves the **static write⇒`AFull` gate** (Pat's Option (ii)) and keeps
   surface + kernel unchanged. **Path-indexed dependent capabilities are a named
   DEFERRED option — do NOT build them, do NOT re-litigate.**

4. **`root` is a directory HANDLE, not a path prefix** (ADR-0017 §3). Resolve
   `openat`-relative, component-by-component. **A stored path-prefix string
   re-resolved per op is the racy, escapable representation this WP exists to
   eliminate.** `check` and `use` **share the resolved fd** — that is what makes
   it race-safe rather than check-then-reopen.

5. **Reuse the existing attenuation machinery** — `attenuate → (Cap,
   AttenuationObligation)` + `discharge_attenuation` (`capabilities.rs`).
   Generalize its scalar `w` to the **product meet** (rights ∩, scope ∩, symlink
   toward `NoFollow`); discharge is **unchanged**. **Narrowing only — there is no
   widen/join/amplify path, and you must not add one.**
   *(This is the semantic operation I-4 protected. It is why I-5 is a
   generalization and not a rebuild.)*

6. **Out of scope, do not reopen:** the I-4 wrapper surface / Option (ii) (write
   stays monomorphic at `AFull`); Ken-callable attenuation (**stays deleted** —
   attenuation is runner/elaborator-side); CLI grants / OS sandboxing (a
   separate concern — this WP is what the driver enforces on a *minted* cap, not
   how it is granted).

## Acceptance criteria (testable)

- **AC0 — no byte-path host bypass.** No byte-path FS operate method survives on
  the `HostHandler` trait surface. Resolution is the sole path-consuming entry;
  all filesystem operations consume its owned handle or pinned-parent result.
- **AC1 — every denial precedes the syscall.** Assert `host.fs_trace()` is
  **EMPTY** on every deny path (the §B discipline). A denial that happens *after*
  the host touched the filesystem is a **failed** denial.
- **AC2 — named variants, never `is_err`.** `RightNotHeld`, `ScopeEscape`,
  `SymlinkDenied`, `AuthorityInsufficient` — assert the **exact** variant.
  Dispatch stays exhaustive; unknown op stays loud.
- **AC3 — ★ THE DISCRIMINATORS, EVERY ONE A NON-DEGENERATE PAIR** (ADR-0017 §5,
  adopted verbatim). **A deny-case alone proves nothing — a path-string
  implementation passes it. The PAIR is the net.** Each needs deny **and** accept
  on the same shape:
  - **`..` traversal** — cap scoped `dir1/sub/`: `dir1/sub/../secret` **DENIED**
    (`ScopeEscape`) **WHILE** `dir1/sub/ok` **ACCEPTED**. *(This is the one that
    catches an unnormalized string-prefix check — the classic real-world bug.)*
  - **Symlink escape** — cap scoped `dir1/`, symlink `dir1/link → /etc` (or
    `→ ../dir2`): op via `dir1/link/x` **DENIED** **WHILE** real `dir1/x`
    **ACCEPTED**.
  - **Absolute-path target** under a subtree cap → **DENIED** (`ScopeEscape`).
  - **Right absent** — read-only cap on a write op **DENIED** (`RightNotHeld`)
    **WHILE** the same cap on a read is **ACCEPTED**.
  - **Attenuation monotonicity (C1↔C2 orientation pair)** — narrowing
    {Read,Write}→{Read} and `dir1/`→`dir1/sub/` **DISCHARGES**; a deviant child
    claiming {Read,Write} from a {Read} parent, or `dir1/`→`dir1/../dir2/`, is
    **UNDISCHARGEABLE**.
  - **TOCTOU — proved STRUCTURALLY, not by a flaky timing test.** Resolve a
    target, rename its parent directory entry, then operate through the resolved
    handle; the operation must still hit the pinned inode. A naive path-string
    implementation must fail this discriminator.
- **AC4 — zero kernel delta.** `crates/ken-kernel/` untouched; `trusted_base()`
  before == after; no new primitive/postulate. (Fixed input 1. If this AC cannot
  be met, the verdict is falsified — **escalate, do not work around.**)
- **AC5 — the static write gate is UNCHANGED.** `writeFile : Cap AFull` stays
  monomorphic; an `APartial` write is still **ill-typed** (not merely denied at
  runtime). I-5 adds runtime confinement *underneath* the static gate; it does
  not replace it.
- **AC6 — conformance seeds (→ CV vote).** The discriminating pairs are pinned as
  black-box conformance cases. Because this touches `conformance/`, **CV is a
  required reviewer.** Seeds must be **honestly RED until the build lands** — do
  not hand-feed them green.
- **AC7 — honest prose.** No comment, doc, error message, or test name claims or
  implies kernel-proved confinement (fixed input 2). Grep your own diff for it.
- **AC8 — scope discipline.** `capabilities.rs` (lattice), `ken-interp` (driver /
  `authorizes` / injectable VFS handler), conformance seeds, and the docs that
  describe them. No kernel, no `Cargo`/lock, no surface-type change.

## Sequencing & review chain

Runtime builds → Runtime QA → **Architect terminal** (he authored ADR-0017;
he should press hardest on **whether enforcement is genuinely `openat`-relative
or a normalized-string check wearing a handle's clothes** — that is the
difference between this WP working and merely appearing to) → **CV** (required —
`conformance/` seeds) → `git_request` to the Steward → honesty gate + publish.
I-5 closes when it lands **and** its §10 retros are in.

**Validate TARGETED only** (`scripts/ken-cargo -p ken-interp` / `-p
ken-elaborator`), **never `--workspace`** — CI owns the locked gate. If you add a
Ken source file or touch `catalog/`/`examples/`, run **both** corpus oracles
(`crates/ken-cli/tests/ken_fmt.rs` **and**
`crates/ken-elaborator/tests/kenfmt_c_capstone.rs`) before release, and **add no
row to `FRAME_LINE_COUNTS`**.
