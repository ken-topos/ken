# Audit the OPERATIVE line, not the stated principle

**Lesson (build-QA playbook review, 2026-07-22).** Asked to attack a draft
mutation-proofing section, I found the load-bearing gap not in the new text but
in the **contradiction between what the document asserts and what it instructs.**

`agent/playbooks/build/qa.md` establishes independence twice in its framing:

```
:3   "Build-team QA. Independent verification gate against /spec …"
:10  "You are the independent verification gate for your team's work."
```

And then the line a QA actually *executes* says:

```
:18  "Your independent gate RE-RUNS the affected tests"
```

★ **Re-running is not re-deriving.** A QA that re-runs the implementer's
mutation inherits the implementer's vantage — including the forms they never
imagined, which for a text-matching mechanism **is the entire failure surface.**
Nothing in the document reconciled the two, and the new section — four
sub-blocks on how to build a *good* mutation — never said the QA must build its
**own**.

## ⛔ CORRECTION — my evidence was wrong; the finding was right for a different reason

**Checked after it landed.** `:18`'s "re-runs" sits inside the **targeted-testing**
rule — *"never run `cargo test --workspace`… re-runs the **affected** tests
through `scripts/ken-cargo -p <crate>`"*. Its subject is **test scope**, not
derivation. And it is the **only** other `re-run` in the file.

⇒ **There was no contradiction.** The playbook was **SILENT** on constructing
your own mutation. I read two senses of one word as a conflict:

| sense | where | what it means |
|---|---|---|
| run the affected subset, not `--workspace` | `:18` | scope of execution |
| re-execute someone else's mutation | my finding | provenance of the probe |

**Same word, two concepts** — the trap I already had indexed, walked into while
writing a lesson about reading documents carefully. Worse, the sub-block that
landed now cites `:18` in my framing, so the corpus carries my error.

**What survives, and it is the more useful shape:** the gap was an **ABSENCE**,
which is *harder* to find than a contradiction and much more common. A
contradiction has two lines you can put side by side; an absence has none, so no
amount of reading the document finds it — you find it only by asking **"what must
the reader DO, and is that anywhere?"** and coming up empty.

⇒ **When you cite a line as evidence, re-read it in its own context, not in the
one your finding is about.** A grep hit is a token, not a meaning.

## Why it hid

The principle is stated in the **role framing**, which reads as settled and gets
skimmed. The contradiction lives in the **procedure**, which reads as detail. A
reviewer checking "does this doc value independence?" finds `:3` and `:10` and
stops — the answer is yes, emphatically, twice.

⇒ **The stated principle is not evidence that the instructions implement it.**
Those are two different claims and the document was only audited for the first.

**And the failure leaves no trace:** a mutation proof inherited wholesale is
**indistinguishable in the verdict** from one derived independently. Nothing in
the output records which happened.

## How to use it

- **When auditing a process doc, find the line the reader will actually
  execute** — the imperative with a verb in it — and check it against the
  principle the doc claims. Grep for the verbs (`re-run`, `check`, `confirm`,
  `verify`), not the nouns (`independent`, `rigorous`, `exhaustive`).
- **Where a doc asserts a property of the reader's *process*, ask what artifact
  would differ if the reader ignored it.** If nothing would, the assertion is
  decorative and needs a reporting clause, not stronger wording.
- **Prefer a disclosure requirement to a prohibition.** The fix here was not
  *"always construct your own"* — sometimes you legitimately cannot — but **"if
  you can only re-run theirs, say so in the verdict."** An inherited mutation is
  not a defect; an inherited mutation **reported as independent** is. That
  converts an invisible degradation into a visible one, which is the same move
  as preferring a post-condition to a mechanism story.
## ⛔ Two ways this move fails — do not hand it over without these

Written the same day, after the Steward put the verb query across the whole
`agent/playbooks/` corpus. **The move above is one cell of a two-cell test**, and
handing it over alone reproduces exactly the defect it finds.

1. **A STRONG verb can be decorative too.** `QA must independently construct its
   own mutation` is perfectly worded and **exactly as dead as `re-runs`** if
   nothing records whether they did. A verb-strength query returns the
   *mismatches* and silently passes every strong-but-unverifiable instruction —
   which is the **larger** population — while returning a short, plausible,
   ranked list that reads as a complete sweep. ⇒ Always run the completing
   query: **what artifact would differ if the reader silently ignored this
   line?** No verdict field, no commit message, no logged value, no CI check ⇒
   decorative, whatever its verb.

2. ⛔ **The tempting fix is a verb rewrite, and a verb rewrite changes nothing.**
   Turning `re-runs` into `independently derives` across forty files produces a
   large satisfying diff, ships nothing, and — the part that makes it worse than
   inaction — **immunizes the corpus against the next audit**: nouns and verbs
   now agree everywhere, so the query goes quiet on a corpus that did not
   improve. **Verb rewriting is documentation's version of "be more careful"** —
   same appeal, same zero yield, and the same tell, that it feels like progress
   *because* it touches everything.

   ★ Check what actually fixed the instance: **the verb in `qa.md` is still
   `construct`.** What changed an artifact was the **disclosure clause** — *"if
   you can only re-run theirs, say so in the verdict."* The fix was never the
   sentence; it was making compliance **observable somewhere**.

⇒ So for each hit, never ask *what should this sentence say.* Ask **what would
have to appear in some output for compliance to be observable** — and where the
honest answer is "nothing plausible," **delete the line rather than sharpen
it.** A decorative instruction is not neutral: it spends the reader's attention,
buys nothing, and makes the document read as covering something it does not.

- **Corollary for describing your own work:** the Steward described this very
  artifact from *intent* rather than by reading it back, and told me a credit was
  in the file when it existed only in the message. Re-read the diff before
  writing the summary of it. See
  [[an-incident-offered-as-corroboration-must-reproduce-your-mechanism]].

Related: [[hunt-the-stand-down-clause-it-lives-in-prose-no-gate-reads]] (the
other prose-resident defect class),
[[a-clean-parallel-result-must-be-withheld-until-the-other-seat-reports]]
(why inherited premises destroy corroboration).
