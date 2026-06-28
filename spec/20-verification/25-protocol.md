# The machine-readable diagnostic protocol

> Status: **DRAFT v0**. Normative for the protocol's shape and stability
> guarantees; the exact JSON schema is given as a reference to be finalized with
> the agent-team software. Contract for WS-V/WS-T **T1** — the agent contract:
> how the write→spec→verify→repair loop (strategy G7) talks to the toolchain.
> Every verdict and diagnostic (`24`) is emitted as stable, schema-valid JSON an
> agent consumes without scraping human text.

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
  "trusted_base_delta": [ Postulate… ],   // holes/axioms added (24 §2)
  "stats": { "obligations": 3, "discharged": 2, "open": 1, "ms": 41 }
}
```

- **`status`** is the rollup over obligations: `proved` (all discharged),
  `disproved` (≥1 refuted, `24 §1/§3`), `incomplete` (≥1 open hole, none
  refuted). This is the surface of the kernel trichotomy (`../10-kernel/12
  §5.2`).
- **`trusted_base_delta`** lists every postulate/open-hole this target
  introduces (`../10-kernel/18 §5`) — so an agent (or a reviewer) sees exactly
  what is *assumed*. A genuinely-verified artifact has an empty delta.

## 4. Obligation objects

```json
{
  "id": "ob:divide#post.0",          // stable across runs (22 §1)
  "goal": { "pretty": "result * d + (n % d) == n",
            "core":  "<core-term ref>" },
  "context": [ { "name": "d", "type": "Int", "from": "param" },
               { "name": "h", "type": "d ≠ 0", "from": "requires" } ],
  "provenance": { "span": "pay.ken:14:11-14:38", "clause": "ensures" },
  "fragment": "D | FO | HO",         // classifier verdict (23 §2)
  "status": "discharged | refuted | open",
  "method": "reflect | smt-direct | kripke | ipc | tactic | hole",
  "diagnostic": Diagnostic | null     // present iff not discharged
}
```

`id`s are **stable** (same program+spec → same ids) so an agent can correlate a
diagnostic across edit/verify cycles and target a specific obligation.

## 5. Diagnostic objects (the four mechanisms of `24`)

A `diagnostic` is one tagged union; all fields are machine-consumable.

```json
// 1. Kripke countermodel (24 §1)
{ "kind": "countermodel",
  "worlds": [ { "id": "w0", "below": [] }, { "id": "w1", "below": ["w0"] } ],
  "forcing": { "w0": { "d ≠ 0": false }, "w1": { "d ≠ 0": true } },
  "fails_at": "w0",
  "verdict": "false | unknown",        // false vs unknown (24 §1)
  "failing_subformula": "result * d + (n % d) == n" }

// 2. typed hole (24 §2)
{ "kind": "hole", "hole_id": "?h.7",
  "goal": "isSorted (insert x xs)", "context": [ … ],
  "runtime": "depends → evaluates to `unknown`" }

// 3. three-region Heyting decomposition (24 §3)
{ "kind": "decomposition",
  "true_region":    "n > 0",
  "false_region":   "n < 0",
  "unknown_region": "n == 0" }

// 4. slice / missing hypothesis (24 §4)
{ "kind": "slice",
  "missing_hypothesis": "xs ≠ nil",
  "bridge": "holds in Γ, (xs ≠ nil); add as a precondition" }
```

Every diagnostic also carries:

```json
"suggested_actions": [
  { "kind": "add_precondition", "detail": "requires xs ≠ nil",
    "edit": { "span": "pay.ken:20:1", "insert": "  requires xs ≠ nil\n" } },
  { "kind": "provide_lemma", "detail": "insert_preserves_sorted", "edit": null }
]
```

`edit` (optional) is a concrete, applyable source patch; when present an agent
can apply it directly, when `null` the action is advisory (`24 §5`).

## 6. Stability guarantees (the contract)

- **Versioned schema.** `schema: "ken.verify/v1"`. Breaking changes bump the
  version; agents pin and migrate. Additive fields are non-breaking.
- **Deterministic.** Same program + spec + prover version → byte-stable document
  (modulo `stats.ms`), so diffs are meaningful (`24 §6`).
- **Stable ids.** Obligation and hole ids are derived from program structure,
  not run order, and survive unrelated edits elsewhere in the file.
- **Total.** Every target produces a document; every non-discharged obligation
  produces a diagnostic. There is no "silent" failure mode.

## 7. The loop, concretely

```
agent: writes definition + spec        →  ken verify --format=json
toolchain: emits verdict document      →  agent reads status/obligations
  status=proved      → done; trusted_base_delta empty → shippable
  status=incomplete  → pick an `open` obligation's `hole`/`suggested_actions`,
                       apply an `edit` or supply a lemma, re-verify
  status=disproved   → read `countermodel.verdict=false` / `false_region`,
                       fix the code or spec at `provenance.span`, re-verify
```

This is the protocol the agent-team software (strategy G7, and the operator's
mootup-based fleet) drives unattended. It is deliberately small and stable so
the loop is robust across model and toolchain versions.

## 8. What WS-V/WS-T must deliver here (T1)

The versioned schema (verdict + obligation + the four diagnostic objects +
suggested_actions + trusted_base_delta), the batch and server emitters, and the
stability guarantees (§6). Acceptance: the agent-team loop completes a
write→spec→verify→repair cycle consuming **only** the JSON (no human-text
scraping), landing a `proved`, empty-`trusted_base_delta` result (strategy G7).
Conformance: `../../conformance/verify/protocol/` — schema validation of each
status/diagnostic kind and an id-stability test across an unrelated edit.
