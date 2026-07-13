# `Text.Codec` — safe UTF-8 and ASCII views

`Text.Codec` gives names to the safe byte/text operations already provided by
the language and adds a small, total ASCII classifier. Decoding remains a
`Result`; looking at an absent byte remains `None`.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Laws and examples](#3-laws-and-examples)
4. [Trust and derivation](#4-trust-and-derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Definition](#2-definition)
- *Practitioner* → [Definition](#2-definition) →
  [Laws and examples](#3-laws-and-examples)
- *Researcher* → [Laws and examples](#3-laws-and-examples) →
  [Trust and derivation](#4-trust-and-derivation)

## 1. Motivation

Raw bytes are not automatically text. `decode_utf8` therefore preserves the
landed `Utf8Error` result, while `ascii_view` classifies one requested byte and
preserves the distinction between an absent byte and a present non-ASCII byte.

## 2. Definition

```ken
fn decode_utf8 (bs : Bytes) : Result Utf8Error String = bytes_decode bs

fn byte_is_ascii (byte : UInt8) : Bool = leq_int (uint8_to_int byte) (127 : Int)

fn classify_ascii_result (found : Option UInt8) : Option Bool =
  match found {
    None ↦ None Bool;
    Some byte ↦
      Some
        Bool
        (byte_is_ascii byte)
  }

fn ascii_view (bs : Bytes) (index : Int) : Option Bool =
  classify_ascii_result (bytes_at bs index)

proof definition for decode_utf8
  (bs : Bytes)
  : Equal (Result Utf8Error String) (decode_utf8 bs) (bytes_decode bs) =
  Refl

lemma codec_roundtrip_anchor (p : BytesRoundTripLaw) : BytesRoundTripLaw = p

lemma ascii_view_none
  (bs : Bytes)
  (index : Int)
  (h : Equal (Option UInt8) (bytes_at bs index) (None UInt8))
  : Equal (Option Bool) (ascii_view bs index) (None Bool) =
  cong
    (Option UInt8)
    (Option Bool)
    (bytes_at bs index)
    (None UInt8)
    classify_ascii_result
    h

lemma ascii_view_some
  (bs : Bytes)
  (index : Int)
  (byte : UInt8)
  (h : Equal (Option UInt8) (bytes_at bs index) (Some UInt8 byte))
  : Equal (Option Bool) (ascii_view bs index) (Some Bool (byte_is_ascii byte)) =
  cong
    (Option UInt8)
    (Option Bool)
    (bytes_at bs index)
    (Some UInt8 byte)
    classify_ascii_result
    h
```

## 3. Laws and examples

The classifier's boundary cases and its propagation of `bytes_at` failure are
checked by reduction. The final declaration keeps the package's round-trip
surface tied to the landed one-way `BytesRoundTripLaw`; this package does not
claim the false reverse byte round trip.

```ken example
const ascii_a_view : Option Bool = ascii_view (bytes_encode "A") (0 : Int)

const ascii_a_missing_view : Option Bool = ascii_view (bytes_encode "A") (1 : Int)

const utf8_lead_view : Option Bool = ascii_view (bytes_encode "é") (0 : Int)
```

## 4. Trust and derivation

**Public API:** `decode_utf8`, `byte_is_ascii`, `ascii_view`,
`decode_utf8::definition`, `ascii_view_none`, `ascii_view_some`, and
`codec_roundtrip_anchor`.

**Derivation.** Every operation is ordinary Ken over the landed total
`bytes_decode`, `bytes_at`, `uint8_to_int`, and `leq_int` operations. The
round-trip anchor consumes the already-landed `BytesRoundTripLaw`; it does not
mint a replacement and does not cross the unrelated `String`/`List Char`
bijection boundary.

**Trust delta:** zero. The checked fences contain no `Axiom`, primitive,
postulate, or opaque declaration.
