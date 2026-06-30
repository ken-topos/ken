# Constant-time `@ct` discipline conformance — seed cases (Sec1ct)

Format: `../../README.md`. These pin the **`@ct` constant-time discipline** of
`spec/60-security/61-information-flow.md §5a` — Sec1ct elaborates the Sec1
*hook* (parse/attach/carry/reject-a-leakage-sink-flow) to the enforced
**discipline** (the `CT` axis §5a.1, the sealed `LeakSink` set §5a.2, the
`L-CT-SINK` rule §5a.3, the CT-promise/`Q` export §5a.4, declassify-ends-span
§5a.5, the honest split §5a.6/§H). They sit beside the Sec1 IFC seed
(`../ifc/seed-ifc.md`) and **absorb** its two `@ct`-hook cases (F1/F2 — see
"Hook absorption" below); they reuse Sec1's lattice, `pc` discipline,
declassify, and labeled effects — **extended, not rebuilt**.

> **Acceptance namespace.** These are **Sec1ct AC1–AC7** (the `@ct` discipline,
> this file's coverage map) — a *different* numbering from the Sec1 AC1–AC5 in
> `../ifc/seed-ifc.md`. Where this file says "AC*n*" it means **Sec1ct AC*n***.

Grounding (landed `§`-bodies on this branch, content-reconciled — not the plan):
`61 §5a.1`–`§5a.6` (the six-part discipline), `61 §7` (worked examples
`cmp`/`route`/`cmp_ok`/`ct_eq`), `61 §9` (the level table `CT`-factor rows + the
Sec1ct conformance note), `61 §H` (honest limits + the three `[Sec1-*]`
triggers); `36 §3.1` (the cross-workstream effect-sink contract: the `@ct` taint
row "a label whose sink is a distinguished `Vis` op", and branch-guard /
memory-index / variable-time-primitive as a distinguished effect sink);
`71 §2` (the assume-guarantee contract `guarantees` (`Q`) channel); `63 §5a`
(the discharge attestation — field #1 binds the `71` contract hash, field #4
carries `Ward`'s timing-validation result); `64 §4.2` (timing is
codegen/hardware-relative under a stated leakage model); `61 §4`/`62` + `25 §3`
(declassify is capability-gated/audited and shows in `trusted_base_delta`).

## Reading these cases — the Sec1ct-specific disciplines

**`@ct` is unary taint by typing; the observable is accept/reject — NEVER a V3
verdict (`61 §5a.3`).** `@ct` does **not** route through the by-proof relational
engine (`61 §5`, the strength-2 product-program path). A `@ct` leak is an
**elaboration type error** (reject); a clean program **accepts**, full stop. A
case mapping a `@ct` leak to `disproved`/`unknown` is a **mode-confusion bug**:
the verdict-mapping silence is pinned at source (`61 §5a.3`), foreclosing the
cross-mode trap. So every case below asserts **accept** or **reject**, never a
verdict tag and never a runtime two-run output diff.

**Taint orientation — the `[Sec1-dual]` discipline (`61 §5a.1`/`§H`).** `@ct` is
a **taint** axis: the **order-dual of confidentiality**, oriented exactly like
integrity (§2.2) — `ct⊥` is *safe* (low taint), `ct⊤ = @ct` is *the taint*, and
a **leakage sink demands `ct⊥`**. `@ct` joins **upward** (`L-COMBINE`: any `@ct`
input ⇒ `@ct` result — you cannot compute `@ct` away), and a sink carries
clearance `κ.ct = ct⊥`, so a `@ct` value (`ct⊤`) reaching it satisfies
`ct⊤ ⋢ ct⊥` → **reject**. Getting the order *backwards* (confidentiality-style,
sink demands `ct⊤`) **silently inverts** accept/reject. The orientation is held
by the **non-degenerate distinguishing pair** (AC1 rejects *while* AC4 accepts
on the **same branch shape**, CT-B1) — a real distinguishing case, **not**
green-vs-green.

**The `LeakSink` set is sealed and exhaustive-by-construction (`61 §5a.2`;
COORDINATION §7, the omission-hole discipline).** Exactly three members, each
with its exact trigger: `BranchGuard` (a non-`ct⊥` discriminant steers
`if`/`match`), `MemIndex` (a non-`ct⊥` data-dependent index feeds an indexing
`Vis` op), `VarTimePrim` (a non-`ct⊥` operand reaches a primitive flagged
`var-time` in its effect signature, `36 §3.1`). There is **no `_ => non-sink`
catch-all**: a new leaky op left unclassified must be a **compile error**, never
a silent non-sink. CT-A covers **one distinct member per case** (A1/A2/A3) so
the corpus exercises the **whole** set, not a single trigger three ways; the
sealedness itself is the structural net (cross-case sweep).

**N1 — `@ct` labels are erased before the kernel, so conformance is the SOLE net
(`61 §5a.6`/`§9`/`§H`).** A flow-rule bug — a flipped `CT` order, a dropped
`pc.ct`-join, a label-dropping `bind`/`incl`, a sink omitted from the sealed
set — emits a **well-typed core term the kernel accepts** while the CT
precondition is violated. The kernel **cannot** net it; the `L-CT-SINK` rule and
the `LeakSink` classification are **trusted**, and these flip cases are the
**only** backstop (the §H design-level argument + this discriminating corpus,
never the kernel) — identical in shape to Sec1's N1 for the confidentiality flow
rules (`../ifc/seed-ifc.md` group C).

**QA gate — route a REAL `@ct` value through a REAL sink, never predicate about
a synthetic label (the 2-team build-qa lesson).** A test that asserts
`is_ct(lit) && is_sink(op)` over hand-assigned literals **guards nothing** — it
predicates about the classification instead of exercising its downstream
consequence. Each negative case below **drives a real `@ct` value into a real
`LeakSink` op and observes the reject**; each positive case drives the same
value/shape past the same sink and observes the accept. The verdict **flips**
on the exact bug the case targets.

**Honest split — Ken owns `Q` (the source precondition), `Ward` owns the binary
(`61 §5a.6`/`§H`; `64 §4.2`, `63 §5a`).** The `L-CT-SINK` check gives the
**source-level** constant-time precondition `Q` (no `@ct` value steers a
`LeakSink`) — **not** constant-time *execution* (which needs CT-preserving
lowering + an empirical leakage-model validation, both `Ward`'s). **No case
asserts Ken proves constant-time execution** — that is the exact over-claim §H
forecloses (AC7). The CT-promise `Q` (CT-D1) is a **source-level** guarantee
clause, paired **1:1** with a `63 §5a` `Ward` discharge result.

**Tags.** `(soundness)` = a real constant-time-precondition commitment that must
never regress. `(oracle)` = an expected result to confirm against Ken's
reference elaborator once it exists; and per defer-spelling-not-concept the
**literal surface token `@ct`** stays `OQ-syntax`-deferred and the **literal
`Q`-clause field token** stays B1-/`71`-deferred — cases pin the value-set
(`{ct⊥, ct⊤}`) + the flow/contract invariants, **not** the surface/field tokens.

---

## CT-A. `@ct` steers a leakage sink → reject (the sealed set + `pc`) — AC1–AC3

> Strength-1, by typing. Each case drives a real `@ct` value (`ct⊤`) into a real
> `LeakSink` member; `L-CT-SINK` (`61 §5a.3`) rejects when
> `(ℓ.ct ⊔ pc.ct) ⋢ ct⊥`, naming `ℓ.ct`, `pc.ct`, and the sink site.

### security/ct/ct-value-steers-branch-guard-rejected
- spec: `61 §5a.3` (`L-CT-SINK`), `§5a.2` (`BranchGuard`), `36 §3.1`, `61 §7`
  (`cmp`)
- given: `cmp (k : Bytes @ ct) (g : Bytes)` whose body
  `branch_on (k[0] == g[0]) …` makes the `@ct` value `k` the **scrutinee** of a
  control-flow branch (a `BranchGuard` leakage sink)
- expect: **rejects** — `L-CT-SINK`: `(k.ct ⊔ pc.ct) = ct⊤ ⋢ ct⊥`, an IFC-CT
  static error naming `k.ct`, `pc.ct`, and the branch site
- why: (soundness) **AC1**, the dominant CT leak — a key steering a secret-
  dependent branch. Verdict **flip**: the same branch shape on a `ct⊥` value
  (or `k` un-annotated, hence `ct⊥`) **accepts** — right=accept / wrong=reject
  on `ct⊤ ⊑ ct⊥`. Routes a **real** `@ct` value into a **real** `BranchGuard`
  (QA gate), not a predicate over `is_ct`/`is_sink` literals. Absence-gate: goes
  green-vs-red under exactly the orientation-flip bug (a sink mis-built to
  demand `ct⊤`, `[Sec1-dual]`) — the bug N1 says the kernel cannot net.

### security/ct/ct-value-steers-memory-index-rejected
- spec: `61 §5a.3`, `§5a.2` (`MemIndex`), `36 §3.1`
- given: `lookup (i : Nat @ ct) (t : Array A)` whose body `t[i]` makes the
  `@ct` value a **data-dependent memory index** feeding an indexing `Vis` op
  (a `MemIndex` sink — data-dependent cache access)
- expect: **rejects** — `(i.ct ⊔ pc.ct) = ct⊤ ⋢ ct⊥`, naming the index site
- why: (soundness) **AC2**, the cache-timing leak (a secret-dependent table
  index). Verdict **flip**: a `ct⊥` index, or a constant-time full-table scan
  primitive, **accepts**. Exercises the **second** sealed-set member with a
  **distinct trigger** (data-dependent index, not a branch) — so the discipline
  nets index sinks, not only branches; the per-member split is what makes the
  sealed set's totality testable (sweep).

### security/ct/ct-value-into-var-time-primitive-rejected
- spec: `61 §5a.3`, `§5a.2` (`VarTimePrim`), `36 §3.1`
- given: `(k : Bytes @ ct)` passed as the operand of a primitive whose effect
  signature is flagged **`var-time`** (a naïve bignum compare/divide, a non-CT
  `==` — run time depends on the operand **value**), e.g. `naive_eq k g`
- expect: **rejects** — the `var-time` primitive's operand is the timing-
  relevant operand; `(k.ct ⊔ pc.ct) = ct⊤ ⋢ ct⊥`
- why: (soundness) **AC3**, the operand-value-timing leak. Verdict **flip**: the
  **same operand** into a **constant-time** primitive (`ct_byte_eq`, *not*
  flagged `var-time`) **accepts**. The discriminator is the **`var-time` effect-
  signature flag** (`36 §3.1`), **not** the syntactic operator — so the case
  exercises the signature-flag mechanism (the third sealed-set member), and a
  primitive that *is* var-time but whose signature **omits** the flag is the
  omission-hole the sealedness guards against.

### security/ct/ct-guarded-branch-implicit-leak-rejected
- spec: `61 §5a.3` (the `pc.ct`-aware rule, `L-OBSERVE` projected onto `CT`),
  `36 §3.1`
- given: a leakage op whose **own** operand is `ct⊥` placed **inside** a branch
  guarded by a `@ct` value — e.g.
  `if (k[0] == g[0] : Bool @ ct) then t[j] else t[j']` where the indices
  `j`, `j'` are themselves `ct⊥` (a `MemIndex`/`BranchGuard` op under a `@ct`
  `pc`)
- expect: **rejects** — branching on the `@ct` value raises `pc.ct = ct⊤` in
  **both** branches, so the inner op is caught even though its operand is `ct⊥`:
  `(ct⊥ ⊔ pc.ct) = ct⊤ ⋢ ct⊥`, the **implicit** CT flow
- why: (soundness) **AC1, the implicit-flow discriminator** (the analog of Sec1
  A3 `implicit-flow-pc-rejected`). The leak is the **timing of the choice**, not
  the inner datum. Verdict **flip**: a checker that **drops the `pc.ct`-raise**
  in `L-OBSERVE` wrongly **accepts** (the inner op's operand is `ct⊥`) —
  green-vs-red on exactly the `pc.ct` bug. Names the exact guard (the
  `pc.ct`-join), per the absence-gate.

## CT-B. The axes are independent — the `[Sec1-dual]` distinguishing pair (AC4)

### security/ct/secret-not-ct-branches-freely-accepted
- spec: `61 §5a.1` (the `CT` factor is independent of `Conf`/`Integ`), `61 §7`
  (`route`), `§H` (`[Sec1-dual]`)
- given: `route (p : Tag @ Secret) (g : Bytes)` whose body
  `if p == Admin then handleA else handleB` branches on a value that is
  `Secret` (confidentiality) but **not** `@ct` (so `p.ct = ct⊥`)
- expect: **accepts** — branching on `p` leaks nothing to a **timing** observer;
  `(p.ct ⊔ pc.ct) = ct⊥ ⊑ ct⊥`. (Its confidentiality is a *separate* product
  factor, constrained by the Sec1 `L-SINK`/`L-OBSERVE` cases, not by `@ct`.)
- why: (soundness) **AC4** — the `Conf` and `CT` axes are **orthogonal product
  factors** (`61 §5a.1`). This is the **non-degenerate distinguishing pair**
  that holds the `[Sec1-dual]` orientation: it **accepts** on the **same
  `BranchGuard` shape** where CT-A1 **rejects** — the *only* difference is
  `p.ct = ct⊥` vs
  `k.ct = ct⊤`. A `CT`-order flip (sink demands `ct⊤`) would invert **both**,
  rejecting this and accepting A1 — so A1∧B1 together pin the orientation
  (right-accept here / right-reject there), **not** green-vs-green.

## CT-C. Declassify ends the `@ct` span — the sole terminator (AC5)

### security/ct/declassified-ct-value-steers-sink-accepted-and-listed
- spec: `61 §5a.5` (declassify is the sole span terminator), `61 §4`/`62`
  (capability-gated/audited), `25 §3` (`trusted_base_delta`), `61 §7` (`cmp_ok`)
- given: `cmp_ok (k : Bytes @ ct) (d : Cap_declassify[ct⊤→ct⊥]) (g : Bytes)`
  that first `let k' = declassify d (ct_eq k g)` (so `k' : Bool @ ct⊥` — blinded
  via a CT compare, now safe to act on) and **then** `branch_on k' …` (the same
  `BranchGuard` sink as CT-A1)
- expect: **accepts** — post-authorised-declassify `k'` is `ct⊥`, so its
  influence into the sink is unconstrained `(ct⊥ ⊔ pc.ct) = ct⊥ ⊑ ct⊥`; **and**
  the `@ct` declassify authority appears in `trusted_base_delta`
- why: (soundness) **AC5** — declassify is the **sole** span terminator on the
  `CT` axis (no `constant_time{}` region). Verdict **flip** vs CT-A1: the **same
  sink on the same value**, the only difference the **authorised, audited**
  downgrade → accept here / reject there. The declassify **machinery** itself
  (capability-gating present-vs-absent → accept/reject, delta-completeness) is
  the **reused** `61 §4` mechanism already netted by `../ifc/seed-ifc.md` B1–B3
  — **not** re-derived here (subsume-don't-proliferate); this case pins only the
  **CT-axis-specific** behavior (`ct⊤ → ct⊥` ends the span) + that the `@ct` cap
  surfaces in the delta.

## CT-D. CT-in-parameter promise + the `Q` export (AC6)

### security/ct/ct-in-parameter-promise-checked-emits-Q
- spec: `61 §5a.4` (the CT-promise + `Q` export), `71 §2` (the `guarantees`
  (`Q`) channel), `63 §5a` (the `Ward` discharge attestation), `61 §7` (`ct_eq`)
- given: `ct_eq (k : Bytes @ ct) (g : Bytes) : Bool @ ct` **declaring**
  constant-time-in-`k`, with body `fold_and (map2 ct_byte_eq k g)` (no
  `LeakSink` op's timing-relevant operand depends on `k`)
- expect: **accepts** AND **structurally emits a source-level guarantee clause**
  onto the `71` `guarantees` (`Q`) channel — "constant-time-in-`k` at the source
  level, relative to the stated leakage model" — content-hashed into the `71`
  contract. **Assert the emitted boundary obligation structurally** (the clause
  is present, names parameter `k`, is a **source-level precondition** not a
  timing guarantee, and pairs 1:1 with a `63 §5a` discharge field), **not merely
  "accepts."** A sibling body that did `branch_on k[0]` instead is **rejected**
  (the promise is checked **by typing**, the §5a.3 rule applied with the named
  parameter as the `@ct` source) — the **flip**.
- why: (soundness) **AC6** — the promise is a *checked* obligation, not a
  decoration, and an accepted promise **produces** a `Q` clause the boundary can
  rely on. The structural claim (clause present + well-formed) guards against
  the silent-omission hole: an accepted function emitting **no** clause would
  read "constant-time-promised" to a consumer while supplying nothing (the V2
  completeness-of-extraction backstop, in the CT domain). **`(oracle)`:** the
  **literal field-token spelling** of the `Q` clause is B1-/`71`-deferred
  (defer-spelling-not-concept) — pin the concept (a `Q`-channel clause naming
  the parameter), the value-set/invariants (source-level, 1:1 with a `63 §5a`
  `Ward` discharge, relative to a leakage model), and **content-hash stability**
  (renaming the field after binding is a contract break, `63 §5a` #1); do
  **not** freeze the token.

## CT-E. Honest limits — no over-claim (AC7)

### security/ct/timing-guarantee-delegated-not-claimed
- spec: `61 §5a.6`/`§H` (the proven-vs-delegated split + the three `[Sec1-*]`
  triggers), `64 §4.2`, `63 §5a`
- given: the full `@ct` corpus above (A–D), read as a body of claims about what
  Ken proves
- expect: **no case asserts Ken proves constant-time *execution*.** Ken proves
  only the **source-level precondition `Q`** (no `@ct` value steers a
  `LeakSink`), **by typing**, with the `L-CT-SINK` rule + `LeakSink`
  classification **trusted** (labels erased, N1). The **timing guarantee** is
  **delegated to `[Ward]` + the toolchain** under a stated leakage model
  (`64 §4.2`, `63 §5a`) — and §H carries the three kernel-blind surfaces as
  **named scoped work**: `[Sec1-dual]` (the `CT`/integrity dual-ordering check),
  `[Sec1-launder]` (real `Vis`-routed `CT`-index preservation), `[Sec1-reduce]`
  (`Φ_post` reduction-faithfulness — named for the strength-2 increment; it does
  **not** gate the `@ct` unary path)
- why: (AC7, the scope-honesty rule) over-claiming a delegated guarantee is
  itself the security failure. A case asserting "well-typed ⇒ constant-time
  execution," or "Ken proves no timing leak," would over-claim past Ken's locked
  granularity — a **wrong case**. This case pins the honest scope: the
  precondition is real and checked (the trusted-by-typing boundary), the timing
  is `[Ward]`'s, and the three reify-triggers are **present, not silent**
  (the honest-limits non-silence rule). Mirrors `../ifc/seed-ifc.md` G2
  (the by-typing meta-theorem is the trusted boundary).

---

## Coverage map (Sec1ct AC → cases)

- **AC1** `@ct` branch-guard → reject (+ the implicit `pc.ct` flow) → CT-A1,
  CT-A4.
- **AC2** `@ct` memory-index → reject → CT-A2.
- **AC3** `@ct` var-time-primitive → reject → CT-A3.
- **AC4** `Secret`-but-not-`@ct` branches freely → accept (the `[Sec1-dual]`
  distinguishing pair) → CT-B1.
- **AC5** declassify ends the span (accept + in delta) → CT-C1.
- **AC6** CT-in-parameter promise checked + emits `Q` (structural) → CT-D1.
- **AC7** honesty — timing delegated, no over-claim, three `[Sec1-*]` triggers
  named → CT-E1 (and the `(oracle)`/defer-tags throughout CT-D1, CT-E1).

## Cross-case consistency sweep (pre-handoff gate)

- **The `@ct`-reject class {A1, A2, A3, A4}** — all four agree: a `ct⊤` value
  (or a `ct⊥` value under a `ct⊤` `pc`) reaching **any** sealed `LeakSink`
  member is a **reject**, observable = elaboration accept/reject, **never** a V3
  verdict (`61 §5a.3`); a case mapping a `@ct` leak to `disproved`/`unknown`
  is a **mode-confusion bug**. The three sink members are **exhaustively**
  covered (A1 `BranchGuard`, A2 `MemIndex`, A3 `VarTimePrim`) with **distinct
  triggers**, so the sealed set's totality is exercised, not one trigger
  thrice; the sealedness (no `_ => non-sink` catch-all) is the structural net
  for the omission-hole (N1 — an unclassified leaky op the kernel cannot see).
- **The accept class {B1, C1, D1-accept-arm}** — all agree: a value that is
  **`ct⊥`** at the sink **accepts**, whether because it was never `@ct` (B1,
  `Secret`-but-not-`@ct`), or was `@ct` and **authorisedly declassified** to
  `ct⊥` (C1), or is the clean body of a CT-promise that steers nothing (D1).
- **The `[Sec1-dual]` orientation pin — {A1, B1} as a distinguishing pair.** A1
  rejects **while** B1 accepts on the **same `BranchGuard` shape**, differing
  only in `ct⊤` vs `ct⊥`. A flipped `CT` order inverts **both** — so the pair,
  not either case alone, holds the orientation (non-degenerate, not
  green-vs-green; the `[Sec1-dual]` net).
- **The N1 trust-boundary cases {A1, A2, A3, A4, C1} are the SOLE net.** `@ct`
  labels are **erased** before the kernel, so an over-accepting `L-CT-SINK`
  (flipped order, dropped `pc.ct`, a sink omitted from the sealed set) emits a
  **kernel-valid** core — the discriminating flips here, **not** the kernel, are
  the trust boundary (the Sec1 N1 lesson, `61 §9`/`§H`, in the `CT` axis).
- **Absence-gate cases {A1, A3, A4, C1, D1, E1}** — each names its exact guard
  and passes "would this also be absent/accepted under the precise bug it
  targets?": A1 the `CT` orientation (`ct⊤ ⋢ ct⊥`); A3 the `var-time`
  signature flag; A4 the `pc.ct`-join; C1 the declassify-`ct⊥` span end +
  delta entry; D1 the **emitted** `Q` clause (a producer omission the consumer
  cannot see); E1 the three **present** `[Sec1-*]` reify-triggers + the absent
  CT-execution claim.

## Hook absorption (reconcile note)

This file **supersedes** the two `@ct`-*hook* cases in `../ifc/seed-ifc.md`
(retired in this WP, replaced by a one-line pointer there):

- `ifc/ct-value-steers-leakage-sink-rejected` (Sec1 F1, the hook-level
  branch/index/var-time reject, one case) → **split** into the discipline-level
  CT-A1 / CT-A2 / CT-A3 (one discriminating case per sealed `LeakSink` member,
  the proper `L-CT-SINK` rule) + the implicit-flow CT-A4.
- `ifc/ct-label-parses-carries-and-defers-timing` (Sec1 F2, the carry +
  defer-timing honesty) → **absorbed** into CT-E1 (the honest split, now naming
  the three `[Sec1-*]` triggers — the Sec1 F2 reify-trigger pointed at *this*
  WP `[Sec1ct]`, now landed, so the trigger is updated to `[Ward]` for timing).

The Sec1 `ifc/` cases for confidentiality/integrity/no-laundering/by-proof
(groups A–E, G) are **untouched** — Sec1ct extends only the `@ct` axis.

## Build-sequencing note

All Sec1ct cases (A–D) exercise the elaborator's **flow-typing pass** (the
`L-CT-SINK` rule §5a.3, the `pc.ct` discipline, the sealed `LeakSink` match) —
the **same** untyped-then-erased path as the Sec1 by-typing cases, needing **no
kernel feature** beyond landed K1.5 and **no** by-proof/product-program
machinery (`@ct` is unary taint, §5a.3 — the relational engine is *not* on this
path). CT-D1's `Q` export rides the **`71` guarantees channel** and couples to
**B1** (the export emitter, Kernel/WS-B) — the `Q`-clause **field token** is
`(oracle)`-tagged and bound by B1, so CT-D1 asserts the clause **structurally**
(present, names the parameter, source-level, 1:1 with a `63 §5a` discharge),
not the literal spelling. The literal surface token `@ct` stays
`OQ-syntax`-deferred (`(oracle)`); cases pin the value-set `{ct⊥, ct⊤}` +
invariants, not the surface token. `[Ward]` (runtime timing validation under a
leakage model) and the three `[Sec1-*]` triggers are the named follow-on work —
the enforcement of constant-time *execution* lands there, not in Sec1ct.
