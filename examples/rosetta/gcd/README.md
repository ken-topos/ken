# Greatest common divisor

Compute gcd(a, b) for two non-negative integers using the Euclidean algorithm.

Reference: <https://rosettacode.org/wiki/Greatest_common_divisor>

## Status

**Working** (expected). All declarations should elaborate. `main = gcd(12, 8) = 4`.

## Implementation notes

- Uses the subtraction-based GCD: repeatedly subtract the smaller from the
  larger until they are equal; that value is the GCD.
- Fuel-bounded recursion: `natGcdFueled` takes an explicit `fuel : Nat`; the
  SCT accepts it because `f` is a structural sub-term of `Suc f`. The fuel
  is set to `a + b` in the `natGcd` wrapper, which is sufficient for the
  subtraction-based algorithm.
- `natSub` is monus (floors at 0 if the subtrahend exceeds the minuend).
- `natCmp` uses structural recursion on both arguments simultaneously.
- `OrdResult` / `Lt` / `Eq` / `Gt` come from the L3 prelude (no redefinition
  needed).

## Alternative: Euclidean modulo-based GCD

The idiomatic form `gcd(b, a mod b)` would require `natMod`. That function is
structurally well-founded (remainder < divisor), but the SCT checker may not
verify this automatically without a contract annotation. Deferred as a
follow-on once SCT/contract tooling matures.

## Oracle

`main` evaluates to `4`.
