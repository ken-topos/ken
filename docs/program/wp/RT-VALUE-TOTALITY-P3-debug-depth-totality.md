# RT-VALUE-TOTALITY P3 — `Debug` is the last host-recursive `Value` traversal

**Node:** [`RT-VALUE-TOTALITY`](../issues/RT-VALUE-TOTALITY.md) · **Owner:**
Runtime · **Size:** S–M · **Gate:** none

**Fixed inputs, measured at `origin/main = 5df415c1`. ⛔ Re-derive at point of
use — these are current-state claims and they perish.**

| input | pin |
|---|---|
| the defect | `crates/ken-runtime/src/values.rs` blob **`a2921904`** — `#[derive(Debug)]` at **`:130`**, immediately above `pub enum Value` |
| the mechanism to share | same file, `impl Drop for Value` **`:381`** and `impl Clone for Value` **`:399`**, and their doc block at **`:368–380`** describing the explicit **LIFO** heap worklist |
| the postorder sibling | `crates/ken-runtime/src/canonical.rs` blob **`bc2579a9`** — `project_operational_to_canonical` at **`:107`**, `Visit`/`Finish` worklist, `Finish` arm at **`:165`** |
| the AC text | `RT-VALUE-TOTALITY.md` **`AC-V11`** — read it; this frame does not restate its reasoning |
| the corroborating probe | `runtime-implementer`, `evt_2119bqa3tnz0a`: landed `Debug` dies of stack overflow at **`D = 131072`**, out of process |

## 1. Why this is its own WP and not a line in P2

P1 made the canonical encoder, `Clone` and `Drop` iterative. P2 split the
representation and removed the derived `Eq`/`Ord`/`Hash`. **`Debug` survived
both**, because P2's subject is *representation* and depth is not
representation — `Debug` appears in this node exactly once, inside a quoted
derive line, so it had no P2 edit to ride on.

⭐ **What makes it worth a WP rather than a cleanup:** every other deep-recursion
site left is reached from a *deliberate* call — an identity comparison, an
encode. `Debug` is reached from `{:?}` in a **panic handler, a log line, or an
`assert_eq!` failure message**. ⇒ The abort fires **while a maintainer is
diagnosing something else**, and it destroys the diagnostic that was being
produced. That is a different failure class from the others, not a smaller one.

## 2. Deliverable

**One hand-written `impl Debug for Value` driving the same explicit heap
worklist `Clone`/`Drop` already use** (`values.rs:368–380`), replacing the
derive at `:130`.

⛔ **Not a second traversal mechanism beside the existing one.** The `AC-V9`
prohibition applies here for exactly the reason it applied there: two worklists
diverge, and the next depth defect lands in whichever one the reader did not
check. If the existing worklist cannot be shared as-is, **that is a finding to
report, not a licence to fork it.**

## 3. Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-P3a` | ⭐⭐ **The mechanism is pinned, not the depth.** `Debug` traverses the same explicit heap worklist as `Clone`/`Drop`, therefore its depth is heap-bounded rather than host-stack-bounded. | ⛔ **A discharge whose whole content is "it survives `D = 131072`" is NOT this AC** — that is green against one depth on one platform and re-derives nothing if the traversal changes. State the structural argument; cite the measurement **beside** it |
| `AC-P3b` | A `{:?}` at the **same `D` that `AC-V1` exercises** returns. | ⛔ Run it **out of process** — a stack overflow aborts, it does not unwind, so an in-process control cannot observe its own failure. ⚠ **State the depth as a number BEFORE running.** A control that renders nothing reports the same green as one that renders a deep value |
| `AC-P3c` | ⭐ **The control actually rendered.** | Assert something about the produced string that is **impossible at shallow depth** — e.g. its length, or an occurrence count of a nesting token that scales with `D`. This is the positive control for `AC-P3b`; without it, `AC-P3b` passes for any reason including the value never being built |
| `AC-P3d` | The derive is **gone**, not shadowed. | `#[derive(Debug)]` no longer appears above `pub enum Value`. ⛔ A hand-written impl added while the derive remains does not compile — but a derive moved to a wrapper type would, and would leave the recursive path reachable. Say which you did |
| `AC-P3e` | No regression, and **no output contract is asserted**. | ⚠ `Debug` output is unspecified, and that is not a reason to skip this WP — **the claim under test is *does it return*, not *what does it print*.** Equally: ⛔ do not add a test that pins the rendered text, which would freeze an unspecified surface. Targeted suites green in CI |
| `AC-P3f` | Trusted-base delta is **zero**. | `Debug` renders; it decides nothing. No `trusted_base()` change, no new primitive |

## 4. Scope

**IN:** `impl Debug for Value` and whatever minimal sharing of the existing
worklist it needs; the controls above.

⛔ **OUT:**
- ⛔ **Any other `Value` traversal.** P1 and P2 landed; do not revisit them.
- ⛔ **Changing what `Debug` prints** beyond what a worklist rewrite forces. If
  the output does change, **report the difference** — do not pin it.
- ⛔ **`ken-foundation`'s twin.** P2's `AC-V10` owns that crate's disposition.
- ⛔ **Deriving `Debug` for any other type.** The subject is `Value` alone.

## 5. Contention check

**Measured at `5df415c1`, not assumed.** `values.rs` was last moved by P2, which
is merged; no open WP branch names it. `RT-FNSPLIT-C1` and `B2F` are held and
touch `cranelift_backend/`, not `values.rs`.

⭐ **The licence is that this WP changes one trait impl on a type whose public
shape P2 already settled** — not that no other node mentions `Value`. Several
do. If a candidate finds itself editing the enum's *variants*, the premise has
failed and it comes back to me.

## 6. Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `COORDINATION.md §12`). `scripts/ken-cargo
test -p ken-runtime`, plus the out-of-process control for `AC-P3b`/`AC-P3c`.
Workspace, `--locked` and conformance run **in CI**.

⚠ `ken-cargo` is a machine-wide `flock` with one slot and several rings are
live on it — coordinate the turn **in-thread**; ⛔ never sample `ps` to decide
it is free.

## 7. Reporting

Return exact SHA/tree/base, and specifically: **the structural argument for
`AC-P3a`** (which worklist, shared how); **the depth number stated before the
run** and the out-of-process result for `AC-P3b`; **the positive-control
assertion for `AC-P3c`** and what it would have caught; and whether `Debug`'s
rendered output changed.
