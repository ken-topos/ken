# Hunt the stand-down clause — it lives in prose, where no gate reads

**Lesson (2026-07-22, three instances in one session).** A **stand-down clause**
is a rule whose function is to tell a reader *not to look*, justified by
something with no bearing on whether anything is there. They are the
highest-yield target this seat has, for a structural reason: **they live in
messages, not artifacts.** No CI job reads a kickoff. No gate fails on a
sentence. The committed artifact can be correct while the prose carrying it to
two rings is wrong — which happened three times in one day.

## The three

1. **I proposed one.** After three accepted findings on one clause I asked the
   Steward whether to hard-stop after N and batch the rest. They refused: *"a cap
   keyed on a number is a stand-down clause — a rule whose function is to tell
   you not to look, justified by something with no bearing on whether anything is
   there."* I had written the fleet lesson against that construct **that
   morning** and then proposed it wearing a process hat.
2. **`COORDINATION.md` §14(5)'s "the failure is loud."** True-sounding, and it
   removes the reason the rule says *inspect*. Corrected, and the correction
   carried a fresh overstatement of its own.
3. **"Shard 4/4 red on that test is not your defect — do not chase it."** Issued
   in a kickoff, **pre-armed against a signal that did not yet exist.** The next
   WP necessarily edited a cited source (`px4b_native_production.rs:907`), so the
   gate would go red *for a true reason about the ring's own diff*, on the same
   test and the same shard the notice had already excused.

## The tell

★ **An instruction that pre-classifies a future observation.** *"If you see X, it
isn't real."* That is a prediction about evidence that does not exist yet, issued
by someone who will not be the one looking at it. The hedge that usually
accompanies it — *"any other failure is real"* — does not save it, because the
suppressed case is the one that matches the description exactly.

The direction of the error is always toward **less** looking, and it always
arrives feeling like efficiency: fewer wasted cycles, less noise, don't chase
ghosts. That is why they survive review — they read as considerate.

## How to use it

- **Hunt them before the signal arrives.** Afterwards the suppression has already
  worked and there is nothing to see: the seat did not chase it, so no evidence
  was generated. This is the rare case where speed matters for an advisory seat.
- **Ask whether the excluded class can contain a true positive.** Not *"is this
  instruction reasonable?"* — it always is — but *"what would a real failure
  wearing this description look like, and would anyone notice?"*
- **Check the diff of the WP the instruction governs.** Instance 3 fell out of
  one command: the next WP's target file was itself the cited source. The
  instruction was written before anyone looked at what the next candidate would
  touch.
- **Read the prose beside a correct artifact.** Gates verify artifacts. Nothing
  verifies the message announcing them, and the message is what reaches the
  rings as instruction. The Steward's form: *"check what was broadcast, not only
  what was committed"* — and their closing line, worth keeping verbatim:
  **"A structural guarantee does not protect the prose you write next to it."**
- **Expect to generate them yourself.** I did, six hours before catching two.
  Any rule you are about to propose that reduces looking is in this class until
  you have named the true positive it would suppress.

Related: [[hunt-the-correction-it-inherits-the-defect-class]],
[[an-incident-offered-as-corroboration-must-reproduce-your-mechanism]],
[[the-post-merge-yield-is-vantage-not-seat-quality]].
