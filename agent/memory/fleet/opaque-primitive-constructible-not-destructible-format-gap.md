---
scope: fleet
audience: (see scope README) — anyone framing a "show"/"format"/
  "serialize"/"to-string" deliverable over an opaque primitive (`Int`,
  `String`, `Bytes`)
source: CC2 `Text.Numeric` (`show_int`) ruling, grounded @ main `6088e0b8`
---

# An opaque primitive is constructible but not destructible — the format gap

Ken's opaque primitives (`Int`, `String`, `Bytes` — `OpaqueType`) are
**constructible but not destructible**, and any operation on the
*destructor* side hits a substrate gap. Ruled for CC2 `Text.Numeric`
(`show_int`).

**The asymmetry.** Opaque `Int` exposes only
`add_int`/`sub_int`/`mul_int`/`eq_int`/`leq_int` (grep the registration
site in `crates/ken-elaborator/src/numbers.rs`, or `add_int`/`sub_int`/
etc. as string literals across `crates/ken-interp/src/eval.rs` and
`crates/ken-runtime/src/runtime_ir_evaluator.rs`) — **no
`div`/`rem`/`mod`/`int_to_nat`/destructor** anywhere (currency-checked
2026-07-28: none of those primitive names exist in the tree). So **parse
(`String → Int`) is buildable** — fold digits `acc' = add_int (mul_int acc
10) digit`, pure *construction*. But **format (`Int → String`) is NOT** —
extracting decimal digits needs repeated `div/mod 10` (absent) or
structural recursion on the value (impossible: `Int` is opaque — no
destructor to case-split, and a repeated-subtraction loop fails structural
termination checking because the decreasing arg isn't a structural
subterm). No trick rescues it under a zero-new-primitive boundary.

**The ruling shape (reuse it):**
1. **Ship the constructor direction** (parse → the opaque value) fully —
   it's sound, no destructor needed.
2. **Ship a STRUCTURAL formatter** over a *destructible* type (`Nat`,
   `List DecimalDigit`) — `Nat/List Digit → String` IS buildable and
   genuinely useful (renders any structural numeric / the parsed digit
   form). Not a stub.
3. **Defer the opaque→structural hop as a NAMED substrate gap** (`show_int
   : Int → String` needs a `div`/`rem`-or-`int_to_nat` primitive [trust
   delta] or a structural-`Int` bridge + extensionality cert). Fast-follow.
   **Never fake it** — no bounded-range lookup posing as total, no
   smuggled primitive.
4. **Keep verified round-trip laws at the structural level** — `parse_digits
   (format_digits ds) = ds`, crossing NEITHER the opaque `Int` NOR the
   opaque `String` boundary. A `parse_int (show_int n) = Ok n` law is
   forbidden: the opaque↔structural hop is un-provable (no destructor to
   prove it), exactly as a `String ↔ List Char` retraction is deferred to
   a keys cert. Confine the whole opaque↔structural debt to the one
   deferred gap.

**Recognize it early:** before framing a "show"/"format"/"serialize"/
"to-string" deliverable over an opaque primitive, grep its op inventory for
a destructor (`div`/`rem`/`*_to_nat`/case-eliminator). If absent, the
format direction is a named gap, not a build task — ship the structural
formatter and defer the hop. Same discipline as a String-keys injectivity
cert and a Bytes-keys substrate descope — ship what rides landed facts,
name the gap, don't mint trust under pressure.
