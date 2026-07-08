# SURF-named-proof-claims — attached proof declarations and proof references

**Steward frame -> Spec enclave, then Team Language.** Owner after design:
Language. Gate: Spec/conformance design approval for syntax/meaning,
Architect boundary review, Language QA, and CI.

**Status:** framed, not kicked off.
**Size:** M. **Risk:** medium — surface/elaboration work over proof terms,
with high readability value and low kernel risk if kept as ordinary proof
definitions.

## Trigger

Catalog review surfaced a readability and proof-usability gap around small
functions such as `list_append`. A signature like:

```ken
fn list_append (A : Type) (xs : List A) (ys : List A) : List A = ...
```

states the executable shape, but not the semantic guarantee a human reader
expects: the result contains the elements of `xs` followed by the elements of
`ys`. Pushing the guarantee into the return type can be correct, but it also
changes the function's use surface. For ordinary executable functions, the
preferred human form is:

```ken
prop AppendsTo (A : Type) : List A -> List A -> List A -> Omega where
  nil :
    (ys : List A) ->
    AppendsTo A nil ys ys

  cons :
    (x : A) ->
    (xs ys zs : List A) ->
    AppendsTo A xs ys zs ->
    AppendsTo A (cons x xs) ys (cons x zs)

fn list_append (A : Type) (xs : List A) (ys : List A) : List A = ...

proof appends for list_append
  (A : Type) (xs : List A) (ys : List A)
  : AppendsTo A xs ys (list_append A xs ys) = ...
```

A downstream proof should be able to reference that proof term explicitly:

```ken
proof caller_property for some_caller
  (A : Type) (xs : List A) (ys : List A)
  : SomeCallerClaim A xs ys =
    ... (proof appends for list_append) A xs ys ...
```

The reference phrase resolves to the canonical attached-proof path:

```ken
list_append::appends
```

The two spellings have different jobs: `proof appends for list_append` is the
human declaration/reference form, while `list_append::appends` is the stable
path used for export, import, diagnostics, and desugared terms.

By contrast, this WP intentionally does not require changing the executable
result type to carry the claim:

```ken
fn list_append (A : Type) (xs : List A) (ys : List A)
  : (result : List A ** AppendsTo A xs ys result) = ...
```

That dependent return style remains available when callers should receive a
strengthened value directly. Attached proofs target the case where the ordinary
runtime result should stay a plain `List A`, but the semantic guarantee should
still be available to proof clients.

The explicit repeated telescope is intentional. This should be rejected if the
subject signature changes and the proof declaration is not updated:

```ken
proof appends for list_append
  (A : Type) (xs : List A)
  : AppendsTo A xs ys (list_append A xs ys) = ...
```

Callers should then be able to refer to that proof explicitly when their own
correctness depends on it, without pretending the function's runtime return type
is something other than `List A`.

## Objective

Design and implement a surface vocabulary for proposition-oriented catalog
source:

1. A `prop` declaration names a reusable proposition family, classified in
   `Omega`.
2. A `proof ... for ...` declaration states and inhabits a proposition about an
   existing function/value.
3. The proof is a real proof term checked by the existing kernel/prover path,
   not metadata, a comment, or a trusted annotation.
4. The original function keeps its ordinary computational return type.
5. A caller can explicitly reference the attached proof in another proof.

The design goal is source that reads like literature: the function says what it
computes, the `prop` says what semantic vocabulary is being used, and the nearby
proof says what semantic claim has been established.

## Settled direction

- **Add `prop` as surface sugar.** `prop` is the proposition-family analogue of
  `data`: it declares an `Omega`-classified claim shape. It must elaborate to
  existing checked machinery for admissible proposition families; it does not
  add a kernel declaration class or trusted proposition table.
- **This is syntactic sugar over ordinary proof terms.** Do not add a kernel
  declaration class or trusted proof table. The elaborated result must be an
  ordinary named proof definition with a stable path.
- **The proof is load-bearing.** It must be usable by downstream proofs. A
  comment or documentation-only claim is not acceptable.
- **The function return type is not automatically strengthened.** The point is
  to preserve a simple executable signature while exporting a separately named
  proposition for proof clients.
- **The proof should live in the subject's namespace.** The discussion favored a
  reference style like `list_append::appends` for proof use, while the
  declaration reads `proof appends for list_append`. The declaration spelling
  and reference spelling need not be identical if the relation is obvious and
  stable.
- **`::` is the reference punctuation.** The canonical attached-proof path is
  `subject::proof_name`, e.g. `list_append::appends`. This marks the proof as a
  named item in the subject's attached-proof namespace, not as record projection
  or decimal syntax.
- **Do not use `use` as the proof-reference keyword.** `use` already carries
  import meaning in the surface language. Human source should use the same
  proof-selector phrase as the declaration: `proof appends for list_append`.
  That phrase resolves to the canonical path `list_append::appends`.
- **Repeat the subject signature explicitly.** Generic attached proofs must
  restate the subject's explicit telescope rather than silently inheriting it.
  This is noisier, but it makes the proof's shape readable at the proof outset,
  and a subject signature change should force every attached proof to be
  revisited through a mismatch error.
- **Multiple proofs per subject are unordered.** A subject may have many
  attached proof names, but duplicate names on the same subject are errors and
  declaration order must not affect resolution or meaning.
- **Same-subject proofs are independent in this WP.** One attached proof for a
  subject must not depend on another attached proof for the same subject,
  directly or through an ordinary helper proof. If two claims need a shared
  lemma, factor that lemma out as an ordinary named proof rather than making the
  attached proofs depend on each other.
- **Do not use operational/WP names.** Proof names are public source surface.
  They should be durable semantic names such as `appends`, `assoc`, `left_unit`,
  or `preserves_length`.
- **Named props should be semantic vocabulary.** It is acceptable for a tiny
  teaching example such as `list_append` / `AppendsTo` / `appends` to have
  parallel recursive structure. That should not become the general pattern of
  writing propositions that merely restate one implementation step-for-step.
  A proposition should name reusable domain semantics that another
  implementation, proof, or caller could consume independently.

## `AppendsTo` in the example

`AppendsTo` is not a built-in. It is the named generic proposition family that
states what it means for one list to be exactly the concatenation of two others:

```ken
prop AppendsTo (A : Type) : List A -> List A -> List A -> Omega where
  ...
```

The complete statement of `list_append` is therefore distributed across three
readable declarations:

- `prop AppendsTo` defines the claim shape.
- `fn list_append` computes a `List A`.
- `proof appends for list_append` proves that this implementation's result
  satisfies `AppendsTo`.

For small structural functions, the function, proposition, and proof may look
similar. The distinction is still useful: `AppendsTo` becomes reusable
vocabulary for other append implementations, length lemmas, parser/token
concatenation claims, or caller-side proofs. For larger catalog functions, the
proposition should usually be less algorithm-shaped: sortedness, permutation,
preservation, lookup correspondence, safety, parser soundness, and similar
semantic properties.

## Resolution model to design against

- The subject in `proof appends for list_append` resolves first, using ordinary
  value/global name resolution at the declaration or reference site.
- Attached proof lookup happens only after the subject is resolved. The proof
  name is looked up in that subject's attached-proof namespace.
- The stable path is `subject::proof_name`. If the subject is qualified, the
  proof path qualifies through the subject path, for example
  `Collections.List.list_append::appends`.
- Attached proof names do not enter the ordinary value namespace as bare names.
  A bare `appends` should not silently resolve to `list_append::appends`.
- If two visible subjects have the same unqualified name, ordinary subject
  resolution is ambiguous before attached-proof lookup. The author must qualify
  the subject.
- Export/import should preserve the pair: exporting a subject's public API may
  export its attached proof namespace; importing the subject makes attached
  proofs available through explicit proof-selector syntax. D0 should decide the
  exact manifest/export knob, but attached proofs must not become ambient
  imports.
- For this WP, attaching proofs to foreign subjects is out of scope. The
  subject and its attached proofs should be declared in the subject's owning
  module/package. Third-party extension-proof namespaces can be a later design
  if needed.
- Diagnostics should name both sides: the resolved subject path and the attached
  proof name, e.g. `duplicate attached proof list_append::appends`.

## Caller-use model to design against

Calling a function does not automatically expose its attached proofs. If a
caller proof needs the semantic claim, it asks for the proof explicitly with the
same arguments:

```ken
let zs = list_append A xs ys
let h = (proof appends for list_append) A xs ys
...
```

The proof term `h` is then passed like any ordinary proof term: to `transport`,
to a rewrite/congruence helper, to an induction step, or into a class-law field.
If the caller goal mentions the exact call `list_append A xs ys`, the proof's
result should line up directly. If the caller stores or transforms the result
under another expression, the caller still uses the existing equality/transport
machinery to move the attached claim to the goal shape. This WP should not add
ambient automation that discovers attached proofs automatically.

## Design details to pin in D0

The Spec enclave should pin these details before Language D1:

- **Name resolution.** Define how attached proof names are resolved, imported,
  shadowed, exported, and shown in diagnostics. The result must be stable enough
  for package APIs.
- **Same-subject dependency rule.** Because multiple proofs are unordered, one
  attached proof must not depend on another attached proof of the same subject
  in this WP. D0 should check whether transitive references can reintroduce an
  order dependency and either reject those cycles or route them through ordinary
  named proofs outside the subject namespace.
- **Caller use.** Show how a downstream proof imports/references an attached
  proof and passes it to transport, rewrite, induction, or a class law field.

## Deliverables

### D0 — Spec design

- Update the relevant surface/verification spec chapters with:
  - grammar for `prop` declarations;
  - grammar for attached proof declarations;
  - grammar/reference form for using attached proofs;
  - elaboration rule from `prop` to ordinary checked `Omega` proposition
    families;
  - elaboration rule to ordinary proof definitions;
  - name-resolution/export/import behavior;
  - rejection rules for missing subjects, duplicate proof names, wrong proof
    sort, subject-signature mismatch, same-subject proof dependency, and claims
    that do not type-check.
- Include a small source example centered on `list_append`:
  - `prop AppendsTo ...`;
  - `fn list_append ... : List A`;
  - `proof appends for list_append ...`, repeating the explicit subject
    telescope;
  - a caller proof that references the attached proof.
- Specify that `proof appends for list_append` in expression/reference position
  resolves to the canonical attached-proof path `list_append::appends`.

### D1 — Parser/AST/resolution

- Add the declaration syntax and AST node(s) for attached proofs.
- Add the `prop` declaration syntax and AST node, keeping it distinct from
  proof definitions and from computational `data`.
- Add parser tests for:
  - one `prop` declaration;
  - one proof attached to a function;
  - multiple proofs attached to the same function;
  - duplicate proof names rejected;
  - missing subject rejected;
  - explicit subject-signature mismatch rejected;
  - `proof <name> for <subject>` reference syntax accepted in a proof
    expression.

### D2 — Elaboration and proof use

- Elaborate an attached proof to an ordinary proof definition whose stable
  surface path is `subject::proof_name`.
- Elaborate `prop` to an existing checked `Omega` proposition-family
  declaration, with no new trusted core.
- Ensure downstream proofs can refer to the attached proof explicitly.
- Add an end-to-end test where a caller writes `proof appends for list_append`
  and the resolved proof path is `list_append::appends`.
- Ensure package export/import preserves the attached proof path.

### D3 — Catalog pilot

- Apply the feature to a small catalog function, preferably `list_append` in the
  collections package or a similarly small list function.
- Keep the executable function signature simple.
- Add the attached proof claim and a caller-side proof that consumes it.
- Update the package manifest to map the function to its attached proof claims.

## Acceptance Criteria

- **AC1 — Real proof term.** An attached proof elaborates to a real, checked
  proof definition; removing or breaking the proof body makes the package fail.
- **AC2 — Function signature preserved.** The executable subject keeps its
  original computational return type unless the author independently chooses a
  refinement/dependent return type.
- **AC3 — Caller-visible semantics.** A downstream proof can explicitly use
  the attached proof by its stable source path.
- **AC4 — No kernel delta.** No `crates/ken-kernel` change and no
  `trusted_base()` growth.
- **AC5 — Diagnostics are specific.** Duplicate proof names, missing subjects,
  wrong proof sort, and failed proof bodies report the attached proof name and
  subject.
- **AC6 — Catalog readability improves.** The pilot demonstrates that a human
  reader can find the function and its semantic guarantee from the source shape,
  not only from comments or a manifest.

## Guardrails

- Do not make attached proofs ambient automation in this WP. Explicit reference
  by callers is enough; automated proof search over attached claims can be a
  later WP if needed.
- Do not replace refinements or dependent result types. Those remain the right
  tool when callers should receive a strengthened value by type.
- Do not make proof declarations mere metadata. If the claim is not inhabited by
  a proof term, it is not an attached proof.
- Do not overfit to lists. `list_append` is the teaching example; the mechanism
  must work for ordinary functions, class methods, records, and later catalog
  laws where appropriate.

## Dependencies and sequencing

- **Can start:** after the current NC executable sequence is closed and the
  catalog frontier is selected.
- **Runs before:** broad catalog refinement that wants functions to expose
  semantic claims beside executable signatures.
- **Feeds:** collections/list refinement, lawful class law presentation, and the
  broader catalog style goal that small functions teach Ken idiom through their
  signatures plus attached proofs.
- **Does not block:** current catalog functionality work. Until this lands,
  catalog packages should continue using ordinary named proof definitions with
  durable names.
