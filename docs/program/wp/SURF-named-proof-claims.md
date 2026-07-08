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
fn list_append (A : Type) (xs : List A) (ys : List A) : List A = ...

proof appends for list_append :
  AppendsTo A xs ys (list_append A xs ys) = ...
```

Callers should then be able to refer to that proof explicitly when their own
correctness depends on it, without pretending the function's runtime return type
is something other than `List A`.

## Objective

Design and implement a surface form for **named proof claims attached to a
definition**:

1. A `proof ... for ...` declaration states and inhabits a proposition about an
   existing function/value.
2. The proof is a real proof term checked by the existing kernel/prover path,
   not metadata, a comment, or a trusted annotation.
3. The original function keeps its ordinary computational return type.
4. A caller can explicitly reference the attached proof in another proof.

The design goal is source that reads like literature: the function says what it
computes, and the nearby proof says what semantic claim has been established.

## Settled direction

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
- **Do not use operational/WP names.** Proof names are public source surface.
  They should be durable semantic names such as `appends`, `assoc`, `left_unit`,
  or `preserves_length`.

## Open design questions

The Spec enclave should answer these before Language D1:

- **Reference punctuation.** Is `subject::proof_name` the right selector, or is
  another punctuation clearer for "semantic claim attached to subject"? Dot
  suggests record/module projection; `::` suggests namespace/path. The chosen
  spelling must not collide confusingly with existing module paths, decimal
  syntax, or future record projection.
- **Reference keyword.** The discussion found `use list_append::appends` highly
  readable in caller proofs, but `use` already exists as an import form in
  `spec/30-surface/32-grammar.md`. Decide whether contextual `use` is acceptable
  inside proof expressions, or choose a different proof-reference form.
- **Name resolution.** Define how attached proof names are resolved, imported,
  shadowed, exported, and shown in diagnostics. The result should be stable
  enough for package APIs.
- **Multiple proofs per subject.** A function may carry several claims:
  `appends`, `length`, `assoc`, `identity`, etc. Specify ordering and duplicate
  rejection.
- **Generic parameters.** Decide whether the proof declaration repeats the
  subject telescope explicitly, inherits it implicitly, or supports both. The
  readable source form should avoid surprising implicit binders.
- **Caller use.** Show how a downstream proof imports/references an attached
  proof and passes it to transport, rewrite, induction, or a class law field.

## Deliverables

### D0 — Spec design

- Update the relevant surface/verification spec chapters with:
  - grammar for attached proof declarations;
  - grammar/reference form for using attached proofs;
  - elaboration rule to ordinary proof definitions;
  - name-resolution/export/import behavior;
  - rejection rules for missing subjects, duplicate proof names, wrong proof
    sort, and claims that do not type-check.
- Include a small source example centered on `list_append`:
  - `fn list_append ... : List A`;
  - `proof appends for list_append ...`;
  - a caller proof that references the attached proof.
- Decide whether `use` is acceptable as the proof-reference keyword despite the
  existing import use, and record the reason.

### D1 — Parser/AST/resolution

- Add the declaration syntax and AST node(s) for attached proofs.
- Add parser tests for:
  - one proof attached to a function;
  - multiple proofs attached to the same function;
  - duplicate proof names rejected;
  - missing subject rejected;
  - reference syntax accepted in a proof expression.

### D2 — Elaboration and proof use

- Elaborate an attached proof to an ordinary proof definition whose stable
  surface path is the chosen subject/proof selector.
- Ensure downstream proofs can refer to the attached proof explicitly.
- Add an end-to-end test where a caller uses `list_append::appends` or the
  chosen equivalent to discharge a small property.
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
