# Sec6 (discharge attestation) conformance — seed cases

Format: `../../README.md`. These pin the **Ken-visible half** of the
discharge-attestation contract (`spec/60-security/63-supply-chain.md §5a`,
`OQ-discharge-attestation` DECIDED — Ward finalized its half at ward `f33276b`).
The attestation is a **`Ward` artifact**; Ken emits/enforces/depends on
**exactly** the ratified field set (`63 §5a`, all already B1-emitted,
`71 §2.1`), and the two **hard** invariants are the trust-boundary ones: the
**one-way gate** (no outcome promotes a `T` to `proved`, `71 §5.1` I4) and the
**Ward-internal boundary** (no Ken correctness judgment reads Ward's mechanism).

## Grounding (content-verified against the landed targets)

- `63 §5a` (ratified body) — the Ken-visible field set (`export.hash`/
  `.contractVersion`, `ward.version`, `obligations[].id`/`.field`/`.outcome`,
  `signature`), the four-way outcome `discharged/bounded/monitored/failed`
  (total; `bounded-to-`k`` widened to `bounded`), the Ward-internal
  must-not-read set (`policy`/`bound`/`evidence`/`ct.method`/`regression`), and
  I4.
- `71 §5.1` — **invariant I4** the one-way gate: a discharge re-enters Ken only
  as a `trusted_base_delta`/attestation record tagged `delegated`/`tested`,
  **never** re-stamped `proved`; realized as the **absence of a code path** (no
  function from a `Ward` verdict to `proved`), monotone.
- `71 §2.1` — the `Q`/`P`/`Σ`/`T`/`G` export contract (B1, landed): the four-way
  verification status (`21 §5`) and the field the Ken-visible set is produced
  by.
- `61 §5a` (`OQ-relational`) — the `@ct` timing guarantee is delegated to `Ward`
  under a leakage model; a `Q@ct`-channel obligation carries its verdict but is
  still `delegated`, never kernel-`proved`.

## Scope — these EXTEND B1's export gate, they do not re-pin it

The **base** one-way gate — *a delegated obligation is never promoted; no
`proved`-writing edge exists* — is homed in **B1**
(`../../behavioral/export/seed-export.md`:
`export/delegated-obligation-never-promoted-to-proved`, AC6/I4). These Sec6
cases pin only the **new axis Sec6 introduces** — the discharge-attestation
**outcome vocabulary** and the **Ward-internal field boundary** — over that same
gate (subsume-don't-proliferate; one home per property). AC3 here is the
outcome-axis extension of the B1 case, referenced not duplicated.

## Contract-spec discipline (defer spelling, not concept)

The attestation is a wire contract Ken co-owns with `Ward`. The **literal field
tokens** (`export.hash`, `ward.version`, `obligations[].id/field/outcome`,
`signature`) and the `predicateType` URI (`ward.dev/attestation/discharge/v1`)
are **`Ward`'s wire spelling** — **`(oracle)`-tagged**, finalized with `Ward`
under `OQ-export-wire`. Cases author against the **reference spellings** and
oracle-tag the tokens; the **test logic** — no-promote, reject-missing-required,
accept-ignore-unknown, `id`-stable-across-`export.hash`-change,
no-branch-on-Ward-internal — is **spelling-agnostic** and normative.

**Tags.** `(soundness)` — a trust-boundary commitment whose wrongness is a
soundness bug (I4 no-promotion; the Ward-internal boundary — coupling to `Ward`
mechanism breaks the abstraction I4 draws). `(property)` — an invariant over
many inputs (the `id` stability key). `(oracle)` — the literal wire tokens +
`predicateType` URI, finalized by `Ward`. The **outcome value-set, the four-way
totality, I4, the must-not-read set, and every verdict** are **normative**.

## Static vs runtime face

These pin the **static face** of the two hard invariants — the outcome→status
projection has no `proved` arm; a correctness judgment's field-dependency set ⊆
the Ken-visible set. The **runtime face** — the three-check **deployment gate**
must not promote and must not read Ward-internal at execution — is the named
**Team-Verify build follow-on** (`63 §5a`, WS-Sec), not this WP
(`soundness-AC-static-vs-runtime-face`).

---

## AC3 — the one-way gate: no outcome promotes to `proved`

`63 §5a` / `71 §5.1` I4: **no `outcome` value — not even `discharged` — promotes
a delegated obligation to `proved`.** A discharge projects to `P`/`tested`,
**never `Q`**.

### security/supply-chain/discharged-does-not-promote-to-proved (soundness)
- spec: `63 §5a` (I4, the outcome vocabulary), `71 §5.1` (the one-way gate),
  `21 §5` (the four-way status)
- given: a discharge-attestation record for a **`T`-channel** obligation
  (`obligations[].field = T` `(oracle)` token) whose
  `obligations[].outcome = discharged` `(oracle)` — the **strongest**, most
  proof-like outcome — fed back into the Ken side, re-entering **only** as an
  attestation record / `trusted_base_delta` entry (`63 §5a`), **not** as a
  kernel certificate
- expect: the obligation's Ken-side verification status **stays `delegated`**
  (in `T`) / `tested` (in `P`); it is **never** re-stamped `proved` and
  **never** appears in `Q`. Assert the **projected status**
  (`delegated`/`tested`), the observable — **not** a message string and **not**
  the internal mechanism. Structurally: there is **no code path** from an
  `outcome` value to a `proved` status.
- why: (soundness) AC3/I4 — the **outcome-axis extension** of B1's
  `export/delegated-obligation-never-promoted-to-proved`. `discharged` is the
  **discriminating arm**: a decision-procedure/exhaustive discharge *looks* like
  a proof, so a naïve emitter that reads `outcome = discharged` and stamps
  `proved` is the exact bug — this is the
  [[trusted-by-typing-guarantee-is-not-kernel-proved-Q]] shape (a
  classically-discharged obligation is lower-trust → `P`/`tested`, never kernel
  `Q`). **Structural-flip (anti-green-vs-green):** the projected status is
  `delegated`/`tested` on the correct emitter and would be `proved`/`Q` under
  the promotion bug — opposite observables, not both-reject. **Absence gated,
  named:** the guard is *no `proved`-writing edge from an outcome* (per
  `71 §5.1`'s absence-of-code-path framing), not "the happy path doesn't take
  one." The four-way is **total** and `bounded`/`monitored`/`failed` are
  strictly weaker (partial/observed/negative) — **none** promotes; `discharged`
  is the arm that nets the bug. Does **not** re-pin the B1 base gate
  (referenced).

### security/supply-chain/qct-channel-discharge-stays-delegated (soundness)
- spec: `63 §5a` (AC6 — carry the `@ct` verdict, not the method), `61 §5a`
  (`@ct` delegated to `Ward`), `71 §5.1` I4
- given: a **`Q@ct`-channel** obligation (`obligations[].field = Q@ct`
  `(oracle)` — a constant-time guarantee delegated to `Ward` under a leakage
  model) with `obligations[].outcome = discharged` — `Ward` validated the timing
- expect: the obligation **stays `delegated`**/`tested`; it is **not** promoted
  to kernel-`Q`/`proved` **despite the `Q@ct` channel label**. Ken **carries and
  enforces the verdict**, never the CT-validation *method* (Ward-internal, ward
  §13)
- why: (soundness) the **field-axis** arm — distinct dimension from the
  outcome-axis case above (a guard discriminating on `outcome` can be vacuous on
  `field`; multi-dimensional-guard discipline). The `Q@ct` label is the trap: it
  *reads* like a `Q` guarantee, but a `Ward`-validated `@ct` discharge is
  **trusted-by-typing / classically-discharged → `P`/`tested`, never kernel
  `Q`** ([[trusted-by-typing-guarantee-is-not-kernel-proved-Q]] sharpened for
  `@ct`). **Structural-flip:** stays `delegated` (correct) vs promoted-to-`Q` (a
  build that special-cases the `Q@ct` channel as kernel-certified). Pins that
  the no-promotion holds on **every** channel, not just `T`.

---

## AC4 — the Ward-internal boundary (no judgment reads Ward mechanism)

`63 §5a`: a Ken correctness judgment depends only on the Ken-visible set; the
must-not-read set is `policy`/`bound`/`evidence`/`ct.method`/`regression`.

### security/supply-chain/consumer-reads-ward-internal-rejected (soundness)
- spec: `63 §5a` (the Ward-internal boundary), `71 §5.1` (the abstraction I4
  draws)
- given: the **same** discharge-attestation, consumed two ways by a Ken
  correctness judgment (a gate/consumer): (a) one whose field-dependency set is
  ⊆ the **Ken-visible set** (`outcome`/`field`/`export.hash`/`ward.version`/
  `id`); (b) one that **branches on a Ward-internal field** — `bound` (the
  depth-`k`/sample size), `policy`, `evidence`, `ct.method`, or `regression`
- expect: **the verdict flips.** (a) **accepts** — a well-formed judgment
  depending only on the ratified surface; (b) **rejected** — a correctness
  judgment reading `Ward` **mechanism** is a boundary violation, keyed on the
  structural discriminator *field ∈ the closed Ward-internal set*
- why: (soundness) AC4 — the abstraction boundary I4 draws (*Ken couples to
  **that** `Ward` discharged, never **how***). **Non-degenerate pair** (the
  classification-boundary net, COORDINATION §7): a single accept is
  green-vs-green under a consumer that silently reads mechanism; the **reject
  arm is the guard**, keyed on the **named closed** must-not-read set (not a
  self-reported string). **Disconfirming check:** would (b) also reject if it
  read a *visible* field? **No** — (a) reading `outcome`/`ward.version` accepts
  — so the reject is gated on the Ward-internal set, not coincidental. Producer:
  the judgment's **field-access set** ⊆ the Ken-visible set; any read of the
  must-not-read set fails. (Static face; the deployment-gate runtime face is the
  Team-Verify follow-on.)

---

## Contract surface — required fields, unknown fields, and the regression key

### security/supply-chain/reject-missing-ward-version-required-field (soundness)
- spec: `63 §5a` (the ratified field set + the one trust edge), `71 §5`
  (faithfulness holds relative to the pinned `Ward` version)
- given: (a) an attestation **missing `ward.version`** `(oracle)` (the
  load-bearing trust edge); (b) one **missing `export.hash`** (the binding /
  revocation key); and (c) a well-formed attestation with an **extra unknown
  field**
- expect: (a) **rejected** — no `ward.version` means the faithfulness argument's
  version-bounded assumption has no anchor (`71 §5`), so the attestation is not
  admissible; (b) **rejected** — no `export.hash` means it cannot be bound to
  the export it discharges (the gate's hash-match, fail-closed); (c)
  **accepted** — **unknown fields are ignored** (forward-compat under the
  contract-spec stability discipline)
- why: (soundness for (a)/(b); contract-spec for (c))
  reject-missing-**required** / accept-ignore-**unknown** is the wire-contract
  stability discipline ([[contract-spec-defer-spelling-not-concept]]) —
  **spelling-agnostic** (the literal tokens are oracle-tagged, the *logic* is
  normative). `ward.version` is the **soundness-load-bearing** required field:
  it is the one trust edge, so its absence is not mere ill-formedness but a
  missing faithfulness anchor. Discriminating: reject-on-missing-required
  **while** accept-on-unknown — a parser that rejects unknown fields (breaks
  forward-compat) or accepts a missing `ward.version` (drops the trust edge)
  fails the opposite arm.

### security/supply-chain/id-stable-across-export-hash-change (property)
- spec: `63 §5a` (`obligations[].id` = "stable key across `export.hash`
  changes"), `71 §2.1` (the export content-hash), B1
  `export/removing-assume-shrinks-P-and-changes-hash`
- given: two attestations for the **same** obligation identity (same `Σ`-scoped
  obligation) where the **`export.hash` differs** — e.g. an `assume` was
  removed, shrinking `P` and changing the B1 content-hash (`71 §2.1`) — but the
  obligation itself is unchanged
- expect: `obligations[].id` is **identical** across the two, **while**
  `export.hash` differs. The `id` is `Ward`'s **regression key** — it tracks the
  same obligation across export re-hashes
- why: (property) the regression-key stability the attestation exists to
  provide. **Structural-flip (anti-green-vs-green):** assert `id` **unchanged**
  *while* `export.hash` **changed** — a bug that derives `id` **from**
  `export.hash` (coupling them) flips this: `id` would change on every re-hash,
  breaking `Ward` regression tracking. Pins that `id` is stable over obligation
  **identity**, not over the export content-hash. (References the B1 hash-change
  scenario; does not re-pin the hash mechanism.)

### security/supply-chain/outcome-vocabulary-is-total-four-way (AC2)
- spec: `63 §5a` (four-way total `discharged/bounded/monitored/failed`)
- given: an attestation whose `obligations[].outcome` carries a **fifth** value
  outside the four-way vocabulary
- expect: **rejected** — the outcome classification is **total** and **closed**;
  there is **no fifth outcome**. The four reference spellings are `(oracle)`;
  the **closedness** (value-set is exactly the four) is normative
- why: (AC2) the value-set lock of the contract-spec discipline — the four-way
  classifies **epistemic status** (decided / partial-under-bound / observed /
  negative) and is stated total, so an out-of-vocab outcome is a contract
  violation, not a silently-accepted extension. `bounded` covers **both**
  model-check depth and sampled coverage (the `bounded-to-`k`` widening); the
  distinguishing **source** is Ward-internal (AC4), so no fifth label splits it.

---

## Coverage map

- **AC3** (one-way gate over outcomes, soundness):
  `discharged-does-not-promote-to-proved`,
  `qct-channel-discharge-stays-delegated`.
- **AC4** (Ward-internal boundary, soundness):
  `consumer-reads-ward-internal-rejected`.
- **AC1/contract surface:** `reject-missing-ward-version-required-field`
  (soundness), `id-stable-across-export-hash-change` (property).
- **AC2** (outcome vocabulary total): `outcome-vocabulary-is-total-four-way`.

## Cross-case consistency sweep

- **No-promotion holds on every channel.**
  `discharged-does-not-promote-to-proved` (`T`) and
  `qct-channel-discharge-stays-delegated` (`Q@ct`) are one story: **no** outcome
  on **any** `obligations[].field` promotes to `proved`/`Q`. A case letting a
  particular channel (e.g. `Q@ct`) or a particular outcome (`discharged`)
  promote would contradict this class and I4.
- **The Ken-visible set and the Ward-internal set partition the attestation.**
  `consumer-reads-ward-internal-rejected` (a judgment reads only
  the visible set) and `reject-missing-ward-version-required-field` (the
  required visible fields) agree: the visible set (`export.hash`,
  `.contractVersion`, `ward.version`, `obligations[].*`, `signature`) is exactly
  what Ken depends on, and the must-not-read set
  (`policy`/`bound`/`evidence`/`ct.method`/`regression`) is exactly what it does
  not. A case requiring a Ward-internal field for a correctness judgment, or
  treating a visible field as optional-to-depend-on, would contradict the
  partition.
- **`id` tracks obligation identity, `export.hash` tracks export content.**
  `id-stable-across-export-hash-change` pins these as **independent**
  keys; a case coupling `id` to `export.hash` (id changes when the export
  re-hashes) would break the regression key and contradict `63 §5a`.

## Subsumed / not-duplicated (one home per property)

- **The base one-way gate** — *a delegated obligation is never promoted; the
  emitter has no `proved`-writing edge* — is **B1's**
  (`../../behavioral/export/seed-export.md`:
  `export/delegated-obligation-never-promoted-to-proved`, AC6/I4). Sec6 AC3
  **extends** it over the discharge-attestation **outcome vocabulary** + the
  `Q@ct` channel; it does **not** re-pin the base absence-of-code-path net.
- **The `Q`/`P`/`Σ`/`T`/`G` status→field projection** (the no-over-claim pair,
  the alphabet, the export content-hash) is **B1's** (`71 §2.1`, same seed).
  Sec6 references the projection (a discharge lands `delegated`/`tested`) but
  re-pins none of it.
- **The `@ct` taint/leakage enforcement** (the constant-time discipline, the
  sealed `LeakSink` set, declassify) is **Sec1ct's** (`../ct/seed-ct.md`). Sec6
  carries only the `@ct` **verdict** on the attestation, not the enforcement or
  the CT-validation method (Ward-internal).
- **Provenance/signature transport + the governance ladder** (keyless signing,
  the policy-attestation ladder) is **Sec5's** governance (`../policy/`,
  `65 §5`) and `63 §5`. Sec6 depends on `signature` + `ward.version` as the
  trust edge, re-pinning neither the signing mechanism nor the ladder.
