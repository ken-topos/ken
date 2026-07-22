---
id: DOC-W1
title: "documentation Wave 1 — the read-Ken spine, taught from checked fragments"
status: closed
owner: doc
size: L
gate: none
depends_on: [DOC-W0, DOC-CURRENCY-ANCHOR]
blocks: []
github: null
origin: research/librarian-documentation-program-proposal.md Wave 1; re-scoped by operator ruling 2026-07-22 (docs/program/12-documentation-program.md §4b)
---

**✅ RELEASED 2026-07-22 — both dependencies are closed.** `DOC-W0` closed on
its deliverables; **`DOC-CURRENCY-ANCHOR` closed at `origin/main @ 10da4cf6`**,
acceptance re-derived against the landed `scripts/gen-doc-status.sh` through the
git object store. `status:` is now `ready`.

> ⛔ **AC-2 is the reason the gate existed, and it is now DISCHARGEABLE rather
> than discharged.** Wave 1 is precisely where DOC-W0's unmet half bites: every
> cited spec section's currency must be anchored by the **content-currency**
> mechanism — the check that reads a source's bytes *at `REVISION`* — **not by a
> recorded revision that merely names a real ancestor.** That distinction is the
> entire content of the eight-finding DOC-W0 defect class (`12-documentation-
> program.md` §4a). **State which predicate each citation rests on**; a green
> gate must not assert more than it checked.

The program frame is `docs/program/12-documentation-program.md`. **Read §4a
(the failure mode this program is designed against) and §4b Wave 1 before
scoping.** §1's four decisions are **settled** — do not reopen them.

## 1. Objective

The first complete human reading path: short enough to finish, deep enough to
change how a reader reviews code.

## 2. Deliverables

| artifact | action | note |
|---|---|---|
| `library/introduction.md` | ⚠ **REVISE** | **It already landed in Wave 0** and is in `manifest.toml`. The proposal assigns "write the introduction" to Wave 1. **Revise, do not author** — or you will duplicate it. |
| `library/quickstart.md` | author | install/use the current toolchain, check and run one program, format it, then a short trust-aware reading exercise |
| `learn/reading-ken/01-anatomy.md` | author | orient in a source file |
| `learn/reading-ken/02-types-contracts-and-proofs.md` | author | read the promise before the body |
| `learn/reading-ken/03-assurance-and-trust.md` | author | proved / tested / delegated / unknown, and the TCB |
| `learn/reading-ken/04-effects-capabilities-and-authority.md` | author | what the program may do |
| `learn/reading-ken/05-packages-and-provenance.md` | author | what it imports and inherits |
| `learn/reading-ken/06-execution.md` | author | interpreter, compiler, runtime assumptions, traps |
| `library/learn/exercises/` | author | checked reading exercises, **solutions in separate files**, explicit learning objectives |

Every chapter ends with a **"reader can now answer"** checklist.

**⛔ Chapter `07-review-worked-example.md` is NOT in this wave.** It is Wave
1b, deferred by the operator pending the basic capability surface. Do not
write it, and do not fold a partial version into chapter 06.

## 3. Fixed inputs — settled, do not reopen

- **Wave 1 is FRAGMENT-BASED** (operator, 2026-07-22: *"we're still focusing
  on basic capabilities. defer complete program work (revise wave 1)"*). Teach
  the reading discipline from **real checked package fragments** under
  `catalog/packages/`, which exist today in volume and are already
  fence-checked.
- **⛔ Do NOT build the curriculum around one whole program.** The catalog
  contains exactly **one** actual program — `catalog/examples/CommandLine/
  Forge.ken.md`, 55 lines — and it is **pure spec-data with no effects**, so
  chapters 04 and 06 would have nothing local to teach from, forcing exactly
  the *"unrelated snippets"* the proposal forbids. That is the whole reason
  the wave was re-scoped.
- **⛔ Do NOT substitute a purpose-built toy** for a real fragment. The
  proposal's *"an existing catalog artifact, not a toy syntax collage"*
  constraint survives the re-scoping intact.
- **`library/` is explanatory and derived; `spec/` is the sole normative
  authority** (D1). A page that states a language rule on its own authority is
  a defect **regardless of correctness**. Cite the exact spec section.
- **Planned syntax may never appear in a checked current example** (§2).
  Label capability honestly: current / partial / planned / unavailable.
- **Targeted builds only** — `scripts/ken-cargo -p <crate>`, never
  `--workspace`. Workspace-green means green in **CI**.

## 4. Acceptance criteria

1. **Every page declares its authority class and sources in
   `library/manifest.toml`**, and the Wave 0 gates pass on the full set.
2. **Every cited spec section resolves, and its currency is anchored by the
   `DOC-CURRENCY-ANCHOR` mechanism** — not by a recorded revision that merely
   names a real ancestor. State which predicate each citation rests on.
3. **Every fragment shown is a real, checked `catalog/packages/` fragment**,
   cited to its file, and it still checks at the candidate SHA. Report the
   mechanism used to establish that, not the assertion.
4. **The exercises are checked**, and their solutions live in separate files
   so a reader cannot see them by accident.
5. **Exit property demonstrated, not asserted** — see §5. Report the run.
6. **No page introduces normative language.** A reviewer must be able to check
   this cheaply; state the mechanism you used to sweep for it.

## 5. Exit property

> **A technically experienced human unfamiliar with dependent types can read a
> real Ken declaration or package fragment and accurately state its contract,
> its assurance class, and the authority it requires** — without yet being
> asked to synthesize a whole program.

**This is a property, not a page count.** It is deliberately narrower than the
proposal's original Wave 1 exit; the removed half is Wave 1b's.

**Demonstrate it.** The honest available instrument is a **cold-context agent
seat** given only the Wave 1 path and asked to state a fragment's contract,
assurance class, and required authority — scored on whether it **cited the
authority it used** and whether it **refused rather than improvised** where the
path does not cover something. That is a proxy for the human reader and it
must be **labelled as one in the retro**, not reported as the property itself.

## 6. Framing traps

- **Every anchor in this frame is perishable.** The fragment citations, the
  `manifest.toml` shape, and the gate names were true when written. **Re-verify
  at pickup; if a fixed input turns out false against the landed corpus, say so
  and escalate — do not quietly build around it.**
- **A wave that exits on a proxy poisons every wave that inherits its
  substrate**, and waves run sequentially, so there is no parallel track to
  catch it. §4a of the program frame is not background reading.
- **The doc ring's QA is the Librarian, who also edits `library/`.** Its
  approval is therefore not the independent check a build QA's is — **the
  gates are the independent oracle.** Prove every new gate fails on a planted
  violation.
