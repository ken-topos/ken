---
id: KW-ORACLE-CLOSURE
title: "close the KW-THEOREM source oracle structurally — the occurrence sweep is never applied, and the file population is a five-arm hand enumeration"
status: merged
owner: language
size: S
gate: none
depends_on: [KW-THEOREM]
blocks: []
github: 986
origin: Adversary post-merge hunt on landed `origin/main` = `c72be0b0`, evt_4q06tgtrw6bv (thr_2seh2bm1kr5mh). Two findings, both in `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs`, **both with zero live instances** — the adversary ran each missing check itself against the full landed population and the corpus is clean. Filed anyway because each is one call site / one predicate wide, and because `RT-FNSPLIT-B2F` is not the only consumer that will lean on this oracle. Steward triage 2026-07-25.
---

> ## ✅ MERGED — PR #986, `origin/main` = `9b6d4d16`, landed tree `50c485ce`
>
> ⚠ **The title and §P1 below are in the PRESENT TENSE and describe the DEFECT, not
> current state.** Both are now historical: the occurrence sweep **is** applied to
> the corpus, and the population **is** derived structurally. Verified by blob
> identity — `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs` =
> `c2119e62` on `main`, identical to candidate `79acbabb`.
>
> ⭐ **It took two candidates, and the reason is the reusable part.** `980bb047`
> was **BLOCKED by QA**: `AC-C1`'s required **population-side** mutation (one line
> of ordinary prose outside a `ken` fence in a real corpus file) returned **exit 0
> / 1 passed**. The build had run a **detector-side** mutation instead (*"head-only
> occurrence scan"*), which reddened — so its report that *"each control reddened
> its intended named test"* was **literally true** while the exact defect this WP
> exists to close sat under a green control. `79acbabb` discharges it
> population-side: **exit 101** at
> `exact_candidate_has_no_unclassified_retired_occurrences`.
>
> ⇒ Promoted to `agent/playbooks/tools/pin-a-property.md §10`: **detector-side and
> population-side mutations are not interchangeable controls.** ⛔ Deliberately
> distinguished from the existing *"edited the detector along with its subject"*
> row — that is mutating **both**; this is mutating the detector **instead of** the
> population, and it reports clean.
>
> **Frame:** `docs/program/wp/KW-ORACLE-CLOSURE-structural-population.md` — the
> re-derived anchors, the AC→control map, and ⭐ the measured design fact that made
> P2 small: `candidate_inputs()` **already** enumerated the whole tree via
> `git ls-tree -r`, so the narrow part was `classify`, not the enumeration.
>
> ### ✅ RETROS IN — and they REFUTED the repair I would have made
>
> Leader `evt_6nh73m6j0zkwd` · implementer `evt_5xqacdzfjmkh2` · QA
> `evt_45b3h0xmpw9gw`. **This node is CLOSED.**
>
> I asked each seat one question I could not answer myself, and all three
> answered without softening:
>
> - **QA — was `AC-C1` ambiguous about the operand, or clear and skipped?**
>   ⭐ **"Clear and skipped."** The row *did* name the corpus-side operand. ⛔ So
>   the repair is **not** an AC-authoring fix — reaching for one would have
>   hardened prose that was already correct, which is exactly why the question
>   had to be asked rather than guessed.
> - **Implementer — what made the cheaper operand look right?** ⭐ The seam in
>   front of them was a `declaration_lines` helper; mutating it was **cheap,
>   isolated, compile-preserving, and it reddened the correctly named test.**
>   They *"varied the operand named by the code seam rather than the operand
>   named by the reach claim."* **The code seam supplies a default operand, and
>   it is not the AC's.**
> - **Leader — what was visible at your seat?** ⭐ **"Nothing."** Branch, tree,
>   scope, hygiene and a reddening named test **do not say which operand the AC
>   requires to move.** ⛔ That rules out a leader-review step here.
>
> ⇒ Folded back into `pin-a-property.md §10` as a **correction**: naming the
> operand in the row is **necessary and measured not sufficient**, and the
> load-bearing obligation is a **reported field** — every AC→control handoff
> carries *(property · operand that moved · observed boundary)* stated, not
> inferable, because that is the only one of the three that changes what a
> reviewer can **see**.

> ## ⚠ THERE IS NOTHING HIDING TODAY — THAT IS NOT THE POINT
>
> The adversary **measured both gaps closed**: applying the uncalled occurrence
> predicate to the same scanned lines finds **the identical single hit** the
> declaration sweep finds — the enumerated `AC-2(d)` control at
> `conformance/surface/declarations/seed-named-proof-claims.md:43` — and every
> unclassified `ken`-fence line under
> `library/`/`catalog/` — 37 of them — is clean.
>
> ⛔ **Do not close this WP by re-running those measurements.** A measurement that
> the population is currently empty is not the deliverable; the deliverable is
> that the instrument can *see* the population at all. Both findings are about
> **an oracle whose reach is narrower than the property it advertises** — and a
> clean corpus today is exactly the condition under which that goes unnoticed.

## P1 — the occurrence sweep is never applied to a corpus file

`retired_occurrence_offsets` (`kw_theorem_source_oracle.rs:189-195`) has exactly
one caller: **its own self-test at `:278`.** It never sees a corpus file.

What actually runs against the corpus is `is_retired_declaration` (`:166-173`),
which matches a line only when its head word is `lemma` or `pub lemma`. That
answers *"is this line a retired declaration head?"* — but `AC-1` asks for
`lemma` / `lemmas` / possessive and plural forms **plus surface-derived
identifiers and anchors**, and the head matcher can express **none** of those.
The instrument that can express them is the uncalled one.

⭐ **The self-test is the sharpest part of the finding, not a mitigation.** At
`:274-282` it probes `"LEMMA lemmas lemma's lemma_identifier"` and asserts all
four are seen — case, plural, possessive, derived. **A rigorous test of an
uncalled function is the most convincing possible form of no coverage**, because
the rigor is real and it is aimed at the wrong question. A reviewer auditing this
file sees a careful occurrence test and concludes occurrences are covered.

**Deliverable:** the occurrence predicate runs against the corpus population, and
a mutation that widens a corpus file's occurrence set beyond a declaration head
**reddens**. Cost to close is a call site plus the control that proves the call
site exists.

## P2 — the file population is a five-arm hand enumeration

`classify` (`:55-84`) admits `.ken.md`, `.ken`, `conformance/**/seed-*.md`,
`library/agents/evaluations/results-*.toml`, and one fixture path. **Ken code
inside an ordinary `.md` is outside the population.** Of the files the
`KW-THEOREM` merge itself changed, **10 carry `ken`-family fences and are
unclassified**, three of them in the adversary's own lane:
`library/agents/core/write-ken.md` (2 fences),
`library/learn/reading-ken/02-types-contracts-and-proofs.md` (2),
`library/learn/reading-ken/03-assurance-and-trust.md` (1).

⛔ **The non-vacuity assertion cannot notice this.** `:103-107` requires that the
exact candidate populate every structural source/oracle **class**, and it is
discharged by the five classes being non-empty. It says nothing about *which
files reach them*. **That is the same shape as the finding it exists to guard
against, one level up.**

⛔ **`AC-1` was already amended once for exactly this**, because the
hand-enumerated root list *"was wrong three ways"*; that amendment warns that
**"a sweep that grew one arm per missed file has reproduced the bug it exists to
prevent."** `classify` is a five-arm hand enumeration of file kinds, and the
population question survived into it.

**Deliverable: a STRUCTURAL closure, not a sixth arm.** The population must be
derived (every file the tree contains that carries a `ken`-family fence is in
scope, with exclusions stated as a closed complement) rather than enumerated.
⛔ A fix that adds `.md` to the arm list has reproduced the bug a third time.

## What the adversary attacked and could NOT break — do not re-litigate

- ⭐ **`library/SOURCE-ATTESTATIONS` is fully current, 50/50.** Every recorded
  blob OID re-derived against `c72be0b0:<path>`; zero stale, zero missing. The
  re-canonicalization moved four attested catalog blobs and the fold landed in
  the same candidate.
- **No stale line citation.** The four files whose line counts shifted (+2 each)
  are cited by none of the 11 line/fragment citations; the three cited by line
  have unchanged line counts.
- **The fence-info list is complete** — exactly `ken`, `ken ignore`, `ken reject`,
  `ken example` across the whole tree, no other `ken`-family spelling. (Note
  `info` is only `trim_end`ed, so a leading-space ` ```ken` *would* escape;
  there are none today. Worth folding into P2's structural closure.)
- **Re-canonicalization moved no signature content** — only re-wrapping, in 5
  files.

## The `provide_lemma` row — ruled, and independently flagged

`SuggestedAction::ProvideLemma` / `"kind": "provide_lemma"` remains on the
protocol wire (`crates/ken-elaborator/src/diagnostics.rs:184`,
`protocol.rs:145-148`). The Steward reported this to the operator as a
deliberate residue under the operator's 2026-07-24 `lemma` directive
(*"retired entirely from the language; may remain in comments or
documentation"*), reading it as an **API token, not a language construct**, and
recommended leaving it.

⚠ **The adversary reached the same boundary independently and declined to rule
on it**, noting that `provide_lemma` is an **identifier on the wire, not prose**,
while `AC-1`'s occurrence set explicitly includes *"surface-derived identifiers
and anchors"* — so this is the one in-scope row where the cell is genuinely
ambiguous. That is corroboration rather than agreement: the adversary measured
the frame's routing table itself (*"English word (prose about a helper result)"*
→ leave) rather than inheriting the Steward's reading, and it landed on the
narrower, sharper statement.

⛔ **This WP does not change `provide_lemma`.** It is recorded here so the
ambiguity is review-visible, per the `KW-THEOREM` frame's requirement that
leave-decisions be as review-visible as change-decisions. **Only the operator
reopens it.**

## Scope note the adversary stated, and it binds this WP too

The hunt covered `crates/`, `catalog/`, `library/` only — **not** `spec/`,
`conformance/`, `docs/program/`, `tooling/`, or `agent/`. P2's structural closure
must therefore **derive** its population over the whole tree rather than inherit
the adversary's scanned scope, or it will freeze that scope in as the answer.
