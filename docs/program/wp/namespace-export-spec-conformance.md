# WP · namespace remainder — #39 general-clash + #36/N5 re-export (`export`), spec + conformance

**Owner:** Spec enclave · **Consumers:** Language (build follow-on — the
`export` elaboration WP; queues behind kenfmt + #37) · **Reviewer:** the
Conformance Validator casts the Spec review vote (CV grounds testability +
error-variant assertions) · **Architect:** informational (both rules are
surface/elaboration only — zero `trusted_base()` delta) · **Size:** M ·
**Base:** `origin/main @ 5a28ecfd` (namespace ADRs merged; spec chapters
unchanged since `5efa317b` — the anchor line numbers below still hold) ·
**Status: READY** — operator-directed 2026-07-13 (namespace-remainder design
round folded: ADR 0014 #39 amendment + ADR 0016 Option B).

Two spec edits that touch the **same chapter** (`33-declarations.md` §3.2–§3.3,
§4, §5.5.1) ship as **one coherent enclave spec + conformance WP**, because
#36's re-export interlocks with #39's clash rule by design (both key on
canonical identity — ADR 0016 §1.2). Splitting them would double-edit §3.3 and
risk drift between the general-clash rule and the re-export carve-out that
rides inside it.

## Objective

Elaborate the settled ADR 0014 (#39 general-clash amendment) and ADR 0016
(Option B `export` declaration) into normative spec text in `33-declarations.md`
(§3.2/§3.3/§4/§5.5.1), `32-grammar.md` (§1), and `31-lexical.md` (§4), plus a
black-box conformance corpus that pins the identity-keyed clash + carve-out and
the `export` form's identity-preservation behaviour.

## Fixed inputs — SETTLED; the enclave elaborates, it does not re-derive

The two design decisions below are **closed**. The enclave writes spec text
*from* them; it does **not** reopen any fork, re-decide the spelling, or
re-derive the semantics.

- **ADR 0014 — MRES-6 general-clash amendment (#39) + MRES-9 status flip.**
  `docs/adr/0014-cross-package-resolution-and-fail-closed-collision.md`
  (`architect/work`). The #39 rule (SETTLED): **one order-independent,
  fail-closed rule over all four pairings** among `{local def,
  selective/renamed import, prelude}` — whenever more than one of these binds a
  single unqualified name to **DISTINCT canonical declarations**, resolution
  raises `AmbiguousReference`. Keyed on **canonical identity**, not spelling.
  **Same-identity carve-out:** two surface paths reaching **one** canonical
  identity are NOT a clash (idempotent). MRES-9 flips to "form DESIGNED AND
  DECIDED."
- **ADR 0016 — re-export surface form, Option B (#36/N5).**
  `docs/adr/0016-re-export-surface-and-canonical-identity.md`
  (`architect/work`), read in full. The form is the dedicated **`export`
  declaration** (operator pick, Option B, 2026-07-13):
  `export_decl ::= "export" ( module_path selection_list | export_item_list )`.
  - `export M (…)` = **facade republish**: names taken directly from `M`'s
    exports; **no prior `import M` required**, and it does **NOT** bind those
    names into the current module's body scope. Is itself a loader dependency
    edge to `M` (§3.2.1 path identity).
  - `export foo, Bar` = republish names **already in scope** (imported or
    local).
  - `export M (foo as bar)` / `export foo as bar` = renamed republish;
    identity unchanged.
  - Semantics (SETTLED): canonical-identity invariant (defined-at owns the
    one `GlobalId`; re-export republishes, mints **no** second identity;
    invariant under rename and hops); identity-keyed collision (re-export-site
    collision = two identities under one name → hard error; idempotent
    otherwise); MRES-4d instance-surface carry at the §5.5.1 boundary; **zero
    TCB**; abstract export composes (re-exporting an abstract type re-exports
    the opaque constant, constructors stay hidden). Reserve the `export`
    keyword — a **net +1 reserved keyword**: ADR 0015 retired `use` as a
    *surface form* but #37 keeps `use` **reserved** (not freed), so `export`
    does not reuse a freed slot. (The just-merged ADR 0014/0016 "budget-neutral"
    phrasing is being self-corrected by the Architect; write §31 to the +1
    reality.)

### Current-spec anchors (base `5efa317b`; **verify at pickup** — the enclave edits these exact sites)

- **§3.2 import forms** — `spec/30-surface/33-declarations.md:98-166`. Three
  import forms (`import M`, `import M as N`, `import M (…)`) at `:99-110`;
  boundary-header subsection §3.2.1 at `:145-166`. No `export` form exists.
- **§3.3 name resolution / clash rule** — `:168-195`. **Currently specs only
  two pairings**: the top-level **local×import** clash (`:176-182`,
  `AmbiguousReference`) and the **local×prelude** clash (prelude floor,
  `:187-191`, `AmbiguousReference`). Narrower lexical shadowing is orthogonal
  (`:183-186`). **Missing** (the enclave adds): the **import×import** and
  **import×prelude** pairings, the unified 4-pairing framing, the **same-
  identity carve-out**, and **re-export participation** — all keyed on canonical
  identity. (Note: ADR 0014's "already built" prose says intra-unit two-opens
  of one name already raise `AmbiguousReference` at the reference site; the spec
  text here does not yet state the full 4-pairing rule.)
- **§4 visibility** — `:197-236`. §4.1 private-by-default `pub` (`:199-215`);
  §4.2 abstract export = opaque constant (`:217-236`). **Missing:** interface =
  `pub` defs ∪ `export`ed names; the `defined-at` vs `re-exported-at`
  distinction; abstract export composes under `export`. Add as a §4.3 (or
  extend §4.1).
- **§32 declaration grammar** — `spec/30-surface/32-grammar.md:8-76`. `decl`
  productions `:21-39` (no `export_decl`); `import` production `:16-19`;
  anonymous-header prose `:71-76`. Add `export_decl` + supporting productions.
- **§31 keyword list** — `spec/30-surface/31-lexical.md:349-366`. Keyword
  block `:351-356` **still lists `use`** (line `:352`) — a straggler: `use` was
  retired (ADR 0015) but never removed here. The retirement paragraph
  `:358-364` names `view` retired but **omits `use`**. Add `export` as
  reserved; **also** fix the straggler (remove/retire `use`).
- **§5.5.1 program/package admission** — `:524-619`. The **re-export
  SPEC-NOW/BUILD-LATER bullet** at `:606-610` ("When public re-export lands
  after MRES-9, re-exporting a name also carries the instance surface …") is
  the one to **flip normative** for the MRES-4d carry. The closing
  SPEC-NOW/BUILD-LATER framing at `:616-619` and the §5.7 scope note
  (`:648-655`, "public re-export propagation … remain the clearly marked
  SPEC-NOW / BUILD-LATER rules") must be reconciled to reflect the flip.

## Mandated deliverable outline

### Part 1 — SPEC edits (each a concrete, pinned change)

1. **§3.3 (`33-declarations.md:168-195`) — the identity-keyed 4-pairing clash
   rule.** Replace the two-pairing framing with **one order-independent,
   fail-closed rule keyed on canonical identity** over `{local def,
   selective/renamed import, prelude}`: if more than one binds a single
   unqualified name to **distinct canonical declarations**, raise
   `AmbiguousReference`. Encode all four pairings explicitly (local×import,
   local×prelude, import×import, import×prelude). Encode the **same-identity
   carve-out**: two surface paths to **one** identity is NOT a clash
   (idempotent). Encode **re-export participation**: an `export`ed name entering
   a consumer's scope participates in this rule, keyed on identity (same-identity
   is not a clash; two identities under one name is). State the escape hatches
   (drop/rename a selective item, qualify, rename the local; prelude clash
   resolved only by renaming the local).
2. **§3.2 + §3.2.1 (`33-declarations.md:98-166`) — introduce the `export`
   form.** Add a §3.2 subsection (parallel to import forms) specifying the two
   `export` forms and their **import/export split**: `export M (…)` = facade
   republish (no prior import, does NOT bind into body scope, is itself a loader
   dependency edge to `M` via §3.2.1 path identity); `export foo, Bar` =
   republish names already in scope; renamed forms in both. State that a
   name-list item resolving to nothing in scope is an ordinary unresolved-name
   surface error.
3. **§32 (`32-grammar.md:21-39`) — the `export_decl` grammar + reserve
   `export`.** Add the pinned Option B productions to the `decl` list:
   ```
   export_decl ::= "export" ( module_path selection_list | export_item_list )
   selection_list   ::= "(" export_item ("," export_item)* ")"
   export_item_list ::= export_item ("," export_item)*
   export_item      ::= name ( "as" name )?
   ```
   Add prose that `module_path` uses §3.2.1 role-blind path identity; that no
   production mints a new `GlobalId`; that options A/C are not taken.
4. **§4 (`33-declarations.md:197-236`) — visibility of re-exports.** Add §4.3
   (or extend §4.1): **interface = own `pub` defs ∪ `export`ed names**; the
   **`defined-at` owns identity / `re-exported-at` republishes it** normative
   distinction; provenance is grep-recoverable from the `export` statement;
   **abstract export composes** (`export`ing an abstract type re-exports the
   opaque constant, constructors hidden — visibility travels with identity, not
   the surface path); `export foo` for a local = `pub foo`, mints no second
   identity (`pub`-on-definition stays idiomatic for locals).
5. **§5.5.1 (`33-declarations.md:606-610`) — flip the re-export bullet
   normative.** Convert the SPEC-NOW/BUILD-LATER re-export bullet to a
   normative MRES-4d rule: re-exporting a name carries the **canonical
   structure instances** whose `(class, head-type)` key's head-type or class is
   part of the re-exported public surface into an admitting consumer's
   **direct-use set**; property (Ω-valued) instances carry trivially; a
   transitive instance NOT carried by a re-export stays **coherence-only**
   (direct dispatch still requires admitting its defining package →
   `UnadmittedInstance`); the carry is an elaboration-time direct-use-set
   computation at the admission boundary — **no new TCB**. Reconcile the closing
   SPEC-NOW/BUILD-LATER framing (`:616-619`) and the §5.7 scope note
   (`:648-655`) so "public re-export propagation" is no longer listed as
   deferred.
6. **§31 (`31-lexical.md:351-364`) — reserve `export`; fix the `use`
   straggler.** Add `export` to the reserved keyword list. **Also** remove `use`
   from the keyword-*form* usage but record it as **still reserved** in the
   retirement paragraph (`:358-364`) alongside `view` — ADR 0015 retired `use`
   as a surface form while #37 keeps the token reserved (so `export` is a net
   +1 reserved keyword, not a reuse of a freed slot).

### Part 2 — CONFORMANCE fixtures (assert **specific** error variants, not
    `is_err`)

Home under `conformance/surface/declarations/` (verify path at pickup). Each
fixture asserts the named diagnostic variant / a positive identity check:

1. **import×import reject** — two selective imports binding one unqualified
   name to **distinct** identities → `AmbiguousReference`.
2. **import×prelude reject** — a selective import of a name distinct from a
   prelude binding of the same unqualified name → `AmbiguousReference`.
3. **same-identity NON-clash (positive).** `import M (foo)` and `import P
   (foo)` where `P` re-exports `M.foo` — one identity via two paths →
   **accepts** (no `AmbiguousReference`).
4. **re-export identity preservation (positive).** `export M (foo)` republishes
   `foo` with the **same `GlobalId`** as `M.foo` (assert identity equality, not
   just presence).
5. **renamed re-export.** `export M (foo as bar)` publishes surface name `bar`
   with `defined-at` identity `identity(M.foo)` unchanged.
6. **facade vs in-scope forms.** `export M (foo)` with **no** prior `import M`
   (facade; `foo` NOT bound into body scope) vs `import M (foo)` + `export foo`
   (in-scope republish; `foo` usable locally). Assert the body-scope
   distinction.
7. **re-export-site collision reject.** A module `export`ing two **different**
   identities under one surface name (e.g. local `pub const foo` = identity A
   and `export M (foo)` = identity B) → hard re-export-site error. Assert the
   specific variant.
8. **MRES-4d instance carry (positive).** A consumer that admits `P` dispatches
   a structure instance carried by `P`'s re-export **without** listing the
   instance's defining package `Q` → **accepts**; the complementary negative
   (dispatching a non-re-exported transitive instance) → `UnadmittedInstance`.
9. **abstract re-export keeps constructors hidden.** `export`ing an
   abstractly-exported type at a client — the type is visible, a
   `match`/construct on a hidden constructor is a surface error (name not in
   scope).

## Acceptance criteria (testable)

- **AC1 — §3.3 4-pairing rule normative.** §3.3 states one order-independent,
  fail-closed, identity-keyed rule covering all four `{local, import, prelude}`
  pairings, with the same-identity carve-out and re-export participation
  explicit; no residual two-pairing-only framing remains.
- **AC2 — `export` form specified end to end.** §3.2 (both forms +
  import/export split), §32 (`export_decl` grammar + reserved `export`), and
  §4 (interface = pub ∪ export; defined-at/re-exported-at; abstract composes;
  local `export` = `pub`) are mutually consistent and cite one another.
- **AC3 — §5.5.1 re-export bullet is normative.** The MRES-4d carry is stated
  as a current rule (not SPEC-NOW/BUILD-LATER); the closing framing and §5.7
  scope note are reconciled; the no-new-TCB invariant is explicit.
- **AC4 — §31 currency.** `export` is reserved; `use` is removed from the
  keyword block and recorded retired alongside `view`.
- **AC5 — conformance corpus green with specific variants.** All nine fixtures
  land; every reject asserts the **named** diagnostic (`AmbiguousReference`,
  `UnadmittedInstance`, the re-export-site collision variant, out-of-scope
  name), every positive asserts the identity/acceptance it targets (identity 4
  asserts `GlobalId` equality). No fixture asserts bare `is_err`.
- **AC6 — spec examples satisfy their own rules.** Every `export`/import
  example written into §3.2/§3.3/§4 obeys the clash rule and grammar it
  illustrates.
- **AC7 — no-regression = GREEN IN CI.** Workspace-green / no-regression is
  the GitHub CI `--locked` + conformance run at merge, **never** a local
  `--workspace` build (COORDINATION §12). Local checks are targeted only.

## Do-not-reopen guardrails

- **Spelling is DECIDED.** Option B (`export` declaration) is the operator's
  pick. Do **not** reopen Options A (`pub import`) or C (per-item `pub`), nor
  the rejected `pub const` transparent-alias shape. Do not re-surface a
  spelling fork.
- **Semantics are SETTLED.** The canonical-identity invariant, the
  identity-keyed clash rule + carve-out, and the MRES-4d carry are ADR-decided.
  The enclave writes spec text from them; it does not re-derive or re-decide.
- **Surface + elaboration ONLY — zero TCB.** Do **not** touch the kernel,
  `trusted_base()`, Cargo, or `.github/`. If any edit implies a kernel rule /
  judgment / former, it has mis-scoped.
- **Build is a SEPARATE follow-on.** This WP is **spec + conformance only**.
  The Language `export` elaboration (reserve the keyword in the parser, parse
  `export_decl`, republish the source identity, extend module-interface
  computation, enforce the import/export split, wire the MRES-4d carry into the
  §5.5.1 admission gate) is a **named separate follow-on WP** (ADR 0016 §6.3),
  which the Steward queues behind Language's kenfmt + #37. Do not implement it
  here.

## Follow-on (named, not in scope)

**Language build WP — `export` elaboration** (ADR 0016 §6.3): parser keyword +
`export_decl`, identity republish (no new `GlobalId`), module-interface
extension, import/export split enforcement (`export M (…)` does not bind into
the body), MRES-4d instance-surface carry wired into the §5.5.1 admission gate.
Queues behind kenfmt + #37 in Language's lane (Steward sequencing).
