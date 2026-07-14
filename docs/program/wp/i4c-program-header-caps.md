# WP — Runtime I-4 §C: program-header capability clause (enclave lead)

**Owner:** spec enclave (spec-leader / spec-author / conformance-validator) ·
**Size:** M · **Base:** `origin/main @ 66c6e15f` · **Feeds:** §B Runtime I-4
build (record mint) + a Language grammar-parser dependency · **Design source of
truth:** Architect I-4 ruling `architect/work:docs/program/wp/i4-program-caps-ruling.md
@ be52d82b`, §C + §A.2/§A.5.1/§A.7. **Design-only for the enclave** (spec +
grammar + ADR + contract wording + typed-API signature contract); no
`crates/**` impl here — that is §B / the Language dependency.

## Objective

Add a **per-family capability clause** to the anonymous `program` header — the
**sibling of `admits`** — as the **declared** source of a program's effect
authority. Amend ADR-0014 to record it, and update the Program-I contract so a
program's capabilities are **minted from its declaration**, not from a launch
grant. This realizes Pat's reframe: **capabilities are a program-*validity*
property declared in the program** (concern (1) — "what the language accepts as
a valid program"), **not** a runtime/CLI constraint (concern (2) — SELinux, CLI
grants — which Ken does not own at present).

## Settled inputs — PINNED, do not re-derive (cite ruling `be52d82b`)

- **The reframe (Pat, 2026-07-13).** Ken owns concern (1) only. Capabilities
  are declared *in* the program. The CLI launch-grant fork is **gone** (it was
  concern (2)). Do **not** reintroduce any grant/deny/default CLI surface.
- **Home = the anonymous `program` header** (identity is the path, no name
  token; ADR-0014 MRES-4 / MRES-4e) that already carries `admits`. The
  capability clause is a **second manifest on that same header**.
- **`admits` and the capability clause are ORTHOGONAL manifests — not one
  list.** `admits` = the **instance-dictionary** channel (which packages'
  instances resolve ambiently — a coherence/dispatch property, **zero
  authority**; read by the **elaborator** admission gate → `UnadmittedInstance`,
  unchanged). The capability clause = the **authority** channel (which effect
  families the program may perform, at what level — a security property, **zero
  dispatch**; read by the **runner** at mint). **Same home, different namespace,
  different reader, different check — do NOT couple them.**
- **Per-family authority declaration** — e.g. `FS AFull`. The **concept is
  fixed**; the clause keyword spelling (`grants` / `requires` / `caps` /
  `authority` / …) is **deferred to you** (defer spelling, not concept) — pick
  one, justify in a line.
- **Source = declaration.** The runner reads the header's declared FS authority
  `a` → mints `ProgramCaps a = MkProgramCaps (Cap a)` at that `a`. This replaces
  Program-I contract §3.1 "minted exactly from the launch grant" → "minted
  exactly from the declaration." **No external grant counterparty exists.**
- **`main`** = `proc main (input : ProcessInput) (caps : ProgramCaps a) :
  HostIO ExitCode`, with `a` the header-declared authority. Consistency
  ("header-declared ⊇ what the body performs") is **ordinary cap-passing
  well-typedness** (kernel-backed, spec §36 §2.5 / §62 §1), **not** a runner
  grant-compare. Family-level containment is therefore **fully static**: a
  program with **no** declared `Cap FS` cannot construct **any** FS op.
- **Pat resolved §A.5.1 → Option (ii): static authority-*level* enforcement via
  a typed capability-API wrapper.** A thin typed layer (`writeFile : Cap AFull
  -> …`, `readFile : Cap APartial -> …`) sits over the **UNCHANGED** polymorphic
  I-3 producers; an `APartial` program that attempts a write is **ill-typed**.
  §C **specifies the typed-API signature contract**; §B (Runtime) **implements**
  the package. **Do NOT re-type the raw I-3 producers** — that would reopen the
  I-3 op surface (guardrail); the wrapper is the non-reopening realization.
- **Zero-TCB.** The clause is surface grammar + a runner read; **no kernel
  rule, no trusted primitive.** If authoring finds itself adding one, **stop
  and flag the Steward** (contract breach).

## Deliverables — mandated outline (each ends in a concrete choice)

1. **Grammar** (`spec/30-surface`, the N4 `program`-header production). Add the
   capability-clause production as a sibling of the `admits` list. **Choose**
   the clause keyword and the per-family authority syntax; give the exact
   production **and one example header carrying both `admits` and the capability
   clause**. Ensure no grammatical collision with `admits` (if a shared spelling
   is forced, resolve it and say why).
2. **ADR-0014 amendment — FOLDED INTO THIS WP's landing** (Architect lean:
   atomic with the grammar it documents, no cross-doc drift; a standalone
   amendment only earns its own WP if it grows past a clause). A short
   subsection: capabilities as a program-header manifest **alongside** `admits`;
   the two orthogonal channels; the **reader split** (elaborator vs runner);
   source = declaration. **Not** a new ADR (subsume-don't-proliferate).
3. **Program-I contract wording** (`docs/program/ken-cli-program-i-contract.md`
   §1.3-step-3, §3.1): "minted exactly from the launch grant" → "minted exactly
   from the program's declared capability clause"; note the reframe (concern-(1)
   only; CLI/OS runtime constraint is out of scope).
4. **Typed capability-API signature contract** (spec-level, for §B to
   implement). Specify `writeFile : Cap AFull -> …`, `readFile : Cap APartial ->
   …` over the polymorphic core, and **resolve the attenuation ergonomics**: a
   read+write program must attenuate its `Cap AFull` to `Cap APartial` for reads
   — either expose a **Ken-callable `attenuate`**, or keep `readFile`
   authority-polymorphic. **Pick one**, note the trade. Signatures only — no
   package implementation here.
5. **Conformance** (`conformance/`): assert the static gates on **named
   variants** — (a) a header with the capability clause parses and carries the
   declared authority; (b) a program performing an FS effect with **no**
   capability clause is **rejected ill-typed** (missing-capability diagnostic —
   the family-containment gate); (c) `admits` and the capability clause are
   **independent** (admit instances with no capability, and declare a capability
   with no admits). **No bare `is_err`.** If a gate needs §B to be reachable,
   mark it **RED-until-§B** honestly (conformance-oracle-grounding discipline) —
   do not hand-feed a green.

## Acceptance criteria (testable)

- Grammar production defined; example header validates; `git diff --check` clean.
- ADR-0014 amendment folded in; two-channel orthogonality + reader split stated.
- Contract §1.3/§3.1 wording updated (source = declaration).
- Typed-API signatures specified with the attenuation-ergonomics choice made.
- Conformance asserts specific named variants (or honest RED-until-§B).
- **Validate TARGETED only** (`scripts/ken-cargo`), never `--workspace` — CI
  owns the locked gate. Anchor line numbers verified against landed spec at
  pickup.

## Do-not-reopen guardrails

- **No CLI launch-grant / grant-deny / default surface** (concern (2), out of
  scope).
- **Do NOT couple** the `admits` admission check with the capability mint — two
  orthogonal readers.
- **Do NOT re-decide** Option (ii) (static typed-wrapper) or the reframe.
- **Do NOT reopen** the I-3 op producers — the typed API **wraps** them.
- **@conformance-validator casts the Spec/Fidelity vote** (+ testability of the
  named-variant gates).

## Sequencing

§C is the **lead** — it defines the surface. §B Runtime build consumes §C's
minimal surface (§B steps 1/3/4 can start on a minimal declared-authority read
while the full grammar lands). The **grammar-parser implementation** of the
clause (parsing the header) is a **Language dependency** — the Steward decides
at §B authoring whether it is a small Language sub-WP or folded into §B. The
Architect is **on-call** for design questions during build (ruling `be52d82b`
is authoritative).
