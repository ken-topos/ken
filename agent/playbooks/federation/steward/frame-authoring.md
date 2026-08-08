# Authoring a frame: fixed inputs, audits, and acceptance criteria

Steward task procedure. Read at the point of use. Governing playbook:
`../steward.md`. The release sequence is in `release-and-handoff.md`.

The brief must pin every settled decision as a fixed input (cite `/spec` and
the OQ register — never leave a decided fork open for a lower-tier model to
relitigate), give a mandated deliverable outline where each section ends in a
concrete implementable choice rather than a survey, list testable acceptance
criteria, and state the do-not-reopen guardrails. This is the frame — scope,
acceptance, sequencing, settled-decision pinning — not the full spec.

**Never ship a frame without this clause:** *"treat every anchor as perishable;
if a fixed input turns out false against the landed code, say so and escalate —
do not quietly build around it."* A T1-authored frame is still wrong sometimes,
and that clause is the only thing between a bad pin and a ring confidently
building the wrong thing.

**Frame by objective and acceptance, and treat any current-implementation-state
claim as perishable.** A frame saying "seam X currently does Y, patch this
hole" can go stale between authoring and elaboration, and a stale *"what is
broken"* is worse than a stale *"what is done"*: it misdirects the team to
rebuild removed unsoundness. Prefer describing the goal and acceptance, and tag
any current-state claim *"verify against the landed code, not this line."*

## Before you pin a fixed input: five audits, every time

A fixed input is only as good as the substrate it stands on, and **grounding
the names is not grounding the obligations.**

### (a) Dependency-DAG check

If the WP introduces an abstraction an existing package will consume, draw the
load order and look for the cycle. **An abstraction module must never depend on
its clients:** home each instance with the carrier it is over, and make the
generic module define its own parameterized result and error carriers. The
moment it reaches for a client's concrete type, the cycle returns.

Measured on CC3: a `ByteCursor` instance homed in the new `Cursor` module *and*
`Cursor` ordered before the CAT-5 that declares its `Source`, giving
Cursor to CAT-5 to Decoder to Cursor. **The tell was wanting cosmetic
symmetry** — "both instances in one module" — and cosmetic symmetry created the
cycle.

### (b) Constructibility audit, for every promised carrier field

For each field pinned at a structural type (`Nat`, `List`, ...), ask whether
the landed primitive can actually produce it. Opaque primitives (`Int`,
`Bytes`, `String`) are constructible but not destructible — reading a length,
index, or size *out* of one is exactly the hop that does not exist.

Measured on CC3: `remaining : Nat` pinned over raw `List Bytes`, but
`bytes_length : Bytes -> Int` and no `Int -> Nat` bridge exists. The field was
unproducible. **Opaque representation boundaries are design constraints, not
implementation details.** The landed idiom is a proof-carrying cached-`Nat`
wrapper: carry the `Nat` and prove it agrees with the opaque length. Never
convert, and never mint the missing primitive — that is a TCB delta and it goes
to the operator, not into a build WP.

### (b-prime) Seam and ABI audit: can the landed interface carry the value?

(b) asks whether a primitive can *produce* a pinned type. This asks whether the
landed interface can *carry* it. **Trace the value end-to-end through every
seam it must cross, not just the one the design names.**

Measured on I-5: ADR-0017's central security property is *"check and use share
the resolved fd."* The seam at `authorizes` was correctly verified as pre-cut,
but the seam below it, `HostHandler`, speaks only `fs_*(&[u8])`, and
`fs_dispatch` hands the original path bytes to the handler after the check.
Check and use cannot share an fd. The design was coherent, the TCB verdict was
right, and it was unbuildable through the landed ABI. **The tell: a design that
names one seam as already pre-cut. Check the seams it did not name.**

### (b-double-prime) Genericization audit

When a WP makes an existing concrete path generic over a trait, "is the trait
public?" is the wrong question. The right one: **can the generic version
perform every step the concrete version performs?**

**The tell is greppable** — the concrete type's inherent methods that the call
path uses, which the trait does not declare:

```sh
rg 'concrete_host\.\w+\(' <the path>   # methods the path calls
rg '^\s*fn \w+' <the trait>            # methods the trait declares
# the DIFFERENCE is the gap: a blocker, not a detail
```

Measured on I-6: `run_io<H>`, `HostHandler`, and `CaptureHost` were all
verified public and re-exported — all true — and a generic `run_program<H:
HostHandler>` was framed anyway. But the runner mints the program's capability
via `PosixHost::mint_fs_cap`, an inherent method, and `HostHandler` has no mint
operation at all. One line, and the whole WP was unbuildable as framed.

### (b-triple-prime) The general form: expressibility

Architect synthesis, 2026-07-14. **This subsumes the genericization audit**,
which is its trait-shaped special case. Full statement:
`agent/memory/fleet/never-pin-a-shape-that-cannot-state-its-own-contract.md`.

> **Every obligation this shape must carry — where does it get written, and is
> that place inside the shape's own checked vocabulary, or a reach outside it?**

Three consecutive expensive near-misses were one failure. A shape is pinned — a
trait, a type, an effect op, a spec claim — that must carry an obligation. The
obligation is not false and not expensive to prove. It is **unsayable in the
shape as pinned**:

- **I-6** — the trait had nowhere to say *"any host mints its own-identity
  cap"*, so the obligation lived on the concrete type.
- **The primitive-`Op` erratum** — conversion had nowhere to say
  `byteLength "abc" = 3`, so the spec said it in prose the kernel cannot check.
- **I-8** — `monotonic_now : {Clock} -> Int` had nowhere to say *"this read is
  at least the last"*: two `Int`s, no handle relating them.

**Why it is invisible in a green diff.** Tests exercise values; the gap is in
the type, ABI, or relation surface. The values compile and pass because the
missing thing was never a value — it was a place to write a guarantee. No
amount of green touches it. **The audit is not "run the suite." It is: try to
write each obligation in the shape's own vocabulary and see if your pen has
anywhere to land.**

**The escape hatches are the doors it arrives through:** a comment, a
per-consumer `Axiom` (unbounded TCB), a new trusted primitive (TCB growth), a
caller-fabricated value that manufactures the missing binding, a concrete-only
method the generic path cannot reach. All say the same thing — the shape had no
home for the obligation, so we put it somewhere nothing checks or somewhere
that costs trust.

**The audit:** enumerate the shape's contract obligations; for each, name the
in-shape checkable home (term, type, handle, method) where it is written and
checked. Any obligation whose only discharge is a reach-outside is an
expressibility gap. Either extend the shape to give it a home — and if that
home grows the TCB, it is the operator's call — or descope it honestly. I-8
shipped wall-clock-only with no ordering law, because a wall clock genuinely
has none; **the absence of a law is the truthful statement, not a gap.**

**This is structurally the design pass's catch, not QA's.** The implementer
builds values and they compile; QA tests values and they pass. Only the design
pass asks whether the shape can express its contract. That is why the
design-review edge exists at all.

### (c) Corpus-oracle enumeration

If the WP adds a file to a globbed directory (`catalog/`, `examples/`,
`conformance/`), it must satisfy every corpus-wide oracle, and those live in
crates the WP never touches. Targeted per-crate validation cannot see them, so
they surface as red CI at publish — after review, after the merge Decision, the
most expensive place to find them.

```sh
rg 'collect\(.*catalog|examples/rosetta' crates/*/tests/
```

Name each one in the ACs. "The formatter gate" is rarely the only one. (CC3:
AC6 named `ken_fmt.rs` and missed `kenfmt_c_capstone.rs`, giving red CI on a WP
that had passed QA, the Architect, and the author's own honesty gate.)

**When one of those oracles is a frozen baseline table, do not re-baseline it
to make the build pass.** A file created after the frame has no honest
pre-frame value, so the row you add is fabricated and its check is vacuous
forever. Re-scope the oracle to its own historical set and let a live-anchored
property cover new files — confirming that live net exists first, or you trade
a rubber stamp for a hole.

### (c-prime) Cite prose sites by GREP-ABLE PHRASE. A coordinate is a time-sensitive operand.

**Promoted 2026-08-08 from `RT-CONTSPEC-LEDGER`, and it binds the frame author
first because that is where the defect originated.**

**The rule: cite by a grep-able phrase with no number.** Reach for `path:line`
only when no phrase is stable, and then only at **the SHA you are handing off**.

> **Do not weaken this to "always attach a base to your coordinate." That
> repair PASSES on the case that produced the rule and still fails.**
>
> That frame qualified its coordinates **three ways** — *"All anchors are in
> `…/planning/static_transition.rs`"*, *"Measured by the Steward at
> `0fd9f6e8…`"*, and *"read every line number as an anchor to re-find, never as
> a value to check."* All three sat in one section. `D4` cited `:4729` and
> `:6304` **ninety lines below it** and inherited none of them in practice.
>
> ⇒ **Distance defeats every qualifier equally.** A qualification that lives in
> a different section does not travel to the citation. If a reader must scroll
> to learn which file and which tree a number belongs to, it is unqualified at
> the point of use.
>
> **And the base it named was true and irrelevant** — `0fd9f6e8` was one commit
> behind the `5da614ba` the node was released at, so the coordinates were
> correctly measured against **a tree no reader of that frame would ever check
> out.** That is worse than an unqualified number, for the reason the Adversary
> gave about the wrong filename: **it carries visible evidence of care.**

Downstream those coordinates acquired the wrong filename, and an auditor
checking whether `D4` discharged would have opened live unrelated production
code and concluded the deliverable never landed.

⇒ **A citation pointing at a real thing that is not the thing is worse than one
pointing at nothing.** A bad path 404s and gets fixed; a plausible-but-wrong
path gets believed, and it fails in the direction of "the work is missing" when
the work is present.

**Both numbers were also stale on the day they were written** — grepped at one
`main` and published in a frame based three merges later. Ask the frame's own
standing question of the coordinate: **which state was this measured in?** We
routinely ask it of a before/after figure and forget that a line number is the
same kind of operand — **and one destroyed by the very change the deliverable
performs.** A frame whose job is editing those lines has guaranteed its own
coordinates are wrong by the time anyone checks them. That is the spent-oracle
trap wearing a different hat.

**The phrase is the durable instrument.** It survives the edit; the number does
not. Where a number genuinely helps a reader navigate, write it as *"at
`<sha>`, around `path:NNNN`"* — an anchor to re-find, never a value to check.

### (d) Reuse must be proved behaviorally, not structurally

When a WP is framed as a specialization of landed substrate ("consume CC1-CC6,
do not rebuild them"), **the ordered shared-`ElabEnv` harness makes reuse look
true even when it is false.** Loading a dependency is not using it. A package
can declare the landed `Decoder` — so it appears in the closure and every
import check passes — and then shadow it with a private byte loop. A green
suite hides this perfectly.

**The AC must be behavioral: the landed abstraction must be driven.** The
Architect's phrase is the test to keep — *genuinely driven, not
declared-then-shadowed.* Press the mechanism, not the imports, and write the AC
so a reviewer can tell the difference.

## Frame patterns by WP type

Each of these was promoted after the shape cost a merge or a red `main`.

**A contract or boundary WP that cites in-flight builds** must make the merge
Decision a hard gate on those builds being green and merged. Cite the gates,
never restate their verdicts, so the audited contract equals the surface the
code exposes the day it lands. Never freeze a transient pre-build code state.
K-api's freeze-gate held the contract open exactly long enough to catch a
reversed quotient-respect `cast` direction, and released the instant they
converged.

**A capability-gate un-stage or reopen frame** must require the author to
re-verify that each un-staged net's per-branch obligation falls *within* the
landed capability's power — not merely that the capability merged. The K4/K5
arc proved this twice: an un-stage wrote "K4 landed, therefore the laws are
provable" flat, name-matching the merged capability, but the
concrete-`Eq`-conclusion laws escaped K4 into K5. State the conclusion-shape
axis as a hard AC: for each un-staged net, show its obligation reduces within
the landed capability; if any reduces to a further primitive, it stays gated on
that next capability.

**A frame that adds a new kernel `Term` variant** must enumerate *every*
soundness-relevant exhaustive walker that needs the new arm, not just one. K5's
AC6 named the termination walker (`sct.rs::collect_calls`) but not the
trust-accounting walker (`foreign.rs::collect_consts_in_tb`), whose omission
undercounts a postulate in `trusted_base_delta` and launders trust surface. It
surfaced as CI-red mid-merge. Enumerate the walker set as a hard AC — at
minimum termination and trust-accounting, plus subst, conv, and children — each
with the arm and, for the soundness-relevant ones, a neuter-the-arm flip test.

**A frame for a kernel reduction or completeness change** (whnf, iota,
`eq_reduce`) must require full-workspace-green validation and must not assert a
kernel-only diff. A sound completeness change makes an already-reducing path
reduce more completely, forcing migration of every downstream proof term that
was riding the old incompleteness — and those live both in the crate and in
shipped `catalog/packages/` proofs. K7's frame asserted "the `ken-kernel` diff
is the only diff", the build validated `-p ken-kernel` (153 green), and
`lawful_classes.ken` rode the same incompleteness, giving red `main` and an
Architect hold. So the frame must: (i) distinguish the soundness surface
(kernel-only, legitimately asserted) from the landing unit (workspace-wide);
(ii) make the no-regression AC **workspace-green in CI**, never a local `cargo
test --workspace` — local agents build and test only the touched crate,
`COORDINATION §12`; (iii) state up front that downstream proofs migrate
land-together in one workspace-green unit.

**A WP built on a cross-repo or external handoff** must name the epistemic
boundary: mark which facts are locally verifiable and which are
externally-sourced-and-trusted, and route confirmation to the cross-repo owner
rather than leaving the author to launder an unverifiable citation into a
normative spec. A *narrowing* of a co-owned contract is ambiguous by
construction — new divergence versus catching up to the counterparty's
finalized contract — so it routes to the cross-repo owner, never asserted
settled. The principle underneath: **Ken classifies epistemic status, never the
counterparty's mechanism.**

## Authoring acceptance criteria

Load `pin-a-property` (`agent/playbooks/tools/pin-a-property.md`) before
writing or amending an AC. You write the ACs the ring discharges mechanically,
so a badly phrased one costs a review round at build-team scale.

**Name the property first; the artifact is downstream of it.** Three of this
project's framing defects were one shape: a requirement stated in terms of the
artifact most recently looked at — a *population* requirement as a struct
change, an *authority* requirement as a call count, a *module-boundary*
requirement as a spelling class.

- **Require the per-pin evasion attempt in the frame itself.** A per-candidate
  reminder gets satisfied by the most salient control and silently skips the
  rest.

  > **And it must land as an AC, not as a hazards note.** Measured on
  > `RT-FNSPLIT-B2O` while this rule was already written: the requirement went
  > into the issue file under a heading reading *"Standing hazards for whoever
  > builds this."* The implementer ran one evasion attempt of several, and ran
  > the rest only after the same sentence arrived in a message — immediately
  > finding a real overclaim. **The paragraph was not wrong, not unread, and
  > not unclear. It was in a section whose grammatical mood is advice.** ACs
  > get discharged because something checks them; hazards get noted.
  >
  > **A sentence in a frame that tells someone to do something is an AC.** Give
  > it a per-pin enumeration (never "each pin" as a quantifier the reader
  > resolves), a named positive control that would fire if the work were
  > skipped, and a place to record the result per pin. If you cannot name the
  > control, you have stated a hope, not a requirement.
  >
  > **Audit your own frames before release:** read every advisory section and
  > ask which sentences are actually obligations. Those are the unguarded ones
  > by construction.

- **Ask which mechanism already enforces the property** before demanding a
  detector. The compiler is a legitimate answer and usually the strongest;
  never specify a test for something the language already refuses.
- **Give the honest answer a cell.** If an AC list has nowhere to record
  *"guarded by review, not by CI,"* it will be recorded as *"guarded."* State
  every residual arm.
- **When an AC is defeated repeatedly, diagnose before narrowing.** Ask what
  the defeats share; a granularity error is cheap to test and common. **A
  defeat count never licenses "unenforceable"** — that conclusion weakens a
  gate, so it must be demonstrated by building the candidate mechanism and
  showing it cannot work.
- **Narrowing an AC is a frame amendment, and it is yours to author and
  publish.** An amendment that is not on a fetchable ref has not happened; the
  ring reads the frame, not your message.
