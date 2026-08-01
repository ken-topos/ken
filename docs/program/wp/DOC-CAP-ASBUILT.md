# DOC-CAP-ASBUILT — the chapter's "no checked exemplar" claim is now false

**`library/learn/reading-ken/04-effects-capabilities-and-authority.md` tells the
reader the authority discipline exists only as spec pseudocode: *"the catalog
has not instantiated [it] as checked code."* `CAT-CAPEX` instantiated it. The
chapter is now false on exactly that claim, and true everywhere else.**

**Owner:** Team Doc (`doc-leader` + `doc-author`, Librarian as QA).
**Branch:** `wp/DOC-CAP-ASBUILT`. **Size:** S.
**Risk:** low — one page, one narrow claim, `library/` only.
⚠ **AMENDED 2026-08-01 — the shape is one PAGE plus its two first-time source
declarations, not one FILE.** See `D5`/`AC-6`. ⭐ `AC-4` (`library/`-only) is
unchanged and was never breached: `library/manifest.toml` and
`library/SOURCE-ATTESTATIONS` are both inside `library/`.

**Status:** Steward frame, shovel-ready. ⛔ **Gated on `CAT-CAPEX` landing on
`main`** — see `§7`. Do not start before it does.

---

## 1. Fixed inputs

| path | blob at `d6df571e` |
|---|---|
| `library/learn/reading-ken/04-effects-capabilities-and-authority.md` | `75efaf88d015ec2ea26c00029962fa537a102352` |
| `spec/60-security/62-authority.md` | `7b6b1b7299ee6438211690562167e5bf37e99316` |

⚠ The spec entry is a **read-only reference**. ⛔ This WP does not edit `spec/`.

⭐ **The new inputs land with `CAT-CAPEX`** and must be re-derived at pickup:
`catalog/packages/Capability/Filesystem/Authority.ken.md` and
`crates/ken-elaborator/tests/cat_capex_authority.rs`.

---

## 2. The measurement

Two spots in the chapter carry the stale claim:

| line | text |
|---|---|
| `:115` | `§7's worked examples (unavailable in checked form — spec pseudocode, not a catalog fragment)` |
| `:125` | `The authority specification is normative prose that the catalog has not instantiated as checked code.` |

Against `CAT-CAPEX`: the catalog now carries
`Capability/Filesystem/Authority.ken.md`, a **checked** fragment taking an
explicit `(cap : Cap a)` over the landed `read_bytes`, elaborated by a named
test with paired positive/negative controls.

⇒ **The "unavailable in checked form" label and the "has not instantiated"
sentence are the defect. Nothing else in the section is.**

---

## 3. ⭐ Steward-discharged — the repair PROMOTES the chapter's own argument

⚠ **Read this before editing, because the obvious edit is wrong.**

`:126-128` already says:

> *"The boundary is also **not merely** 'no checked fragment exists yet' — it is
> that `attenuate`/`revoke` are, by design, never going to be something a Ken
> program calls at all."*

⭐ **That sentence is correct, and `CAT-CAPEX` makes it *more* true.** The
chapter was hedging against a reader who might think the gap was just missing
work. Now the "no checked fragment" half is **discharged by fact**, and what
remains is exactly the designed part the chapter was pointing at.

⇒ **The repair is not a deletion and not a rewrite.** It retires a hedge that
has been overtaken, and lets the real claim stand on its own. ⛔ Do **not**
weaken, qualify, or restructure the `attenuate`/`revoke` argument — it is
landed truth (`38 §1.3.1` requires all three unbound; `62 §4` puts narrowing in
the trusted host).

---

## 4. ⛔ Banned shapes

- ⛔ **Do not edit `spec/`, `catalog/`, or `crates/`.** `library/` only.
- ⛔ **Do not fix the chapter's one `Cap_FS` occurrence.** ⚠ It sits in a
  refinement-type example paraphrasing `62 §2.2`, and **`62` itself still
  writes `Cap_FS`** — the spec's staleness is filed separately as
  `SPEC-AUTH-EX`. Correcting the doc ahead of the chapter it cites manufactures
  a doc/spec mismatch. **Report it under `D3`; it follows `SPEC-AUTH-EX`.**
- ⛔ **Do not claim the exemplar covers more than it does.** `CAT-CAPEX`
  exhibits authority-as-signature, the authority index, and the no-ambient
  rejection. It does **not** mint, attenuate, revoke, or exercise a runtime
  capability — those are host/runner-side and named as excluded in the
  fragment's own `D5`. ⭐ Reproduce that boundary honestly; do not upgrade
  "the catalog now has an exemplar" into "the catalog exhibits the whole
  discipline."
- ⛔ **No new CI gate or test asserting facts about doc, spec, or catalog
  lines** (operator test policy). ⚠ Including a "reports drift" form.
- ⛔⛔ **DO NOT RUN `scripts/gen-source-attestations.sh`, and do not regenerate
  `library/STATUS.md`.** ⭐ **Measured by the Steward 2026-08-01 at
  `main = 2b424842`: `scripts/gen-doc-status.sh --check` ALREADY EXITS 1 with
  27 drifted cited sources.** That population is the outstanding as-built audit
  and it is ⛔ **not this WP's to clear.** A wholesale regeneration would
  re-stamp all 27 in one commit — laundering the exact evidence the audit
  exists to examine. ⇒ **Hand-write the two rows of `D5` and nothing else.**
  ⚠ `gen-doc-status.sh` cannot complete while those 27 stand, so `STATUS.md`
  **cannot** be regenerated here; leaving it untouched is correct, and the
  audit slice regenerates ledger and `STATUS.md` together under Librarian
  review.

---

## 5. Deliverables

- **`D1`** — the `:115` label and the `:125` sentence corrected to state what
  landed, citing the catalog fragment **by path** and its elaborating test
  **by name**. ⭐ A reader must be able to go run the thing.
- **`D2`** — the `:126-128` boundary argument **preserved and sharpened** per
  `§3`, with the overtaken hedge retired rather than the claim softened.
- **`D3`** — a short report: the `Cap_FS` occurrence left in place with its
  reason (`§4`), and any other spot in the chapter that reads as stale against
  the landed exemplar.
- **`D4`** — a **closed** statement of what the chapter now claims the catalog
  exhibits **and what it still does not**, matching the fragment's own
  complement.
- **`D5`** ⭐ **(added 2026-08-01)** — **declare the two sources this edit now
  cites.** Add to this chapter's `sources` in `library/manifest.toml`, and add
  the matching rows to `library/SOURCE-ATTESTATIONS`, for exactly:

  | path | attest at this OID |
  |---|---|
  | `catalog/packages/Capability/Filesystem/Authority.ken.md` | `3ada325be9479b96993f0e13d4df60901c95fae1` |
  | `crates/ken-elaborator/tests/cat_capex_authority.rs` | `23e8a161a8f803b805c832e5df3522b3cb45672e` |

  ⭐ **These are FIRST-TIME attestations of never-attested paths, not
  re-stamps.** ⚠ Re-derive both OIDs at your base with `git ls-tree HEAD --
  <path>`; ⛔ do not copy them from this frame if your base differs.
  ⚠ Ledger rows are `<oid>\t<path>`, **sorted by path** — the gate rejects an
  unsorted ledger. `Authority.ken.md` sorts **after** `Capability/Console/
  Text.ken.md` and **before** `Capability/Filesystem/Errors.ken.md`;
  `cat_capex_authority.rs` sorts **before**
  `crates/ken-elaborator/tests/ds1_empty_dec_acceptance.rs`.

---

## 6. Acceptance criteria

- **`AC-1`** — every claim `D1` writes about the catalog is **true of the
  landed fragment**. **Control:** quote the fragment's actual signature and
  name the test; ⛔ do not describe it from this frame — read the file at your
  base. ⭐ This frame was written **before** the fragment landed and is not
  authority on its final contents.

- **`AC-2`** ⭐ **(load-bearing)** — the `attenuate`/`revoke`/`strengthen`
  boundary survives **undiluted**. **Control:** show the before/after of that
  passage; the claim that these are unbound **by design** and that narrowing
  happens outside Ken must be at least as strong as before. ⛔ If the edit
  makes the security argument weaker or more hedged, it has failed — that
  argument is the reason the section exists.

- **`AC-3`** — `D4` names its complement. **Control:** state what the exemplar
  does **not** exhibit (minting, attenuation, revocation, admission,
  settlement, audit). ⛔ An empty exclusion list is a failed measurement and
  an over-claim.

- **`AC-4`** — scope is `library/` only. **Control:** `git diff --name-only`
  shows no path outside `library/`.

- **`AC-5`** — no link or section anchor is broken. **Control:** the chapter's
  existing spec cross-references still resolve; ⚠ you are editing prose that
  carries several `../../../spec/...#anchor` links.

- **`AC-6`** ⭐ **(added 2026-08-01 — the anti-laundering control)** — the
  ledger edit **adds two rows and changes nothing else.**
  **Control:** `git diff library/SOURCE-ATTESTATIONS` shows **exactly 2 added
  lines and 0 removed lines.** ⛔ **Any removed line means a drifted row was
  re-stamped — that is the laundering this WP is forbidden to do, and it is a
  hard stop, not a fixup.**

- **`AC-7`** ⭐ **(added 2026-08-01 — the positive control on `AC-6`)** — the
  drift population is **unchanged in both directions.**
  **Control:** run `scripts/gen-doc-status.sh --check` before and after your
  edit. It exits 1 both times. **The listed drifted paths must be the SAME 27,
  byte-for-byte** — ⛔ not 26, ⛔ not 29, and ⛔ neither new path may appear.
  ⭐ **Why this is a real control and not ceremony:** the ledger's
  sortedness check and the cited-set exact-match check both run **before** the
  drift check, so a malformed, misplaced, or missing row changes *which error
  the script reports*. An unchanged 27-path drift block is therefore positive
  evidence that the two rows landed well-formed **and** that nothing else moved.

- **`AC-8`** ⭐ **(added 2026-08-01 — Librarian `BLOCK 1`)** — every source
  the edit **quotes or paraphrases** matches that source's blob **at your
  base**, not a remembered version. ⚠ Specifically:
  `catalog/packages/Capability/Filesystem/Errors.ken.md` **no longer says
  authority is unconfined** — it now scopes `Full` to its `FsScope` with
  downstream resolution enforcing confinement. **Control:** quote the current
  blob. ⛔ The candidate's own rewrite of that paragraph makes this a live
  derived claim, ⛔ **not** a frozen historical quotation that may stand stale.

---

## 7. ⛔ Sequencing — this WP is GATED

⛔ **Do not start until `catalog/packages/Capability/Filesystem/Authority.ken.md`
is present on `origin/main`.** Until then the chapter's statement is **true**
and editing it would make the corpus wrong in the other direction.

⭐ **How to check — the fact, not a report of it:**

```sh
git fetch origin && git cat-file -e \
  origin/main:catalog/packages/Capability/Filesystem/Authority.ken.md && echo PRESENT
```

⚠ ⛔ Do **not** gate on a message from me saying it landed; gate on the file.
`CAT-CAPEX` was in CI when this frame was written.

**Contention:** `library/learn/reading-ken/` — the doc track runs concurrently
with build work by standing operator exception, and no build ring touches
`library/`. ⚠ Re-measure at pickup.

---

## 8. Hard stop

⛔ Route to the Steward if:

- the landed fragment does **not** support a claim this frame assumes — ⭐ the
  frame was written pre-merge and the fragment is authority over it; **or**
- correcting `:115`/`:125` appears to require editing `spec/` or restating
  `62 §7` — it does not, and `62 §7`'s staleness is `SPEC-AUTH-EX`; **or**
- the honest-boundary passage cannot be preserved while making the claim true.
  ⛔ Do not resolve that by weakening the boundary.
