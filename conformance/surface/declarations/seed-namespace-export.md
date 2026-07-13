# Namespace clash and re-export conformance seed

This seed is the black-box oracle for ADR 0014 MRES-6's general-clash
amendment and ADR 0016's dedicated `export` declaration. It extends the
existing module corpus without changing its homes for loader traversal,
ordinary import spelling, abstract-export representation, or source-package
admission.

The harness must drive source through the real loader, parser, module-interface
builder, name resolver, and admission gate. It must not pre-populate an export
map, forge a `GlobalId`, or call a collision helper with hand-built bindings.
Every identity assertion below compares the `GlobalId` emitted for the
defining declaration with the ID observed through the named surface path.

All cases are **RED UNTIL the Language namespace/export follow-on** unless a
case says otherwise. In particular, the current parser has no `export`
declaration, and the current import×import implementation records an ambiguity
for a later lookup instead of rejecting every latent clash at binding time.
The red label names an absent producer; it is not permission to accept a parse
error or an earlier unrelated failure.

## A. Identity-keyed general clashes

### surface/declarations/import-import-distinct-identities-reject-latently

- rule: ADR 0014 MRES-6 general-clash amendment; `33 §3.3`
- fixture: modules `M` and `N` each publicly define a distinct `foo`. A client
  writes `import M (foo)` and `import N (foo)`. Run both import orders. In each
  arm, declare no later reference to `foo`.
- expect: both arms reject at binding time with
  `AmbiguousReference { name = "foo", sources }`, where `sources` contains the
  canonical origins `M.foo` and `N.foo`. The source order may not select a
  winner, and the unused name may not defer the error to a lookup.
- discriminator: changing only the second selection to `foo as nfoo` accepts;
  bare `foo` resolves to `identity(M.foo)`, and bare `nfoo` resolves to the
  distinct `identity(N.foo)`. This rejects last-import-wins, first-import-wins,
  and use-site-only implementations.

### surface/declarations/import-prelude-distinct-identities-reject-latently

- rule: ADR 0014 MRES-6/MRES-10; `33 §3.3`
- fixture: module `M` publicly defines a user declaration named `Bool`, whose
  `GlobalId` is distinct from the registered prelude `Bool`. A client writes
  `import M (Bool)` and never references bare `Bool` afterward. The prelude
  remains installed in both arms.
- expect: binding rejects with
  `AmbiguousReference { name = "Bool", sources }`; `sources` identifies both
  `M.Bool` and the registered prelude declaration. It is not `UnboundName`, a
  warning, or an implicit local/import preference.
- discriminator: changing only the selective item to `Bool as MBool` accepts.
  `MBool` resolves to `identity(M.Bool)` while `Bool` still resolves to the
  prelude `GlobalId`. No prelude opt-out or environment mutation participates.

### surface/declarations/same-identity-through-two-imports-is-not-a-clash

- rule: ADR 0014 MRES-6 same-identity carve-out; ADR 0016 §1.1–§1.2
- fixture: `M` publicly defines `foo`. Facade module `P` contains only
  `export M (foo)`. A client writes both `import M (foo)` and
  `import P (foo)`, then defines a value whose body is bare `foo`.
- expect: accepted in both import orders. The two bindings and the body
  reference all equal `identity(M.foo)`; no second global is allocated and no
  `AmbiguousReference` is emitted.
- discriminator: replace only `P`'s export source with a different public
  `N.foo`. The same client now rejects with `AmbiguousReference`. Thus the
  positive cannot pass merely by deduplicating equal spellings or silently
  choosing one import.

## B. Re-export identity and scope

### surface/declarations/facade-reexport-preserves-global-id

- rule: ADR 0016 §1.1/§2.1; `33 §3.2`/`§4`
- fixture: `M` publicly defines `foo`; a distinct facade `P` has no prior
  import of `M` and writes `export M (foo)`. A client imports `P (foo)` and
  references it in a transparent value body.
- expect: `P`'s public-interface entry for `foo`, the client's binding, and
  the body constant all equal the exact `GlobalId` of `M.foo`. The flat `Σ`
  contains no declaration owned by `P` for `foo`; `trusted_base()` is
  unchanged.
- discriminator: presence under `P.foo` is insufficient. A wrapper or
  transparent alias that allocates a second ID fails the equality and flat-Σ
  assertions even if its value happens to unfold to `M.foo`.

### surface/declarations/renamed-reexport-preserves-defined-at-identity

- rule: ADR 0016 §1.1/§2.1; `33 §3.2`/`§4`
- fixture: `M` publicly defines `foo`; facade `P` writes
  `export M (foo as bar)`. A client selectively imports `P (bar)` and uses
  bare `bar` in a transparent value body.
- expect: `P` exports surface name `bar`, does not export `foo` under that
  spelling, and maps `bar` to `identity(M.foo)`. The client binding and body
  constant carry that same ID. Provenance remains `defined-at M.foo` and
  `re-exported-at P as bar`.
- discriminator: a fresh `P.bar` ID, an interface entry still named `foo`, or
  provenance that reports `P` as the defining module fails. Renaming changes
  spelling only.

### surface/declarations/facade-and-in-scope-export-have-distinct-body-scope

- rule: ADR 0016 §2.1 import/export split; `33 §3.2`
- fixture: use the same public `M.foo` in three source runs:
  1. `module P { export M (foo) }` with no prior import;
  2. the same facade plus `const localUse : Nat = foo` in `P`'s body;
  3. `module P { import M (foo); export foo; const localUse : Nat = foo }`.
- expect: run 1 accepts, publishes `P.foo`, and its interface entry equals
  `identity(M.foo)`. Run 2 rejects the body declaration specifically with
  `UnboundName { name = "foo" }`: the facade form is a loader edge and
  interface operation, not a body binding. Run 3 accepts; both `localUse` and
  `P.foo` resolve to `identity(M.foo)`.
- discriminator: the controlled negative differs from the facade success only
  by the body lookup. A producer that treats facade export as an import accepts
  run 2; one that treats in-scope export as facade-only rejects run 3.

### surface/declarations/reexport-site-distinct-identities-collide

- rule: ADR 0016 §1.2/§2.1; `33 §3.3`/`§4`
- fixture: `M` publicly defines `foo`. Module `P` defines its own public
  `foo`, then writes `export M (foo)`. Run the reverse declaration order too.
  The two declarations have distinct canonical IDs.
- expect: both orders reject at interface construction with the structured
  **`ReExportCollision`** oracle
  `{ surface_name = "foo", existing = identity(P.foo),
  incoming = identity(M.foo) }`. The diagnostic retains both defined-at
  origins, is raised at the re-export site, and never reaches the kernel.
  `ReExportCollision` pins the Language error-variant spelling for this
  previously unbuilt diagnostic.
- discriminator: replace `P`'s local declaration with an import of the same
  `M.foo`, then export it under `foo`; that arm accepts idempotently and exposes
  one interface entry with `identity(M.foo)`. This distinguishes identity from
  spelling and prevents a blanket duplicate-surface-name rejection.

## C. Visibility and instance-surface carry

### surface/declarations/reexport-carries-only-its-public-instance-surface

- rule: ADR 0014 MRES-4c/MRES-4d and ADR 0016 §1.3; `33 §5.5.1`
- fixture: source package `Q` owns structure classes `Render` and `HiddenMark`,
  head types `Widget` and `Hidden`, and the sole orphan-valid canonical
  instances for `(Render, Widget)` and `(HiddenMark, Hidden)`. Source package
  `P` admits `Q` for its own build and contains this facade:

  ```ken
  export Q (Render, Widget)
  ```

  This is the only `P → Q` loader edge; `Q` has no edge to `P`, so the source
  graph is acyclic. A program admits only `P`, imports `P (Render, Widget)`,
  and forces real implicit dispatch of `Render Widget`.
- expect: the dispatch accepts without `admits Q`. Its resolved dictionary is
  the canonical Q-defined instance, with its original `GlobalId`, canonical
  key, and `defining_package = Q`. The harness also verifies `P` did not mint a
  wrapper dictionary or a second instance registration.
- complementary negative: with the same package graph and `admits P`, the
  program imports `Q (HiddenMark, Hidden)` only to make the ordinary names
  available and dispatches `HiddenMark Hidden`. Because `P` did not re-export
  either key participant, dispatch rejects specifically with
  `UnadmittedInstance { defining_package = Q, class = "HiddenMark",
  head_type = "Hidden", instance_id = identity(Q's instance) }`.
- discriminator: the positive and negative share the same transitive
  coherence closure. A gate that admits the whole closure accepts the negative;
  a gate that ignores carried surfaces rejects the positive. Ordinary import
  affects nameability only and does not itself admit Q's instances.

### surface/declarations/abstract-reexport-does-not-widen-constructors

- rule: ADR 0016 §1.1/§4; `33 §4.2–§4.3`
- fixture: module `M` has the existing abstract interface
  `pub data Token = MkToken`, which publishes the opaque `M.Token` identity and
  withholds `MkToken`. Facade `P` writes `export M (Token)`. A client imports
  `P (Token)`. One arm uses `Token` in a type position; a controlled second arm
  adds `match t { MkToken |-> 0 }`.
- expect: the type-only arm accepts, and its `Token` binding equals the opaque
  `identity(M.Token)`. The match arm rejects at the surface with
  `UnresolvedCon { name = "MkToken" }`; `MkToken` appears in neither `P`'s
  interface nor the client scope. The failure is not `TypeMismatch` or
  `KernelRejected`.
- discriminator: the only added operation is the constructor pattern.
  Re-exporting the type cannot reconstruct, widen, or separately publish the
  hidden constructor; a producer that copies all defining-module members into
  `P` accepts the wrong arm.

## Coverage map

- **General clash, all missing pairings:**
  `import-import-distinct-identities-reject-latently` and
  `import-prelude-distinct-identities-reject-latently`.
- **Same-identity carve-out:**
  `same-identity-through-two-imports-is-not-a-clash` and the idempotent control
  in `reexport-site-distinct-identities-collide`.
- **One defined-at identity through facade and rename:**
  `facade-reexport-preserves-global-id` and
  `renamed-reexport-preserves-defined-at-identity`.
- **Import/export split:**
  `facade-and-in-scope-export-have-distinct-body-scope`.
- **Identity-keyed interface collision:**
  `reexport-site-distinct-identities-collide`.
- **MRES-4d direct-use carry and its coherence-only complement:**
  `reexport-carries-only-its-public-instance-surface`.
- **Abstract visibility composes:**
  `abstract-reexport-does-not-widen-constructors`.

## Cross-case consistency sweep

- Every import or export path is a surface path to one defined-at `GlobalId`.
  None allocates a declaration in the flat `Σ`; identity equality, not equal
  spelling or definitional equality, is the positive oracle.
- Every clash is checked before kernel admission and is order-independent.
  Distinct identities reject even if unused; the same identity is idempotent
  even when reached through two modules.
- Facade export changes the public interface and loader graph only. It does not
  alter body scope. In-scope export requires an existing body binding and does
  not create a second one.
- Public name carry and instance carry use the same re-export surface. The
  positive MRES-4d arm grants only the canonical instance whose key is carried;
  the complementary Q instance remains coherence-only and reaches the
  structured `UnadmittedInstance` gate.
- Visibility travels with identity. The abstract type remains the original
  opaque constant through `M → P → client`; the missing constructor remains a
  surface-resolution failure and never becomes a kernel visibility feature.

## Build-forward: Language namespace/export follow-on

The follow-on must add the reserved `export` token and grammar, source-driven
facade and in-scope export producers, identity-preserving module-interface
entries, binding-time general-clash checks, the structured
`ReExportCollision` variant, and MRES-4d direct-use carry. It must exercise
these cases through ordinary source entry units and the N2 loader. Parse errors,
hand-built export maps, copied `GlobalId`s, or test-only admission flags do not
discharge the red gates.

The follow-on remains surface and elaboration only: no kernel declaration,
`trusted_base()` entry, Cargo feature, or persisted package-manager artifact is
authorized by this seed.
