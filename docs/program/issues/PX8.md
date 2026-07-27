---
id: PX8
title: "partial/positioned IO — the completion program's root; closure condition"
status: draft
owner: runtime
size: L
gate: none
depends_on: [PX8-F-CAP-41, PX8-WROTE-ABS]
blocks: [ABI-R3, PX9]
github: null
origin: docs/program/09-posix-linux-abi-campaign.md (charter, PX-C phase); closure condition added 2026-07-22 (operator-approved)
---

> ## ⚠ STATUS CORRECTED `active` → `draft` — 2026-07-25 (Steward, tracker honesty)
>
> **Nothing is building this, and its `depends_on` is unmet** (PX8-F-CAP-41 (draft), PX8-WROTE-ABS (draft)). The tracker's own
> legend defines `draft` = *not framed / **deps unmet***, `ready` = *deps met,
> unassigned*, `active` = ***a team is building***. So `active` was a false claim
> that a seat held this node, and it polluted the releasable-frontier read that the
> next sequencing pass depends on.
>
> ⛔ **Blocked-ness is DERIVED, never spelled in `status`.** `gen-progress.sh`
> computes the frontier as *`ready` **and** every `depends_on` merged*, and lists
> blockers separately. ⇒ Do not invent a "blocked" status; fix `depends_on` and let
> the generator say so.
>
> ⇒ **Flip to `ready` only when every `depends_on` entry is `merged`.**

**This issue exists because PX8 had none.** `docs/program/10-linux-abi-completion.md`
makes **15 of its 18 items** unblock on `PX8 --> ABI-R3` and `PX8 --> PX9`, and
**nothing in the repository defined when that happens.** The sub-issues existed;
the parent did not. So "is PX8 done?" was a judgement call with no artifact
behind it — which is the same *restated-instead-of-derived* defect that
**ABI-R3 exists to fix**, applied to the program's own root.

> **Why `blocks:` is empty.** `ABI-R3` and `PX9` are the two items this gates,
> but **neither has an issue file yet** (they are unframed items in
> `10-linux-abi-completion.md`, not tracked issues), so there is no id to point
> at. Minting stubs purely to satisfy a schema field would be inventing scope.
> **The gate is bound in prose below and in that document's §5 graph.** When
> `ABI-R3` and `PX9` are framed, each takes `depends_on: [PX8]` and this note
> comes out.

## ⛔ Closure condition — a PROPERTY, not a checklist

Per `10-…:239-241` (*"Frames must state the mechanism, not name one"*), the
gate is the property. The sub-issue list below is the **currently known
sufficient set**, not the definition — if these all merge and the property does
not hold, **PX8 is not done.**

> **PX8 is closed when: every value the positioned/partial IO path reifies into
> checked Ken code is (a) correct against the LOCKED text of
> `spec/30-surface/38-ffi-io.md`, asserted ABSOLUTELY rather than differentially,
> and (b) indexed to the same request/span/buffer as every other value in the
> same reply — on BOTH the interpreter and the native backend.**

Two clauses, both load-bearing, both learned the hard way:

- **(a) absolutely, not differentially.** `RT-PARITY` merged green with six
  differentials over a **shared wrong basis**. A differential is a *relative*
  oracle: it establishes that two implementations agree, never that either is
  right. Where the spec is the authority, the oracle asserts against the
  **normative text**. This is exactly how `BUDGET-EFF`'s defect survived.
- **(b) co-indexed.** `SPAN-SEAL`'s whole subject. `BUDGET-EFF` is the same
  shape arriving through **reification** rather than through a producer: the
  span is correct and the count's `remaining` is indexed to the *raw* request
  while the span is indexed to the *effective* one — two halves of one
  `ReadSome` describing different requests.

## Known sufficient set

| sub-issue | status | note |
|---|---|---|
| `SPAN-SEAL` | ✅ merged | co-indexing for carrier producers |
| `RT-PARITY` | ✅ closed | interp/native parity — **necessary, not sufficient**, see (a) |
| `BUDGET-EFF` | ✅ merged | `remaining` bounded by the effective request |
| `SEAL-2` | ✅ merged | derived carrier-producer enumeration (PR #912 @ 4ac9141e, CI green) |
| `RT-ESCAPE` | ✅ merged | native-lowering completeness (PR #911 @ 238a5c5d, CI green) |
| `RT-SPLIT` | ✅ merged | ruled NOT a PX8 semantic dependency (see below) — does not gate |

> **The known-sufficient set went fully green as of `origin/main = 4ac9141e`
> (2026-07-23) — and PX8 STILL DID NOT CLOSE.** The Architect's grounded
> closure-property verdict (`evt_163mfgjs7fkh8`) derived the live reified-value
> population from the closed Rust sums (`ReadEof`, `ReadSome(BufferSpan,
> TransferCount)`, `Wrote(TransferCount)`, `SemanticErrorV1` — `crates/ken-host/
> src/effect_v1.rs:2089-2101`) rather than from the WP list, and found the
> property **fails on both clauses**. This is exactly the outcome this file was
> created to catch: the named mechanisms all merged, and the property still had
> holes. **Three closure-insufficiency items now gate PX8** (do not close until
> all three discharge AND the property is re-verified):
>
> | gap | clause | item | shovel-readiness |
> |---|---|---|---|
> | A1 — closed-endpoint `start==capacity` host-rejects instead of deriving `ReadEof` | (a) behavior | [[PX8-F-CAP-41]] | bounded fix + RED conformance oracle; **operator queuing gate** |
> | A2 — interp capped-short `Wrote` no absolute oracle; PR-C error identities unreached | (a) evidence | [[PX8-WROTE-ABS]] | `Wrote` oracle shovel-ready; error-row set needs a normative scoping call |
> | (b) — `BufferSpan` has no originating-buffer identity; `freeze` accepts a same-shape foreign span | (b) provenance | [[PX8-SPAN-PROV]] | ✅ **MERGED @ `cbf6a298` (PR #914), 2026-07-23 — clause-(b) DISCHARGED**; deferred native cells → [[RT-NATIVE-FNSPLIT]] |
>
> **UPDATE 2026-07-23:** clause-(b) [[PX8-SPAN-PROV]] MERGED (@ `cbf6a298`,
> PR #914) — **DISCHARGED**. **Two clause-(a) items remain:** [[PX8-F-CAP-41]]
> (A1 — operator queuing gate RELEASED; running as Foundation Track 2 alongside
> [[RT-NATIVE-FNSPLIT]]) and [[PX8-WROTE-ABS]] (A2 — needs a normative scoping
> call on the PR-C error-row set). PX8 does NOT close until both discharge AND
> the closure property is re-verified (absolute, co-indexed, both engines).
>
> PX8 stays `active`; **ABI-R3 and PX9 stay held.** This IS the artifact-backed
> gate working — a list-green close would have shipped three real defects.

> ## ⚖️ CLAUSE-(a) SCOPE RULED 2026-07-27 — Architect `evt_5h884g6xhtts3`
>
> **Question routed** (Steward `evt_2bh3tqwcvyb7n`, corrected by
> `evt_7aeprfb7qrwfh`): does clause (a)'s **universal** absolute-evidence claim
> bind a value population with **no production route on one engine**? Raised
> because the interpreter capped-short `Wrote` value looked unreachable.
>
> ### ⛔ ANSWER: it BINDS. No exclusion, no seam.
>
> - ⛔ **No named durable exclusion.** It would let an **admitted closed-sum
>   arm** escape the very absolute oracle this property exists to require. *"If a
>   future backend produces a short write, the same reifier runs; correctness
>   cannot begin only when that environment becomes convenient to reproduce."*
> - ⛔ **No production seam** — and none is needed.
> - ✅ **The discharge is component-boundary evidence:** a test-local
>   `HostEffectBackendV1` → the real `dispatch_host_op_v1` → the real minted
>   `TransferCountV1` → the existing reifier → the `§38`-derived literal.
>   Production unchanged. See [[PX8-WROTE-ABS]].
>
> **Three durable boundaries decided it:** this node defines closure over every
> value the path **reifies**, derived from the **closed Rust sums** and not from
> whichever end-to-end fixtures are convenient; LOCKED `§38.1.7.2` explicitly
> admits `write returns 0 < n <= effective request -> Wrote n, including a short
> write`, with `remaining = effective - n`; and the shared dispatcher accepts
> exactly that range and mints the private count itself.
>
> ### ⭐⭐⭐ The generalizable rule — it governs the other three instances too
>
> **Evidence must distinguish every normatively different behavior over the
> component's ADMITTED SEMANTIC DOMAIN.** Mutations that are *extensionally
> equal over that domain* need no discriminator. ⛔ Conversely, a genuinely
> uninhabited value — excluded by the **closed type / validator / contract
> itself** — does not bind merely because one can write suggestive prose about
> it.
>
> ⇒ ⛔ **Derive the domain from the authoritative constructor and admission
> boundary FIRST, then ask reachability. Never infer semantic absence from the
> lack of a convenient top-level producer.** ⭐ Carry this to
> [[CONF-FMT8-LEVELTOK]], [[SEC1-IFC-R3]], and [[CONF-SEC4-REFL-PAIR]] — the
> other three instances of *"an operand I cannot construct is byte-identical, to
> any reader, to one not yet built."* **Three of the four sit under universally
> quantified properties, and this ruling is the test for all of them.**

### ⚠ RT-SPLIT is bundled into PX8 by the docs, and probably should not be

`RT-SPLIT` is a **pure maintainability decomposition** of
`cranelift_backend.rs` (22,081 lines) and its own text says it *"feeds no
G-gate."* `10-…:272-275` lists it among PX8's in-flight set, but on its own
terms it carries **no semantic obligation** in the closure property above.

**Ruling (Steward, 2026-07-22): `RT-SPLIT` does NOT gate PX8.** It is real work
and stays queued for the Runtime ring, but `ABI-R3` and `PX9` **do not wait for
it**. Holding 15 items behind a file-splitting refactor would be a pure
sequencing loss. If someone believes it *is* a semantic dependency, the burden
is to name which clause of the closure property it discharges.

## Why this matters downstream

`ABI-R3` (the derived operation inventory) and `PX9` (cross-domain
`System.Error`) are the two gates. **Nothing else in the completion program
moves until they open**, and between them they transitively gate every item
except `ABI-R1` and `ABI-S3`. `PX9` alone gates six items including the whole
of Track T.

## Notes

- **Do not close this issue on the sub-issue list going green.** Re-check the
  property. The list is evidence, not the definition — that distinction is the
  entire reason this file exists.
- The completion program's coverage record is `10-linux-abi-completion.md` §9.
  Update it when this closes.
