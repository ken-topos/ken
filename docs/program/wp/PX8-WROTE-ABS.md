# PX8-WROTE-ABS — the interpreter's capped-short `Wrote` oracle

**The interpreter has capped-full read, capped-short read, and capped-full
write. The fourth cell — capped-short write — does not exist. Native has both
write cells. The corpus's own comment, twelve lines above the gap, says
capped-short is *"the load-bearing pair member."***

**Owner:** Team Verify (`verify-leader` + `verify-implementer` + `verify-qa`).
**Branch:** `wp/PX8-WROTE-ABS`. **Size:** S.
**Risk:** low — one test added to an existing module, no `src` behavior change.

**Status:** Steward frame, shovel-ready. ⭐ **On the Linux ABI I critical
path** — `PX8` does not close until this and `PX8-F-CAP-41` discharge, and 15
of the program's 19 nodes descend from `PX8`.

---

## 1. Fixed inputs

| path | blob at `origin/main = 12a5ef4f` |
|---|---|
| `crates/ken-interp/src/eval.rs` | `8ecf5d4b7ce063fa060c57411a6abd5c3ca2c797` |
| `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/effects.rs` | `24edfa47e17f386d99c66ad2d22992ed9bc4dd91` |
| `spec/30-surface/38-ffi-io.md` | `56c3b3d5f1090f8920cc66286e0d7ba3729f0113` |
| `conformance/behavioral/buffer-io/seed-buffer-io.md` | `0364b230742e08f67fc59a2c2421221744b051e0` |

⚠ `38-ffi-io.md` is **LOCKED** and is the absolute oracle. ⛔ This WP does not
edit `spec/`, `conformance/`, or any `src/**` behavior.

---

## 2. The measurement

The `budget_eff_*` oracle family in `crates/ken-interp/src/eval.rs`:

| cell | interpreter | native |
|---|---|---|
| capped-**full** read | ✅ `:6469` | ✅ `effects.rs:~430` |
| capped-**short** read | ✅ `:6554` | ✅ `effects.rs:~440` |
| capped-**full** write | ✅ `:6629` | ✅ `effects.rs:453` |
| capped-**short** write | ⛔ **ABSENT** | ✅ `effects.rs:453` |

Native's test
`budget_eff_native_wrote_capped_full_and_short_reify_effective_not_raw_remaining`
asserts both write cells in one test, the short one being
`raw 8 / effective 4 / count 2 -> remaining 2`, labelled in-source
*"NOT 6 == raw 8 - count 2, the pre-fix defect this WP closes."*

⇒ **The interpreter has no corresponding assertion.**

### ⭐ The corpus already argues for this test, in its own words

`eval.rs:6461-6467`, the comment introducing the whole family:

> *"capped-full ALONE is satisfiable by the wrong shortcut `effective :=
> count` (both diverge from raw budget only because they happen to equal the
> count in the full-buffer case). capped-short is the load-bearing pair member
> — it is the only shape where `effective` and `count` differ, so it is the
> only shape that can catch an implementation that quietly substitutes one for
> the other."*

That reasoning was applied to the read pair and **not** to the write pair.
The wrong shortcut on the write side is the same one: the reifier arm at
**`eval.rs:5316-5326`** is green under capped-full because both formulae yield
`remaining == 0`. The operand is `:5322`:

```rust
let count = transferred.get();
let effective = transferred.effective_request();   // :5322 — the operand
let remaining = buffer_nat_value(effective.checked_sub(count)…)?;  // :5324
```

> ### ⛔ LOCATOR CORRECTED 2026-07-27 — this frame cited `eval.rs:4981-4997`
>
> **That range is wrong and it was wrong when written.** `:4981-4997` is the
> `FsReadAt` / `PrivateFsWriteAt` **request-argument narrowing** block
> (`narrow_host_u64` on offset/start/length) — it does not compute `effective`
> at all, so `effective := count` is not expressible there. Re-measured at
> `origin/main = 5fbbc67e`, where the `eval.rs` blob is **unchanged** from the
> `12a5ef4f` this frame was pinned at ⇒ ⛔ not drift, an authoring error.
>
> ### ⭐⭐ And the correction matters more than a line number
>
> **`:5303` is the `ReadSome` arm and carries a byte-identical expression** —
> `let effective = transferred.effective_request();`. ⇒ A ring that mutates
> "the `effective` computation" and sees red may have hit the **read** arm and
> reddened the **pre-existing** read tests (`:6469`, `:6554`), concluding the
> control discharged while the write arm was never exercised at all.
>
> ⛔ **The mutation must be on the `Wrote` arm at `:5322`, and the redden must
> be attributed by test name.** ⚠ **This hazard is unchanged and still
> load-bearing** — but see **`AC-1‴`**, not `AC-1`: the control is now three
> mutations, and the load-bearing one must redden `D1`'s new component test.

⚠ **The source formula is presently right.** This is an *evidence* gap, not a
behavior gap — the Architect's clause-(a) verdict (`evt_163mfgjs7fkh8`)
classes it as *absolute-not-differential evidence not discharged*. ⛔ Do not
open this expecting to find a bug in `eval.rs`; expect to find that nothing
would notice if one appeared.

---

## 3. ⭐⭐ Steward-discharged — and one question I deliberately did NOT answer

### 3a. Scope is **A2a only**. A2b is split out.

`docs/program/issues/PX8-WROTE-ABS.md` carries two gaps. **A2b** — the five
PR-C error identities (`MalformedResource`, `InvalidBounds`,
allocation-failure-distinct-from-`BufferLimit`, unsupported-nonblocking
posture, host-I/O-failure-distinct-from-`Interrupted`) with no independent
reaching evidence — **needs a normative scoping call before it can be sized**,
per the Architect's own second route. It is filed as `PX8-ERRID-SCOPE` and is
**not yours**. ⛔ Do not widen into it, and ⛔ do not treat its absence as a
reason this WP cannot close.

### 3b. ⛔ The fixture construction is the WP, and I am not prescribing it

> ⛔⛔ **STOP — this section was ruled FALSE, and then the ruling itself was
> REVERSED. `§3b` and `§3c` are both dead. Two things you must read before
> acting on anything in `§3`–`§3c`: the SUPERSEDED banner at the end of `§3b`
> (which corrects the dichotomy) AND its `ROWS 3 AND 4` correction (Architect
> `evt_5h884g6xhtts3`, which restores `D1`). ⇒ ✅ **`D1` IS available at the
> component boundary — see `§5 D1`.** ⛔ A comment-only deliverable does NOT
> discharge `PX8`.**
>
> The text below is kept only as the record of what was originally asked.

The discriminating shape requires **`count < effective < requested`**. On the
read path that is produced by a short source file (`:6554` reads 2 bytes into
a capacity-4 buffer against a length-8 request).

⚠ **I have not verified that the write path can produce it**, and I am not
going to guess in a frame. The open question is exactly:

> On `PrivateFsWriteAt`, is `effective` the buffer's **capacity** or its
> **installed window length**?

- If **capacity**: install a 2-byte window in a capacity-4 buffer, request 8
  ⇒ `count 2`, `effective 4`, `remaining 2` — discriminating, and the test is
  a direct mirror of `:6554`.
- If **installed window length**: `count == effective == 2 ⇒ remaining 0`, the
  same coincidence that makes capped-full non-discriminating, and **the cell
  is not merely missing — it is inexpressible on this path.**

⭐ **Both outcomes are acceptable deliverables.** See `§5 D3`.

> ## ⛔⛔ SUPERSEDED 2026-07-27 — the dichotomy above is FALSE
>
> **Ruled `evt_1grq3fcfkz4yy` on `verify-implementer`'s hard stop, measured at
> `origin/main = 06722d2b` and verified independently. Durable here because a
> ruling that lives only in the channel is not an input.**
>
> **Neither branch is the mechanism.** `effective` **is** capacity-backed
> (`effect_v1.rs:1784-1786`: `effective = min(requested, capacity - start)`), so
> branch (b) is false — **and branch (a)'s fixture is still unbuildable.**
> `count < effective` is **unreachable** on the interpreter write path, by a
> composition of four facts — ⛔ **and that conclusion is FALSE; see the
> correction directly below the table:**
>
> | # | fact | site |
> |---|---|---|
> | 1 | `effective = min(requested, capacity - start)` — capacity-backed | `effect_v1.rs:1784-1786` |
> | 2 | the whole `[start, start+effective)` must be in the installed live window ⇒ a 2-byte window returns exact `InvalidBounds` **before any write** | `:1798-1800` → `:681-692` |
> | 3 | `InterpreterHostBackend` does **not** override `fs_resource_write_at`; it inherits the direct POSIX call ⇒ **no injectable short-write seam** | `eval.rs:4399-4697`, `effect_v1.rs:1356-1364` |
> | 4 | `TransferCountV1::new` is **`pub(crate)`** to `ken-host` ⇒ `ken-interp` cannot hand-construct `count 2 / effective 4` either | `effect_v1.rs:2188` |
>
> ### ⛔⛔ ROWS 3 AND 4: TRUE FACTS, FALSE CONSEQUENCE (`evt_5h884g6xhtts3`)
>
> ⚠ **Read this before using the table.** All four rows are individually
> accurate and they remain in the frame for a reason (`AC-6′`). But the
> conclusion drawn from them — *"`count < effective` is unreachable, so `D1` is
> unavailable"* — is **wrong**, and rows 3 and 4 are where it went wrong:
>
> - **Row 3 censused the wrong object.** The short-write seam is the **`pub
>   trait HostEffectBackendV1`** (`effect_v1.rs:1214`), not the one concrete
>   `InterpreterHostBackend` that declines to override it. `dispatch_host_op_v1`
>   calls **`backend.fs_resource_write_at(…)`** through the trait at
>   **`:1801-1803`**. ⇒ A **test-local** impl returning `2` for `effective 4` is
>   an injection point that already exists. ⭐ `ken-interp` **already implements
>   this trait** at `eval.rs:4399`.
> - **Row 4 is irrelevant, not decisive.** Nothing needs to forge a
>   `TransferCountV1`: **ken-host mints it itself** at **`:1811`**
>   (`TransferCountV1::new(written, effective)`), after validating `written != 0`
>   (`:1804`) and `written <= effective` (`:1807`). ⇒ `pub(crate)` never blocked
>   this route.
>
> ⇒ ⭐ **What the four rows actually prove is that no *end-to-end regular-file*
> interpreter fixture exists — a fixture-reachability limit, NOT a semantic
> absence.** `count < effective` is squarely inside the admitted domain: LOCKED
> `§38.1.7.2` admits `0 < n <= effective` including a short write and defines
> `remaining = effective - n`, and the dispatcher accepts exactly that range.
>
> ### ⭐⭐⭐ The generalizable rule — carry this to the other three cases
>
> **Architect, `evt_35wf94pv5q28v`:** evidence must distinguish every
> **normatively different behavior over the component's admitted semantic
> domain.** Mutations that are *extensionally equal over that domain* need no
> discriminator — but `effective := count` is equal to the real formula **only
> over the current full-write fixture subset**, not over `§38`'s `Wrote n`
> domain. ⛔ And conversely: a genuinely uninhabited value — excluded by the
> closed type / validator / contract itself — does not bind merely because one
> can write suggestive prose about it.
>
> ⇒ ⛔ **Derive the domain from the authoritative constructor and admission
> boundary FIRST, then ask reachability. Never infer semantic absence from the
> lack of a convenient top-level producer.** That inversion is exactly the error
> this section records.
>
> ### ⭐⭐ Why this frame got it wrong — the matrix's symmetry is an artifact
>
> `§2`'s 2×2 presented four cells as four instances of one shape. **They are
> three different shapes:**
>
> - **read** cells get their discriminating count **free from file size** —
>   `:6554` writes a 2-byte source and the real POSIX read returns 2;
> - **writes have no analogue** — POSIX `write()` to a regular file writes
>   everything or errors. There is no "short source" for a write;
> - **native's** cells test the **lowered arithmetic**, not a host reply —
>   `run_checked_bounded_nat_fixture(2, 0, 8, 4, …)` takes the scalars as
>   arguments.
>
> ⇒ ⭐ **Drawing them in one table made a genuinely absent capability look like a
> missing test.** ⛔ Do not read a matrix cell as a comparable instance without
> checking that the cells share a production route.
>
> ### ✅ Authorized: `D3` second form, and nothing else
>
> ⛔ **No write fixture. No seam. No `src/**` behavior change.** ⭐ A
> production short-write seam added so a test can force `count 2` is a
> production change driven by test convenience — banned, and unnecessary.
>
> ✅ The measurement **may** be committed as a comment adjacent to the `Wrote`
> arm (`eval.rs:5316-5326`), in the style of the existing `:6456-6465` note.
>
> ⚖️ **ANSWERED — that routed consequence is resolved.** Architect
> `evt_5h884g6xhtts3`: clause (a) **still binds** capped-short `Wrote`; ⛔ no
> named durable exclusion (it would let an admitted closed-sum arm escape the
> oracle the property exists to require) and ⛔ no production seam. The
> discharge is `D1`'s component-boundary oracle.

### 3c. ⛔⛔ DEAD SECTION — the cell is NOT inexpressible

> ⛔ **Superseded 2026-07-27 (Architect `evt_5h884g6xhtts3`).** `count <
> effective` is inside the admitted domain (LOCKED `§38.1.7.2`), and `D1` is
> available at the component boundary. ⛔ **Do not deliver an inexpressibility
> finding.** The reasoning below is retained only because its *general* point —
> that an unconstructible cell reads identically to an unwritten one — is
> correct and is what made this frame wrong twice. ⭐ The corrective is in the
> `§3b` `ROWS 3 AND 4` block: derive the domain from the authoritative
> constructor and admission boundary FIRST, then ask reachability.

⚠ A test cell that *cannot* be constructed is byte-identical, to any reader,
to one that simply has not been written yet. This corpus has two other
instances of exactly that class already filed (`CONF-FMT8-LEVELTOK`, and
`SEC1-IFC-R3`'s synthetic `Disproved` verdicts).

⇒ If you establish that `count < effective` is unconstructible on the write
path, **say so with the measurement and stop** — do not manufacture a fixture
that reaches the shape by some other route, and ⛔ do not weaken the assertion
to something the coincidental shape can satisfy. Route it to me under `§8`.

---

## 4. ⛔ Banned shapes

- ⛔ **No change to any `crates/**/src/**` behavior.** The formula is correct;
  you are adding the assertion that would notice if it stopped being.
- ⛔ **Do not edit `spec/` or `conformance/`.** `38-ffi-io.md` is LOCKED and is
  the oracle you assert *against*. Editing a cited source moves its OID.
- ⛔ **Do not touch the three existing `budget_eff_*` tests**, their shared
  helpers (`nat_value`, `allocate_buffer`, `open_file`, `release_resource`,
  `rt_parity_root`), or the `:6450-6467` comment block beyond appending to it.
- ⛔ **Do not assert differentially against the native test.** Clause (a) is
  *absolute-not-differential*: the expected `remaining` must be stated as a
  literal derived from `38`, not as "whatever native produced." ⭐ A
  cross-lane equality check is a different property and does not discharge
  this one.
- ⛔ **No `--workspace` run.** Targeted only (`COORDINATION §12`).

---

## 5. Deliverables

- **`D1`** ✅ **RESTORED AND RE-SPECIFIED 2026-07-27** (Architect,
  `evt_5h884g6xhtts3`) — the interpreter capped-short `Wrote` absolute oracle
  **is available**, at the **component boundary** rather than end-to-end.
  ⛔ Production is unchanged: no seam, no `pub` constructor, no `cfg(test)`
  production hook, no relaxed visibility.

  **The route, verified to exist:**

  1. a **test-local** type in `eval.rs`'s inline test module implementing the
     already-`pub` **`ken_host::HostEffectBackendV1`** (`effect_v1.rs:1214`),
     whose `fs_resource_write_at` returns **`2`** for a valid effective request
     of **`4`**;
  2. call the **real** shared dispatcher **`ken_host::dispatch_host_op_v1`**
     (`effect_v1.rs:1409`, re-exported by `pub use effect_v1::*`) — it validates
     `written != 0` (`:1804`) and `written <= effective` (`:1807`), then **mints
     the private `TransferCountV1` itself** at `:1811`;
  3. pass the resulting `CanonicalOutcomeV1` through the **existing**
     `reify_host_reply_v1` `Wrote` arm (`:5315-5331`);
  4. assert the `§38`-derived literal **`remaining = 2`** (`effective 4 −
     count 2`), with an in-source line naming the rejected formula in
     `:6554`'s style.

  ⚠ **The buffer still needs a 4-byte installed window** or `initialized_slice`
  (`:1798-1800`) rejects first — reuse `:6629`'s setup, which installs exactly 4
  bytes through a real `FsReadAt`.
  ⚠ **Shape is the ring's call:** a standalone test-local backend, or a newtype
  delegating to `InterpreterHostBackend` (`eval.rs:4399`) overriding only
  `fs_resource_write_at`. ⭐ Check which trait methods lack defaults before
  choosing. The Architect authorized *"a test-local implementation … or an
  equivalent existing host-dispatch fixture."*

  ⭐ **This is the same component-boundary shape native already uses** —
  `run_checked_bounded_nat_fixture(2, 0, 8, 4, …)`. Requiring an interpreter OS
  fixture while accepting native's component evidence was an **accidental
  asymmetry**, and that asymmetry is what this frame previously read as
  inexpressibility.
- **`D2`** — the `§38` clause the literal is derived from, cited by section in
  a comment. ⭐ A reader must be able to check the number against the spec
  without re-deriving it.
- **`D3`** ⛔⛔ **SUPERSEDED 2026-07-27 by the Architect** (`evt_5h884g6xhtts3`)
  — ⚠ **it was ruled twice and the second ruling reverses the first.**

  **What survives:** the four-site measurement, as proof that **no end-to-end
  regular-file interpreter fixture** can force `count < effective`. Keep it in
  the comment; `AC-6′` still requires all four sites.

  **What is struck:** the claims that (i) `D1` is unavailable and (ii) the
  measurement **alone** discharges clause (a). Both are false — rows 3 and 4
  censused the concrete backend and the constructor's visibility, neither of
  which is the seam (see the `§3b` correction). ⇒ ⛔ **A comment-only
  deliverable does NOT discharge `PX8`.**

  ⭐ **Deliver `D1` by the component-boundary route** and keep the measurement
  as the *reachability* result it actually is — the honest framing is *"no
  end-to-end fixture; here is the component oracle instead"*, not *"the value
  cannot exist."*
- **`D4`** — a one-line statement of whether the `:6450-6467` comment's
  "load-bearing pair member" reasoning is now true of **both** pairs, or
  remains true of the read pair only and why.
  ⭐ **AMENDED — `D4` now also records the two-shortcut split** in `AC-1‴`: the
  `:6456-6465` comment's reasoning is about `effective := count`, and it is
  **correct about reads** (where the capped-short read cell kills it). On writes
  that shortcut is not merely unkilled by the capped-full cell — it is
  **unkillable**, while the *historical* shortcut `effective := requested`
  **is** killed by that cell. ⇒ ⛔ Do not write `D4` as "the write pair has no
  load-bearing member"; it has one, against a different shortcut.

---

## 6. Acceptance criteria

- **`AC-1`** ⛔ **SUPERSEDED AS WRITTEN 2026-07-27** (`evt_5m964d2fygxpj`, this
  amendment). It required *"show the **new test** reddens"* — and `D3` withdrew
  the new test. ⚠ **It is also vacuous on its own terms**, which is the part the
  earlier note got wrong: the shortcut it names (`let effective = count;`) is a
  **semantic no-op** on every reachable write, because `D3` proves `count ==
  effective` there. ⇒ No test could redden under it, and **the prior claim that
  this control "still applies and still matters" was false.**

  ### ⭐⭐ Because there are TWO wrong shortcuts, with opposite detectability

  | mutation of `:5322` | on the write path | caught by |
  |---|---|---|
  | `effective := requested` — **the historical defect** | `remaining` becomes `requested - count` = **4**, not 0 | ✅ the capped-**full** write test, `:6629` |
  | `effective := count` — the `:6456-6465` shortcut | identity **only over the current full-write fixture subset** | ⛔ nothing **today** — ✅ `D1`'s new test once it exists |

  ⇒ ⭐ **The write cell is NOT evidence-free.** It has real discriminating power
  against the shortcut that actually occurred in this codebase — the test's own
  comment at `:6689-6690` predicts `remaining` 4. `AC-1` named **only the other
  one**, which the existing fixture population cannot reach.

  ⚠ **CORRECTED by the Architect (`evt_35wf94pv5q28v`): row 2 says "nothing,
  ever" and that is wrong.** `effective := count` is extensionally equal to the
  real formula only over the **full-write subset**; on the admitted **short**
  subset (`count < effective`) it is **not** an identity and yields the wrong
  checked value. ⇒ It is discriminable — by `D1`'s component-boundary test, which
  supplies exactly that input. **"Vacuous" was scoped to the reachable fixtures,
  not to the domain.**

- **`AC-1‴`** ⭐ **(load-bearing — supersedes `AC-1` and `AC-1″`)** — Architect,
  `evt_5h884g6xhtts3`. Three mutations of `eval.rs:5322`, each applied **alone**
  and restored **byte-identically**, each outcome reported **by test name**:

  | # | mutation | required outcome |
  |---|---|---|
  | 1 | `let effective = count;` | ⛔ **MUST redden `D1`'s new component test** — this is the load-bearing control |
  | 2 | `let effective = <raw `FsWriteAt` length>` | ✅ must redden `budget_eff_capped_full_write_reifies_effective_not_raw_remaining` (`:6629`) at its `assert_eq!(remaining, 0, …)` |
  | — | both | ✅ capped-short **read** `:6554` **unaffected**; `:5303` byte-identical |

  ⭐ **Mutation 1 is the whole point of the WP** and it is now expected to
  **redden**, not stay green — that is precisely what `D1` buys. ⛔ If it stays
  green with `D1` in place, `D1` is not exercising the short subset: stop and
  route, do not report it as discharged.

  ⭐ Mutation 2 is retained as the **historical-shortcut positive control** —
  keep it. `request: &CanonicalRequestV1` is already a parameter of
  `reify_host_reply_v1` (`:5219`), so it stays a `Wrote`-arm-only edit.

  ⛔⛔ **Attribute every redden to a named test.** `:5303` is the `ReadSome` arm
  with a byte-identical expression; a redden you cannot name may be the read
  tests and says nothing about the write arm.

- **`AC-2`** — the capped-**full** write test's behavior is **reported, not
  asserted green**. Under mutation 1 it is expected green (full-write is where
  the shortcut is an identity); under mutation 2 it **must** redden. ⛔ If it
  reddens under mutation 1, the mutation was wider than intended.

- **`AC-3`** — the asserted `remaining` is an **absolute literal** traceable to
  `38-ffi-io.md`. **Control:** quote the governing clause. ⛔ "Matches native"
  fails this AC.

- **`AC-4`** — scope. **Control:** `git diff --name-only` shows
  `crates/ken-interp/src/eval.rs` and nothing else.

- **`AC-5`** — targeted green. **Control:**
  `scripts/ken-cargo test -p ken-interp --lib` — the whole lib test suite, not
  a single `--test` filter, because these tests live in `src/eval.rs`'s inline
  module. ⚠ Re-derive build-slot availability first; `ken-cargo` blocks
  silently up to 30 minutes on lock contention.

- **`AC-6`** ⛔ **WITHDRAWN AS WRITTEN 2026-07-27** (`evt_1grq3fcfkz4yy`). It
  required naming *"the code that forces `effective` to the installed window"* —
  **nothing does; `effective` is capacity-backed** (`effect_v1.rs:1784-1786`).
  The AC presupposed a branch of the `§3b` dichotomy that is false, so it was
  not satisfiable by the true finding.
  **REPLACED — `AC-6′`, re-scoped 2026-07-27 (`evt_5h884g6xhtts3`):** the
  measurement names, by file and line, the code that makes `count < effective`
  **unreachable through an end-to-end regular-file interpreter fixture**.
  ⛔ **NOT "unreachable" simpliciter** — it is reachable at the component
  boundary, which is what `D1` now delivers. **Control:** all four sites of the
  `§3b` superseding table are cited — capacity-backed `effective`, live-window
  admission, the absent backend override, **and** the `pub(crate)`
  `TransferCountV1::new`. ⛔ Citing fewer than four is an incomplete census: each
  one independently closes a different route, so omitting one leaves that route
  looking open. ⛔ A prose assertion that it "appears not to be possible" still
  does not discharge this.

---

## 7. Contention

`crates/ken-interp/src/eval.rs` — **no live WP branch touches it.** Checked:
the three branches with `ken-interp` deltas (`wp/ABI-S3`, `wp/BUDGET-EFF`,
`wp/BUDGET-EXHAUST`) are all **merged** nodes. The seven active rings are on
`ken-runtime`/cranelift (Runtime), surface spaces (Language), `ken-kernel`
(Kernel), `ken-elaborator/tests` (Verify's current WP, Ergo), `spec/30`
(enclave), and `library/` (Doc).

⚠ **Re-measure at pickup** — `origin/main` moves.

---

## 8. Hard stop

⛔ Route to the Steward if:

- `count < effective` is unconstructible on the write path — ⭐ that is `D3`'s
  second form and a real result; deliver the measurement, do not improvise a
  fixture around it; **or**
- ⚠ **RESTORED 2026-07-27, second pass.** An intermediate amendment weakened
  this bullet to *"a no-redden mutation stops only if no companion mutation on
  the same line reddens"* — a workaround for the (now superseded) belief that
  `D1` was unavailable. **With `D1` restored the contradiction is gone and the
  original stop stands unweakened:** ⛔ **mutation 1 of `AC-1‴`
  (`effective := count`) reddening NOTHING is a hard stop** — it means `D1` is
  not exercising the admitted short subset. ⛔ So is mutation 1 reddening the
  capped-full test (too wide), or mutation 2 reddening nothing; **or**
- discharging `D1` appears to require editing `spec/`, `conformance/`, or any
  `src/**` behavior — it does not, and if it does, the gap is not the one this
  frame describes; **or**
- you find the source formula is actually **wrong** (not merely unasserted).
  ⭐ That would be a behavior gap, which is `PX8-F-CAP-41`'s class, not this
  one — stop and route it rather than fixing it here.
