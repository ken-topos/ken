# PX8-N — Runtime checked-host bounded-`Nat` reply-lowering compiler prereq

> **Architect ruling `evt_22y0emyj6cwf2` (2026-07-17).** PX8-F hard-stopped:
> the checked buffer surface needs `BufferSpan` to carry a **constructor-private
> structural `Nat`** budget that transparent `writeAll` eliminates, but the host
> reply is **scalar-only** and the native lanes are **represented-unavailable** —
> there is no dynamic-scalar→structural-`Nat` lowering. PX8-N adds the **minimum
> sound mechanism**: a compact semantic **`BoundedNat` lowering carrier** minted
> **only** while reifying a validated host reply, whose zero/successor elimination
> and recursion are observationally the source `Nat`. This is a **separate Runtime
> compiler WP** from the PX8-X observation/schema train — the Architect ruled the
> two are kept split (independent failure modes) and **may run in parallel**, each
> with its own QA + §14. Ward remains external throughout.

- **ID:** PX8-N · **Owner:** **Team Runtime** (leader `agt_37reqrd72cg00` /
  implementer `agt_37reqg3nync00` / qa `agt_37reqvb6ce400`) · **Size:** M ·
  **Risk:** High (a **checked-host result** mechanism that mints a semantic value
  the kernel eliminates/recurses over — soundness-critical: the carrier's
  `Zero | Suc` elimination and recursion **must** be observationally a structural
  `Nat`, minted only inside the private checked result path, never caller fuel).
- **Branch:** `wp/px8n-bounded-nat-reply-lowering` off **`origin/main @ 78ef39eb`**
  (the handoff-gate `git reset --hard origin/main` gives the ring current code).
- **Route:** **Architect §14** (the lowering-carrier soundness — the minting
  invariant, the `Zero | Suc` elimination/recursion equalling structural `Nat`,
  no scalar public budget) **+ Runtime QA**. No `spec/`/`conformance/` change
  expected → CV only if the candidate touches those paths. Own Decision, own SHA
  — **independent of the PX8-X train.**
- **Parallel with PX8-X** (both Runtime): PX8-N is the compiler carrier; PX8-X is
  the observation/wire schema unification. They have independent failure modes;
  coordinate on their two branches but do not bundle. **PX8-F is held until BOTH
  land.**

## Objective

Add a **compact semantic `BoundedNat` lowering carrier**, minted **only** while
reifying a validated host reply at the checked-result boundary, whose ordinary
`Zero | Suc predecessor` elimination and computational recursion over a strictly
decreasing predecessor are **observationally the source `Nat`** — so the checked
buffer surface's constructor-private `BufferSpan` structural-`Nat` budget (and the
`TransferCount` projections) can be constructed from the host's scalar reply, and
transparent `writeAll` can eliminate it in real native execution. Admit **and
completely lower** the four settled host ops (`BufferAllocate`, `BufferFreeze`,
`FsReadAt`, `FsWriteAt`) so their checked results are constructible on the native
lane, not merely represented-available. **This is a general checked-host result
mechanism, not a `writeAll` intrinsic or a resource-specific rewrite.**

## Fixed inputs — DO NOT REOPEN (settled; do not re-ask the operator/Architect)

- **Architect ruling `evt_22y0emyj6cwf2` — the binding mechanism.** Transcribe,
  do not redesign:
  - **Mint only at the validated response boundary.** Validate the operation/tag
    **and the full progress invariant BEFORE minting**: ReadSome/Wrote count is
    **positive**; count is **within the effective request**; ReadSome span
    **length equals count**; `start + length` is checked and **within the admitted
    buffer range**; **ReadEof is the only zero read**; a **zero write remains
    `NoProgress`** (never a minted count). A mint that skips any clause is
    non-conforming.
  - **One mint, no divergence.** Reify the private `BufferSpan` budget as
    `BoundedNat(count)` and reify the corresponding private `TransferCount`
    projections **from the same validated scalar truth** — a single mint, so span
    length, transfer count, and attempt budget **cannot diverge**.
  - **The carrier is a real `Nat`.** It implements ordinary `Zero | Suc
    predecessor` elimination and **computational recursion over a strictly
    decreasing predecessor**, retaining the exact recursive **IH and binder
    environment**. Not a `writeAll` intrinsic; a general result mechanism.
  - **Compact internally, structural externally.** The representation **may remain
    one machine scalar internally**, but **every source-level observation must
    equal the structural `Nat` semantics**. It must **not** expose the scalar as
    **caller-selected fuel**, **not** add an `Int` budget field, and **not** permit
    construction **outside** the private checked result path. No eager unary chain
    allocation.
  - **Admit + fully lower the four ops.** `BufferAllocate`, `BufferFreeze`,
    `FsReadAt`, `FsWriteAt` — including their **exact request fields, success
    constructors, and all existing structured resource/I/O errors**. Flipping
    availability without **constructing the checked result** is non-conforming
    (the exact PX8-F failure mode: "admitting the op IDs would move the failure,
    not solve it").
  - **No ABI expansion.** The existing tag-dependent `HostReplyV1` scalar layout
    is sufficient; **host-effect ABI bytes stay stable**. Do **not** expand the
    ABI for this prerequisite.
  - **No compatibility resurrection.** This prerequisite does **not** revive any
    V1 observation/export byte-preservation promise — that is superseded by the
    operator's single-schema ruling (`evt_291b8gcwde32v`, PRINCIPLES transient T).
    PX8-N touches the **host-reply→checked-result lowering**, not the
    observation/wire schema (that is PX8-X).
- **Landed anchors on `f2f60083`/`78ef39eb` (verify before editing; do not
  fabricate; `git show origin/main:<path>`):**
  - **Scalar host truth:** `crates/ken-host/src/abi_v1.rs:963-978` —
    `HostReplyV1.detail : u64` transports ReadSome as `detail = transferred` +
    tag-specific `bytes.len = start`, and Wrote as `detail = transferred`.
    `crates/ken-host/src/effect_v1.rs:2050-2076` stores only `u64 {start, length,
    count}`. `BufferSpanV1 { start, length }` + `TransferCountV1(u64)` are scalar
    host values.
  - **The represented-unavailable native gap:** the checked-native consumer list
    rejects the four ops; the first red is `cranelift_backend.rs:3323-3330`
    (`FSOp.1026` / `FsReadAt` "represented unavailable lane"). The minimized
    reaching fixture is Foundation's `crates/ken-cli/tests/px8f_buffer_native.rs`
    (preserved at `preserved/px8f-held-60a481b5`; see PX8-F).
  - **Host op ids (landed):** `FsReadAt=0x030D`, `FsWriteAt=0x030E`,
    `BufferAllocate=0x0402`, `BufferFreeze=0x0403` (`effect_v1.rs:34-38`); real
    `pread`/`pwrite` dispatch with progress; **write-zero→`NoProgress`**.
  - **Interpreter reference behavior:** the interpreter already **mints the `Nat`
    by Rust recursion** (Foundation reported catalog + interpreter green). PX8-N's
    native carrier must **agree with** that interpreter reference on the same
    reply — the native lane is the gap, not the semantics.
- **R2 closed / no in-language affine machinery** — irrelevant here; the carrier
  is a **compiler lowering** value, not a language-surface type. Do **not** add a
  public `Int -> Nat` primitive or a surface constructor.

## Mandated deliverables (each ends in a concrete implementable choice)

- **N-D1 — the `BoundedNat` carrier.** Add the compact semantic carrier (one
  internal machine scalar) with ordinary `Zero | Suc predecessor` elimination and
  computational recursion over a strictly-decreasing predecessor, retaining the
  exact recursive IH + binder environment. State the representation (e.g. a
  `u64` bound + eliminator that observes `Zero` at `0` and `Suc pred` at `n>0`
  with `pred = n-1`) and prove the source-level observations equal structural
  `Nat`. **No eager unary allocation; no public constructor; no `Int` field; no
  caller-selected fuel.** Construction is confined to N-D2's private path.
- **N-D2 — the validated response-boundary mint.** At the host-reply reification
  site, before minting: validate op/tag + the full progress invariant (count>0,
  within effective request, `ReadSome` span length = count, `start+length` within
  admitted buffer, `ReadEof` only zero-read, zero-write ⇒ `NoProgress`). On pass,
  **one mint** produces `BoundedNat(count)` and the `TransferCount` projections
  from that same scalar. On any clause failure, the existing structured error is
  produced (never a minted count). This is the **only** construction site.
- **N-D3 — admit + fully lower the four ops.** Flip `BufferAllocate`,
  `BufferFreeze`, `FsReadAt`, `FsWriteAt` from represented-unavailable to
  **checked-result-constructing** on the native lane at `cranelift_backend.rs`
  (the `3323-3330` region + the consumer list), wiring each op's exact request
  fields, success constructors (using N-D2's mint for the count/span/budget), and
  **all existing structured resource/I/O errors** (`Closed`, `MalformedResource`,
  `ResourceKindMismatch`, `RightNotHeld`, invalid offset/window/bounds,
  buffer-limit/alloc failure, `NoProgress`, `Interrupted`, host-I/O failure). No
  ABI byte change.
- **N-D4 — interpreter/native agreement.** The native carrier + mint agree with
  the landed interpreter `Nat`-minting reference on the same host reply
  (count/order/terminal outcome), so a checked program produces the **same**
  structural-`Nat`-eliminating result on both lanes. This is what makes the PX8-F
  `writeAll` native witness reachable (evidence-only overlay in N-T3 below).

## Required test classes (Architect-mandated; each independently reaching; §7)

The Architect fixed **four** test classes — all four are required:

1. **N-T1 — resource-independent bounded-`Nat` units.** `Zero`/`Suc` elimination;
   multi-step recursion; **exact predecessor/IH order**; malformed/over-bound
   **rejection**. These exercise the carrier's `Nat` semantics with **no resource
   op** — proving it is a general mechanism, not a `writeAll`/buffer intrinsic.
2. **N-T2 — exact response-boundary positives + negatives.** Positives: `ReadEof`
   (zero read is the only zero), a **positive short `ReadSome`** (count < request,
   span length = count), a **positive short `Wrote`**. Negatives: **zero/over-bound
   progress rejected** (zero write ⇒ `NoProgress` not a minted count; count >
   request rejected; `start+length` outside admitted buffer rejected). Assert the
   **specific** outcome/variant, never `is_err`.
3. **N-T3 — the real PX8-F linked-native fixture as an evidence-only overlay.**
   Include Foundation's minimized `px8f_buffer_native` fixture (from
   `preserved/px8f-held-60a481b5`) as **evidence only**, showing transparent
   `writeAll` recursion **reaches native execution and agrees with the
   interpreter**. This is a reachability witness for N-D3/N-D4, **not** PX8-F's
   terminal gate (PX8-F rebases + re-proves after PX8-N + the PX8-X train land).
4. **N-T4 — decrement/scalar-substitution discriminator.** A mutation that
   **breaks the strictly-decreasing decrement** or **substitutes the raw scalar**
   for the structural eliminator makes the exact result/proof fixture **fail to
   elaborate/execute**. This proves the carrier's `Nat` semantics are load-bearing,
   not decorative.

## Acceptance criteria (testable)

- **AC1** — N-D1..N-D4 landed: the `BoundedNat` carrier + validated-boundary mint
  + the four ops fully lowered (checked results constructed on the native lane,
  not merely admitted); native agrees with the interpreter reference (N-D4).
- **AC2** — the mint is confined to the private checked result path: **no public
  constructor, no `Int` budget field, no caller-selected fuel, no construction
  outside N-D2** (grep the construction sites; the carrier type's constructors are
  private to the lowering module).
- **AC3** — all **four** test classes green: N-T1 units, N-T2 boundary
  positives/negatives (asserting exact variants), N-T3 evidence overlay reaching
  native + agreeing with the interpreter, N-T4 discriminator failing on
  broken-decrement / raw-scalar substitution.
- **AC4** — **host-effect ABI bytes unchanged** (no ABI expansion); the
  `HostReplyV1` scalar layout is untouched.
- **AC5** — **no-regression = GREEN IN CI** (never a local `--workspace` run;
  COORDINATION §12). Build/test **targeted only** (`scripts/ken-cargo -p ken-host
  …` / `-p ken-runtime …` / `-p ken-cli --test px8f_buffer_native` / `--test
  <name>`), **plus** run the ken-cli native integration suites the lowering
  implicates before release.

## Do-not-reopen guard

- Do **not** add a public `Int -> Nat` primitive, a surface constructor, or a
  caller-selected fuel parameter — the carrier is minted only inside the private
  checked result path (N-D2).
- Do **not** allocate an eager unary chain — the carrier stays compact (one
  machine scalar) with structural-`Nat` **observations**.
- Do **not** weaken the PX8-F private structural budget or theorem, and do **not**
  bury this mechanism inside the PX8-X observation/schema train — it is a separate
  Runtime compiler WP with its own QA + §14.
- Do **not** expand the host-effect ABI or change `HostReplyV1` bytes.
- Do **not** resurrect any V1 observation/export byte-preservation promise — that
  is superseded (PRINCIPLES transient T); PX8-N is host-reply→checked-result
  lowering only.
- Do **not** "admit the op ids" without constructing the checked result — that
  moves the failure, it does not solve it (the exact PX8-F hard-stop).
- If the bounded-`Nat` carrier cannot be minted soundly from the validated scalar
  reply without an ABI change or a surface primitive, **HARD-STOP to the Steward**
  rather than weakening the mechanism.

## Sequencing

`{ PX8-X train ∥ PX8-N } → PX8-F rebased terminal gate → Phase-C exit`.
PX8-N and the PX8-X unification train are **independent Runtime efforts**
running in parallel (own branch, own QA, own §14, own Decision). **Both
must land before PX8-F resumes:** Foundation then rebases the preserved
candidate
(`preserved/px8f-held-60a481b5`) onto the combined main, removes obsolete
V1-obligation assertions, reruns the reaching linked-native `writeAll` proof
(now constructible because PX8-N supplies the native structural-`Nat`), and takes
fresh QA/§14/CV. Ward is external throughout.
