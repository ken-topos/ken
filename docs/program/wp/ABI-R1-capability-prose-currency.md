# `ABI-R1` — bring `Capability/Filesystem/Errors.ken.md`'s security boundary
# paragraph current with the landed capability surface

**Owner:** Foundation · **Size:** S · **Gate:** none ·
**Authority:** `docs/program/10-linux-abi-completion.md` §4, Track R.

> ## ⛔ THE TRAP THIS WP EXISTS INSIDE — read before touching the file
>
> The paragraph being replaced is **false**. The obvious replacement is **also
> false**, in the other direction. `ABI-R2` was **withdrawn** from this very
> program because its premise turned out false, and `DOC-W0` is the same family:
> ★ *a true statement standing in for the property that mattered.*
>
> **This is not a find-and-replace. Its entire content is deciding what the
> capability now guarantees, clause by clause, at the site that enforces it.**
> A rewrite that says "authority is now path-confined and symlink policy is
> enforced" would trade one false sentence for another.

## The current text (verbatim, `origin/main` = `d3b9f36c`, lines 7–10)

> Security boundary: the current authority check is coarse and is **not
> path-confined**. An `AFull` capability permits writes and deletes anywhere the
> host process can access. Scoped rights, symlink policy, and TOCTOU-resistant
> resolution are deferred to CA4/I-5.

Three claims. **All three are now wrong, and each is wrong differently.**

## Fixed inputs — the landed surface, verified on `d3b9f36c`

Authoritative file: **`crates/ken-host/src/capability.rs`** (332 lines).
⚠ Treat every line below as **perishable**: re-verify against the landed code at
pickup, and if any pin is false, **say so and escalate — do not build around
it.**

1. **`RightSet` is a 7-right bitset** (`:94-125`): `READ`, `WRITE`, `CREATE`,
   `DELETE`, `ENUMERATE`, `METADATA`, `CHANGE_MODE`; `ALL = (1 << 7) - 1`.
2. **`SymlinkPolicy` has exactly two states** (`:128-132`): `NoFollow`,
   `FollowWithinScope`. ⚠ It is a **per-scope choice**, not a single global
   policy.
3. **`FsScope`** (`:167-173`) carries `rights`, `root: FsHandle`,
   `lineage: Vec<FsIdentity>`, `symlink`, `empty`.
4. **`FsIdentity::Posix { device, inode }`** (`:161-165`) — identity is
   device+inode, i.e. **handle identity, not a path string**.
5. **`rights_for_authority`** (`:209-220`): `AUTH_FULL → RightSet::ALL`;
   `AUTH_PARTIAL → READ ∪ ENUMERATE ∪ METADATA`; otherwise `NONE`.
6. **`FsRootSpec`** (`:9-56`) is a *checked root spelling* resolved **once at
   executor initialization**, consuming the execution-start cwd handle; **no
   operation retains the specification.**
7. **`check_fs_capability`** (`:311-332`) is documented as *"the sole runtime FS
   capability gate used by interpreter and native lanes."*

## ★ The decisive finding — what the gate does and does NOT enforce

**Read `check_fs_capability`'s body, not its doc comment.** It performs exactly
three checks and then **returns the scope to its caller**:

| check | axis | enforced here? |
|---|---|---|
| `scope.rights.contains(operation.required_right())` | rights | **yes** |
| `authority_flows_to(required_authority, cap.authority())` | authority | **yes** |
| `scope.empty` → `ScopeEscape` | scope | **only the empty-scope case** |
| path confinement within `root`/`lineage` | scope | **no — returns `Ok(scope)`** |
| symlink policy | symlink | **no — `SymlinkDenied` is never returned here** |

⇒ `CapabilityDenied::ScopeEscape` and `CapabilityDenied::SymlinkDenied` **exist
as variants**, and this gate produces `ScopeEscape` **only** for an empty scope
and `SymlinkDenied` **never**. Confinement and symlink policy are **carried by
the capability and enforced by the resolver downstream.**

★ **Therefore: "the capability is path-confined" is a claim about the RESOLVER,
not about the gate.** The prose must be grounded where enforcement happens.
⛔ **Do not cite `check_fs_capability` as evidence of confinement.** It is
evidence of *rights* enforcement.

## ★★ The `AFull` precision trap — two axes the old sentence conflates

The old sentence — *"An `AFull` capability permits writes and deletes anywhere
the host process can access"* — is **half right, and the half that is right is
the half people will delete.**

- **Rights axis: UNCHANGED.** `rights_for_authority(AUTH_FULL)` is
  `RightSet::ALL`. `AFull` **does** hold `WRITE` and `DELETE`. ⛔ **Do not write
  that `AFull` is now restricted in what operations it permits.** That is false.
- **Reach axis: CHANGED.** Those rights are exercised **within an `FsScope`**
  rooted at a handle with a device+inode lineage — not "anywhere the host process
  can access."

⇒ The correct correction changes **"anywhere"**, and leaves **"writes and
deletes"** alone.

## Deliverables

**D1 — replace the security-boundary paragraph**, one clause per landed fact,
each clause grounded at its enforcement site. It must state:
- rights are **named and per-operation** (name the count and the mechanism, not
  the list — the list belongs to the code);
- authority `Full`/`Partial`/`None` maps to rights by `rights_for_authority`, and
  `Full` retains **all** rights while being **scope-bounded**;
- scope is **handle-and-lineage** based, with the root spelling resolved once at
  startup and not retained;
- symlink policy is a **two-state per-scope choice**, so the honest statement is
  that the *mechanism* is expressible and carried — **not** that one policy is
  globally enforced.

**D2 — state the enforcement split explicitly**, because it is the part a reader
will otherwise get wrong: the runtime gate checks **rights and authority**;
**confinement and symlink policy are enforced at resolution.** One sentence, but
it is the sentence that keeps this paragraph honest as the resolver evolves.

**D3 — the deferral sentence.** `CA4`/`I-5` are no longer the right pointers for
what landed. Either name what genuinely **remains** deferred, or delete the
sentence. ⛔ **Do not simply flip "deferred" to "landed"** — that is precisely
the `ABI-R2` failure mode. If you cannot ground a residual deferral, say nothing
rather than something reassuring.

**D4 — update the attestation ledger row.** Editing this file **moves its blob
OID**, which is true even for a locator-only change.

```
library/SOURCE-ATTESTATIONS  row 9
  59fbe76dde61a9ab3a1d4599088c60f04502ea89  catalog/packages/Capability/Filesystem/Errors.ken.md
```

⚠ **That row and OID were re-derived from `origin/main` on 2026-07-25 after
`DOC-W2` landed** — the row moved from 7 to 9 when DOC-W2 added three rows.
**Re-derive again at pickup**; do not trust this line if anything has merged
since. Regenerate via `scripts/gen-source-attestations.sh` and confirm the
proposed ledger matches the committed one exactly.

## Acceptance criteria

- **AC-1 — no claim without a citation.** Every clause of the new paragraph
  names the landed construct it rests on (`RightSet`, `SymlinkPolicy`, `FsScope`,
  `rights_for_authority`, `FsRootSpec`), and each is verified present on
  `origin/main` at pickup.
- **AC-2 — the `AFull` rights claim is NOT weakened.** The paragraph must not
  assert or imply that `Full` authority lost `WRITE` or `DELETE`. Reviewer check:
  `rights_for_authority(AUTH_FULL) == RightSet::ALL`.
- **AC-3 — the enforcement split is stated** (D2), and the paragraph does **not**
  attribute confinement or symlink enforcement to `check_fs_capability`.
- **AC-4 — no new overclaim.** For each sentence, name the file and construct
  that makes it true. **A sentence that cannot be grounded is cut, not softened.**
  ⛔ Specifically: no unqualified "TOCTOU-resistant" or "path-confined" claim
  unless grounded at the resolution site.
- **AC-5 — ledger currency.** `scripts/gen-doc-status.sh --check` and
  `gen-source-attestations.sh` are green on the merge result, with the new OID
  committed in the same diff as the prose change. ⚠ Both files land **together**
  or the currency gate reddens.
- **AC-6 — scope.** The diff touches **exactly**
  `catalog/packages/Capability/Filesystem/Errors.ken.md` and
  `library/SOURCE-ATTESTATIONS` (plus `library/STATUS.md` if regenerated).
  Deliberately grep-checkable.
- **AC-7 — the `.ken` code block is untouched.** This is a prose correction; the
  `renderIOError`/`renderFileError` definitions do not change. Verify by
  extracting the fenced block from both sides and diffing it to empty.
- **AC-8 — no regression in CI.** Catalog sources are literate `.ken.md` and are
  elaborated by corpus-wide oracles that a targeted build **cannot see**. ⛔ Do
  not run `--workspace`/`--locked` locally (COORDINATION §12) — CI gates it.
  ⚠ Grep for every test enumerating `catalog/` before assuming a prose-only edit
  is inert.

## Do-not-reopen guardrails

- ⛔ **Do not change the capability implementation.** This WP is prose +
  attestation only. If the code looks wrong, that is a **separate finding** —
  route it, do not fix it here.
- ⛔ **Do not extend the paragraph into a capability tutorial.** It is a security
  boundary note. Subsume, don't proliferate.
- ⛔ **Do not re-litigate the rights vocabulary or the two symlink states.** They
  are landed and settled; this WP describes them.
- The `Other Int` sentence and the rendering-is-package-policy framing are
  **correct and stay**.
