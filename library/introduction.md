# Introduction

Ken is a **software-engineering language**: it is written by agents and
read by humans, and its job is to prove what can be proven and state
honestly what must be tested, legibly for a sufficiently-educated human
(`docs/PRINCIPLES.md` §1). That asymmetry — agents write, humans read — is
the axis every other design decision in Ken hangs from, this documentation
included.

## Who this is for

Ken's primary reader is not a novice meeting a new syntax for the first
time. It is a **human reviewer** deciding whether to trust a program an
agent wrote: what does it claim, what actually backs the claim, and what
does it still leave unproven? This library teaches that reading task before
it teaches authoring, because reviewing agent-written software is the
harder and more frequent job — and it is the job Ken exists to make
possible.

## What "verified" means here

Most correctness assertions today stop at tests (empirical, sampled) or
static typing (rules out whole classes of error, but says nothing about
whether a function computes the *intended* result). Ken's thesis is that a
third level — a machine-checked proof that code satisfies a stated
specification — is reachable with the ergonomics of ordinary static typing,
not just by proof engineers in an interactive assistant
(`spec/00-overview.md` §1). **"Verified," in Ken, means exactly that
machine-checked level** — a passing test suite is not the claim.

## The assurance thesis

Ken's kernel is the **only** trust root. It is small, permanent, and
auditable by construction — the de Bruijn criterion — so trusting a Ken
proof never means trusting the much larger elaborator, prover, or standard
library that produced it (`docs/PRINCIPLES.md` §5). The kernel's audited
boundary has exactly three parts: the kernel itself, the primitive
declarations, and the postulates an artifact explicitly assumes — nothing
else is trusted (`spec/60-security/64-trust-model.md` §1).

Every claim a Ken program makes carries one of four honest, exported
statuses: **proved**, **tested**, **delegated**, or **unknown**
(`spec/20-verification/21-spec-syntax.md` §5). A `tested` or `delegated`
claim is never silently promoted to `proved`. Where Ken cannot prove
something, saying so plainly is a design value, not an admission of failure
(`docs/PRINCIPLES.md` §8) — and it is the property this library is bound to
as well: a page that states a language rule on its own authority is a
defect regardless of whether the rule happens to be correct.

## Explicit non-goals

Ken is not:

- a general-purpose scripting language optimized for terse, throwaway code;
- a proof assistant whose primary audience is proof engineers rather than
  software reviewers;
- a restatement of everything its specification covers and excludes — that
  scope is fixed in `spec/00-overview.md` §5, and this page does not
  reproduce it.

## Where to go next

[`quickstart.md`](quickstart.md) takes you from a clean checkout through
building the toolchain, checking and running one real program, formatting
it, and a short trust-aware reading exercise applying the four-class
vocabulary above. It is available now (Wave 1).

The fuller reading curriculum this introduction anchors — program anatomy,
contracts and proofs, assurance and trust, effects and capabilities,
packages and provenance, and execution — is in progress under
[`learn/reading-ken/`](learn/reading-ken/fragments.md) (Wave 1); a worked
review chapter is deferred to Wave 1b. See [`STATUS.md`](STATUS.md) for
what is landed versus planned.

---

**Grounds this page:** `docs/PRINCIPLES.md` §§1, 5, 8;
`spec/00-overview.md` §§1, 5; `spec/60-security/64-trust-model.md` §1;
`spec/20-verification/21-spec-syntax.md` §5. Authority class:
`explanatory` — this page orders and interprets those sections for a new
reader; it does not assert a rule the cited spec sections do not already
state.
