# KM-sigma-projection-execution

**Owner:** Mechanism lane, initial owner Language for interpreter/elaborator
audit and build; Kernel becomes co-owner only if D0 proves the kernel checker or
conversion rules must change.
**Reviewer:** Architect mandatory. Language QA mandatory. Kernel QA mandatory
if any `crates/ken-kernel` path changes.
**Branch:** `wp/KM-sigma-projection-execution` when released.
**Status:** Steward frame. Ready to release; blocks CAT-5 D3.
**Size:** M. **Risk:** medium/high until D0 classifies the TCB touch.

## 0. Trigger

CAT-5 D3 was re-released after `KM-literal-trust-accounting` landed at
`origin/main @ 9f3ac540be8676cb649a9cd109e1a81a0a4848e7`. Language then stopped
at D0 with no candidate head:

- implementer stop: `evt_24hyqzasbzndh`;
- leader re-derivation: `evt_2wvcfz3qpkxsr`;
- Architect ruling: `evt_7rq52cgqsphz8`;
- Language acknowledgments: `evt_5bcm3adbnmpp5`, `evt_7kaveh89ry67h`,
  `evt_16tnkk3q70ezt`.

The Boolean grammar/parser/printer surface can be written, but CAT-5 D3's
executable gates fail at the class-backed `Source` producer boundary. A
D1-style concrete fixture,

```ken
instance Source T { ... }
const s : Source = Source_instance_T
```

leaves `sourceBytes s` and `sourceLength s` evaluating to `Unknown`. Therefore
`parseBoolExpr s ...` cannot reduce far enough for checked round-trip witnesses
to pass.

Architect ruled that CAT-5 D3 stays held and that the immediate unblocker is
not dependent-constructor/GADT syntax. The narrow seam is executable projection
from concrete Sigma/class-backed records.

## 1. Objective

Make projection of concrete Sigma/class-backed record values compute the
demanded component without forcing unrelated proof-tail components, including
after transparent definition unfold.

The motivating client is a concrete class-backed `Source` value whose early
computational fields are executable and whose later proof fields may be
noncomputational. The result must let CAT-5 D3 resume unchanged against the
existing `Source` contract.

Acceptable implementation strategies include a projection-aware interpreter
path or a lazy pair representation. D0 must classify which layer is actually
responsible before implementation.

## 2. Scope

In scope:

- projection execution for `proj1`/`proj2` chains over concrete Sigma values;
- class/record projection lowering that becomes `proj1(proj2^idx(base))`;
- transparent unfold of class-instance definitions before projection;
- interpreter behavior for projection when only the requested field is needed;
- focused regression fixtures for class-backed `Source`;
- existing kernel convertibility/projection regression checks.

Out of scope:

- CAT-5 D3 implementation or changes to `packages/parsing`;
- weakening `Source` into a string-only gate or unconstrained constructor;
- fake CAT-5-local literal/trusted-base exceptions;
- dependent constructor binder syntax, GADT syntax, or parser changes;
- broad lazy evaluation for ordinary pairs unless D0 explicitly audits and pins
  the changed behavior.

## 3. Required D0 Audit

Before implementation, post a D0 audit that answers:

1. Does the failing `Source` projection reduce to an interpreter
   `Term::Proj1`/`Term::Proj2` chain over a transparent class-instance pair?
2. Does strict pair evaluation in `crates/ken-interp/src/eval.rs` force an
   unrelated noncomputational proof tail before projection can select the early
   field?
3. Is kernel conversion already able to beta-reduce the analogous projection
   through transparent constants? If not, what exact kernel rule is missing?
4. Can the fix stay interpreter/elaborator-only while preserving kernel
   re-checking and trusted-base accounting?
5. What existing whole-pair `Unknown` behavior is intentionally preserved?

The audit must name exact files/functions touched or ruled out.

## 4. Acceptance Criteria

- **AC1 -- concrete Source projection.** A concrete class-backed `Source`
  fixture evaluates `sourceBytes s` to concrete bytes and `sourceLength s` to a
  concrete `Nat`, even when later proof fields or `record_nil_val` are
  noncomputational.
- **AC2 -- proof tail stays noncomputational.** Projecting a proof field whose
  proof is not executable still evaluates to `Unknown` or the existing
  noncomputational representation. The fix must not manufacture proof evidence.
- **AC3 -- requested unknown stays unknown.** If the requested computational
  field itself is `Unknown`, the projection returns `Unknown`.
- **AC4 -- whole-pair behavior preserved or repinned.** Existing whole-pair
  strict/unknown behavior remains unchanged unless the WP explicitly audits it,
  documents the new rule, and adds regressions for both old and new surfaces.
- **AC5 -- kernel projection regressions green.** Existing conversion and
  projection tests still pass. If `crates/ken-kernel` changes, Architect and
  Kernel QA review the trusted-surface delta.
- **AC6 -- CAT-5 blocker classified.** The original D3 `Source` execution
  blocker is re-run or reconstructed. After this WP, `sourceBytes` and
  `sourceLength` no longer block D3; any remaining failure is named as a
  distinct follow-on.
- **AC7 -- forbidden scope empty.** No parser syntax, package-level `Source`
  weakening, CAT-5 D3 implementation, literal-accounting exception,
  `Cargo.lock`, `spec`, or `conformance` movement occurs unless explicitly
  rerouted by Steward/Architect.
- **AC8 -- workspace green.** Focused regressions and
  `scripts/ken-cargo test --workspace` pass.

## 5. Guardrails

- Preserve the CAT-5 package contract. This WP exists so CAT-5 does not route
  around the lower-layer execution seam.
- Do not make proof fields executable by erasure, proof irrelevance, `Axiom`,
  postulate, primitive, or trusted-base hiding.
- Do not fold the broad dependent-constructor/GADT surface feature into this
  mechanism. That feature is a separate language WP.
- Do not treat an interpreter-only execution improvement as a kernel proof. The
  kernel remains the proof authority.

## 6. Downstream

CAT-5 D3 remains held with no candidate head until this WP lands and closes.
After closure, Steward re-releases CAT-5 D3 unchanged on the landed base. The
Language team must rerun the original executable parse/print/format gates plus
the strict zero trusted-base-delta checks.
