---
scope: enclave
audience: (see scope README)
source: private memory `trusted-primitive-refinement-codomain-witness`
---

# A trusted primitive with a refinement-typed codomain must establish it

**The distinction (why refinement codomains are different).** For a bare opaque
type (`IntN`), a trusted primitive producing an out-of-range value is only a
**wrong value** in the tested-not-trusted ring — never a false proof (no witness
to violate); conversions' `int_to_intN_raw` is fine on that basis. But a
**refinement-typed** codomain (`Char = {c:Int | isScalar c}`,
`isScalar c = IsTrue(inRangeBool c) = Equal Bool (inRangeBool c) True`) carries
a **proof**. A `declare_primitive` (trusted, no body check) whose declared type
is `String → List Char` PROMISES every element satisfies `isScalar`. If its Rust
impl produces a non-scalar codepoint typed as `Char`, a downstream projection
(`(c:Char) → IsTrue(inRangeBool (proj c))`, provable from the refinement) yields
`IsTrue(inRangeBool <non-scalar>) = Equal Bool False True → Bottom`. **A
fabricated refinement witness is a genuine soundness hole**, even though the
witness is *erased* at runtime (`{x:A|φ}`→`A`) — the hole is at the TYPE level
(the primitive's type is a promise the kernel can't check).

**Ruling posture (prefer, will rule definitively on the implementer's
proposal).** Establish the refinement **by construction through the landed
reducing check**, failing CLOSED — not by trusting an external invariant. For
L3-strings `s2l`:
- **Preferred (b):** route each decoded codepoint through the existing
  refinement-checked `intToChar : Int → Option Char` / `inRangeBool`
  (`decimal_char.rs:225,253`) — the witness is honest-by-the-reducing-check; a
  non-scalar (decoder bug, malformed UTF-8, domain drift) yields `None`/decode-
  error, never a smuggled `Char`. Minimizes trust to "UTF-8 byte→codepoint
  decode is correct" (netted by the round-trip law + boundary corpus), REUSES
  the construction I already gated (subsume-don't-proliferate).
- **(a) decode-direct trusting Rust's `char`** is ALSO sound iff the domain
  identity is proven+documented as a trusted bridge (Rust `char` =
  `[0,0xD7FF]∪[0xE000,0x10FFFF]` = `inRangeBool`'s range = `55295`/`57344`/
  `1114111`, which DO match) — but it ADDS that bridge to the trust base and
  fails OPEN (drift/decoder-bug fabricates a witness silently). Inferior; accept
  only with a measured perf justification (per-codepoint `inRangeBool` = two
  `leq_int`s, not a cliff).
- **Malformed input:** clarify whether `String` is guaranteed-valid-UTF-8 (frame
  says Rust `String`, so yes — surrogates structurally impossible from valid
  UTF-8). Either way `s2l` must never fabricate a `Char` from a non-scalar.

**The discriminating gate test I'll require (AC1):** a construction path
reaching a surrogate/out-of-range codepoint yields NO `Char`
(decode-error/`None`), never a `Char` value — flips against a trust-Rust-direct
build that would smuggle it. Plus round-trip identity (AC2) + the UTF-8 boundary
corpus (AC3: U+0000/007F/ 0080/07FF/0800/FFFF/10000/10FFFF + surrogate guard).
Kernel-untouched, tested- not-trusted, workspace-green — same posture as
F1/conversions.

**CONSULT DELIVERED (`evt_4c2azcd2rz7b2`, 2026-07-03).** Implementer chose
option (a) decode-via-`str::chars()` + a `debug_assert!` cross-check, and gave a
sound reason (b)-literal is wrong: `prim_reduce` is a pure `&[EvalVal]->EvalVal`
with no way to re-enter `eval` on elaborated terms — re-entering `intToChar`
mid-native-reduction has no precedent. **I UPDATED from my banked "prefer
(b)"**: the witness is established by the INPUT INVARIANT (Rust `String`=valid
UTF-8 -> `str::chars()` yields only scalars, structurally excluding surrogates)
and TRANSPORTED, not fabricated — no non-scalar can reach the `Char` ctor, so
(a) is genuinely SOUND here, not merely acceptable. Ruled ACCEPT + 4 conditions:
(1) document the range identity as a NAMED trusted bridge (Rust char range =
inRangeBool range = 55295/57344/1114111); (2) `debug_assert!` accepted as a
DRIFT TRIPWIRE (not the mechanism — the kernel trusts the declared TYPE
statically, the refinement is erased at eval-time, so NO runtime value->proof
reflection exists), must re-state the exact predicate; fail-open-in-release
accepted HERE ONLY because the input invariant makes the check dead on every
reachable input; (3) honesty flag — under a validated-`String` input the
surrogate/isScalar assertion is GREEN-BY-CONSTRUCTION not discriminating (no
fabrication path to catch; the real discriminating test lives at `intToChar`),
must be named as transported-not- flipped (anti-green-vs-green); (4) `l2s`
back-path must be safe `char::from_u32` (not `_unchecked`=UB) with a named inert
fallback (String is bare-typed -> wrong char = wrong value, never a false
proof).

**Generalizes:** for any trusted primitive producing refinement-typed values
(future proof-carrying/FFI boundary), the FIRST question is whether the input
type already establishes the refinement — if yes, trust+document the bridge; if
no (unconstrained domain), route through the checked construction and fail
closed. Cross-ref ADR 0010 + carrier canonicity axis for lawful class laws —
this witness axis is DISTINCT (refinement-witness fabrication, not eq→Equal).
