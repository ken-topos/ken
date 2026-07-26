---
scope: fleet
audience: (see scope README) — anyone removing a gate, test, oracle, or checker, and
  anyone reviewing such a removal (QA, Architect, the Steward at the merge gate)
source: 2026-07-26, two instances in one day. (1) The operator removed the library
  currency gate from the publisher; the check went, its success sentence stayed and
  kept printing "the currency checker is green on origin/main" — including on the
  very merge that broke the ledger. (2) LIB-GATE-DECOUPLE deleted five live-document
  tests and disclosed the residual as "citations may drift"; the Architect held
  terminal approval because that was true and covered a fraction of what was retired.
---

# Deleting a check has a text surface, and the text outlives the check

⛔ **A check is not only code. It is also every sentence that tells a reader what
assurance they have. Delete the code and leave the sentences, and you have not
removed a gate — you have converted it into a false claim that no test can fail.**

Both failures below are the same edit boundary drawn too narrowly. Neither was
caught by a test, because in both cases the surviving text was **unexecutable**.

## Instance 1 — the success message outlived its check

The publisher's currency gate was removed at `scripts/scripted-pr-automerge.sh:531`
by operator ruling. Its **success sentence** at `:670` was not:

> *"Post-merge verification: landed tree `X` matches the checked tree, and the
> currency checker is green on `origin/main`."*

⇒ It printed that on **PR #1031 — the merge that broke the ledger — and again on
#1034.** ⭐ The one reader most likely to trust it is the next Steward deciding
whether `main` is healthy, and the sentence sits exactly where that decision is
made.

⛔ **A message is part of a gate's surface.** Remove the claim in the same edit as
the check, or the removal ships a lie with a timestamp on it.

## Instance 2 — the residual was true and underreported

`LIB-GATE-DECOUPLE` deleted five tests whose verdict depended on live documentation
content. The handoff's stated residual: *"a `library/` page may cite a source that
has since changed and nothing will report it."*

**Accurate. And a fraction of the truth.** The Architect enumerated what else those
five entrypoints were the only consumers of: 11 validation routes over the live
library corpus, the live agent manifest/schema/module contracts, `measured_tokens`,
manifest↔pack parity, the pack graph, evaluation task/pack/fixture/result closure,
manifest validation-token agreement, and the real manifest's line layout.

⭐ **The tell: I wrote the residual describing THE BUG I WAS FIXING, not the
CAPABILITY I WAS DELETING.** Citation drift was the symptom that made `main` red,
so it was what I had in mind. The deletion's actual scope was everything those
entrypoints reached.

⇒ ⛔ **When you remove a checker, enumerate what it CHECKED — not what you were
fixing.** The two coincide only by accident.

### ⭐ And split RETIRED from DEFERRED, because they do not cost the same

Re-measuring turned one alarming residual into two honest ones:

| assurance | disposition |
|---|---|
| source currency, generated currency, cited-source attestations | **DEFERRED** — the generator scripts are kept and unchanged; they run at release points. What changed is *when*, not *whether*. |
| live agent manifest/schema/module/measurement/pack checks, evaluation-corpus closure, manifest token + layout, 9 of 11 validation routes | **RETIRED** — zero remaining automated consumer, measured |

⛔ **Reporting the union as "retired" overstates the loss; reporting it as
"deferred" understates it.** A reviewer can only weigh the removal if the two are
separated, and separating them takes one grep per item.

## How to apply

- **Grep for the claim before you delete the code.** The distinctive phrasing of
  what the check *promised* — in output strings, comments, READMEs, frame ACs,
  runbooks. ⛔ A removal whose diff touches only the check is almost always
  incomplete.
- **Write the residual by enumerating the deleted thing's reach**, then for each
  item ask: *is there another consumer?* Zero consumers ⇒ retired. A kept script or
  release process ⇒ deferred; **name it**, so the reader knows where the assurance
  went.
- ⚠ **Confirm each "no other consumer" hit is the intended SENSE of the word, not
  just the token.** Measured here: grepping `evaluation` matched the publisher's
  *base-advance evaluation window* and nearly produced two false consumers of
  `library/agents/evaluations/`. **A consumer census keyed on a homonym reports
  coverage that does not exist.**
- ⭐ **State whether machinery survives the entrypoints.** `LIB-GATE-DECOUPLE`
  deleted five `#[test]` functions and left `VALIDATION_GATES` and its 11 check
  functions in place as dead code. That is defensible — but it is a **re-arm door**,
  and a reader who is told "the coupling is removed" will not guess the mechanism is
  still sitting there.

## Positioning

- ⛔ **The removal moved the gate rather than deleting it, and that has its own
  cost.** The same currency check also ran as a CI test, so removing it from the
  publisher moved the firing from *before* the merge to *after* — where it lands on
  the next, innocent PR and reads as that PR's own failure. **Ask where else the
  mechanism fires before calling it removed.**
- [[a-mechanism-claim-in-a-comment-is-structurally-exempt-from-execution]] — why the
  surviving text cannot be caught by a test: it is in a position nothing executes.
- [[a-later-note-saying-a-deliverable-is-false-does-not-replace-the-deliverable]] —
  the sibling: there the stale text is a superseded deliverable, here it is a
  superseded *guarantee*.
- [[a-deferral-is-honest-a-deferral-that-reads-as-delivery-is-not]] — the
  retired-vs-deferred split is that rule applied to assurance.
