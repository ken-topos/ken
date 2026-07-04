# KNOWN-GAP: GAP-itree-w-style-match — surface `match` cannot destructure
# `ITree` (any instantiation)

## What's missing

The `[State]` effect (VAL2 #10) landed and its **construction** side is
fully usable from surface `.ken` source — `get`/`put`/`bind`/`runState`
all type-check and compose correctly (see `accumulator-factory.ken`,
which builds a genuine hidden-state accumulator program end-to-end up
through `finalResult : ITree Eff RespEff (Pair Bool Nat)`).

But there is no surface way to **observe** an `ITree` value from a
`.ken` program. The only surface destructuring mechanism is `match`,
and `match` on `ITree` — any instantiation, including one whose `f`
("other effects") row is uninhabited-in-practice — is unconditionally
rejected:

```
Internal("dependent match (Gap B): W-style or indexed recursive fields
are out of scope for this WP")
```

`ITree`'s `Vis` constructor has a continuation field
`k : Resp op -> ITree E Resp R` — a Pi-bound (W-style) recursive
occurrence, not a direct one like `List`'s `Cons x xs2`. `elab.rs`'s
match-compiler (`recursive_args`-based IH-slot emission, `elab.rs:824`)
explicitly rejects building an induction-hypothesis slot for any
W-style/indexed recursive field, rather than risk mis-building one
(documented in-place as a deliberate scope limit, not an oversight).
This fires **unconditionally** — regardless of whether the match's
motive is dependent, and regardless of whether the `Vis` arm is
actually reachable at runtime (confirmed empirically: even a trivial
non-dependent `match r { Ret v => v ; Vis op k => <dummy> }` hits it).

## What's NOT the gap

This is **not** a `[State]`-effect-specific gap — `get`/`put`/`bind`/
`runState` are correctly built and landed (VAL2 #10 is genuinely done).
It is a narrower, previously-latent limitation of the match compiler
that this WP's re-authoring newly surfaced: nothing before this WP
tried to `match` on `ITree` from surface `.ken` source (the existing
`state_effect_build_*` test suite drives everything via hand-built
Rust `Term` construction, never surface `match`).

## Impact

Any `.ken` program that constructs an `ITree`/`runState` value and
needs to consume its result (print it, branch on it, feed it to
another computation) is blocked — the effect can be *built* but not
*run and observed* from surface source.

## Fix needed

Extend the match compiler's IH-slot emission to handle a W-style
recursive field (Pi-wrapped, matching the kernel's own `method_type`
construction in `ken-kernel/src/inductive.rs`) — the same class of
extension `L-match-ih-fix` made for non-nullary *non*-W-style fields.
Routed to language-leader / Steward as its own capability WP; likely
elaborator-only (no kernel delta — `Term::Elim`/`method_type` already
handle this shape, per `effects/state.rs`'s own doc comment on why
`runState` itself was hand-built with `Term::Elim` instead of surface
`match`).

## Intended program (once resolved)

The rest of `accumulator-factory.ken` (already written, cut off above
the `main` in the current file): `unwrapRet` pattern-matches
`finalResult`'s `Ret`/`Vis` constructors to extract the
`Pair Bool Nat`, then `pairFst` extracts the discriminating check
(`r1 = 2 && r2 = 5`, i.e. two independently-driven calls against the
same hidden accumulator returning correct running totals), folded to
the usual `"PASS"`/`"FAIL"` oracle string.
