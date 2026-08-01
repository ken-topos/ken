# WP frame — `DOC-W3-DEPDATA` (Wave 3 slice 3, the dependent-data guide page)

Node: `docs/program/issues/DOC-W3-DEPDATA.md`. Program:
`docs/program/12-documentation-program.md` §4b Wave 3. Owner: doc ring
(doc-leader, doc-author, Librarian as QA).

Depends on `DOC-W3-GUIDE`: this page sits in the `library/guide/` directory that
slice created, and follows its checked-page conventions.

**Size S. One candidate. One page. Doc-only.**

## What this WP is, in one sentence

Wave 3 names seven guide subjects; six of them already have explanatory pages
in `library/`, and this WP writes the seventh.

⚠ **The measurement behind that claim is in the node, and it is the reason this
is an `S`.** Do not treat the other six subjects as unwritten. Writing them is
banned scope, not deferred scope.

## Fixed inputs

Measured at `origin/main = c777d2d4`.

| input | measured value |
|---|---|
| `library/guide/` | 3 pages, all checked literate: `decomposition-abstraction.ken.md` (5 `ken` fences), `proof-techniques.ken.md` (16), `surface-reference.ken.md` (26) |
| their classification | all three `kind = "explanatory"`, `authority = "explanatory"`, `availability = "current"` |
| the spine | `library/learn/reading-ken/` — 6 numbered chapters, **all also `explanatory`/`explanatory`**, plus `fragments.md` (`reference`/`derived-reference`) |
| `Vec` in `library/` | **zero occurrences.** Same for `Fin` |
| `Vec` elsewhere | one contrastive mention in `catalog/packages/Data/Collections/Derived.ken.md`; the challenge `conformance/challenge/C4-indexed-vec-head/vec-head.ken`. **There is no catalog package for the family** |
| the normative source | `spec/50-stdlib/60-length-indexed-vectors.md`, seven sections, declared Normative in `spec/SPEC-PROGRESS.md` |
| landed vs gated | spec §4: `vnil`/`vcons` **landed**, `head` **landed**, `Fin` decl **landed**, `tail` **landed** (DS-5b), `zip` **gated** (DS-5c), `lookup` **gated** (DS-5c) |
| `DS-5c` | referenced by the spec; **no tracker node exists** (`docs/program/issues/` holds only `DS-9` among `DS-*`) |
| `KERNEL-NESTED-IND` | `status: active`. **Nested-positive inductives are a different capability from indexed families** — see judgment 4 |
| checked-page mechanism | settled: a checked `library/**/*.ken.md` is a valid first-class manifest document (Librarian, `evt_453cj700kw59a`) |

Reproduce, read-only:

```sh
grep -rc '\bVec\b\|\bFin\b' -r library --include=*.md | grep -v ':0$'
sed -n '/^## 4. The total API/,/^## 5\./p' spec/50-stdlib/60-length-indexed-vectors.md
ls docs/program/issues/ | grep '^DS-'
for f in library/guide/*.ken.md; do printf '%-50s ' "$f"; grep -c '^```ken' "$f"; done
```

## Four judgments, settled here so the ring does not stop for them

### 1. The page goes in `library/guide/`, not in the spine

`learn/reading-ken/` is a **reading curriculum over catalog fragments** — every
chapter walks a real `catalog/packages/` file, and `fragments.md` is its
registered index. This subject has **no catalog package** (fixed inputs), so it
has nothing for a spine chapter to read. Wave 3 also names `library/guide/` as
the thing it produces.

⇒ **One new page under `library/guide/`.** Do not add a seventh spine chapter,
and do not renumber the existing six.

### 2. The page carries its own checked fences — it is a `.ken.md`

Its central claim is that the length index makes `head` **total**: no `Option`,
no partiality, no runtime emptiness check. That claim is exactly the kind that
must be checked rather than asserted. Its only existing checked witness lives in
`conformance/`, which the page may cite but must not depend on for its own
correctness — a prose page restating a guarantee proved somewhere else is the
drift-prone duplicate D1 exists to prevent, and it will look authoritative while
it rots.

⇒ **Author it as a checked literate page**, in the shape the three sibling guide
pages already use, so `ken check` over the destination is a real control on the
page's own content.

### 3. Scope is the totality showcase. The equational theory is out

Spec §5 scopes itself to the totality showcase and **defers the laws by name** —
`tail`/`lookup` computation, `zip`/`map` naturality, the `zip`-`unzip`
round-trip, and the length/`to_list` bridge. The page inherits that scope
exactly. Explaining why indexing buys totality is the subject; developing the
equational theory is not, and doing it here would put a second, unowned
specification of those laws into `library/`.

⇒ **Where the spec defers a law by name, the page names it as deferred too**, in
one sentence, and moves on.

### 4. Gated operations get labels, and nested inductives are a different gate

Two boundaries have to stay visible and they are distinct:

- **`zip` and `lookup` are gated on `DS-5c`.** They are specified and their
  design is settled; only their elaboration is gated. A sentence describing what
  `zip` does, written in the present tense, is aspirational syntax by another
  name. Label them and say why in a sentence.
- **`KERNEL-NESTED-IND` is not this page's gate.** Indexed families like `Vec`
  and `Fin` elaborate **today**; nested strictly-positive inductives are a
  separate, `active` capability. Conflating them would tell a reader that the
  landed showcase is unavailable.

⇒ **Say what works now, label what does not, and do not borrow one gate to
describe the other.**

## Deliverables

- **D1 — one page under `library/guide/`**, a checked `.ken.md`, explaining
  dependent data through the length-indexed family: what a parameter and an
  index are and why the distinction is load-bearing, why `Fin n` is a witnessed
  in-bounds index with no side-proof, and why `head` on `Vec A (Suc n)` is total.
- **D2 — the checked fences.** The declarations and the totality showcase appear
  as fences that `ken check` accepts over the destination file. A claim the page
  makes about what elaborates is carried by a fence, not by prose alone.
- **D3 — manifest registration**: `kind = "how-to"` is wrong here — use
  `kind = "explanatory"`, `authority = "explanatory"`, with audience,
  availability, `sources`, `validation`, and owner consistent with the three
  sibling `library/guide/` records.
- **D4 — the availability labels for the gated operations.** `zip` and `lookup`
  labelled with the one-sentence reason (judgment 4).
- **D5 — ledger consistency**: if a newly cited path lacks a row, install the
  generator's `.proposed` output; never hand-write a row.

## Acceptance criteria

- **AC-1 — the page's own fences check.** `ken check` over the destination
  `.ken.md` passes, and the fence count is recorded in the handoff.
  *Control:* the command and its output, plus the count.
- **AC-2 — every landed claim matches spec §4 at the candidate's base.** The
  page calls landed exactly what §4's table calls landed.
  *Control:* the §4 table beside the page's claims, row by row.
- **AC-3 — no gated operation is described in the present tense**, and each
  carries its one-sentence reason.
  *Control:* read every sentence mentioning `zip` or `lookup`.
- **AC-4 — the page does not restate spine or guide material.** Background on
  proofs, effects, packages, or execution is linked, not re-derived.
  *Control:* the page's background references resolve to a `learn/` or `guide/`
  page, and the page carries no section on another subject.
- **AC-5 — `KERNEL-NESTED-IND` and `DS-5c` are not conflated** (judgment 4).
  *Control:* every gate the page names resolves to the right capability.
- **AC-6 — every cited source is attested.** No page cites a path absent from
  `library/SOURCE-ATTESTATIONS`.
  *Control:* the `sources` path set minus the ledger's path set is empty.
- **AC-7 — no hand-written ledger row.** Re-running
  `scripts/gen-source-attestations.sh` produces a `.proposed` byte-identical to
  the installed ledger.
  *Control:* generate and `diff`; must be empty.
- **AC-8 — `library/STATUS.md` regenerated** by `scripts/gen-doc-status.sh` with
  no arguments. Generated, never hand-edited.
  *Control:* rerun it, then `git diff --quiet -- library/STATUS.md`. Use
  `--quiet`; `--stat` always exits 0 and would pass vacuously.
- **AC-9 — the candidate touches `library/` and `docs/program/` only.** No
  `crates/`, no `spec/`, no `catalog/`, no `conformance/`, no CI path.
  *Control:* the path list.
- **AC-10 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No pages for the other six subjects.** contracts, proofs, effects,
  security, packages, and execution already have explanatory pages, listed in
  the node's table. If one of them looks thin, that is a finding to report, and
  it is a *revision* to an existing page owned by a later slice — not a new
  guide page authored here.
- **No renumbering or restructuring the spine**, and no seventh spine chapter
  (judgment 1).
- **No filing a `DS-5c` node.** That it has no tracker node is a real finding
  and it is **Steward-owned**. Report it; do not create the node, and do not
  make the page's labels depend on one existing.
- **No developing the deferred equational theory** (judgment 3).
- **No `crates/`, `spec/`, or `conformance/` edits.** A defect found in the spec
  while writing is a finding for the enclave. A documentation candidate carrying
  a source change makes the verification record unauditable.
- **No new CI gate, and no reviving `VALIDATION_GATES`.** That registry is
  unreachable code and the finding is Steward-owned.
- **No campaign or WP history in the page** — a reader does not care which WP
  landed a feature (Wave 3 exit property).
- **No new test asserting facts about source, catalog, or documentation lines**
  (operator test policy).

## Contention

The doc track runs concurrently with build work by standing operator exception
because it touches `library/` and `agent/` rather than `crates/`. This candidate
writes `library/` and `docs/program/` only.

AC-1 needs the `ken` binary, so it needs a build turn:
`scripts/ken-cargo build -p ken-cli`, targeted, **never `--workspace`**. Probe
for the lock without blocking before taking it; the runtime ring holds the build
turn. If it is contended, draft the page first — but **do not hand off a
candidate whose fences have not been checked**, because AC-1 is the control that
separates this page from an asserted one.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A fence that the spec says is landed does not check** at your base. That is
   a spec-vs-implementation divergence and it is worth more than the page.
   Report what you ran and what it said; do not demote the claim to prose to get
   the page out.
2. **Spec §4's landed/gated split differs at your base** from the fixed-inputs
   table. The capability moved under the frame; the labels need re-deriving.
3. **The page cannot be written without restating a concept**, because the
   spine or guide page that should cover it is missing or wrong. That is a
   finding about the existing corpus and it outranks this page.
4. **A newly cited path has no ledger row and the generator will not produce
   one.** Report the path.
5. **The subject turns out to be covered after all** — a page you had not found
   already explains indexing at conceptual depth. Report it and stop. Retiring
   this node is a legitimate and valuable outcome, and it is strictly better
   than writing a duplicate.
