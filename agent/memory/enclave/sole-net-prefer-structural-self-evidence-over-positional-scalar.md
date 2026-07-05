---
scope: enclave
audience: (see scope README)
source: private memory
  `sole-net-prefer-structural-self-evidence-over-positional-scalar`
---

# When a check is the sole runtime net, prefer a self-evidencing representation

**D3 ruling on `wp/fs-read-file-lines-flip` (Cap runtime representation),
2026-07-04, my soundness lane.** The choice: bless the landed
`EvalVal::Int(level)` projection as the runtime `Cap` value, or require a real
opaque `EvalVal::Cap(capabilities::Cap)`. I ruled **`EvalVal::Cap`** and it
generalizes.

**The shape.** The runtime capability gate `authorizes` (`ken-interp/eval.rs`)
is the SOLE net for FS authority (I'd proved this by isolation-flip in FS-driver
Phase 2 — the driver's read-permit flips on this call alone). It decoded
`EvalVal::Int(n) → Authority(n) → Cap::mint(...)` — i.e. **re-minted a cap from
a bare scalar**. `EvalVal::Int` is *also* every ordinary integer; the same
variant means both "the number n" and "authority level n," told apart ONLY by
the position it sits in (`read_bytes`'s `Cap`-typed first arg). That is a
representation **collision**.

**Sound-today ≠ robust.** The scalar rep IS sound today, but only via a
three-part NON-LOCAL argument: (i) the kernel type-gate keeps a bare `Int` out
of the `Cap`-typed position; (ii) the reachability precondition — the gate only
ever runs on a kernel-checked term (tested not trusted posture needs
reachability precondition); (iii) no surface intro/attenuate can transform the
bound value. The gate itself holds NO evidence its input is a genuine mint. Any
future weakening of (i)–(iii) silently converts an attacker-influenced scalar
into authority.

**The ruling principle.** For the SOLE soundness net, the input's integrity
should be **structural and local**, not contingent on a non-local
type-gate+reachability chain every future change must preserve. A dedicated
opaque variant (`EvalVal::Cap(Cap)`) makes the net self-evidencing: it consumes
the real minted token carrying its exact authority, no re-mint from a scalar,
and a stray non-cap value fails **closed by construction** (`_ => false`), not
"by presumed absence." Forgery is structural too: the variant has exactly one
Rust producer (the trusted mint site) and no surface syntax / `eval` rule yields
it.

**Why it's proportionate, not gold-plating (the honest boundary).** I named
`EvalVal::Int` as sound-but-fragile, NOT a live hole — the ruling buys
robustness of the sole net, it doesn't fix a bug. It clears the bar because: (a)
it's the *sole* net (structural self-evidence is warranted precisely there); (b)
cost is one OUTER-RING enum arm — `EvalVal` is an interp runtime value, NOT a
`Term`/`Decl` variant, NOT in the kernel, NOT in `trusted_base`, so the
"kernel-untouched / zero trusted_base delta" AC stays grep-clean; (c) the
collision (re-mint-from-bare-int) is real in the landed code, not hypothetical.
When those three hold, prefer the self-evidencing rep. When the cost WOULD touch
the TCB/kernel/`Term`/`Decl`, the calculus flips — then the non-local argument
may be the right trade and you document the reachability precondition instead.

Sibling of kernel backed claim grep the emission not the name (verify the trust
LEVEL of a check) and differential verify which mechanism is the net (know which
call is load-bearing before ruling on it). The "sole net" status is what earns
the stricter representation.
