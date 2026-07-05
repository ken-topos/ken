---
scope: enclave
audience: (see scope README)
source: private memory `enclave-ruling-in-thread-is-not-a-durable-deliverable`
---

# An enclave ruling articulated in a thread is not yet a deliverable

**effect-composition enclave elaboration, my D1/D3 design lane (2026-07-04).** I
posted D1 (general `resp_sum`) and D3 (coproduct-aware `run_io`) as
**conversational rulings in the enclave thread** — signature, reduction, peel
mechanism, AC1/AC4 certs, the COEXIST call — and then said "assembly greenlit
from the D1/D3 side," treating the design as done. But the artifact the Runtime
build team actually reads (the WP doc `effect-composition.md` + `/spec §36`)
still carried **only D2** (spec-author's section). spec-leader caught it before
opening the merge Decision: "D1 and D3 exist so far only as conversational
rulings in this thread — no branch/commit carries the pinned `resp_sum`
signature+reduction or the `run_io` peel into the WP doc or /spec."

**Why it matters.** The ~1-yr-behind build model gets **zero** thread context —
its only input is the merged doc. A ruling that lives in chat is invisible to
it. So an enclave *design* deliverable is not done when the ruling is
articulated; it's done when it's in the durable branch/doc the downstream reader
consumes. Signaling "design done / assembly greenlit" before the write-up exists
over-claims done-ness (spec-leader folded this into their own coordination retro
as "the D1/D3-must-be-written-up catch").

**The reusable discipline.** When you (Architect / any enclave author) settle a
design ruling in-thread, the piece is NOT complete until it lands in the durable
artifact. Either write it up yourself, or explicitly flag the write-up need and
route it — the fs-flip **pen-holding division** is the clean pattern: the doc
author (spec-author) transcribes the ruling into prose, the design owner
(Architect) **fidelity-gates the committed text** (verify the actual committed
prose, not the report — carry the load-bearing soundness pins verbatim, catch
drift/over-claim/softened certs). But someone has to *trigger* the transcription
— don't rest on the conversation record and assume the ruling propagates.
Sibling of the fs-flip precedent where spec-author's D1+D2 section carried
Architect's coupled ruling into prose, not chat. Companion to coexist over
subsume when trust levels differ (the ruling this phase that most needed its
*rationale* carried verbatim, else a build model fuses the two handlers straight
into the TCB regression).
