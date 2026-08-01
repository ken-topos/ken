# Dependent data — making invalid lengths unrepresentable

> **Availability:** current for the checked `Vec` and `Fin` declarations and
> total `head` below. `zip` and `lookup` are unavailable pending `DS-5c`.
> **Authority:** explanatory; the normative contract is the
> [length-indexed-vector specification](../../spec/50-stdlib/60-length-indexed-vectors.md).

Dependent data lets a type record a fact about a value. For a vector, that fact
is its length. The type of `head` can therefore require a non-empty vector
instead of returning an `Option` or checking for emptiness at run time.

## Parameters and indices

In `Vec A n`, `A` is a **parameter**: every constructor keeps the same element
type. The length `n` is an **index**: each constructor chooses the result index
that describes the value it constructs. `VNil` constructs length `Zero`, while
`VCons` constructs length `Suc n`.

The distinction is checked by the declaration itself:

```ken
data Vec (A : Type) : Nat → Type where {
  VNil : Vec A Zero;
  VCons : (n : Nat) → A → Vec A n → Vec A (Suc n)
}
```

This is the brace/`where` declaration form the current parser accepts.
The normative specification displays the canonical constructors as lowercase
`vnil` and `vcons`; copying that display literally does not parse today, so this
checked page names the accepted surface form instead of silently presenting it
as the canonical spelling. The same constructor-name rule is why the checked
`Fin` declaration below uses `FZero` and `FSuc` rather than the specification's
lowercase `fzero` and `fsuc`.

The index is not a comment or a separate proof attached later. It is part of
the constructor's result type, so every `Vec A n` carries its length in the
type checked by the kernel.

## A bounded index

`Fin n` represents an index strictly below `n`. Its constructors make the
bound structural: neither constructor can produce `Fin Zero`, and extending a
smaller bounded index produces one for a successor bound.

```ken
data Fin : Nat → Type where {
  FZero : (n : Nat) → Fin (Suc n);
  FSuc : (n : Nat) → Fin n → Fin (Suc n)
}
```

A value of `Fin n` is therefore already a witnessed in-bounds index. An API can
accept it directly instead of taking an unrestricted `Nat` plus a separate
side-proof.

## Total `head`

The input type `Vec A (Suc n)` excludes the empty constructor. That makes
`head` total: the definition needs no empty case, returns `A` directly, and
performs no run-time emptiness check.

```ken
fn head (A : Type) (n : Nat) (v : Vec A (Suc n)) : A =
  match v {
    VCons m x xs ↦ x
  }
```

This omitted `VNil` arm is exhaustive because `VNil` constructs
`Vec A Zero`, which cannot inhabit the required `Vec A (Suc n)`. The landed
indexed-match mechanism discharges that impossible branch and still gives the
kernel a total eliminator. These declarations and `head` are the
[landed rows in the normative API table](../../spec/50-stdlib/60-length-indexed-vectors.md#4-the-total-api).

## Availability boundary

- **`zip`: unavailable pending `DS-5c`.** Its specified two-vector recursive
  step needs the follow-on dependent-match convoy capability.
- **`lookup`: unavailable pending `DS-5c`.** Its specified recursion over a
  `Fin` index and the vector tail needs the same follow-on capability.

`DS-5c` is the elaboration-completeness boundary described by the
[normative specification](../../spec/50-stdlib/60-length-indexed-vectors.md#6-dependent-match-refinement--tail-landed-ziplookup-on-ds-5c).
It is distinct from
[`KERNEL-NESTED-IND`](../../docs/program/issues/KERNEL-NESTED-IND.md), which
tracks nested strictly-positive inductives; indexed families elaborate without
that separate capability.

The equational theory remains deferred with the specification: this page does
not add `tail`/`lookup` computation laws, `zip`/`map` naturality, the
`zip`-`unzip` round trip, or the length/`to_list` bridge.

For the general `data` and `match` surface, continue with
[`data` and `match`](surface-reference.ken.md#3-data-and-match). For proof
construction, use [Proof techniques](proof-techniques.ken.md).
