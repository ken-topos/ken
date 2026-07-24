---
id: DOC-GATE-CONTROL-BINDING
title: "validation-gate registry: make the two DOC-GATE-RECORD-AXIS checks orphan-proof by lifting them to pure detectors with committed controls"
status: ready
owner: verify
size: S
gate: none
depends_on: [DOC-GATE-RECORD-AXIS]
blocks: []
github: null
origin: adversary finding G1 on DOC-GATE-RECORD-AXIS (64b0811f), side thread thr_2seh2bm1kr5mh evt_4j8fschh7v4vx, 2026-07-24. Steward-filed (agents cannot create tracked work per COORDINATION §2); Steward triage = CONFIRMED, and the overclaim half is a Steward defect.
---

> ## The remedy for the orphaning defect is itself orphanable
>
> `DOC-VALIDATION-BINDING` existed to kill one failure mode: *"deleting or
> renaming a gate left its token silently orphaned while the suite stayed
> green."* `DOC-GATE-RECORD-AXIS` then closed two coverage gaps — **with two
> bare assertions inside test bodies, either of which can be deleted without
> anything reddening.** The cure was applied in a form that lacks the cure.

## Objective

Convert the two checks that `DOC-GATE-RECORD-AXIS` added
(`crates/ken-cli/tests/library_documentation_gates.rs`) from removable inline
assertions into **pure detectors with committed tests**, matching a pattern the
same file already carries.

⛔ **This does not reopen `DOC-GATE-RECORD-AXIS`.** Both of its mechanisms are
correct and verified against the real manifest. `VALID_KINDS` (`:434`, `:450`)
rejects `"Status"`/`"statuses"`; the `status_records` assertion (`:588-603`)
fires exactly on a second `kind = "status"` record and names the required
action. **This WP changes their *form*, not their *behavior*.**

⛔ **Not in scope:** the `run: fn(&DocEntry)` refactor. The Adversary explicitly
declined to ask for it twice; do not fold it in.

## The defect

Measured against the tree at `64b0811f`:

```text
added   #[test] lines: 0
removed #[test] lines: 0        <- probe control
total   #[test] in file: 22     <- probe control: the grep does find tests
```

All 18 insertions are the `VALID_KINDS` const, its `else if` arm, one assertion
message reword, the `status_records` collection, and the `assert_eq!`.

- Delete the `assert_eq!` at `:598-603` — nothing reddens.
- Delete the `else if` at `:450-455` — nothing reddens.

Both new checks read `load_manifest()` directly instead of taking input, which
is *the same reason* the open-array parser defect was untestable before the
Librarian found it.

## The precedent — 100 lines up in the same file

When the Librarian found the open-array parser defect, the response was **not** a
one-time proof. It was to extract a pure helper and commit detector tests:

| item | line |
|---|---|
| `fn field_lines_inside_open_arrays(src: &str) -> Vec<String>` | `:700` |
| `fn gate_manifest_rejects_a_field_line_inside_an_open_multiline_array()` | `:730` |
| `fn field_lines_inside_open_arrays_detects_the_reported_shape()` | `:747` |

Mirror that shape exactly. It is in-file, has no new dependency, and needs no
new idiom.

## Deliverables

1. Lift each of the two checks to a **pure detector** taking input rather than
   loading the manifest — `fn(&[DocEntry]) -> Vec<String>` (returning the
   violations found), following `field_lines_inside_open_arrays`.
2. Call each detector from the **real gate**, so production behavior against
   the actual manifest is unchanged.
3. Add **two `#[test]`s over inline fixture entries**:
   - a second `kind = "status"` record ⇒ the record-axis detector reports it;
   - a `kind = "Status"` record ⇒ the vocabulary detector reports it.

## Acceptance criteria

- **AC-1 — the detectors are pure.** Neither takes its input from
  `load_manifest()`; both are callable on inline fixtures.
- **AC-2 — production behavior is unchanged.** The real gates still run against
  the real manifest and still pass. Today's 14 records (`explanatory`×7,
  `tutorial`×4, `portal`, `reference`, `status`) produce no rejection.
- **AC-3 — ⭐ THE CONTROL IS COMMITTED AND REVERSIBLE, AND THIS IS THE POINT OF
  THE WP.** For **each** of the two detectors, demonstrate the break and
  restore it:
  1. Remove the detector's rule (or rename the detector, leaving the gate's
     call site untouched).
  2. **Build/run. The failure must land at the committed test or at the call
     site** — name the exact test and the exact error.
  3. Restore byte-for-byte and re-verify green.

  ⛔ *"It went red"* is not the claim. *"It went red **at this named
  artifact**"* is. A failure anywhere else means the binding is elsewhere and
  the control is decoration. ⚠ Commit the real work **before** any
  mutation-proof reset, or the reset eats it.
- **AC-4 — no vacuity.** Each new test must fail if its detector's rule is
  removed. State which line you removed to prove it.

## ⚠ The wording half — a Steward defect, recorded so it is not repeated

The commit message on `main` at `64b0811f` claims *"binds coverage on the record
axis, with a positive control that fails when the binding is removed."* **Both
halves are false against the tree**: there was no positive control, and the
mechanism pins **one instance** via one hard-coded path literal rather than
binding an axis. A future registry row whose `applies` predicate selects a wider
set than its runner inspects is exactly as unbound as `generated-current` was.

That text was the **Steward's publish description**, authored *after* the last
review gate, and it is now permanent in the git log. **It is the one artifact in
the WP pipeline that no reviewer reviews.** Landing this WP makes the claim true
rather than leaving a correction to prose; the process hole is recorded
separately in `agent/memory/roles/steward/`.

⇒ When writing this WP's own publish text, **state what the mechanism pins, not
what it evokes.** The honest sentence for the current state is *"pins the one
status-kind record the hard-coded runner can serve."*
