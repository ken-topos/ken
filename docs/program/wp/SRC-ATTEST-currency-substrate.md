---
id: SRC-ATTEST
title: "squash-stable whole-source attestation + fresh merge-result authorization"
status: ready
owner: unassigned
size: M
gate: none
depends_on: []
blocks: [ORACLE-VIS-PACKAGING]
github: null
origin: >
  Librarian impossibility proof (evt_6t6wz1aw18291) during ORACLE-VIS-PACKAGING
  re-validation; Architect ruling dec_7q3kes0jcx1kn (evt_5vp06mb9v26mh).
  Framed by the Steward from that contract.
---

## Why this exists

**The Librarian proved that no valid `library/REVISION` can ride a PR that
edits a cited source.** Constructed and executed, both arms:

| choice | outcome |
|---|---|
| `REVISION` = current `origin/main` | **source-currency fails** — the cited source differs between `REVISION` and `HEAD` |
| `REVISION` = the branch tip | **squash-stability fails** — branch-local, not an ancestor of `origin/main` |

⇒ **No third commit is simultaneously an ancestor of `origin/main` and a holder
of bytes that have not merged yet.** That is a proof by the gate's two
predicates, not a tooling inconvenience, and **rebase does not help** — it makes
`main` an ancestor of the candidate, but the cited source still differs from
`main`.

★ **This generalizes to every WP that touches a manifest-cited source, forever.**
`ORACLE-VIS-CHECK` hit it, `ORACLE-VIS-PACKAGING` is held by it, and the next one
will hit it too. It is not a property of any candidate.

## ⛔ The anomaly, and the wrong explanation it nearly got

PR #885 changed `px4b_native_production.rs` — cited — left `REVISION` stale, and
shard 4/4 came back **SUCCESS**. Two explanations were proposed and **both were
wrong**:

- *"The predicate tightened since."* **Measured false** — `gen-doc-status.sh` is
  byte-identical between `dd715950` and `b2fd95ac`.
- *"CI's shallow checkout makes the PR-time gate weaker."* **Measured false** —
  the Librarian reproduced the current merge result at depth 1 and got the same
  red. The shallow-heal path is not the divergence.

**The Architect's graph reconstruction is the actual cause:** `c0890b13` (#885's
base) had **zero** `px4b` citations and its exact-tree check exits 0; **DOC-W1-5
added the two citations at `1e148908`, after #885's green check had already
formed.** Reconstructing the *later* merge result at `dd715950` makes the
**unchanged** checker exit 1.

⇒ **#885 carried a green check for an OLDER merge result after `main` advanced.
It is a TIME-OF-CHECK defect, not a weaker PR-time predicate.**

⚠ **Framing note for whoever builds this, because it cost two seats a wrong
lead:** the citation-presence question is a **PR-base** question and was twice
measured against **post-merge `main`** (`dd715950^`), which is a different point
in the graph. *Asking a pre-merge question of post-merge state returns a
confident, wrong answer.* Check `c0890b13`, not `dd715950^`.

## Part 1 — source-currency representation

Keep `library/REVISION`, but **narrow it to an already-on-`main` provenance /
bootstrap anchor.** It is **no longer** the proof that every current cited byte
existed at that commit, and `STATUS.md` must stop describing it as a "validated
revision" in a way that implies branch-only bytes are in it.

Add a canonical ledger, e.g. `library/SOURCE-ATTESTATIONS`:

```
# ken-source-attestation-v1 object-format=<git object format>
<full blob oid><TAB><normalized repository-relative path>
...
```

**The filename is not load-bearing. These semantics are:**

1. **Population** = every source path selected by the current manifest records to
   which `source-currency` applies.
2. Strip `#anchor` **only** to deduplicate the path. **Anchor existence remains a
   separate gate.**
3. Rows sorted by path, binding each path to its **whole tracked-file Git blob
   OID at `HEAD`**.
   ⛔ **Do NOT hash extracted Markdown spans** — that introduces a new
   section-boundary parser and **weakens** the current conservative whole-file
   predicate.
4. **Exact set equality.** Missing, extra, duplicate, noncanonical, nonexistent,
   non-blob, or symlink rows **fail closed**.
5. **Exact OID equality.** Changed bytes without a fresh Librarian attestation
   are red. ★ **A candidate-time attestation is squash-stable because the blob
   survives even though the branch commit does not** — this is the whole point,
   and it is what dissolves the impossibility.
6. Render both values **distinctly** in `STATUS.md`: **provenance revision** and
   **attested source-set digest/root**. The ledger is the source-currency
   authority; `REVISION` is provenance.
7. ⛔ **Replace** the old cited-byte `REVISION → HEAD` equality with ledger
   `→ HEAD` equality. **Keeping both preserves the impossibility and defeats
   the design.**

## Part 2 — fresh merge-result authorization

**The ledger does not by itself close #885.** A later manifest change can alter
the required population after CI has formed its verdict.

Immediately before **every** merge — **not only `--doc-only`** — the publisher
must:

1. read current `origin/main`;
2. construct the exact squash result with the candidate;
3. run **`origin/main`'s checker**, not the candidate's, on that result;
4. **re-read `origin/main` before merging** and abort/reconstruct if it moved.

Because the publisher is the sole merge router, that makes **the checked result
the result it lands.** ⇒ **Old green CI attached to a prior merge result is not
authorization.**

★ **Steward note:** steps 1–3 already exist in `scripts/scripted-pr-automerge.sh`
as the `--doc-only` currency guard (landed `a9554a07`, F10/F11/F12 folded in).
**The mechanism is built and proved; this WP generalizes its scope to every
merge and adds step 4, which is new.** Read that block before writing a new one —
it carries the F10/F11/F12 reasoning in-file, including why the checker must come
from `origin/main` and why `git merge --squash` must be followed by a commit.

## Acceptance — the Architect's proof matrix, verbatim

Each row is a **required proof, executed and shown**, not an assertion:

1. cited body drift, ledger unchanged → **red**
2. exact candidate-time ledger update → **green** on candidate **and** on the
   synthetic squash result
3. citation add/remove → **set mismatch** until the ledger follows
4. extra / duplicate / wrong-path / wrong-OID / symlink row → **red**
5. old-green CI followed by a citation-bearing `main` advance → **fresh
   publisher check red**
6. **depth-1 and full-history runs agree**
7. `ORACLE-VIS-PACKAGING` + the Librarian's exact whole-source attestation →
   **green without weakening either predicate**

⚠ Row 5 is the one that closes #885 and it is the one a happy-path suite will
omit — it requires *manufacturing* a stale-CI-then-advance sequence. Per the
build-QA playbook (`:299`), enumerate the probes by the state each builds and
name which probe builds row 5. **If it has no probe, the WP is not done.**

⚠ Row 6 is a **freshness/environment** axis, independent of correctness: the
depth-1 case must be executed, not reasoned about. The prior gate carried a
comment saying nobody had proved it **accepts** a real revision in the
environment where it runs — do not repeat that.

## Scope boundary

**In:** `library/SOURCE-ATTESTATIONS` (or equivalent), `scripts/gen-doc-status.sh`,
`scripts/scripted-pr-automerge.sh`, `library/STATUS.md` rendering, and the gates
in `crates/ken-cli/tests/library_documentation_gates.rs`.

**Out:** anchor-existence checking (stays a separate gate, unchanged); any
Markdown span extraction; any relaxation of a predicate to make a candidate pass.

⛔ **The Librarian's attestation remains a human act.** The ledger makes it
*representable*; it must not make it *automatic*. A generated-on-demand ledger
that always matches `HEAD` asserts nothing — it would be the same defect as an
auto-bumped `REVISION`, which the current gate refuses for exactly this reason.
