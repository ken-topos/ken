# WP CP0 · §5.2 — safe Bytes/text ops + placeholder retirement

**Owner:** **Language-led** on the Rust migration (`bytes.rs` sigs + native
Cranelift lowering + the 8-site consumer sweep — the bulk of the work) ·
**Enclave-owned** portion: the `spec/` binary-I/O clause + conformance reconcile
+ the CV vote · **one coupled WP** (the sig/spec/conformance coupling wants
atomic landing — do NOT split) · **Coordination:** Foundation/Spec for the
DS-AC6/L6 fixture re-anchor (below) · **Terminal reviewer:** Architect
(effect-floor contract = his lane) · **Size:** M · **Base:** `origin/main @
1f85ad9a`. *(Ownership per Architect recommendation `evt_55vyqjpfz787t` +
Steward scope call: the bulk is build-team Rust engineering, so the team doing it
leads; the enclave supplies the spec/conformance half and the CV vote.)*

Transcribes the **Architect's ruled effect-floor contract** (`evt_2sxd7pszs42af`,
Deliverable 2). CP0 = the spec-prerequisite that makes the pure Bytes/text ops
**total and safe** and retires the bogus placeholder primitives, so the Console
+ FS floors rest on honest signatures and **CC2 (Text.Codec/ArgParse) can
standardize on the safe shapes.** This is *why CC2 is gated* — land CP0 first.

Runs **in parallel with I-3, no shared-file churn:** **CP0 owns
`crates/ken-elaborator/src/bytes.rs` (+ its native lowering) and the consumer
sweep**; I-3 owns prelude `FSOp` + Path. They meet only at the honesty check
"no bogus FS-write primitive remains" (that deletion is CP0's).

## Deliverable A — retire the placeholder FS/Net primitives (coupled)

The FS floor (I-3) consumes **none** of these; they are bogus
(`Bytes→Bytes→Bytes`, no path/cap/Result/driver arm — `bytes.rs:134-164`) and
type-check invalid programs. **No Ken program calls any of them**, but
retirement is coupled to fixtures — do NOT bare-delete:

- **`write_bytes`, `recv` — cleanly free.** Retire directly.
- **`append` — load-bearing for the DS-AC6 name-hygiene test**
  (`l3_strings_surface_acceptance.rs:398,410`) **+ the conformance seed**
  (`seed-collections.md:785`). Either migrate those fixtures, or — if the DS-AC6
  name-hygiene property is still wanted — **re-anchor it on a genuinely-distinct
  real pair.**
- **`send` — referenced by `l6_acceptance.rs` + `sec2_acceptance.rs`** (Net
  effect-row seed). Same choice: migrate the fixtures or re-anchor the
  effect-row property on a real pair.

**The re-anchor call is a Foundation/Spec coordination** (Architect flagged it —
not host-ABI). Resolve it explicitly; retiring `append`/`send` is a
conformance/collections coordination, **not** a silent delete.

## Deliverable B — safe total signatures for the pure Bytes/text ops

These are **NOT placeholders** — real reductions with native Cranelift lowering.
Migrate neutral-on-invalid → the spec's safe `Option`/`Result` shapes:

| op | current (`bytes.rs`) | safe target |
|---|---|---|
| `bytes_decode` | `Bytes→String` (neutral on invalid UTF-8, `:113`) | `Bytes→Result Utf8Error String` |
| `bytes_at` | `Bytes→Int→Int` | `Bytes→Int→Option UInt8` |
| `bytes_slice` | `Bytes→Int(start)→Int(len)→Bytes` | `Bytes→Int→Int→Option Bytes` |

Native Cranelift lowering updates accordingly + a `bytes_at.bounds` obligation.

**Whole-consumer sweep — CP0 must update every site (Architect inventory):**
Parsing package, Validation package, `examples/rosetta/read-file-lines`, CLI-E2E
`fs_read_file_lines_flip`, and the `l6_acceptance` / `cat5_parsing` / `nc16`
tests + the runtime lowering. **Sweep whole** — a missed consumer either fails to
compile (safe) or silently relies on the old neutral behavior (unsafe). Land the
safe sigs **before** Text.Codec/ArgParse (CC2) freeze the unsafe shapes.

## Deliverable C — spec + conformance reconcile (work-program `:109,:115`)

Reconcile the `spec/` binary-I/O clause to the safe signatures (the pure
`Bytes`/`Int`/`String` neutral-on-invalid vs safe `Option`/`Result` drift) +
update conformance. This is the enclave's core CP0 deliverable.

## Acceptance criteria

- **AC1** — `write_bytes`/`recv` removed from `bytes.rs`; `append`/`send` removed
  **with** their DS-AC6/L6 fixtures migrated or re-anchored on a real distinct
  pair (Foundation/Spec-coordinated); the honesty check "no bogus
  FS-write/Net primitive remains" holds. `trusted_base()` **shrinks** (net TCB
  reduction) — assert before/after.
- **AC2** — `bytes_decode`/`bytes_at`/`bytes_slice` carry the safe
  `Result`/`Option` sigs (Deliverable B) with native lowering + the
  `bytes_at.bounds` obligation; a bounds-violating `bytes_at` returns `None`
  (tested), never a neutral `0`.
- **AC3** — the **whole consumer inventory** compiles + passes against the new
  sigs: Parsing, Validation, `read-file-lines`, `fs_read_file_lines_flip`, `l6`,
  `cat5_parsing`, `nc16`, runtime lowering. No consumer left on the old shape.
- **AC4** — `spec/` binary-I/O clause + conformance reconciled to the safe sigs
  (Deliverable C).
- **AC5** — literal `scripts/ken-cargo build --workspace --locked && test
  --workspace --locked` green on the exact SHA; formatter clean.
- **AC6** — scope: **no prelude `FSOp`/Path delta** (that's I-3), no kernel rule
  change. `bytes.rs` + consumers + `spec/` + `conformance/` only.

## Do-not-reopen guardrails

- Design is settled (`evt_2sxd7pszs42af` Deliverable 2). Don't redesign the safe
  sigs or invent new Bytes ops.
- **Do not bare-delete `append`/`send`** — resolve the fixture coupling first.
- **Do not touch prelude `FSOp`/`fs_resp`/Path** — I-3 owns those; CP0 stays in
  `bytes.rs` + consumers + spec/conformance.
- **Coupled landing, one WP.** Language leads the `bytes.rs` sig + lowering +
  consumer-sweep bulk; the enclave supplies the `spec/` binary-I/O clause +
  conformance reconcile + the CV vote, and the enclave can draft the spec clause
  from this frame's pinned target sigs **in parallel** — but the whole thing
  lands atomically (the `BytesRoundTripLaw` `Ok s` presumption means the sig and
  its spec/conformance must not drift). Coordinate; do not split into two PRs.

## Review chain

Language build (`bytes.rs` safe sigs + native Cranelift lowering + the whole
consumer sweep) **∥ enclave** (`spec/` binary-I/O clause + conformance reconcile)
→ Language QA (locked workspace; the sweep-whole consumer inventory compiles +
passes; `bytes_at` OOB returns `None`) **+ CV conformance vote** (on
`spec/`+`conformance/`) → **Architect** terminal re-confirm (effect-floor
contract satisfied; safe sigs match; TCB shrank; zero prelude/`FSOp` delta) →
`git_request` to Steward → honesty-gate + CI-poll publish → **unblocks CC2.**
