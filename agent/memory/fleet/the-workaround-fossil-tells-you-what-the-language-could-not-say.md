---
scope: fleet
audience: (see scope README) — anyone reading catalog code that looks needlessly contorted
source: SUB-2 framing, 2026-07-14 — grounding the cached-Nat carriers before writing the frame
---

# A workaround fossil tells you exactly what the language could not say

While grounding SUB-2, this turned up in `catalog/packages/Capability/Parsing`:

```ken
fn byte_unit_zero_int (unit : Bytes) : Int = bytes_length unit - bytes_length unit
```

**That is `0 : Int`, spelled as "the length of a thing minus itself."**

Nobody writes that on purpose. It is a **runtime-computed zero** — and it exists
because `bytes_length` is a `PrimReduction::Op`: it computes at runtime but is
**opaque to kernel conversion**, so the author could not get a *definitional* `0`
out of the byte world and reached for one the evaluator would produce.

Two lines away, the same pressure produced a whole **class**:

```ken
class ArgBytes {
  arg_bytes_field        : Bytes;
  arg_length_field       : Nat;                                            -- a CACHE
  arg_length_valid_field : ArgByteLength arg_bytes_field arg_length_field  -- the CALLER must PROVE the cache
}
```

**The caller had to supply the length AND a proof the length was right** — because
the length **could not be computed** in Ken. That is not a design; **that is a
scar.**

## ★ The lesson

**Contorted code in a healthy codebase is EVIDENCE, not noise.** When you find a
function that computes a constant the long way round, caches a value that ought
to be derivable, or makes a *caller* prove something the *callee* should know —
**stop and ask what the language could not express at the moment it was
written.** The contortion is a **fossil of a missing capability**, and it marks
the exact spot where the capability was missing.

**Corollary — when the capability finally lands, GO FIND THE FOSSILS.** SUB-1
added `bytes_nat_length` (a real structural fold). The moment it merged, **every
one of these workarounds became deletable** — but nothing in SUB-1's diff, tests,
or review would ever point at them. They are in *other packages*, they are
**green**, and they will sit there forever unless someone goes looking.

**A capability WP is not finished when it lands. It is finished when the
workarounds it obsoletes are gone** — otherwise the scar tissue outlives the
wound, and the next author cargo-cults the contortion as if it were the idiom.

## The trap on the way out (and it is the same wall)

Retiring the cache tempts you to keep the opaque `bytes_slice` (which takes
`Int`) and feed it a converted structural length. **That needs
`Equal Int (bytes_length bs) (cursor_nat_to_int (bytes_nat_length bs))` — which
is NOT PROVABLE**, because `bytes_length` is opaque to conversion. **It would
have to be a new postulate.** *You would be trading a per-caller fabricated
obligation for a permanent global one — the exact opposite of the point.*

**The escape is to go structural ALL THE WAY** (`take`/`drop`/`nth` on the
`List UInt8` view), so the opaque primitives **never appear in the consumer path
at all.** Zero new trust.

Sibling of [[a-dependency-is-met-when-you-can-write-the-obligation]] — that one
says *try to write the obligation before you kick the dependent*; this one says
*the code that already exists will tell you which obligations were unwritable.*
And [[never-pin-a-shape-that-cannot-state-its-own-contract]] is the design-time
version of the same instrument.
