# The machine-readable diagnostic protocol

> Status: **T1 elaborated** (implementation-ready). Normative for the message
> **shape**, the **status/verdict cross-walk**, and the **stability surface**
> (§6); the exact JSON field *names* are a reference finalized with the
> agent-team software, but the shape and the stable surface are normative.
> Contract for WS-V/WS-T **T1** — the agent contract: how the
> write→spec→verify→repair loop (strategy G7) consumes the toolchain's output.
> **★★ (untrusted):** the protocol *serializes* decisions V3/V4 made (the
> kernel settled `proved` via the certificate, `23 §1.3`; V4 derived the
> diagnostic, `24`); a T1 bug is a **malformed, lossy, or unstable message**
> — an agentic-UX/contract regression, **never** unsoundness. Every verdict
> (`23 §1.2`) and diagnostic (`24`) is emitted as stable, schema-valid JSON an
> agent consumes without scraping human text.

## The wire contract — fidelity to the verdict (the cardinal rule on the wire)

The protocol **serializes** the verdict V3 produced (`23 §1.2`) and the
diagnostic V4 derived (`24`); it **never re-decides** either. This is the
wire image of V4's cardinal rule (`24`, "fidelity to V3's verdict") and the
reason a T1 bug is advisory-UX, not unsoundness (★★): the kernel settled
`proved`/not via the certificate (`23 §1.3`), V4 rendered the structured value,
and T1 only *encodes* it. Every status string, every `verdict` tag, every
diagnostic `kind`, and every `suggested_actions` tag on the wire is a
**projection of the one V3 verdict**, copied at serialization — **not**
recomputed from the JSON evidence.

The single normative cross-walk — a builder reads this table and the §3 rollup,
nothing else, to serialize a result:

| V3 verdict (`23 §1.2`) | obligation `status` (§4) | doc `status` rollup (§3) | diagnostic `kind` (§5) | countermodel `verdict` tag | legal `suggested_actions` (`24 §5`) |
|---|---|---|---|---|---|
| `proved` | `discharged` | `proved` (all discharged) | none — `diagnostic: null` | — | none |
| `disproved` | `refuted` | `disproved` (≥1 refuted) | `countermodel` / `decomposition` (point in `false_region`) | **`false`** | `fix_counterexample` **only** |
| `unknown` | `open` | `incomplete` (≥1 open, none refuted) | `hole` / `countermodel` (unknown) / `decomposition` (unknown_region) / `slice` | **`unknown`** | `add_precondition` · `strengthen_refinement` · `provide_lemma` · `case_split` · `induct_on` |

- **Document-status rollup (§3) — pin the precedence.** A target's `status` is a
  total function of its obligations' statuses: `disproved` if **any** is
  `refuted`, else `incomplete` if **any** is `open`, else `proved`.
  Precedence is `refuted ⊐ open ⊐ discharged` — a refuted obligation is a hard
  error and dominates open holes (`23 §1.2`, `21 §5.3`). State this; do not
  leave the mixed-verdict rollup to the serializer.
- **One source, three fields — they cannot disagree.** The obligation `status`,
  the document `status`, and a countermodel's `verdict` tag are **three
  renderings of the same V3 verdict**; the serializer derives all three from it,
  so `verdict:"false"` ⟺ obligation `refuted` ⟺ doc `disproved`, and
  `verdict:"unknown"` ⟺ `open` ⟺ `incomplete`. A document where they disagree is
  a fidelity bug — the wire instance of the verdict-mapping-silence trap (a
  procedure's output mapped to a status must be pinned, never left for the
  reader to fill).
- **No independent `is_false` flag** (`24 §1`): the `false`/`unknown`
  discriminator **is** the `verdict` field, carried verbatim from V3; the
  serializer must not add a second boolean that could drift from it.
- **Glivenko on the wire (cross-case sweep anchor).** A **classically-valid**
  goal serializes to `status:"incomplete"` / `verdict:"unknown"`, **never**
  `disproved`/`"false"` — `p ∨ ¬p` and `¬¬p ⇒ p` are `unknown`, while
  `p ∧ ¬p` is `disproved`/`"false"` (`24 §1`/§3, `23 §5`). A serializer
  that emits `"false"` for a classically-valid goal has **relabeled** — the
  load-bearing round-trip failure conformance targets.

## 1. Why a protocol

Ken's differentiator is only real if an **agent** can act on a verification
result mechanically. Human-readable error text is not enough; the toolchain
emits a **structured, versioned** document per verification run. The same
document renders to a human view, but the machine view is primary: where current
languages are "human-interface languages", Ken's diagnostics are
agent-interface first.

## 2. Transport and invocation

- The verifier is invokable as a **batch command** (`ken verify <target>
  --format=json`) emitting one **verdict document** per target, and as a
  **server/LSP** mode streaming verdicts as definitions change (for the REPL and
  editors, strategy T2).
- Output is **newline-delimited JSON** (one document per target) or a single
  JSON array; both are schema-valid. No diagnostic is ever *only* in human text.

## 3. The verdict document

One per verified target (definition, `prove`, or module). Reference shape:

```json
{
  "schema": "ken.verify/v1",
  "target": { "name": "divide", "span": "pay.ken:12:1-15:8", "kind": "view" },
  "status": "proved | disproved | incomplete",
  "obligations": [ Obligation… ],
  "trusted_base_delta": [ Postulate… ],   // holes/axioms this target adds (18 §5)
  "stats": { "obligations": 3, "discharged": 2, "open": 1, "ms": 41 }
}
```

- **`status`** is the **rollup** over obligations per the precedence above:
  `proved` (all discharged), `disproved` (≥1 refuted), `incomplete` (≥1 open,
  none refuted) — the wire surface of the **verdict trichotomy** (`23 §1.2`,
  `21 §5.1`), projected to a run status by `21 §5.3`. (It is the per-run rollup,
  **distinct** from V1's per-definition *epistemic* status `21 §5.2`, which an
  agent reads separately.)
- **`trusted_base_delta`** lists every postulate/open-hole this target
  introduces (`18 §5`; holes are postulates, `24 §2`) — so an agent or reviewer
  sees exactly what is *assumed*. A genuinely-verified artifact has an **empty**
  delta (the honesty guard, `21 §5.4`).

## 4. Obligation objects

```json
{
  "id": "ob:divide#post.0",          // stable across runs (22 §1, 24 §6)
  "goal": { "pretty": "result * d + (n % d) == n",
            "core":  "<core-term ref>" },
  "context": [ { "name": "d", "type": "Int", "from": "param" },
               { "name": "h", "type": "d ≠ 0", "from": "requires" } ],
  "provenance": { "span": "pay.ken:14:11-14:38", "clause": "ensures" },
  "fragment": "D | FO | HO",         // classifier route (23 §2)
  "status": "discharged | refuted | open",
  "method": "reflect | smt-direct | kripke | ipc | tactic | hole",
  "diagnostic": Diagnostic | null     // present iff status ≠ discharged (§5)
}
```

- `status` is the per-obligation projection of V3's verdict (the cross-walk):
  `discharged` ⇐ `proved`, `refuted` ⇐ `disproved`, `open` ⇐ `unknown`. The
  `diagnostic` is `null` **iff** `discharged` (`24 §7`, AC5) and otherwise
  carries the §5 shape whose `verdict`/region matches `status` — never a
  mismatched pair.
- `id`s are **stable** (same program+spec → same ids, derived from program
  structure not run order, `22 §1`, `24 §6`) so an agent can correlate a
  diagnostic across edit/verify cycles and target a specific obligation.

## 5. Diagnostic objects (the four mechanisms of `24`)

A `diagnostic` is one **tagged union** discriminated by `kind`; every field is
machine-consumable. Each shape is the wire serialization of the corresponding
`24` *value* — same fields, faithfully (`24 §7`).

```json
// 1. Kripke countermodel  (24 §1 — the KripkeCountermodel value)
{ "kind": "countermodel",
  "verdict": "false | unknown",      // COPIED from V3's verdict (cardinal rule);
                                      //   NOT recomputed from the model below
  "worlds":  ["w0", "w1"],           // finite; w0 the root
  "order":   [["w0", "w1"]],         // the ≤ preorder, pair [lo, hi] = lo ≤ hi
  "forcing": { "w0": [], "w1": ["d ≠ 0"] },   // atoms forced per world (monotone in ≤)
  "failure": { "world": "w0", "subformula": "result * d + (n % d) == n" } }

// 2. typed hole  (24 §2 — the TypedHole value)
{ "kind": "hole",
  "hole_id": "?h.7",                 // stable, deterministic (§6, 24 §6)
  "goal":    "isSorted (insert x xs)",
  "context": [ Binding… ],           // Γ
  "origin":  { "span": "lib.ken:30:3-30:21", "clause": "ensures" },
  "runtime": "unknown" }             // operational face: evaluates to `unknown` (41 §6)

// 3. three-region Heyting decomposition  (24 §3)
{ "kind": "decomposition",
  "true_region":    "n > 0",         // S_φ      (verdict proved)
  "false_region":   "n < 0",         // S_{¬φ}   (verdict disproved) — the `false` part
  "unknown_region": "n == 0" }       // residual (verdict unknown); the ¬¬φ gap

// 4. slice / missing hypothesis  (24 §4)  — `unknown` ONLY
{ "kind": "slice",
  "missing_hypothesis": "xs ≠ nil",
  "bridge": "holds in Γ, (xs ≠ nil); add as a precondition",
  "sufficient": true }               // Γ, ψ ⊢ φ makes V3 return `proved` (24 §4)
```

- The countermodel's `verdict` field **is** the `false`/`unknown` discriminator,
  carried verbatim from V3 (`24 §1`): there is no separate `is_false` flag, and
  `verdict:"false"` holds **iff** some world forces `¬φ` (`failure` then names a
  refuting world). A model that merely **fails to force** `φ` is `unknown`, not
  `false` — the single most actionable bit, preserved on the wire.
- `decomposition.false_region` is non-empty **only** for a `disproved` goal;
  for an `unknown` obligation the failing inputs land in `unknown_region` (the
  `¬¬φ` gap, `24 §3`). A classically-valid goal never populates `false_region`
  (Glivenko, above).
- `slice` appears **only** for an `unknown` obligation (`24 §4`); a `false` goal
  is never repaired by adding a hypothesis. `sufficient: true` asserts the
  verdict-flip (`unknown` without `ψ` → `proved` with `ψ`); minimality of `ψ` is
  a quality SHOULD (`24 §6`), not a wire guarantee.

Every diagnostic also carries **`suggested_actions`** — an ordered list, each
**region-tagged** so an `unknown`-only action never rides a `false` goal (the
`24 §5` fidelity constraint):

```json
"suggested_actions": [
  { "kind": "add_precondition", "region": "unknown",
    "detail": "requires xs ≠ nil",
    "edit": { "span": "lib.ken:20:1", "insert": "  requires xs ≠ nil\n" } },
  { "kind": "provide_lemma", "region": "unknown",
    "detail": "insert_preserves_sorted", "edit": null }
]
```

- `kind` ∈ the `24 §5` set: `add_precondition`, `strengthen_refinement`,
  `provide_lemma`, `case_split`, `induct_on` (all `region:"unknown"`), and
  `fix_counterexample` (`region:"false"`). A `disproved` obligation carries
  **only** `fix_counterexample`; an `unknown` obligation **never** carries it.
- `edit` (optional) is a concrete, applyable patch; present ⇒ an agent can
  apply it directly, `null` ⇒ the action is advisory (`24 §5`). Suggested
  actions are **hints, not commands**, and never alter the verdict (cardinal
  rule).

## 6. Stability guarantees (the contract)

Agents code against this document; the contract is **what is stable vs.
versioned**. A toolchain update must not silently break a pinned parser.

**Stable surface** — renaming, dropping, or re-tagging any of these is a
**breaking change** (bump `schema`) and a **conformance failure**:

- `schema` (the version string itself) and document `status`
  ∈ `{proved, disproved, incomplete}`;
- `obligations[].id` — a stable function of **program structure**, invariant
  under edits unrelated to the clause (`22 §1`, `24 §6`);
- `obligations[].status` ∈ `{discharged, refuted, open}`;
- `diagnostic.kind` ∈ `{countermodel, hole, decomposition, slice}`;
- **`countermodel.verdict` ∈ `{false, unknown}`** — the load-bearing
  discriminator; its presence and these two values are the wire's fidelity
  guarantee;
- `hole.hole_id` — a stable function of the obligation **provenance**, not
  allocation order (`24 §6`);
- `suggested_actions[].region` ∈ `{false, unknown}` and the **known**
  `suggested_actions[].kind` values;
- `trusted_base_delta` — its presence and **emptiness semantics** (empty ⟺
  nothing assumed, `21 §5.4`).

**Versioned / additive** — non-breaking; a pinned agent **ignores unknowns**:

- new **optional** fields anywhere (additive, non-breaking);
- new `suggested_actions[].kind` values (an agent treats an unknown kind as a
  plain advisory hint);
- `stats.*` (informational; `stats.ms` is non-deterministic — excluded from
  byte-stability) and any human-rendered string (`goal.pretty`, `bridge`) —
  display, not contract.

**Other guarantees:**

- **Versioned schema.** `schema: "ken.verify/v1"`. Breaking changes bump to
  `v2`; agents pin a major and migrate. Additive fields are non-breaking.
- **Deterministic.** Same program + spec + prover version → **byte-stable**
  document (modulo `stats.ms` and display strings), so diffs are meaningful
  (`24 §6`).
- **Total.** Every target produces a document; every non-discharged obligation
  produces a diagnostic (the cross-walk's "`diagnostic: null` iff discharged").
  There is **no silent failure mode** — the wire image of routing totality
  (`23 §2.1`) and `24 §6`.

## 7. The loop, concretely

From a serialized result **alone** (no human-text scraping), an agent pivots on
`status` to locate the actionable signal — *fix-the-spec* vs *supply-facts*:

```
read doc.status:
  "proved"     → trusted_base_delta == [] ? ship : review the listed postulates
  "disproved"  → FIX. for each obligation status=="refuted" (diagnostic.verdict=="false"):
                   kind=countermodel  → failure.{world,subformula} + provenance.span
                   kind=decomposition → false_region (the failing input class)
                 apply the `fix_counterexample` action; edit at provenance.span; re-verify
  "incomplete" → SUPPLY. for each obligation status=="open" (diagnostic.verdict=="unknown"):
                   kind=hole          → fill via `provide_lemma` / the goal+context
                   kind=slice         → apply `add_precondition` (missing_hypothesis) — often THE fix
                   kind=decomposition → strengthen the unknown_region inputs
                 apply an `edit` if present, else act on `detail`; re-verify
```

The decision the loop pivots on is the `false`/`unknown` split carried to the
wire (`24 §1`): `disproved`/`false` ⇒ the property is **wrong** — change the
program/spec at `provenance.span`; `incomplete`/`unknown` ⇒ the property **may
hold** — supply more facts (a precondition, lemma, or case split). This is the
protocol the agent-team software (strategy G7) drives unattended; it is
deliberately small and stable so the loop is robust across model and toolchain
versions.

## 8. Reference schema (shape + stability)

A reference validator — the **shape** and the **stable surface** (§6) are
normative; exact field *names* are finalized with the agent-team software. It
must validate the **three statuses**, the **four diagnostic kinds**, and
`trusted_base_delta`:

- **Document** — required `schema` (string), `target`, `status` (enum of 3),
  `obligations` (array); optional `trusted_base_delta` (array, default `[]`),
  `stats`.
- **Obligation** — required `id` (string), `goal`, `status` (enum of 3),
  `provenance`; optional `fragment`, `method`, `diagnostic` (the union or
  `null`); `diagnostic == null` **iff** `status == discharged`.
- **Diagnostic** — a **discriminated union on `kind`** ∈ `{countermodel, hole,
  decomposition, slice}`; `countermodel` **requires** `verdict` ∈
  `{false, unknown}`; every member carries `suggested_actions` (array, possibly
  empty), each action requiring `kind` + `region` ∈ `{false, unknown}` and an
  optional `edit`.
- **Cross-field invariants** (the fidelity constraints conformance pins): the
  obligation `status`, the document `status`, and a `countermodel.verdict`
  **agree** per the §-cross-walk; a `disproved`/`false` diagnostic carries only
  `region:"false"` actions and no `slice`; an `unknown` diagnostic carries no
  `fix_counterexample`; a classically-valid goal is never `verdict:"false"`.
- **Round-trip:** every verdict + diagnostic serializes to a schema-valid
  document and **round-trips losslessly**; a `false` and an `unknown` serialize
  to **distinct, non-confusable** messages (the cardinal-rule fidelity property
  on the wire).

## 9. What WS-V/WS-T must deliver here (T1)

The versioned schema (verdict document + obligation + four diagnostic objects
+ `suggested_actions` + `trusted_base_delta`, §3–§5, §8), the **batch and server
emitters**, and the **stability guarantees** (§6). The load-bearing properties,
all of which conformance pins:

- **Faithful + lossless:** each V3 verdict (`23 §1.2`) + V4 diagnostic (`24`)
  serializes to schema-valid JSON and round-trips without loss; the **`false` vs
  `unknown`** distinction is preserved on the wire — an `unknown` **never**
  serializes as `false`/refuted (the cross-walk + the Glivenko invariant).
- **Stable contract:** the message declares stable vs. versioned fields (§6); a
  conformance case pins the stable surface — a rename/drop of a stable field
  (`obligations[].id`, `countermodel.verdict`, a `kind` tag) **fails**.
- **Agent-consumable:** from a serialized result, an agent reads `status` +
  `suggested_actions` and locates the actionable signal (fix-spec vs
  supply-facts) **without** parsing human text (§7).
- **No regression:** a fully-`proved` result serializes to an empty diagnostic
  set with an **empty** `trusted_base_delta` (AC5; V4/V3 behavior unaffected).

Acceptance: the agent-team loop completes a write→spec→verify→repair cycle
consuming **only** the JSON, landing a `proved`, empty-`trusted_base_delta`
result (strategy **G7**). Conformance: `../../conformance/verify/protocol/` —
validation of each status/diagnostic kind; a **discriminating round-trip**
where a `false` and an `unknown` go to distinct, non-confusable messages (the V4
fidelity property carried to the wire); a **stable-field** test where dropping
`obligations[].id` (or renaming `countermodel.verdict`) fails; and an
**id-stability** test across an unrelated edit elsewhere in the file.
