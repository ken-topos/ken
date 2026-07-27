# DS-9 findings — lawful JSON codec assembly

This report is deliverable D7 for
[`ds-9-json-codec.md`](ds-9-json-codec.md). It records the friction found while
assembling the landed data and parsing tier rather than placing inward-facing
findings in the reader-facing catalog entry.

## F-0 · Blocker: the required recursive value model is a nested inductive

**Observed.** The frame requires an ordinary `Json` whose array arm contains
`List Json` and whose object arm contains `List (Pair String Json)`. The current
kernel rejects the first recursive arm:

```text
PositivityViolation(
  "non-strictly-positive occurrence of D in constructor ... arg 0"
)
```

The committed acceptance test reduces this to
`data JsonNestedProbe = JsonNestedLeaf | JsonNestedArray (List
JsonNestedProbe)` and observes the same rejection. This is the nested-inductive
boundary that `spec/10-kernel/14-inductive.md §8.5` explicitly leaves deferred;
it occurs before the cursor, decoder, prefix, or fuel proofs are admitted.

**Why DS-9 cannot route around it.** Replacing `List Json` with a mutually
recursive `JsonList` is also outside the landed inductive fragment. Flattening
array cells into extra `Json` constructors would weaken D1's value model and
make malformed internal states representable. A Church encoding would put the
recursive carrier in a negative position and would not be the ruled ordinary
inductive.

**Route.** Kernel capability through the enclave. The frame must either wait for
nested strictly-positive inductives or receive a new Architect-approved value
representation. DS-9 does not silently weaken the array/object model.

## F-1 · JSON numbers need a structural printable carrier

**Observed.** `Capability.Parsing.Numeric` parses decimal text into opaque
arbitrary-precision `Int`, but Ken has no total `Int` destructor, division,
remainder, or `Int → Nat` bridge from which a decimal printer can be derived.
The numeric package already records `show_int` as deferred for this reason.

**DS-9 resolution.** `JsonInteger` is a structural canonical decimal integer:
zero, or a non-zero leading digit plus remaining digits, with an optional minus
sign. The codec covers the full unbounded integer subset without adding trust.

**Residual and route.** Fraction and exponent syntax remain outside DS-9.
A future numeric package should supply a structural JSON-number syntax carrier
with a proved parser/printer law. This is a Foundation catalog abstraction,
not a kernel feature.

## F-2 · A lawful JSON string escape codec is not in the landed tier

**Observed.** The tier supplies the total `String ↔ List Char` view and lawful
character equality, but no JSON string escaping/unescaping component with a
round-trip law. Emitting every `String` directly between quotation marks would
silently produce invalid JSON for control characters, quotation marks, and
reverse solidus.

**DS-9 resolution.** The proved domain explicitly requires string values and
object keys to contain only unescaped JSON scalar characters. The runtime
decoder rejects the excluded characters rather than accepting a representation
the encoder cannot lawfully invert.

**Residual and route.** Add a reusable `List Char` JSON-string escape codec with
an explicit safety predicate and round-trip theorem. This is a Foundation
catalog abstraction; ergonomic literal or pattern sugar, if later justified,
routes separately to Ergo.

## F-3 · Decoder combinators expose behavioral operations before equational laws

**Observed.** `Capability.Parsing.Decoder` supplies transparent `map`, `bind`,
`alt`, repetition, recursion, progress predicates, and a whole-input law, but no
equational laws for token success, sequencing, backtracking, or recursive-layer
unfolding. A codec round-trip proof therefore has to reconstruct those equations
at every grammar boundary before it can state the JSON-specific induction.

**Resolution status.** The partial branch identifies the required local
character-token and codec-prefix proof surface. Those proofs cannot be admitted
until F-0 supplies an admissible recursive value model.

**Residual and route.** Promote the reusable equations into the parsing catalog
if a second proved codec needs them. This is Foundation-owned reusable proof
surface, not a kernel-reduction defect.

## Trust accounting

DS-9 introduces zero trusted declarations. Its string and object-key proof
branches depend positively on the already-landed
`string_to_list_char_retraction` axiom. The other constructor branches do not
add a string-view assumption.
