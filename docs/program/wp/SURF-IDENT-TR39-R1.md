# SURF-IDENT-TR39 R1 — make the identifier confusable gate real, or its absence deliberate

**Node:** [`SURF-IDENT-TR39`](../issues/SURF-IDENT-TR39.md) · **Owner:** Ergo ·
**Size:** S–M · **Gate:** none

**Fixed inputs, measured at `origin/main = 78f1f74b`. ⛔ These are current-state
claims and they perish — re-derive at point of use.**

| input | pin |
|---|---|
| the rule as built | `crates/ken-elaborator/src/lexer.rs:148–150` and `:420` — identifiers are **ASCII only** |
| the spec's identifier | `spec/30-surface/31-lexical.md §2` — `[a-z_][A-Za-z0-9_']*` **plus blessed Unicode letters** |
| the security clause | `31 §1a` principle 5 — the lexer *"normalizes/rejects Unicode confusables (the TR39 security profile)"*; restated `§1c` as *"a hard lexer gate"* |
| the non-repair clause | `31 §1d` — *"rejects unblessed confusable identifier characters rather than repairing them into a different binding"* |
| ⛔ the false green | `crates/ken-elaborator/tests/surface_unicode.rs:52` — green, named for the gate, passing because of an ASCII wall |
| the named confusables | `§1a-5`: `⊔`/`U`, `∨`/`v`, `×`/`x`, `ℓ`/`l`, Cyrillic look-alikes |
| already built — ⛔ do not touch | the formatter half of `L-fmt`: `ken fmt`, `lossless.rs`, `layout.rs`, `literate.rs`, `§1d` token-kind canonicalization |

## 1. What this WP is

**Close the gap between a security property the spec asserts three times and a
lexer that does not implement it — in the direction that does not open a hole.**

Pick **Shape A** or **Shape B** (§3). Both are complete outcomes. ⛔ What is not
acceptable is leaving the corpus in its current state, where the capability is
absent, the security property is vacuous, and a green test makes both look
handled.

⚠ **Read `SURF-IDENT-TR39 §3` before writing code.** The naive completeness fix
— "the spec says blessed Unicode letters, so admit them" — is the single action
that converts today's accidental safety into a real homoglyph hole. **Gate
first, admit second, or do not admit.**

## 2. ⭐⭐ The controls problem, stated first because it is the hard part

Today's test asserts three rejections and is green. **All three are rejected by
the same `is_ascii_alphabetic` short-circuit**, so they are one control wearing
three hats, and that control cannot tell an ASCII wall from a TR39 gate.

⇒ **A suite of rejections passes for any reason, including the reason that the
feature was never built.** Whatever shape you choose, the deliverable is only
believable if a control **distinguishes your mechanism from the ASCII wall that
is already there.** That is `AC-T3` and it is the AC this WP turns on.

## 3. The two shapes

### Shape A — build the gate, then admit (spec-complete)

1. A **bounded blessed identifier-character table** — a fixed list, not a
   property test like "any Unicode letter" and not `char::is_alphabetic`.
   `§1a-5` says *"a fixed table, not 'any Unicode'"*; honor that literally.
2. A **specific rejection error** naming the confusable reason, distinct from
   the generic lex error, carrying a span.
3. **Only then**, `§2`'s blessed-Unicode-letter admission.

### Shape B — narrow deliberately (smaller, and honest)

1. Identifiers stay **ASCII-only**, but by decision: a **specific** error that
   names the rule, replacing the incidental fall-through.
2. `§2`'s blessed-letter clause recorded as an **explicit, cited, unimplemented
   completeness gap**, proposed as a document and routed to the Spec enclave.
   ⛔ **Ergo does not edit `spec/`.**

⭐ **Shape B is acceptable for the same reason Language's Shape B was acceptable
on `EFF-SPACE-ENSURES-PRESTATE` (merged 2026-07-27): it leaves the feature
unavailable and says so.** The status quo leaves it *look* available and is
silently wrong. If Shape A's blessed table cannot be settled from `§1b` without
a design call, **take Shape B and say why** — do not invent the table.

## 4. Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-T1` | The chosen shape is implemented in `lexer.rs`. | Name the shape in the commit message and the report. ⛔ A candidate that implements neither, or implements A's admission without A's gate, is rejected on sight |
| `AC-T2` | ⭐ **The existing false green states only what it observes.** | ⛔ **Do not delete `surf1_d3_rejects_unbounded_unicode_identifiers`** — it does catch the naive any-Unicode widening and that is worth keeping. **Rename/re-document it** so it claims the ASCII-boundary property it actually tests, not the confusable gate. This is the same repair Language made to its false-green effects test |
| `AC-T3` | ⭐⭐ **A control distinguishes your mechanism from the ASCII wall.** | **This is the load-bearing AC.** ⛔ A test that only asserts rejections is satisfied by the wall that already exists and proves nothing about your change. <br>**Shape A:** exhibit a **blessed** Unicode identifier character that is **ACCEPTED** *and* a confusable that is **REJECTED**, and show the rejection names the confusable reason. The accept half is what makes the reject half mean something. <br>**Shape B:** show the rejection error for Cyrillic `а` is your **specific** error, and that a **non-confusable** non-ASCII letter (e.g. `字`) yields the **same** error — proving the rule is "ASCII-only by decision," not "TR39 by implication" |
| `AC-T4` | The `ℓ`/`l` axis is addressed or explicitly excluded. | `§1a-5` and `§1d` both single out blessed **operator** glyphs colliding with ASCII **identifier** characters (`⊔`/`U`, `∨`/`v`, `×`/`x`, `ℓ`/`l`). That is a different axis from a Cyrillic homoglyph. Either cover it with a control or state in the report that it is out of your shape's reach and why. ⛔ Silence on it is not acceptable |
| `AC-T5` | A mutation proves each new control is causal. | For every control you add, apply one compile-preserving mutation at the natural detector site, show it reddens **that named test**, restore byte-identically (`git diff --exit-code`). ⛔ An unexpectedly wide redden usually means the build broke — report the test names, not just "it failed" |
| `AC-T6` | **Zero spec, zero conformance, zero kernel, zero trusted-base delta.** | `conformance/surface/lexical/` does not exist; ⭐ **creating it is OUT of scope for this WP** — note its absence in the report and I will route it. If you find yourself editing `spec/`, stop: that is the enclave's, and Shape B routes a proposal instead |

## 5. Scope

**IN:** `crates/ken-elaborator/src/lexer.rs` and its tests.

⛔ **OUT:** the formatter (built and landed) · `spec/` edits · `conformance/`
seeding · `STR-NFC-CONSTRUCTION`, which is about `String` **values** and is live
in the enclave — same word, different concept, do not merge them · widening
`§1b`'s blessed operator table.

## 6. Contention check

**Measured at `78f1f74b`.** This WP touches `lexer.rs` and `surface_unicode.rs`.
Live `ken-elaborator` work at measurement time: `EFF-SPACE-ENSURES-PRESTATE`
(`elab.rs`, `resolve.rs`, effects/V1/formatter/LET tests) and `V4-RESIDUAL`
(`diagnostics.rs`, `v4_acceptance.rs`). **Intersection with both is empty.**

⚠ **An empty path intersection is not by itself a licence to publish** —
re-derive it against current `origin/main` at handoff, and expect the shared
build slot to be held by another ring. Ask for it; do not take it.

## 7. Validation

⛔ **Targeted only — never `--workspace`.** `scripts/ken-cargo test -p
ken-elaborator --test surface_unicode`, plus any suite your change can reach.
Workspace-green is **CI's** job, not this box's.

## 8. Reporting

Return: the shape chosen and why · the exact SHA and tree · `AC-T3`'s control
**with its actual output**, since that is the one that makes the rest mean
something · the `AC-T5` mutations and their restore proof · your `AC-T4`
position on the `ℓ`/`l` axis · and the measured-vs-claimed boundary, naming
plainly what remains unimplemented.
