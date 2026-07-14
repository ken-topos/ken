---
scope: fleet
audience: (see scope README) — anyone who amends a WP frame after the substrate
  moves under it; anyone READING a frame whose §"current state" may be stale
source: CC8 frame, 2026-07-14 — foundation-qa caught it at kickoff; the Steward
  wrote the frame, wrote the amendment, and still left two sections contradicting it
---

# Amending a frame mid-flight? Sweep the GUARDRAILS section — it is the one you forget, and it reads as law

**CC8 was held for hours while the byte substrate landed underneath it.** When
SUB-1b (lawful `DecEq Bytes`) went in flight, the Steward **amended §3.2** of the
CC8 frame — *"an environment key is a plain `Bytes`, compared with `DecEq
Bytes`"* — and kicked the WP.

**Two other sections still said the opposite.** foundation-qa found one at
preflight; sweeping the doc found a second the QA had missed:

| § | Stale text | Reality at kickoff |
|---|---|---|
| **§6 Guardrails** | *"⛔ Do NOT settle `Bytes → Nat`. **No `DecEq Bytes`**… key matching goes through `ArgBytes` + `bytes_at` + `uint8_to_int` + `eq_int`."* | Pat **ruled** it; SUB-1b **landed** `DecEq Bytes`; SUB-2 **deleted** `ArgBytes`. **Every clause false.** |
| **§3.2 item 2** | *"Where CC3's `Cursor` ABI already demands `ArgBytes`, pass it — consume it as-is."* | **`ArgBytes` does not exist.** SUB-2 deleted the class, its cached fields, and its proof obligation outright. **There is nothing to pass.** |

## ★ Why the guardrails section is the dangerous one

**A frame's §"what to build" gets re-read and re-reasoned. Its §"do not do X" gets
OBEYED.** A guardrail is phrased as a prohibition — imperative, absolute, no
argument invited — so a build agent **complies without re-deriving it**. That is
exactly what makes it useful, and exactly what makes a *stale* one poison:

**⇒ A stale guardrail does not merely fail to help. It actively forbids the
correct implementation, in the frame author's own voice, with the frame author's
authority.** CC8's §6 would have sent Foundation to `ArgBytes` — **a class that no
longer exists** — while forbidding the `DecEq Bytes` the WP was un-held *in order
to use*.

## The rule

**When the substrate moves under a held WP, do not patch the section you were
thinking about. Grep the WHOLE frame for every name the change touched** — here:
`DecEq Bytes`, `ArgBytes`, `bytes_at`, `cached-Nat`, `Int → Nat` — **and
adjudicate every hit.** The section you amended is the one you had in mind; the
ones that bite are the ones you didn't.

Same instrument as [[correcting-scope-must-sweep-whole-doc]], but with a sharper
target: **the guardrails/do-not-reopen section is the highest-authority, least-
re-examined prose in the document.** Sweep it first, not last.

## And the corollary for the READER

**foundation-qa caught this at preflight by comparing the frame against the tree,
before writing a line.** That is the whole discipline:

> **Frame text is an INPUT TO AUDIT, not scripture.**

Every frame should carry the perishability clause (*"treat every anchor as
perishable; if a fixed input is FALSE, say so with exact tree anchors and
ESCALATE — do not build around it"*) — **and CC8's did.** It worked. The clause
caught the frame author's own error. **Keep writing it, and keep meaning it.**
Sibling of [[wp-frame-stale-vs-landed-kernel]] and
[[frame-pseudocode-diverges-from-landed-mechanism]].

## Mechanics footnote

If the WP branch is **already checked out in the implementer's worktree**, do
**not** commit the amendment over it from a second worktree — that desyncs them
([[plumbing-commit-onto-held-branch-and-its-desync-risk]]). **Hand the implementer
the verbatim replacement text and have it ride their commit.** Frame authority
stays with the author; the mechanical edit rides the branch holder.
