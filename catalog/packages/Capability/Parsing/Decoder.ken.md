# Capability.Parsing.Decoder

`Capability.Parsing.Decoder` supplies progress-safe parser combinators over an explicit
`CursorOps`. Its result and error carriers remain generic in the cursor's
location type, keeping downstream source and argument diagnostics separate.

## 1. Definition

Ordinary rejection is backtrackable. Zero progress and impossible fuel
exhaustion are named, non-backtrackable failures so repetition cannot silently
loop or truncate.

```ken
data DecoderError loc = DecoderRejected loc | DecoderZeroProgress loc | DecoderFuelExhausted loc

data DecoderResult c loc a = Decoded a c | DecoderFailed (DecoderError loc)

const Decoder (c : Type) (loc : Type) (a : Type) : Type = c → DecoderResult c loc a

fn decoder_error_location (loc : Type) (err : DecoderError loc) : loc =
  match err {
    DecoderRejected at ↦ at;
    DecoderZeroProgress at ↦ at;
    DecoderFuelExhausted at ↦ at
  }

fn decoder_pure (c : Type) (loc : Type) (a : Type) (value : a) : Decoder c loc a =
  λcur. Decoded c loc a value cur

fn decoder_fail
      (c : Type) (el : Type) (loc : Type) (a : Type) (ops : CursorOps c el loc)
    : Decoder c loc a =
  λcur. DecoderFailed c loc a (DecoderRejected loc (cursor_locate c el loc ops cur))

fn decoder_map
      (c : Type) (loc : Type) (a : Type) (b : Type) (f : a → b) (decoder : Decoder c loc a)
    : Decoder c loc b =
  λcur.
    match decoder cur {
      Decoded value next ↦ Decoded c loc b (f value) next;
      DecoderFailed err ↦ DecoderFailed c loc b err
    }

fn decoder_bind
      (c : Type)
      (loc : Type)
      (a : Type)
      (b : Type)
      (decoder : Decoder c loc a)
      (next_decoder : a → Decoder c loc b)
    : Decoder c loc b =
  λcur.
    match decoder cur {
      Decoded value next ↦ next_decoder value next;
      DecoderFailed err ↦ DecoderFailed c loc b err
    }

fn decoder_seq
      (c : Type)
      (loc : Type)
      (a : Type)
      (b : Type)
      (first : Decoder c loc a)
      (second : Decoder c loc b)
    : Decoder c loc b =
  decoder_bind c loc a b first (λignored. second)

fn decoder_alt
      (c : Type) (loc : Type) (a : Type) (first : Decoder c loc a) (second : Decoder c loc a)
    : Decoder c loc a =
  λcur.
    match first cur {
      Decoded value next ↦ Decoded c loc a value next;
      DecoderFailed err ↦
        match err {
          DecoderRejected at ↦ second cur;
          DecoderZeroProgress at ↦ DecoderFailed c loc a (DecoderZeroProgress loc at);
          DecoderFuelExhausted at ↦ DecoderFailed c loc a (DecoderFuelExhausted loc at)
        }
    }

fn decoder_satisfy
      (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) (accept : el → Bool)
    : Decoder c loc el =
  λcur.
    match cursor_peek c el loc ops cur {
      None ↦ DecoderFailed c loc el (DecoderRejected loc (cursor_locate c el loc ops cur));
      Some value ↦
        match accept value {
          True ↦ Decoded c loc el value (cursor_advance c el loc ops cur);
          False ↦ DecoderFailed c loc el (DecoderRejected loc (cursor_locate c el loc ops cur))
        }
    }

fn decoder_token
      (c : Type)
      (el : Type)
      (loc : Type)
      (ops : CursorOps c el loc)
      (equal : el → el → Bool)
      (expected : el)
    : Decoder c loc el =
  decoder_satisfy c el loc ops (λactual. equal actual expected)

fn decoder_many_fuel
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (step : Decoder c loc a)
      (fuel : Nat)
      (cur : c)
    : DecoderResult c loc (List a) =
  match fuel {
    Zero ↦
      match cursor_remaining c el loc ops cur {
        Zero ↦ Decoded c loc (List a) (Nil a) cur;
        Suc rest ↦
          DecoderFailed
            c
            loc
            (List a)
            (DecoderFuelExhausted loc (cursor_locate c el loc ops cur))
      };
    Suc fuel2 ↦
      match step cur {
        DecoderFailed err ↦
          match err {
            DecoderRejected at ↦ Decoded c loc (List a) (Nil a) cur;
            DecoderZeroProgress at ↦ DecoderFailed c loc (List a) (DecoderZeroProgress loc at);
            DecoderFuelExhausted at ↦ DecoderFailed c loc (List a) (DecoderFuelExhausted loc at)
          };
        Decoded value next ↦
          match cursor_nat_lt
            (cursor_remaining c el loc ops next)
            (cursor_remaining c el loc ops cur) {
            False ↦
              DecoderFailed
                c
                loc
                (List a)
                (DecoderZeroProgress loc (cursor_locate c el loc ops next));
            True ↦
              match decoder_many_fuel c el loc a ops step fuel2 next {
                DecoderFailed err ↦ DecoderFailed c loc (List a) err;
                Decoded rest end ↦ Decoded c loc (List a) (Cons a value rest) end
              }
          }
      }
  }

fn decoder_many
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (step : Decoder c loc a)
    : Decoder c loc (List a) =
  λcur. decoder_many_fuel c el loc a ops step (cursor_remaining c el loc ops cur) cur

fn decoder_some
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (step : Decoder c loc a)
    : Decoder c loc (List a) =
  λcur.
    match step cur {
      DecoderFailed err ↦ DecoderFailed c loc (List a) err;
      Decoded value next ↦
        match cursor_nat_lt
          (cursor_remaining c el loc ops next)
          (cursor_remaining c el loc ops cur) {
          False ↦
            DecoderFailed
              c
              loc
              (List a)
              (DecoderZeroProgress loc (cursor_locate c el loc ops next));
          True ↦
            match decoder_many c el loc a ops step next {
              DecoderFailed err ↦ DecoderFailed c loc (List a) err;
              Decoded rest end ↦ Decoded c loc (List a) (Cons a value rest) end
            }
        }
    }

fn decoder_recursive_fuel
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (layer : Decoder c loc a → Decoder c loc a)
      (fuel : Nat)
      (cur : c)
    : DecoderResult c loc a =
  match fuel {
    Zero ↦ DecoderFailed c loc a (DecoderFuelExhausted loc (cursor_locate c el loc ops cur));
    Suc fuel2 ↦ layer (decoder_recursive_fuel c el loc a ops layer fuel2) cur
  }

fn decoder_recursive
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (layer : Decoder c loc a → Decoder c loc a)
    : Decoder c loc a =
  λcur. decoder_recursive_fuel c el loc a ops layer (cursor_remaining c el loc ops cur) cur
```

## 2. Laws

`DecoderProgress` is the obligation on a repeated step. The whole-input law is
the observable contract of a successful repetition: success ends only when the
derived remaining bound is zero.

```ken
fn DecoderProgress
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (step : Decoder c loc a)
    : Prop =
  (cur : c)
    → (value : a)
    → (next : c)
    → Equal
    (DecoderResult c loc a)
    (step cur)
    (Decoded c loc a value next)
    → Equal Bool
    (cursor_nat_lt (cursor_remaining c el loc ops next) (cursor_remaining c el loc ops cur))
    True

fn DecoderConsumesAll
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (decoder : Decoder c loc a)
    : Prop =
  (cur : c)
    → (value : a)
    → (end : c)
    → Equal
    (DecoderResult c loc a)
    (decoder cur)
    (Decoded c loc a value end)
    → Equal Nat
    (cursor_remaining c el loc ops end)
    Zero

fn DecoderRejectsOnlyAtEnd
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (step : Decoder c loc a)
    : Prop =
  (cur : c)
    → (at : loc)
    → Equal
    (DecoderResult c loc a)
    (step cur)
    (DecoderFailed c loc a (DecoderRejected loc at))
    → Equal Nat
    (cursor_remaining c el loc ops cur)
    Zero

fn DecoderManyConsumesAllLaw
      (c : Type)
      (el : Type)
      (loc : Type)
      (a : Type)
      (ops : CursorOps c el loc)
      (step : Decoder c loc a)
    : Prop =
  And (DecoderProgress c el loc a ops step) (DecoderRejectsOnlyAtEnd c el loc a ops step)
  → DecoderConsumesAll c el loc (List a) ops (decoder_many c el loc a ops step)
```

## 3. Using it

Build token decoders with `decoder_satisfy` or `decoder_token`, combine them
with `map`, `bind`, `seq`, and `alt`, and use `decoder_recursive` for a
structurally fuel-bounded recursive layer. Callers never supply repetition or
recursion fuel; both bounds come from `CursorOps.remaining`.

## 4. Design notes

`DecoderFuelExhausted` is observable only when a cursor or recursive layer
violates its stated progress contract. Legal repeated input cannot reach it:
every success consumes at least one unit from a fuel seed equal to `remaining`.

## 5. References

None.

## 6. Trust  derivation

Every combinator is transparent, structurally recursive on `Nat` fuel, and
uses only checked cursor operations. This package adds no axiom or primitive.

## 7. Package  summary

Public surface: location-generic `DecoderError`, `DecoderResult`, `Decoder`,
the core sequencing/token/repetition/recursive combinators, and progress laws.
