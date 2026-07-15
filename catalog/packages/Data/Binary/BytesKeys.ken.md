# `Data.Binary.BytesKeys` — lawful byte equality

`UInt8` equality transports through the existing widening injection into
`Int`. `Bytes` equality then transports through the landed structural view into
`List UInt8`. The only irreducible certificate is the conversion-layer
`uint8_int_retract` registered by SUB-1b; every definition and law below is an
ordinary kernel-checked proof.

## 1. `UInt8` equality

```ken
lemma uint8_to_int_injective
      (left : UInt8)
      (right : UInt8)
      (same_ints : Equal Int (uint8_to_int left) (uint8_to_int right))
    : Equal UInt8 left right =
  trans
    UInt8
    left
    (int_to_uint8_raw (uint8_to_int left))
    right
    (sym UInt8 (int_to_uint8_raw (uint8_to_int left)) left (uint8_int_retract left))
    (trans
      UInt8
      (int_to_uint8_raw (uint8_to_int left))
      (int_to_uint8_raw (uint8_to_int right))
      right
      (cong Int UInt8 (uint8_to_int left) (uint8_to_int right) int_to_uint8_raw same_ints)
      (uint8_int_retract right))

fn uint8_deceq_eq (left : UInt8) (right : UInt8) : Bool =
  eq_int (uint8_to_int left) (uint8_to_int right)

lemma uint8_deceq_sound
      (left : UInt8)
      (right : UInt8)
      (is_equal : IsTrue (eq_int (uint8_to_int left) (uint8_to_int right)))
    : Equal UInt8 left right =
  uint8_to_int_injective
    left
    right
    (int_eq_sound (uint8_to_int left) (uint8_to_int right) is_equal)

lemma uint8_deceq_complete
      (left : UInt8) (right : UInt8) (same : Equal UInt8 left right)
    : IsTrue (eq_int (uint8_to_int left) (uint8_to_int right)) =
  int_eq_complete
    (uint8_to_int left)
    (uint8_to_int right)
    (cong UInt8 Int left right uint8_to_int same)

instance DecEq UInt8 {
  eq = uint8_deceq_eq;
  sound = uint8_deceq_sound;
  complete = uint8_deceq_complete
}
```

## 2. `Bytes` equality

```ken
lemma bytes_to_list_injective
      (left : Bytes)
      (right : Bytes)
      (same_bytes : Equal (List UInt8) (bytes_to_list left) (bytes_to_list right))
    : Equal Bytes left right =
  trans
    Bytes
    left
    (list_to_bytes (bytes_to_list left))
    right
    (sym Bytes (list_to_bytes (bytes_to_list left)) left (bytes_list_roundtrip left))
    (trans
      Bytes
      (list_to_bytes (bytes_to_list left))
      (list_to_bytes (bytes_to_list right))
      right
      (cong
        (List UInt8)
        Bytes
        (bytes_to_list left)
        (bytes_to_list right)
        list_to_bytes
        same_bytes)
      (bytes_list_roundtrip right))

fn bytes_deceq_eq (left : Bytes) (right : Bytes) : Bool =
  list_eq UInt8 uint8_deceq_eq (bytes_to_list left) (bytes_to_list right)

proof sound for bytes_deceq_eq
      (left : Bytes) (right : Bytes) (is_equal : IsTrue (bytes_deceq_eq left right))
    : Equal Bytes left right =
  bytes_to_list_injective
    left
    right
    ((DecEq_instance_List UInt8 DecEq_instance_UInt8).sound
      (bytes_to_list left)
      (bytes_to_list right)
      is_equal)

proof complete for bytes_deceq_eq
      (left : Bytes) (right : Bytes) (same : Equal Bytes left right)
    : IsTrue (bytes_deceq_eq left right) =
  (DecEq_instance_List UInt8 DecEq_instance_UInt8).complete
    (bytes_to_list left)
    (bytes_to_list right)
    (cong Bytes (List UInt8) left right bytes_to_list same)

instance DecEq Bytes {
  eq = bytes_deceq_eq;
  sound = proof sound for bytes_deceq_eq;
  complete = proof complete for bytes_deceq_eq
}
```

## 3. Trust and derivation

This package declares no local trust. `DecEq UInt8` consumes the single
conversion-layer retraction certificate, while `DecEq Bytes` additionally
consumes SUB-1's existing `Bytes` retraction. The generic `DecEq (List a)`
dictionary supplies structural list comparison. No equality primitive or kernel
certificate is added.
