---
scope: roles/steward
audience: (see scope README)
source: 2026-08-05, caught on RT-CONTSRC-CALLABLE-CONTRACT hours after closing
  RT-UNIT-CLOSURE-CONVERT with a clean edge check
---

# Closing a node strands sequencing prose in OTHER frames, and the edge check cannot see it

When closing `RT-UNIT-CLOSURE-CONVERT` I verified `blocks: []`, grepped every
node's `depends_on` for its id, found nothing, and wrote in the node itself:
*"Nothing depends on this node, so closing it strands no work."*

**That was false when I wrote it.** `RT-CONTSRC-CALLABLE-CONTRACT` — a `ready`
node at the frontier — carried in its frame's section 6:

> it must not be run concurrently with `RT-UNIT-CLOSURE-CONVERT` ... Sequence
> them; **the unit-closure node goes first** because it gates a candidate and
> this one does not.

Read literally, that holds the node **forever**: its predecessor can never run.
And section 0 separately routed *"a reader looking for the next kickoff"* to the
same closed node.

**Why the edge check cannot catch this.** `depends_on` and `blocks` are
**frontmatter in `docs/program/issues/`**. Sequencing and contention also get
written as **prose in `docs/program/wp/` frames**, where they are equally
binding on a reader and invisible to every generator. `gen-progress.sh` reads
the edges; nobody reads the prose but the agent about to start the node.

⇒ **A clean `depends_on`/`blocks` sweep is evidence about the graph, not about
the corpus.** The two disagree precisely because contention clauses are the kind
of constraint an author states in a sentence rather than an edge — often because
it is a *negative* ("must not run concurrently with"), which the schema has no
field for.

**Worse than stale: unsatisfiable.** A sequencing constraint whose predecessor
is `closed` is not a weaker constraint that degrades gracefully. It cannot be
discharged by waiting, and it reads as live law to whoever picks the node up.

**How to apply — closing a node is a two-part sweep, and the second part is the
one you will skip:**

```sh
# 1. the graph (what I did, necessary and NOT sufficient)
grep -l "depends_on:.*<ID>" docs/program/issues/*.md

# 2. the corpus (what actually strands)
grep -rn "<ID>" docs/ --include=*.md | grep -v "^docs/program/issues/<ID>.md"
```

Then, for every hit outside the closed node's own files, decide **per hit**:

- a **sequencing or contention clause** — delete it, do not annotate it. An
  annotated unsatisfiable constraint is still read as a constraint by someone
  skimming for their next action.
- a **routing pointer** ("a reader wanting the next kickoff wants X") —
  repoint it at live nodes, and say the old pointer is dead so it is not
  restored from an older revision.
- a **historical citation** ("the same reasoning that closed X") — leave it.
  Those are the majority and they are fine; the closed node's own frame is
  allowed to be history.

**The tell that you are in this failure mode:** you are writing "nothing
depends on this node" into the node you are closing. That sentence is a claim
about the whole corpus, and you will have checked only the frontmatter.

Sibling of
[[an-obligation-can-outlive-its-mechanism-and-the-self-dated-as-of-is-the-tell]]
and [[a-retired-rule-survives-in-the-boilerplate-that-gets-copied-into-new-artifacts]]
— in all three the mechanism is gone and its text keeps issuing instructions.
