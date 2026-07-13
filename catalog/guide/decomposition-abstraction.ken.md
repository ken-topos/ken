# Decomposition & abstraction — reusable design moves

Which shape to reach for is a design choice with real consequences in a
verified language: the wrong one costs a rewrite once the proofs are half
written. This strand distills design moves the fleet has paid for in real
build failures (`agent/memory/enclave/`) into public guide prose, in Ken's
own terms.

## Index

1. [`class` vs. explicit dictionary](#1-class-vs-explicit-dictionary)
2. [Subsume-don't-proliferate, unless trust
   differs](#2-subsume-dont-proliferate-unless-trust-differs)
3. [Structural self-evidence at a trust
   boundary](#3-structural-self-evidence-at-a-trust-boundary)
4. [Reusable moves — quick reference](#4-reusable-moves-quick-reference)

## 1. `class` vs. explicit dictionary

A `class` buys **dispatch**: the caller doesn't say which instance, the
compiler resolves it from the type. That is worth its ceremony exactly when
a component genuinely needs to work over more than one carrier chosen at
different call sites. Below, one generic function is dispatched per call
site by the carrier type:

```ken example
class Combine a {
  combine : a → a → a
}

instance Combine Bool {
  combine = λx
  y.match x {
    True ↦ True;
    False ↦ y
  }
}

instance Combine Int {
  combine = add_int
}

fn combine_twice (a : Type) (d : Combine a) (x : a) : a = d.combine x x
```

When there is exactly **one** carrier in view, or the "class" would only
ever have one instance in this component, skip the class and take the
operation (and, if it has one, its law) as a **bare, explicit parameter** —
the *unbundled* encoding, the same shape
`conformance/challenge/C5-verified-sort/sound-verified-sort.ken` threads a
comparator through. No `class Ord` is needed here: `leq` and (if a law were
required) its hypotheses are threaded as ordinary parameters, fully
generic in `a`:

```ken example
fn max_of (a : Type) (leq : a → a → Bool) (x : a) (y : a) : a =
  match leq x y {
    True ↦ y;
    False ↦ x
  }
```

The unbundled form is not a lesser fallback — it is **semantically
identical** to a bundled dictionary (`Ord a` is exactly the Σ-record of the
same fields), and it sidesteps a real elaborator gap: a class-typed
parameter's fields are only projectable by ordinary Σ-projection when the
dictionary is bound as a plain term (`(d).leq`); projecting one *inside a
type position* (a law field's own signature referencing another field) does
not parse yet. So when a report says "generic-over-an-abstract-type-var is
unbuildable, we need a bundled `where`-constraint first," check the actual
blocker before accepting it: the **implicit** `where`-resolution path
(picking an instance from a concrete type) genuinely needs a registered
head, but the **explicit** path (a Π-bound dictionary or an unbundled
operation-plus-law-hypotheses) does not, and is very often what the
component actually needs.

## 2. Subsume-don't-proliferate, unless trust differs

Prefer folding a new need into an existing general mechanism over minting a
parallel one (`docs/PRINCIPLES.md`) — **but check the trust level of both
sides first.** If the specific component is kernel-re-checked (an ordinary
`declare_def`, proved) and the general mechanism you'd fold it into is
trusted/outer-ring code (a Rust primitive, an unverified driver), folding
the specific case in would **move it out of the kernel's protection** — a
trust-level regression, not a simplification. Rule **coexist** instead: keep
the proof-carrying piece proof-carrying, and satisfy "this is an instance of
the general shape" at the **data/signature** level (the specific case's
type is literally an instance of the general family's type), not by
merging the code paths.

Subsume freely when both sides already sit at the same trust level — that
is the common case and the default. The check is only "does this fold move
something out of kernel re-checking," not a reason to hesitate on every
generalization.

## 3. Structural self-evidence at a trust boundary

When a value crosses a genuine trust or authority boundary (a capability, a
security-relevant token, anything a check treats as *proof* that something
was authorized) and a single check is the **sole** net that stands between
that value and being trusted, prefer a representation where the check's
input is **self-evidencing** — structurally impossible to produce except by
the legitimate path — over reusing a general-purpose type that merely
*happens* to be sound today because of a non-local argument (a type-gate
elsewhere, a reachability precondition, an absence of any other producer).
Prefer a dedicated wrapper for an authority-carrying value:

```ken example
data Cap =
  MkCap Int

fn cap_level (c : Cap) : Int =
  match c {
    MkCap n ↦ n
  }
```

...over reusing the ambient `Int` type directly wherever "a capability
level" is needed. Both are sound *today* if nothing else in the program can
produce that Int in that position — but that soundness now depends on
every future change preserving a non-local invariant, instead of being true
by the value's own shape:

```ken ignore
fn cap_level_reusing_int (level : Int) : Int = level
```

This is not a blanket "wrap everything" rule — it costs a real constructor
and a projection, so reserve it for the value that is genuinely the sole
net for a trust-relevant property. A value with no such role gets no
special treatment; reach for a dedicated wrapper only where the collision
you're avoiding — "this ordinary-looking value is secretly load-bearing for
authority" — is real.

## 4. Reusable moves — quick reference

Use this as a design self-check while writing a catalog entry, or as a
review checklist while reading one:

| Question | If yes |
|---|---|
| Does this component need dispatch across more than one carrier, chosen at different call sites? | `class`/`instance` (§1) |
| Is there exactly one carrier in view, or would the class have one instance? | Explicit/unbundled parameter (§1) |
| Would folding this into a general mechanism move a kernel-checked piece into trusted/outer-ring code? | Coexist — satisfy "instance of" at the data/signature level (§2) |
| Do both sides of a proposed fold already sit at the same trust level? | Subsume freely (§2) |
| Is this value the *sole* check standing between an untrusted input and a trust/authority decision? | Prefer a self-evidencing representation (§3) |
| Does a "this is unbuildable" report name a concrete elaborator/kernel blocker with a file:line? | Trust it — but re-derive it yourself before treating it as a wall (§1's class-dict case is the recurring counterexample: it's very often only the implicit-resolution path, not the mechanism) |

## Findings

None yet — this strand is the V1 draft; the reusable-moves table above is
deliberately short (four moves), not exhaustive. As more of
`agent/memory/enclave/` proves durable through repeated catalog authoring,
fold the next validated move in here (retro-action wiring, `README.md`).

## References

- Wikipedia — [Type
  class](https://en.wikipedia.org/wiki/Type_class) — orientation on §1's
  dispatch mechanism; Ken's own encoding (a class is an ordinary Σ-record,
  no separate kind of declaration) is `spec/50-stdlib/51-lawful-classes.md`.

```ken
proc main (_input : ProcessInput) (_caps : ProgramCaps)
  : HostIO ExitCode visits [Console] =
  host_program (print_line "decomposition guide ok")
```
