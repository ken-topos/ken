# 13 — Documentation migration ledger

**Status:** Wave 0 deliverable of `docs/program/12-documentation-program.md`
(`docs/program/issues/DOC-W0.md`, deliverable 6). A **record**, not the
migration itself — nothing listed below has moved as part of this wave
except where marked "performed."

This ledger tracks the disposition of the existing documentation corpus
against `library/`, per
`research/librarian-documentation-program-proposal.md` §"Migration from the
current corpus." It is process/program material, not a reader-facing
product page, so it lives under `docs/program/` rather than `library/`
(`docs/program/12-documentation-program.md` D3: federation/program history
stays outside the public product-doc navigation).

## Disposition table

| Current material | Disposition | Wave | Status |
|---|---|---|---|
| `README.md` (repo root) | Remains the public front door; shortens over time to thesis, status, and links into `library/`. | — | unchanged this wave |
| `spec/` | Remains normative and structurally unchanged. `library/` reference pages cite or derive from it, never restate it. | — | unchanged (cited from `library/introduction.md`) |
| `catalog/packages/` | Remains the canonical package source and per-package literate rationale. `library/catalog/` will generate discovery/structural reference around it. | 5 | not started |
| `catalog/guide/` (`README.md`, `decomposition-abstraction.ken.md`, `proof-techniques.ken.md`, `surface-reference.ken.md`) | **Migrates into `library/learn/`, `library/guide/`, `library/how-to/`.** ⛔ Gated: these four files are literate `.ken.md` with **checked** `` ```ken ``/`` ```ken example ``/`` ```ken reject `` fences (`crates/ken-elaborator/src/literate.rs`). They must not move until an equivalent fence-checking gate exists and passes for `library/` content — moving them first would silently stop checking them while they kept *looking* checked. | 3 | **not started — the fence gate is the precondition, see below** |
| `agent/playbooks/tools/write-ken.md` | Keeps its workflow trigger. Reusable Ken product facts move into `library/agents/`; the skill selects the appropriate pack. | 2 | not started (`library/agents/` is explicitly out of scope for Wave 0 — `docs/program/issues/DOC-W0.md`) |
| `docs/adr/` | Remains decision records. Conceptual `library/` pages cite the accepted ADR rather than teach from decision history. | — | unchanged |
| `docs/program/` | Remains internal program history and WP material, excluded from the public product-doc navigation (this ledger included). | — | unchanged |
| `conformance/` | Remains executable contract evidence. `library/` pages link cases and reuse checked fixtures rather than duplicate them. | — | unchanged |
| `research/` | Remains advisory background, not part of the ordinary reader path. | — | unchanged |

## The one ordering constraint, restated

`catalog/guide/`'s checked-fence machinery already exists and is exercised
today (`crates/ken-elaborator/tests/ken_md_literate.rs`,
`crates/ken-cli/tests/ken_check_mode.rs`, `ken check`/`ken run` on
`.ken.md` via `ElabEnv::elaborate_ken_md_file`). What Wave 0 did **not**
build is a *library-wide* gate that runs that same machinery over every
checked fence under `library/` as part of CI (Wave 0's gates 1/2/3/6 cover
manifest coverage, links, source anchors, and availability labels — not
fence elaboration, which is the proposal's documentation-gates item 4).
Wave 3 must add that gate **before** moving any `catalog/guide/` content,
per the frame's non-negotiable ordering constraint.

## D4 note — what Wave 0 learned about generation capability, fact-by-fact

The checked-fence extractor (`ken_elaborator::literate::extract_ken_md`)
classifies exactly four fence roles (source, ignore, reject, example) by an
**exact** info-string match and hard-errors on an unrecognized `ken`-tagged
opener — a typo'd role cannot silently downgrade to unchecked prose.

**Correction to this note's first draft** (librarian QA, `thr_74hvpkqnxjp9q`,
finding 4): a `` ```ken reject `` fence only proves its body **fails to
elaborate at all**. `elaborate_ken_md_file` accepts any elaboration error
as satisfying the fence — it carries no mechanism to check *which* error,
or that it matches the reason the surrounding prose claims. A rejection
example is honest evidence the body is rejected, never evidence it is
rejected for the stated reason. The draft's "for the stated reason" was an
overclaim; struck.

**What D4's structural facts need, checked directly against the API**
(`ElabEnv`/`GlobalEnv`, `crates/ken-elaborator/src/lib.rs:104-124`,
`crates/ken-kernel/src/env.rs`) rather than assumed from the return type
of one function:

- **Directly inspectable today, no new extraction surface needed:**
  - which declarations a checked fence introduced —
    `elaborate_ken_md_file` returns `Vec<GlobalId>`, and
    `ElabEnv.globals: HashMap<String, GlobalId>` names them;
  - each declaration's kernel type (`Decl::Transparent`/`Opaque`/
    `Primitive`'s `ty: Term`, or the inductive's telescope) via
    `GlobalEnv::lookup(id) -> Option<&Decl>`, and the whole declaration set
    via `GlobalEnv::decls()`;
  - the trusted-base delta a declaration pulls in, via
    `ken_elaborator::foreign::trusted_base_delta(&env, id)` and
    `GlobalEnv::trusted_base()`;
  - a definition's **surface effect row** — `ElabEnv.effect_rows:
    HashMap<String, effects::RowType>` is populated for already-elaborated
    definitions;
  - the **class/instance registry** — `ElabEnv.class_env: ClassEnv`
    exposes `classes: HashMap<String, ClassInfo>` (each with
    `field_names: Vec<String>` and `field_types: Vec<Term>`, a real
    Σ-telescope) and `instances: HashMap<(String, String), InstanceInfo>`
    directly. **Narrower than this note's prior draft claimed:** `ClassEnv`
    is a class/instance registry, not a distinct law/proof inventory —
    it holds no separate list of `law`/`proof` declarations. An Ω-sorted
    (property) field is a `ClassInfo` field like any other; "which fields
    are laws" is derivable from `field_types`' sorts, not read off a
    dedicated law registry, because there isn't one. The prior draft's
    "law registry" claim promoted the API beyond what it exposes;
    corrected here rather than left standing.
- **Inspectable via derived traversal, not a one-call fact:**
  - a **human-readable signature** — the `ty: Term` above is a raw kernel
    core term. **Correction:** `ken_elaborator::layout::format_ken` does
    **not** render one — its signature is `format_ken(source: &str) ->
    Result<String, ElabError>`; it parses and formats Ken *surface source
    text*, not a kernel `Term`. A grep across `ken-kernel` and
    `ken-elaborator` for a `Term`-to-surface printer (a `Display` impl or
    an equivalent pretty-printer) found none. Rendering a readable
    signature from `ty: Term` needs a core-term printer that does not
    exist yet in either crate — Wave 5 has to build one, not reuse
    `format_ken`;
  - **dependencies** between declarations — walkable from a `Term`'s free
    variables by the same traversal `trusted_base_delta` already performs,
    but no ready-made "package dependency list" call exists yet.
- **Not found in the elaborator crate; needs verification against
  `ken-runtime`/`ken-host` before Wave 5 can rely on either answer:**
  - per-declaration **capability/authority requirements** —
    `ken_elaborator::capabilities` implements attenuation/revocation
    *checking* machinery (`Cap`, `Authority`, `AttenuationObligation`), not
    an enumerable "which capabilities does this declaration need" record;
  - **platform/execution-backend availability** — this looked like a
    `ken-host`/`ken-runtime` concern (cf. the `TARGET_ABI` fact inventory
    Q-RESIDUE's R1 discusses) in a pass over the elaborator crate alone; a
    genuine "no producer" claim here needs that pass too, which this wave
    did not do. Recorded as **unconfirmed**, not as a negative.

Wave 5 should budget accordingly: "write an exporter over already-
inspectable facts" for declarations/trusted-base/effects/class-instances
(deriving law-field facts from `ClassInfo.field_types`' sorts, not from a
dedicated law registry), "build a core-term-to-signature printer plus a
dependency traversal" for the second tier, and "first confirm whether a
producer exists at all" for capabilities and platform availability before
assuming a new surface is needed there too.
Any fact that turns out to have no producer gets **authored and labelled
as authored**, never generated-looking prose
(`docs/program/12-documentation-program.md` D4 note).
