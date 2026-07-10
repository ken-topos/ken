# WP DS-6c — elaborator `IntLit` emission (make DS-6b reduction surface-reachable)

**Owner:** Kernel team. **Steward-framed** (2026-07-10). Base: `origin/main @
b4e3ac30`. **The committed DS-6b fast-follow** (ADR 0013 Layer 2) — this is the
sequenced piece DS-6b explicitly deferred and put on record; **not optional**.
Inner-ring, soundness-relevant (it makes a kernel reduction reachable from
surface `.ken`), so it carries the **Architect soundness gate**.

## Goal

Make `.ken` **integer literals** elaborate to `Term::IntLit`, so DS-6b's
value-reduction arm actually fires on surface code — `Eq Int 5 5` computes to
`Top` (proof `tt` checks) and `Eq Int 5 6` computes to `Bottom` (rejected).
Retire the opaque-postulate + `num_values` side-table hack for the Int path.

## Background (grounded on `origin/main @ b4e3ac30`)

DS-6b landed the **kernel mechanism** but left it **surface-unreachable**:

- `Term::IntLit(num_bigint::BigInt)` exists (`crates/ken-kernel/src/term.rs:255`).
- `eq_reduce` reduces `Eq ty (IntLit m) (IntLit n) ⇝ Top if m==n else Bottom`,
  opt-in gated by `GlobalEnv::deceq_cert` (`crates/ken-kernel/src/obs.rs:75`,
  arm at `:89`/`:98`–`:114`).

But **no surface literal ever becomes an `IntLit`**. The two emission sites —
`elab_num_lit_infer` (`crates/ken-elaborator/src/elab.rs:2419`) and
`elab_num_lit_checked` (`:2442`) — currently mint an **opaque postulate**
(`declare_postulate(cx.env, vec![], expected)`) and stash the value in a
`num_values: HashMap<GlobalId, NumericLitVal>` side-table (`:2434`, `:2499`),
which the runtime resolves by lookup (`crates/ken-interp/src/eval.rs:1423`,
`crates/ken-cli/src/main.rs:387`). So a literal is an opaque constant, not a
value the kernel can compute on — DS-6b's arm never sees an `IntLit`.

## Scope

- **Rewire the Int-literal case** of `elab_num_lit_infer` / `elab_num_lit_checked`
  to emit `Term::IntLit(<bigint>)` directly (at type `Int`), instead of
  `declare_postulate` + `num_values` insert.
- **Retire the `num_values` hack on the Int path** across elaborator + interp +
  CLI: with the value in the term, the `GlobalId → NumericLitVal` lookup is
  dead for Int literals — remove it where it exists solely for that, keeping the
  runtime able to evaluate an `IntLit` directly.
- **Decimal literals are out of scope** — `NumLit` has an `Int` and a `Decimal`
  variant (`crates/ken-elaborator/src/ast.rs:407`); DS-6b only provides `IntLit`.
  Decimal keeps its existing `MkDecimalPair` construction untouched. If Decimal
  still needs `num_values`, that side-table **stays for Decimal** — do not force
  a full retirement. State honestly in the entry which paths were retired.

### Out of scope

- any Decimal carrier work (that sits behind the `90-open-decisions.md` gate);
- new kernel mechanism (`IntLit`/`eq_reduce` already landed — do **not** touch
  `obs.rs`/`term.rs` reduction logic; this WP is emission + wiring only);
- spec/conformance semantic changes (ADR 0013 already fixed the semantics).

## Acceptance criteria

- **AC1 — surface reachability (the point of the WP).** A test (a `.ken`
  fixture or an elaborator acceptance test) shows `Eq Int 5 5` elaborates with
  both operands as `IntLit`, reduces to `Top`, and a `tt` witness type-checks —
  concrete Int-equality **computes end-to-end from surface syntax**. The
  discriminating negative arm — `Eq Int 5 6` reduces to `Bottom`, and both `tt`
  **and** `Refl` are **rejected** — is asserted with the specific error variant,
  not bare `is_err()`.
- **AC2 — hack retired (Int path).** The Int-literal emission no longer mints an
  opaque postulate; the `num_values` Int entries are gone. Grep-confirm in the
  handoff (show the removed `declare_postulate`/`num_values` insert for Int).
- **AC3 — no trust growth (expect a *reduction*).** `trusted_base()` delta is
  empty-or-negative — retiring per-literal opaque `Decl`s should *shrink* the
  opaque surface, not grow it. No new `Decl::Opaque`/`declare_postulate` for
  Int literals. `crates/ken-kernel` reduction code unchanged.
- **AC4 — runtime intact.** The interpreter evaluates an `IntLit` directly; the
  differential / L1 / runtime-IR acceptance suites that consumed `num_values`
  still pass (they either read the value from the term or the driver seeding is
  updated). No regression in the trust-report/differential path.
- **AC5 — build.** Workspace-green in CI at merge (QA re-runs the workspace
  suite independently per K7 discipline). Local: **targeted builds only**
  (`-p <crate> <test>`), never a full local `cargo build`.

## Gate (soundness-relevant, inner-ring)

Kernel ring (kernel-leader → kernel-implementer → kernel-qa) → **@architect
soundness gate** — this is where completeness converts to soundness surface: the
Architect verifies the newly-reachable `IntLit` path is sound (`5==5 ⇝ Top`
computes, `5≠6 ⇝ Bottom` rejects `tt`/`Refl`), the TCB delta is
empty-or-negative, and the `num_values` retirement doesn't open a runtime hole —
→ `git_request` to Steward → **CI-gated** merge. No Spec/CV vote unless a
conformance case is added (welcome, not gating). Own the retro; flag every
judgment call. **No WP-token identifiers in production source** (self-grep the
whole diff before handoff).

**Frame-staleness note:** the kernel moves fast — re-verify these `file:line`
cites against the code at pickup before building; they are grounded on
`b4e3ac30` and may drift.
