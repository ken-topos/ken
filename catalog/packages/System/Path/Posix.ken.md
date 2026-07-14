# `System.Path.Posix` — byte-preserving lexical paths

POSIX paths are sequences of raw bytes. This package parses those bytes once
into a structured `Path`, performs total operations over the structure, and
renders back to `Bytes` only at the boundary. It never decodes through
`String`.

## 1. Structured paths

```ken
data Path = MkPath {path_absolute : Bool, path_segments : List (List UInt8)}

fn path_is_absolute (path : Path) : Bool =
  match path {
    MkPath absolute segments ↦ absolute
  }

fn path_segment_eq (left : List UInt8) (right : List UInt8) : Bool =
  list_eq UInt8 uint8_deceq_eq left right

const path_slash_byte : UInt8 = 47

fn path_byte_is_slash (byte : UInt8) : Bool = (DecEq_instance_UInt8).eq byte path_slash_byte

fn path_finish_segment
      (current : List UInt8) (segments : List (List UInt8))
    : List (List UInt8) =
  match current {
    Nil ↦ segments;
    Cons byte rest ↦ list_append
      (List UInt8)
      segments
      (Cons (List UInt8) current (Nil (List UInt8)))
  }

fn path_split_cons_result
      (slash_result : List (List UInt8)) (ordinary_result : List (List UInt8)) (slash : Bool)
    : List (List UInt8) =
  match slash {
    True ↦ slash_result;
    False ↦ ordinary_result
  }

fn path_split
      (input : List UInt8) (current : List UInt8) (segments : List (List UInt8))
    : List (List UInt8) =
  match input {
    Nil ↦ path_finish_segment current segments;
    Cons byte rest ↦ path_split_cons_result
      (path_split rest (Nil UInt8) (path_finish_segment current segments))
      (path_split rest (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) segments)
      (path_byte_is_slash byte)
  }

fn path_input_is_absolute (input : List UInt8) : Bool =
  match input {
    Nil ↦ False;
    Cons first rest ↦ path_byte_is_slash first
  }

fn path_parse (raw : Bytes) : Path =
  MkPath
    (path_input_is_absolute (bytes_to_list raw))
    (path_split (bytes_to_list raw) (Nil UInt8) (Nil (List UInt8)))
```

## 2. Rendering and ordinary views

```ken
fn path_render_segments (segments : List (List UInt8)) : List UInt8 =
  match segments {
    Nil ↦ Nil UInt8;
    Cons segment rest ↦
      match rest {
        Nil ↦ segment;
        Cons next tail ↦ list_append
          UInt8
          segment
          (Cons UInt8 path_slash_byte (path_render_segments rest))
      }
  }

fn path_render (path : Path) : Bytes =
  match path {
    MkPath absolute segments ↦
      match absolute {
        True ↦ list_to_bytes (Cons UInt8 path_slash_byte (path_render_segments segments));
        False ↦ list_to_bytes (path_render_segments segments)
      }
  }

fn path_join (left : Path) (right : Path) : Path =
  match right {
    MkPath True segments ↦ right;
    MkPath False right_segments ↦
      match left {
        MkPath absolute left_segments ↦ MkPath
          absolute
          (list_append (List UInt8) left_segments right_segments)
      }
  }

fn path_drop_last (segments : List (List UInt8)) : List (List UInt8) =
  match segments {
    Nil ↦ Nil (List UInt8);
    Cons segment rest ↦
      match rest {
        Nil ↦ Nil (List UInt8);
        Cons next tail ↦ Cons (List UInt8) segment (path_drop_last rest)
      }
  }

fn path_parent (path : Path) : Path =
  match path {
    MkPath absolute segments ↦ MkPath absolute (path_drop_last segments)
  }

fn path_segment_cons_has_no_slash (tail : Bool) (slash : Bool) : Bool =
  match slash {
    True ↦ False;
    False ↦ tail
  }

fn path_segment_has_no_slash (segment : List UInt8) : Bool =
  match segment {
    Nil ↦ True;
    Cons byte rest ↦ path_segment_cons_has_no_slash
      (path_segment_has_no_slash rest)
      (path_byte_is_slash byte)
  }

fn path_segment_valid (segment : List UInt8) : Bool =
  match segment {
    Nil ↦ False;
    Cons byte rest ↦ path_segment_has_no_slash segment
  }

fn path_segments_valid (segments : List (List UInt8)) : Bool =
  match segments {
    Nil ↦ True;
    Cons segment rest ↦ bool_and (path_segment_valid segment) (path_segments_valid rest)
  }

fn path_valid (path : Path) : Bool =
  match path {
    MkPath absolute segments ↦ path_segments_valid segments
  }

lemma path_equal_sym (a : Type) (x : a) (y : a) (p : Eq a x y) : Eq a y x =
  J (λy' _. Eq a y' x) Refl p

lemma path_equal_trans
      (a : Type) (x : a) (y : a) (z : a) (p : Eq a x y) (q : Eq a y z)
    : Eq a x z =
  J (λz' _. Eq a x z') p q

lemma path_equal_cong
      (a : Type) (b : Type) (x : a) (y : a) (f : a → b) (p : Eq a x y)
    : Eq b (f x) (f y) =
  J (λy' _. Eq b (f x) (f y')) Refl p

lemma path_cons_no_slash_tail
      (byte : UInt8) (rest : List UInt8)
    : Eq Bool
        (path_segment_cons_has_no_slash
          (path_segment_has_no_slash rest)
          (path_byte_is_slash byte))
        True
      → Eq Bool (path_segment_has_no_slash rest) True =
  match path_byte_is_slash byte {
    True ↦ λno_slash. absurd no_slash;
    False ↦ λno_slash. no_slash
  }

lemma path_append_cons
      (current : List UInt8) (byte : UInt8) (rest : List UInt8)
    : Eq
        (List UInt8)
        (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
        (list_append UInt8 current (Cons UInt8 byte rest)) =
  (proof assoc for list_append) UInt8 current (Cons UInt8 byte (Nil UInt8)) rest

lemma path_split_no_slash_cons
      (byte : UInt8)
      (rest : List UInt8)
      (current : List UInt8)
      (segments : List (List UInt8))
      (ih : Eq
        Bool
        (path_segment_has_no_slash rest)
        True
        → Eq
        (List (List UInt8))
        (path_split rest (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) segments)
        (path_finish_segment
          (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
          segments))
    : Eq Bool
        (path_segment_cons_has_no_slash
          (path_segment_has_no_slash rest)
          (path_byte_is_slash byte))
        True
      → Eq
        (List (List UInt8))
        (path_split_cons_result
          (path_split rest (Nil UInt8) (path_finish_segment current segments))
          (path_split rest (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) segments)
          (path_byte_is_slash byte))
        (path_finish_segment
          (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
          segments) =
  match path_byte_is_slash byte eqn : slash_case {
    True ↦ λno_slash. absurd no_slash;
    False ↦ λno_slash. ih no_slash
  }

lemma path_split_no_slash_end
      (segment : List UInt8) (current : List UInt8) (segments : List (List UInt8))
    : Eq Bool (path_segment_has_no_slash segment) True
      → Eq
        (List (List UInt8))
        (path_split segment current segments)
        (path_finish_segment (list_append UInt8 current segment) segments) =
  match segment {
    Nil ↦ λno_slash.
      path_equal_sym
        (List (List UInt8))
        (path_finish_segment (list_append UInt8 current (Nil UInt8)) segments)
        (path_finish_segment current segments)
        (path_equal_cong
          (List UInt8)
          (List (List UInt8))
          (list_append UInt8 current (Nil UInt8))
          current
          (λcombined. path_finish_segment combined segments)
          ((proof right_unit for list_append) UInt8 current));
    Cons byte rest ↦ λno_slash.
      path_equal_trans
        (List (List UInt8))
        (path_split (Cons UInt8 byte rest) current segments)
        (path_finish_segment
          (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
          segments)
        (path_finish_segment (list_append UInt8 current (Cons UInt8 byte rest)) segments)
        (path_split_no_slash_cons
          byte
          rest
          current
          segments
          (path_split_no_slash_end
            rest
            (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
            segments)
          no_slash)
        (path_equal_cong
          (List UInt8)
          (List (List UInt8))
          (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
          (list_append UInt8 current (Cons UInt8 byte rest))
          (λcombined. path_finish_segment combined segments)
          (path_append_cons current byte rest))
  }

lemma path_split_append_cons_unfold
      (byte : UInt8)
      (rest : List UInt8)
      (suffix : List UInt8)
      (current : List UInt8)
      (segments : List (List UInt8))
    : Eq
        (List (List UInt8))
        (path_split (list_append UInt8 (Cons UInt8 byte rest) suffix) current segments)
        (path_split_cons_result
          (path_split
            (list_append UInt8 rest suffix)
            (Nil UInt8)
            (path_finish_segment current segments))
          (path_split
            (list_append UInt8 rest suffix)
            (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
            segments)
          (path_byte_is_slash byte)) =
  Refl

lemma path_split_no_slash_prefix_cons
      (byte : UInt8)
      (rest : List UInt8)
      (suffix : List UInt8)
      (current : List UInt8)
      (segments : List (List UInt8))
      (ih : Eq
        Bool
        (path_segment_has_no_slash rest)
        True
        → Eq
        (List (List UInt8))
        (path_split
          (list_append UInt8 rest suffix)
          (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
          segments)
        (path_split
          suffix
          (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
          segments))
    : Eq Bool
        (path_segment_cons_has_no_slash
          (path_segment_has_no_slash rest)
          (path_byte_is_slash byte))
        True
      → Eq
        (List (List UInt8))
        (path_split_cons_result
          (path_split
            (list_append UInt8 rest suffix)
            (Nil UInt8)
            (path_finish_segment current segments))
          (path_split
            (list_append UInt8 rest suffix)
            (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
            segments)
          (path_byte_is_slash byte))
        (path_split
          suffix
          (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
          segments) =
  match path_byte_is_slash byte {
    True ↦ λno_slash. absurd no_slash;
    False ↦ λno_slash. ih no_slash
  }

lemma path_split_no_slash_prefix
      (segment : List UInt8)
      (suffix : List UInt8)
      (current : List UInt8)
      (segments : List (List UInt8))
    : Eq Bool (path_segment_has_no_slash segment) True
      → Eq
        (List (List UInt8))
        (path_split (list_append UInt8 segment suffix) current segments)
        (path_split suffix (list_append UInt8 current segment) segments) =
  match segment {
    Nil ↦ λno_slash.
      path_equal_sym
        (List (List UInt8))
        (path_split suffix (list_append UInt8 current (Nil UInt8)) segments)
        (path_split suffix current segments)
        (path_equal_cong
          (List UInt8)
          (List (List UInt8))
          (list_append UInt8 current (Nil UInt8))
          current
          (λnext. path_split suffix next segments)
          ((proof right_unit for list_append) UInt8 current));
    Cons byte rest ↦ λno_slash.
      path_equal_trans
        (List (List UInt8))
        (path_split (list_append UInt8 (Cons UInt8 byte rest) suffix) current segments)
        (path_split_cons_result
          (path_split
            (list_append UInt8 rest suffix)
            (Nil UInt8)
            (path_finish_segment current segments))
          (path_split
            (list_append UInt8 rest suffix)
            (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
            segments)
          (path_byte_is_slash byte))
        (path_split suffix (list_append UInt8 current (Cons UInt8 byte rest)) segments)
        (path_split_append_cons_unfold byte rest suffix current segments)
        (path_equal_trans
          (List (List UInt8))
          (path_split_cons_result
            (path_split
              (list_append UInt8 rest suffix)
              (Nil UInt8)
              (path_finish_segment current segments))
            (path_split
              (list_append UInt8 rest suffix)
              (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
              segments)
            (path_byte_is_slash byte))
          (path_split
            suffix
            (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
            segments)
          (path_split suffix (list_append UInt8 current (Cons UInt8 byte rest)) segments)
          (path_split_no_slash_prefix_cons
            byte
            rest
            suffix
            current
            segments
            (path_split_no_slash_prefix
              rest
              suffix
              (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
              segments)
            no_slash)
          (path_equal_cong
            (List UInt8)
            (List (List UInt8))
            (list_append UInt8 (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) rest)
            (list_append UInt8 current (Cons UInt8 byte rest))
            (λnext. path_split suffix next segments)
            (path_append_cons current byte rest)))
  }

lemma path_finish_valid_segment
      (segment : List UInt8) (segments : List (List UInt8))
    : Eq Bool (path_segment_valid segment) True
      → Eq
        (List (List UInt8))
        (path_finish_segment segment segments)
        (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8)))) =
  match segment {
    Nil ↦ λvalid. absurd valid;
    Cons byte rest ↦ λvalid. Refl
  }

lemma path_segment_valid_no_slash
      (segment : List UInt8)
    : Eq Bool (path_segment_valid segment) True
      → Eq Bool (path_segment_has_no_slash segment) True =
  match segment {
    Nil ↦ λvalid. absurd valid;
    Cons byte rest ↦ λvalid. valid
  }

lemma path_segments_valid_head
      (segment : List UInt8) (rest : List (List UInt8))
    : Eq Bool (path_segments_valid (Cons (List UInt8) segment rest)) True
      → Eq Bool (path_segment_valid segment) True =
  λvalid.
    (proof left for bool_and) (path_segment_valid segment) (path_segments_valid rest) valid

lemma path_segments_valid_tail
      (segment : List UInt8) (rest : List (List UInt8))
    : Eq Bool (path_segments_valid (Cons (List UInt8) segment rest)) True
      → Eq Bool (path_segments_valid rest) True =
  λvalid.
    (proof right for bool_and) (path_segment_valid segment) (path_segments_valid rest) valid

lemma path_slash_is_slash : Eq Bool (path_byte_is_slash path_slash_byte) True =
  (DecEq_instance_UInt8).complete path_slash_byte path_slash_byte Refl

lemma path_split_leading_slash
      (rest : List UInt8) (current : List UInt8) (segments : List (List UInt8))
    : Eq
        (List (List UInt8))
        (path_split (Cons UInt8 path_slash_byte rest) current segments)
        (path_split rest (Nil UInt8) (path_finish_segment current segments)) =
  path_equal_cong
    Bool
    (List (List UInt8))
    (path_byte_is_slash path_slash_byte)
    True
    (λslash.
      path_split_cons_result
        (path_split rest (Nil UInt8) (path_finish_segment current segments))
        (path_split
          rest
          (list_append UInt8 current (Cons UInt8 path_slash_byte (Nil UInt8)))
          segments)
        slash)
    path_slash_is_slash

lemma path_append_nil_sym
      (a : Type) (items : List a)
    : Eq (List a) items (list_append a items (Nil a)) =
  match items {
    Nil ↦ Proved;
    Cons item rest ↦ path_equal_cong
      (List a)
      (List a)
      rest
      (list_append a rest (Nil a))
      (Cons a item)
      (path_append_nil_sym a rest)
  }

lemma path_append_assoc
      (a : Type) (left : List a) (middle : List a) (right : List a)
    : Eq
        (List a)
        (list_append a (list_append a left middle) right)
        (list_append a left (list_append a middle right)) =
  match left {
    Nil ↦ Refl;
    Cons item rest ↦ path_equal_cong
      (List a)
      (List a)
      (list_append a (list_append a rest middle) right)
      (list_append a rest (list_append a middle right))
      (Cons a item)
      (path_append_assoc a rest middle right)
  }

lemma path_split_render_single
      (segment : List UInt8) (segments : List (List UInt8))
    : Eq Bool (path_segment_valid segment) True
      → Eq
        (List (List UInt8))
        (path_split segment (Nil UInt8) segments)
        (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8)))) =
  λvalid.
    path_equal_trans
      (List (List UInt8))
      (path_split segment (Nil UInt8) segments)
      (path_finish_segment segment segments)
      (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8))))
      (path_split_no_slash_end
        segment
        (Nil UInt8)
        segments
        (path_segment_valid_no_slash segment valid))
      (path_finish_valid_segment segment segments valid)

lemma path_split_render_nil
      (segments : List (List UInt8))
    : Eq
        (List (List UInt8))
        (path_split (path_render_segments (Nil (List UInt8))) (Nil UInt8) segments)
        (list_append (List UInt8) segments (Nil (List UInt8))) =
  path_equal_trans
    (List (List UInt8))
    (path_split (path_render_segments (Nil (List UInt8))) (Nil UInt8) segments)
    segments
    (list_append (List UInt8) segments (Nil (List UInt8)))
    Refl
    (path_append_nil_sym (List UInt8) segments)

lemma path_split_render_single_case
      (segment : List UInt8) (segments : List (List UInt8))
    : Eq Bool (path_segment_valid segment) True
      → Eq
        (List (List UInt8))
        (path_split
          (path_render_segments (Cons (List UInt8) segment (Nil (List UInt8))))
          (Nil UInt8)
          segments)
        (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8)))) =
  λvalid.
    path_equal_trans
      (List (List UInt8))
      (path_split
        (path_render_segments (Cons (List UInt8) segment (Nil (List UInt8))))
        (Nil UInt8)
        segments)
      (path_split segment (Nil UInt8) segments)
      (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8))))
      Refl
      (path_split_render_single segment segments valid)

lemma path_split_render_cons_case
      (segment : List UInt8)
      (next : List UInt8)
      (tail : List (List UInt8))
      (segments : List (List UInt8))
      (ih : Eq
        Bool
        (path_segments_valid (Cons (List UInt8) next tail))
        True
        → Eq
        (List (List UInt8))
        (path_split
          (path_render_segments (Cons (List UInt8) next tail))
          (Nil UInt8)
          (path_finish_segment segment segments))
        (list_append
          (List UInt8)
          (path_finish_segment segment segments)
          (Cons (List UInt8) next tail)))
    : Eq Bool
        (path_segments_valid (Cons (List UInt8) segment (Cons (List UInt8) next tail)))
        True
      → Eq
        (List (List UInt8))
        (path_split
          (path_render_segments (Cons (List UInt8) segment (Cons (List UInt8) next tail)))
          (Nil UInt8)
          segments)
        (list_append
          (List UInt8)
          segments
          (Cons (List UInt8) segment (Cons (List UInt8) next tail))) =
  λvalid.
    path_equal_trans
      (List (List UInt8))
      (path_split
        (path_render_segments (Cons (List UInt8) segment (Cons (List UInt8) next tail)))
        (Nil UInt8)
        segments)
      (path_split
        (Cons UInt8 path_slash_byte (path_render_segments (Cons (List UInt8) next tail)))
        segment
        segments)
      (list_append
        (List UInt8)
        segments
        (Cons (List UInt8) segment (Cons (List UInt8) next tail)))
      (path_split_no_slash_prefix
        segment
        (Cons UInt8 path_slash_byte (path_render_segments (Cons (List UInt8) next tail)))
        (Nil UInt8)
        segments
        (path_segment_valid_no_slash
          segment
          (path_segments_valid_head segment (Cons (List UInt8) next tail) valid)))
      (path_equal_trans
        (List (List UInt8))
        (path_split
          (Cons UInt8 path_slash_byte (path_render_segments (Cons (List UInt8) next tail)))
          segment
          segments)
        (path_split
          (path_render_segments (Cons (List UInt8) next tail))
          (Nil UInt8)
          (path_finish_segment segment segments))
        (list_append
          (List UInt8)
          segments
          (Cons (List UInt8) segment (Cons (List UInt8) next tail)))
        (path_split_leading_slash
          (path_render_segments (Cons (List UInt8) next tail))
          segment
          segments)
        (path_equal_trans
          (List (List UInt8))
          (path_split
            (path_render_segments (Cons (List UInt8) next tail))
            (Nil UInt8)
            (path_finish_segment segment segments))
          (list_append
            (List UInt8)
            (path_finish_segment segment segments)
            (Cons (List UInt8) next tail))
          (list_append
            (List UInt8)
            segments
            (Cons (List UInt8) segment (Cons (List UInt8) next tail)))
          (ih (path_segments_valid_tail segment (Cons (List UInt8) next tail) valid))
          (path_equal_trans
            (List (List UInt8))
            (list_append
              (List UInt8)
              (path_finish_segment segment segments)
              (Cons (List UInt8) next tail))
            (list_append
              (List UInt8)
              (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8))))
              (Cons (List UInt8) next tail))
            (list_append
              (List UInt8)
              segments
              (Cons (List UInt8) segment (Cons (List UInt8) next tail)))
            (path_equal_cong
              (List (List UInt8))
              (List (List UInt8))
              (path_finish_segment segment segments)
              (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8))))
              (λprefix. list_append (List UInt8) prefix (Cons (List UInt8) next tail))
              (path_finish_valid_segment
                segment
                segments
                (path_segments_valid_head segment (Cons (List UInt8) next tail) valid)))
            (path_append_assoc
              (List UInt8)
              segments
              (Cons (List UInt8) segment (Nil (List UInt8)))
              (Cons (List UInt8) next tail)))))

lemma path_split_render_step
      (segment : List UInt8) (rest : List (List UInt8)) (segments : List (List UInt8))
    : (Eq
        Bool
        (path_segments_valid rest)
        True
        → Eq
        (List (List UInt8))
        (path_split
          (path_render_segments rest)
          (Nil UInt8)
          (path_finish_segment segment segments))
        (list_append (List UInt8) (path_finish_segment segment segments) rest))
      → Eq Bool (path_segments_valid (Cons (List UInt8) segment rest)) True
      → Eq
        (List (List UInt8))
        (path_split
          (path_render_segments (Cons (List UInt8) segment rest))
          (Nil UInt8)
          segments)
        (list_append (List UInt8) segments (Cons (List UInt8) segment rest)) =
  match rest {
    Nil ↦ λih.
      λvalid.
        path_split_render_single_case
          segment
          segments
          (path_segments_valid_head segment (Nil (List UInt8)) valid);
    Cons next tail ↦ λih. path_split_render_cons_case segment next tail segments ih
  }

lemma path_split_render_segments
      (rendered : List (List UInt8)) (segments : List (List UInt8))
    : Eq Bool (path_segments_valid rendered) True
      → Eq
        (List (List UInt8))
        (path_split (path_render_segments rendered) (Nil UInt8) segments)
        (list_append (List UInt8) segments rendered) =
  match rendered {
    Nil ↦ λvalid. path_split_render_nil segments;
    Cons segment rest ↦ path_split_render_step
      segment
      rest
      segments
      (path_split_render_segments rest (path_finish_segment segment segments))
  }

lemma path_no_slash_head
      (byte : UInt8) (rest : List UInt8)
    : Eq Bool (path_segment_has_no_slash (Cons UInt8 byte rest)) True
      → Eq Bool (path_byte_is_slash byte) False =
  match path_byte_is_slash byte eqn : slash_case {
    True ↦ λno_slash.
      absurd
        (J
          (λslash _.
            Eq
              Bool
              (path_segment_cons_has_no_slash (path_segment_has_no_slash rest) slash)
              True)
          no_slash
          slash_case);
    False ↦ λno_slash. Proved
  }

lemma path_render_cons_head_nil
      (byte : UInt8) (tail : List UInt8)
    : Eq
        (List UInt8)
        (path_render_segments (Cons (List UInt8) (Cons UInt8 byte tail) (Nil (List UInt8))))
        (Cons UInt8 byte tail) =
  path_equal_cong (List UInt8) (List UInt8) tail tail (Cons UInt8 byte) Refl

lemma path_render_cons_head_more
      (byte : UInt8) (tail : List UInt8) (next : List UInt8) (remaining : List (List UInt8))
    : Eq
        (List UInt8)
        (path_render_segments
          (Cons (List UInt8) (Cons UInt8 byte tail) (Cons (List UInt8) next remaining)))
        (Cons
          UInt8
          byte
          (list_append
            UInt8
            tail
            (Cons
              UInt8
              path_slash_byte
              (path_render_segments (Cons (List UInt8) next remaining))))) =
  path_equal_cong
    (List UInt8)
    (List UInt8)
    (list_append
      UInt8
      tail
      (Cons UInt8 path_slash_byte (path_render_segments (Cons (List UInt8) next remaining))))
    (list_append
      UInt8
      tail
      (Cons UInt8 path_slash_byte (path_render_segments (Cons (List UInt8) next remaining))))
    (Cons UInt8 byte)
    Refl

lemma path_render_cons_is_relative
      (byte : UInt8) (tail : List UInt8) (rest : List (List UInt8))
    : Eq Bool (path_byte_is_slash byte) False
      → Eq Bool
        (path_input_is_absolute
          (path_render_segments (Cons (List UInt8) (Cons UInt8 byte tail) rest)))
        False =
  match rest {
    Nil ↦ λhead_not_slash.
      path_equal_trans
        Bool
        (path_input_is_absolute
          (path_render_segments (Cons (List UInt8) (Cons UInt8 byte tail) (Nil (List UInt8)))))
        (path_byte_is_slash byte)
        False
        (path_equal_cong
          (List UInt8)
          Bool
          (path_render_segments (Cons (List UInt8) (Cons UInt8 byte tail) (Nil (List UInt8))))
          (Cons UInt8 byte tail)
          path_input_is_absolute
          (path_render_cons_head_nil byte tail))
        head_not_slash;
    Cons next remaining ↦ λhead_not_slash.
      path_equal_trans
        Bool
        (path_input_is_absolute
          (path_render_segments
            (Cons (List UInt8) (Cons UInt8 byte tail) (Cons (List UInt8) next remaining))))
        (path_byte_is_slash byte)
        False
        (path_equal_cong
          (List UInt8)
          Bool
          (path_render_segments
            (Cons (List UInt8) (Cons UInt8 byte tail) (Cons (List UInt8) next remaining)))
          (Cons
            UInt8
            byte
            (list_append
              UInt8
              tail
              (Cons
                UInt8
                path_slash_byte
                (path_render_segments (Cons (List UInt8) next remaining)))))
          path_input_is_absolute
          (path_render_cons_head_more byte tail next remaining))
        head_not_slash
  }

lemma path_render_relative_is_relative
      (segments : List (List UInt8))
    : Eq Bool (path_segments_valid segments) True
      → Eq Bool (path_input_is_absolute (path_render_segments segments)) False =
  match segments {
    Nil ↦ λvalid. Proved;
    Cons segment rest ↦
      match segment {
        Nil ↦ λvalid. absurd (path_segments_valid_head (Nil UInt8) rest valid);
        Cons byte tail ↦ λvalid.
          path_render_cons_is_relative
            byte
            tail
            rest
            (path_no_slash_head
              byte
              tail
              (path_segment_valid_no_slash
                (Cons UInt8 byte tail)
                (path_segments_valid_head (Cons UInt8 byte tail) rest valid)))
      }
  }

lemma path_split_render_absolute
      (segments : List (List UInt8))
    : Eq Bool (path_segments_valid segments) True
      → Eq
        (List (List UInt8))
        (path_split
          (Cons UInt8 path_slash_byte (path_render_segments segments))
          (Nil UInt8)
          (Nil (List UInt8)))
        segments =
  λvalid.
    path_equal_trans
      (List (List UInt8))
      (path_split
        (Cons UInt8 path_slash_byte (path_render_segments segments))
        (Nil UInt8)
        (Nil (List UInt8)))
      (path_split (path_render_segments segments) (Nil UInt8) (Nil (List UInt8)))
      segments
      (path_split_leading_slash (path_render_segments segments) (Nil UInt8) (Nil (List UInt8)))
      (path_split_render_segments segments (Nil (List UInt8)) valid)

lemma path_parse_list_to_bytes
      (input : List UInt8)
    : Eq Path
        (path_parse (list_to_bytes input))
        (MkPath
          (path_input_is_absolute input)
          (path_split input (Nil UInt8) (Nil (List UInt8)))) =
  path_equal_cong
    (List UInt8)
    Path
    (bytes_to_list (list_to_bytes input))
    input
    (λactual.
      MkPath (path_input_is_absolute actual) (path_split actual (Nil UInt8) (Nil (List UInt8))))
    (list_bytes_roundtrip input)

lemma path_parse_render_relative
      (segments : List (List UInt8))
    : Eq Bool (path_segments_valid segments) True
      → Eq Path (path_parse (path_render (MkPath False segments))) (MkPath False segments) =
  λvalid.
    path_equal_trans
      Path
      (path_parse (path_render (MkPath False segments)))
      (MkPath
        (path_input_is_absolute (path_render_segments segments))
        (path_split (path_render_segments segments) (Nil UInt8) (Nil (List UInt8))))
      (MkPath False segments)
      (path_parse_list_to_bytes (path_render_segments segments))
      (path_equal_trans
        Path
        (MkPath
          (path_input_is_absolute (path_render_segments segments))
          (path_split (path_render_segments segments) (Nil UInt8) (Nil (List UInt8))))
        (MkPath
          False
          (path_split (path_render_segments segments) (Nil UInt8) (Nil (List UInt8))))
        (MkPath False segments)
        (path_equal_cong
          Bool
          Path
          (path_input_is_absolute (path_render_segments segments))
          False
          (λabsolute.
            MkPath
              absolute
              (path_split (path_render_segments segments) (Nil UInt8) (Nil (List UInt8))))
          (path_render_relative_is_relative segments valid))
        (path_equal_cong
          (List (List UInt8))
          Path
          (path_split (path_render_segments segments) (Nil UInt8) (Nil (List UInt8)))
          segments
          (MkPath False)
          (path_split_render_segments segments (Nil (List UInt8)) valid)))

lemma path_parse_render_absolute
      (segments : List (List UInt8))
    : Eq Bool (path_segments_valid segments) True
      → Eq Path (path_parse (path_render (MkPath True segments))) (MkPath True segments) =
  λvalid.
    path_equal_trans
      Path
      (path_parse (path_render (MkPath True segments)))
      (MkPath
        (path_input_is_absolute (Cons UInt8 path_slash_byte (path_render_segments segments)))
        (path_split
          (Cons UInt8 path_slash_byte (path_render_segments segments))
          (Nil UInt8)
          (Nil (List UInt8))))
      (MkPath True segments)
      (path_parse_list_to_bytes (Cons UInt8 path_slash_byte (path_render_segments segments)))
      (path_equal_trans
        Path
        (MkPath
          (path_input_is_absolute (Cons UInt8 path_slash_byte (path_render_segments segments)))
          (path_split
            (Cons UInt8 path_slash_byte (path_render_segments segments))
            (Nil UInt8)
            (Nil (List UInt8))))
        (MkPath
          True
          (path_split
            (Cons UInt8 path_slash_byte (path_render_segments segments))
            (Nil UInt8)
            (Nil (List UInt8))))
        (MkPath True segments)
        (path_equal_cong
          Bool
          Path
          (path_input_is_absolute (Cons UInt8 path_slash_byte (path_render_segments segments)))
          True
          (λabsolute.
            MkPath
              absolute
              (path_split
                (Cons UInt8 path_slash_byte (path_render_segments segments))
                (Nil UInt8)
                (Nil (List UInt8))))
          path_slash_is_slash)
        (path_equal_cong
          (List (List UInt8))
          Path
          (path_split
            (Cons UInt8 path_slash_byte (path_render_segments segments))
            (Nil UInt8)
            (Nil (List UInt8)))
          segments
          (MkPath True)
          (path_split_render_absolute segments valid)))

lemma path_parse_render_valid
      (path : Path)
    : Eq Bool (path_valid path) True → Eq Path (path_parse (path_render path)) path =
  match path {
    MkPath absolute segments ↦
      λvalid.
        match absolute {
          True ↦ path_parse_render_absolute segments valid;
          False ↦ path_parse_render_relative segments valid
        }
  }

lemma path_bool_and_intro
      (left : Bool) (right : Bool)
    : Eq Bool left True → Eq Bool right True → Eq Bool (bool_and left right) True =
  match left {
    True ↦ λleft_true. λright_true. right_true;
    False ↦ λleft_true. λright_true. absurd left_true
  }

lemma path_no_slash_append_cons
      (byte : UInt8)
      (rest : List UInt8)
      (right : List UInt8)
      (ih : Eq
        Bool
        (path_segment_has_no_slash rest)
        True
        → Eq
        Bool
        (path_segment_has_no_slash right)
        True
        → Eq
        Bool
        (path_segment_has_no_slash (list_append UInt8 rest right))
        True)
    : Eq Bool
        (path_segment_cons_has_no_slash
          (path_segment_has_no_slash rest)
          (path_byte_is_slash byte))
        True
      → Eq Bool (path_segment_has_no_slash right) True
      → Eq Bool
        (path_segment_cons_has_no_slash
          (path_segment_has_no_slash (list_append UInt8 rest right))
          (path_byte_is_slash byte))
        True =
  match path_byte_is_slash byte {
    True ↦ λleft_valid. λright_valid. absurd left_valid;
    False ↦ λleft_valid. λright_valid. ih left_valid right_valid
  }

lemma path_no_slash_append
      (left : List UInt8) (right : List UInt8)
    : Eq Bool (path_segment_has_no_slash left) True
      → Eq Bool (path_segment_has_no_slash right) True
      → Eq Bool (path_segment_has_no_slash (list_append UInt8 left right)) True =
  match left {
    Nil ↦ λleft_valid. λright_valid. right_valid;
    Cons byte rest ↦ path_no_slash_append_cons byte rest right (path_no_slash_append rest right)
  }

lemma path_segments_valid_append_one
      (segments : List (List UInt8)) (segment : List UInt8)
    : Eq Bool (path_segments_valid segments) True
      → Eq Bool (path_segment_valid segment) True
      → Eq Bool
        (path_segments_valid
          (list_append (List UInt8) segments (Cons (List UInt8) segment (Nil (List UInt8)))))
        True =
  match segments {
    Nil ↦ λsegments_valid.
      λsegment_valid.
        path_bool_and_intro (path_segment_valid segment) True segment_valid Proved;
    Cons head tail ↦ λsegments_valid.
      λsegment_valid.
        path_bool_and_intro
          (path_segment_valid head)
          (path_segments_valid
            (list_append (List UInt8) tail (Cons (List UInt8) segment (Nil (List UInt8)))))
          (path_segments_valid_head head tail segments_valid)
          (path_segments_valid_append_one
            tail
            segment
            (path_segments_valid_tail head tail segments_valid)
            segment_valid)
  }

lemma path_finish_preserves_valid
      (current : List UInt8) (segments : List (List UInt8))
    : Eq Bool (path_segment_has_no_slash current) True
      → Eq Bool (path_segments_valid segments) True
      → Eq Bool (path_segments_valid (path_finish_segment current segments)) True =
  match current {
    Nil ↦ λcurrent_valid. λsegments_valid. segments_valid;
    Cons byte rest ↦ λcurrent_valid.
      λsegments_valid.
        path_segments_valid_append_one
          segments
          (Cons UInt8 byte rest)
          segments_valid
          current_valid
  }

lemma path_non_slash_singleton
      (byte : UInt8)
    : Eq Bool (path_byte_is_slash byte) False
      → Eq Bool (path_segment_has_no_slash (Cons UInt8 byte (Nil UInt8))) True =
  match path_byte_is_slash byte eqn : slash_case {
    True ↦ λnot_slash. absurd not_slash;
    False ↦ λnot_slash.
      path_equal_cong
        Bool
        Bool
        (path_byte_is_slash byte)
        False
        (λslash. path_segment_cons_has_no_slash True slash)
        slash_case
  }

lemma path_split_preserves_valid_cons
      (byte : UInt8)
      (rest : List UInt8)
      (current : List UInt8)
      (segments : List (List UInt8))
      (slash_ih : Eq
        Bool
        (path_segment_has_no_slash (Nil UInt8))
        True
        → Eq
        Bool
        (path_segments_valid (path_finish_segment current segments))
        True
        → Eq
        Bool
        (path_segments_valid
          (path_split rest (Nil UInt8) (path_finish_segment current segments)))
        True)
      (ordinary_ih : Eq
        Bool
        (path_segment_has_no_slash (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))))
        True
        → Eq
        Bool
        (path_segments_valid segments)
        True
        → Eq
        Bool
        (path_segments_valid
          (path_split rest (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) segments))
        True)
    : Eq Bool (path_segment_has_no_slash current) True
      → Eq Bool (path_segments_valid segments) True
      → Eq Bool
        (path_segments_valid
          (path_split_cons_result
            (path_split rest (Nil UInt8) (path_finish_segment current segments))
            (path_split rest (list_append UInt8 current (Cons UInt8 byte (Nil UInt8))) segments)
            (path_byte_is_slash byte)))
        True =
  match path_byte_is_slash byte eqn : slash_case {
    True ↦ λcurrent_valid.
      λsegments_valid.
        slash_ih
          Proved
          (path_finish_preserves_valid current segments current_valid segments_valid);
    False ↦ λcurrent_valid.
      λsegments_valid.
        ordinary_ih
          (path_no_slash_append
            current
            (Cons UInt8 byte (Nil UInt8))
            current_valid
            (path_non_slash_singleton byte slash_case))
          segments_valid
  }

lemma path_split_preserves_valid
      (input : List UInt8) (current : List UInt8) (segments : List (List UInt8))
    : Eq Bool (path_segment_has_no_slash current) True
      → Eq Bool (path_segments_valid segments) True
      → Eq Bool (path_segments_valid (path_split input current segments)) True =
  match input {
    Nil ↦ λcurrent_valid.
      λsegments_valid.
        path_finish_preserves_valid current segments current_valid segments_valid;
    Cons byte rest ↦ path_split_preserves_valid_cons
      byte
      rest
      current
      segments
      (path_split_preserves_valid rest (Nil UInt8) (path_finish_segment current segments))
      (path_split_preserves_valid
        rest
        (list_append UInt8 current (Cons UInt8 byte (Nil UInt8)))
        segments)
  }

lemma path_parse_valid (raw : Bytes) : Eq Bool (path_valid (path_parse raw)) True =
  path_split_preserves_valid (bytes_to_list raw) (Nil UInt8) (Nil (List UInt8)) Proved Proved

lemma path_parse_render_parse
      (raw : Bytes)
    : Eq Path (path_parse (path_render (path_parse raw))) (path_parse raw) =
  path_parse_render_valid (path_parse raw) (path_parse_valid raw)
```

## 3. Lexical normalization

`path_normalize` is **lexical, not canonical**. It removes `.` and resolves
`..` without filesystem contact; it does not resolve symlinks or imply that two
lexically equal paths identify the same filesystem object.

```ken
const path_dot_byte : UInt8 = 46

fn path_is_dot (segment : List UInt8) : Bool =
  match segment {
    Nil ↦ False;
    Cons first rest ↦
      match rest {
        Nil ↦ (DecEq_instance_UInt8).eq first path_dot_byte;
        Cons second tail ↦ False
      }
  }

fn path_is_dotdot (segment : List UInt8) : Bool =
  match segment {
    Nil ↦ False;
    Cons first rest ↦
      match rest {
        Nil ↦ False;
        Cons second tail ↦
          match tail {
            Nil ↦ bool_and
              ((DecEq_instance_UInt8).eq first path_dot_byte)
              ((DecEq_instance_UInt8).eq second path_dot_byte);
            Cons third remaining ↦ False
          }
      }
  }

const path_dot_segment : List UInt8 = Cons UInt8 path_dot_byte (Nil UInt8)

const path_dotdot_segment : List UInt8 =
  Cons UInt8 path_dot_byte (Cons UInt8 path_dot_byte (Nil UInt8))

data PathOrdinarySegment : Type where {
  MkPathOrdinarySegment :
    (bytes : List UInt8)
    → Equal Bool (path_is_dot bytes) False
    → Equal Bool (path_is_dotdot bytes) False
    → PathOrdinarySegment
}

fn path_ordinary_bytes_of (segment : PathOrdinarySegment) : List UInt8 =
  match segment {
    MkPathOrdinarySegment bytes not_dot not_dotdot ↦ bytes
  }

proof not_dot for path_ordinary_bytes_of
      (segment : PathOrdinarySegment)
    : Equal Bool (path_is_dot (path_ordinary_bytes_of segment)) False =
  match segment {
    MkPathOrdinarySegment bytes not_dot not_dotdot ↦ not_dot
  }

proof not_dotdot for path_ordinary_bytes_of
      (segment : PathOrdinarySegment)
    : Equal Bool (path_is_dotdot (path_ordinary_bytes_of segment)) False =
  match segment {
    MkPathOrdinarySegment bytes not_dot not_dotdot ↦ not_dotdot
  }

data PathNormalForm = MkPathNormalForm Nat (List PathOrdinarySegment)

fn path_normal_form (segments : List (List UInt8)) : PathNormalForm =
  match segments {
    Nil ↦ MkPathNormalForm Zero (Nil PathOrdinarySegment);
    Cons segment rest ↦
      match path_is_dot segment eqn : not_or_is_dot {
        True ↦ path_normal_form rest;
        False ↦
          match path_is_dotdot segment eqn : not_or_is_dotdot {
            True ↦
              match path_normal_form rest {
                MkPathNormalForm parents ordinary ↦ MkPathNormalForm (Suc parents) ordinary
              };
            False ↦
              match path_normal_form rest {
                MkPathNormalForm Zero ordinary ↦ MkPathNormalForm
                  Zero
                  (Cons
                    PathOrdinarySegment
                    (MkPathOrdinarySegment segment not_or_is_dot not_or_is_dotdot)
                    ordinary);
                MkPathNormalForm (Suc parents) ordinary ↦ MkPathNormalForm parents ordinary
              }
          }
      }
  }

fn path_repeat_dotdot (count : Nat) : List (List UInt8) =
  match count {
    Zero ↦ Nil (List UInt8);
    Suc rest ↦ Cons (List UInt8) path_dotdot_segment (path_repeat_dotdot rest)
  }

fn path_forget_ordinary (segments : List PathOrdinarySegment) : List (List UInt8) =
  map PathOrdinarySegment (List UInt8) path_ordinary_bytes_of segments

fn path_forget_relative (form : PathNormalForm) : List (List UInt8) =
  match form {
    MkPathNormalForm parents ordinary ↦ list_append
      (List UInt8)
      (path_repeat_dotdot parents)
      (path_forget_ordinary ordinary)
  }

fn path_forget_absolute (form : PathNormalForm) : List (List UInt8) =
  match form {
    MkPathNormalForm parents ordinary ↦ path_forget_ordinary ordinary
  }

fn path_no_special_cons_result (tail : Bool) (special : Bool) : Bool =
  match special {
    True ↦ False;
    False ↦ tail
  }

fn path_segments_no_dot (segments : List (List UInt8)) : Bool =
  match segments {
    Nil ↦ True;
    Cons segment rest ↦ path_no_special_cons_result
      (path_segments_no_dot rest)
      (path_is_dot segment)
  }

fn path_segments_no_dotdot (segments : List (List UInt8)) : Bool =
  match segments {
    Nil ↦ True;
    Cons segment rest ↦ path_no_special_cons_result
      (path_segments_no_dotdot rest)
      (path_is_dotdot segment)
  }

fn path_normalize_raw (path : Path) : Path =
  match path {
    MkPath True segments ↦ MkPath True (path_forget_absolute (path_normal_form segments));
    MkPath False segments ↦ MkPath False (path_forget_relative (path_normal_form segments))
  }

fn path_relative_ordered_cons_result
      (ordinary_seen : Bool)
      (tail_false : Bool)
      (tail_true : Bool)
      (is_dot : Bool)
      (is_dotdot : Bool)
    : Bool =
  match is_dot {
    True ↦ False;
    False ↦
      match is_dotdot {
        True ↦
          match ordinary_seen {
            True ↦ False;
            False ↦ tail_false
          };
        False ↦ tail_true
      }
  }

fn path_relative_ordered (ordinary_seen : Bool) (segments : List (List UInt8)) : Bool =
  match segments {
    Nil ↦ True;
    Cons segment rest ↦ path_relative_ordered_cons_result
      ordinary_seen
      (path_relative_ordered False rest)
      (path_relative_ordered True rest)
      (path_is_dot segment)
      (path_is_dotdot segment)
  }

fn path_normalized (path : Path) : Bool =
  match path {
    MkPath True segments ↦ bool_and
      (path_segments_no_dot segments)
      (path_segments_no_dotdot segments);
    MkPath False segments ↦ bool_and
      (path_segments_no_dot segments)
      (path_relative_ordered False segments)
  }

fn path_normalize_result (path : Path) (raw : Path) (already_normalized : Bool) : Path =
  match already_normalized {
    True ↦ path;
    False ↦ raw
  }

fn path_normalize (path : Path) : Path =
  path_normalize_result path (path_normalize_raw path) (path_normalized path)

lemma path_no_dot_cons
      (segment : List UInt8) (rest : List (List UInt8))
    : Equal Bool (path_is_dot segment) False
      → Equal Bool (path_segments_no_dot rest) True
      → Equal Bool
        (path_no_special_cons_result (path_segments_no_dot rest) (path_is_dot segment))
        True =
  match path_is_dot segment {
    True ↦ λnot_dot. λtail_clean. absurd not_dot;
    False ↦ λnot_dot. λtail_clean. tail_clean
  }

lemma path_no_dotdot_cons
      (segment : List UInt8) (rest : List (List UInt8))
    : Equal Bool (path_is_dotdot segment) False
      → Equal Bool (path_segments_no_dotdot rest) True
      → Equal Bool
        (path_no_special_cons_result (path_segments_no_dotdot rest) (path_is_dotdot segment))
        True =
  match path_is_dotdot segment {
    True ↦ λnot_dotdot. λtail_clean. absurd not_dotdot;
    False ↦ λnot_dotdot. λtail_clean. tail_clean
  }

lemma path_ordinary_no_dot
      (segments : List PathOrdinarySegment)
    : Equal Bool (path_segments_no_dot (path_forget_ordinary segments)) True =
  match segments {
    Nil ↦ Proved;
    Cons segment rest ↦ path_no_dot_cons
      (path_ordinary_bytes_of segment)
      (path_forget_ordinary rest)
      ((proof not_dot for path_ordinary_bytes_of) segment)
      (path_ordinary_no_dot rest)
  }

lemma path_ordinary_no_dotdot
      (segments : List PathOrdinarySegment)
    : Equal Bool (path_segments_no_dotdot (path_forget_ordinary segments)) True =
  match segments {
    Nil ↦ Proved;
    Cons segment rest ↦ path_no_dotdot_cons
      (path_ordinary_bytes_of segment)
      (path_forget_ordinary rest)
      ((proof not_dotdot for path_ordinary_bytes_of) segment)
      (path_ordinary_no_dotdot rest)
  }

lemma path_no_dot_head
      (segment : List UInt8) (rest : List (List UInt8))
    : Equal Bool
        (path_no_special_cons_result (path_segments_no_dot rest) (path_is_dot segment))
        True
      → Equal Bool (path_is_dot segment) False =
  match path_is_dot segment {
    True ↦ λclean. absurd clean;
    False ↦ λclean. Proved
  }

lemma path_no_dot_tail
      (segment : List UInt8) (rest : List (List UInt8))
    : Equal Bool
        (path_no_special_cons_result (path_segments_no_dot rest) (path_is_dot segment))
        True
      → Equal Bool (path_segments_no_dot rest) True =
  match path_is_dot segment {
    True ↦ λclean. absurd clean;
    False ↦ λclean. clean
  }

lemma path_no_dot_append
      (left : List (List UInt8)) (right : List (List UInt8))
    : Equal Bool (path_segments_no_dot left) True
      → Equal Bool (path_segments_no_dot right) True
      → Equal Bool (path_segments_no_dot (list_append (List UInt8) left right)) True =
  match left {
    Nil ↦ λleft_clean. λright_clean. right_clean;
    Cons segment rest ↦ λleft_clean.
      λright_clean.
        path_no_dot_cons
          segment
          (list_append (List UInt8) rest right)
          (path_no_dot_head segment rest left_clean)
          (path_no_dot_append rest right (path_no_dot_tail segment rest left_clean) right_clean)
  }

lemma path_equal_cong0
      (a : Type) (b : Type) (left : a) (right : a) (f : a → b) (same : Equal a left right)
    : Equal b (f left) (f right) =
  J (λright' _. Equal b (f left) (f right')) Refl same

fn path_list_tail (items : List UInt8) : List UInt8 =
  match items {
    Nil ↦ Nil UInt8;
    Cons item rest ↦ rest
  }

lemma path_dotdot_not_dot : Equal Bool (path_is_dot path_dotdot_segment) False = Proved

lemma path_repeat_dotdot_no_dot
      (count : Nat)
    : Equal Bool (path_segments_no_dot (path_repeat_dotdot count)) True =
  match count {
    Zero ↦ Proved;
    Suc rest ↦ path_no_dot_cons
      path_dotdot_segment
      (path_repeat_dotdot rest)
      path_dotdot_not_dot
      (path_repeat_dotdot_no_dot rest)
  }

lemma path_dotdot_is_dotdot : Equal Bool (path_is_dotdot path_dotdot_segment) True =
  (proof intro for bool_and)
    ((DecEq_instance_UInt8).eq path_dot_byte path_dot_byte)
    ((DecEq_instance_UInt8).eq path_dot_byte path_dot_byte)
    ((DecEq_instance_UInt8).complete path_dot_byte path_dot_byte Refl)
    ((DecEq_instance_UInt8).complete path_dot_byte path_dot_byte Refl)

lemma path_ordered_ordinary_cons
      (ordinary_seen : Bool) (segment : List UInt8) (rest : List (List UInt8))
    : Equal Bool (path_is_dot segment) False
      → Equal Bool (path_is_dotdot segment) False
      → Equal Bool (path_relative_ordered True rest) True
      → Equal Bool
        (path_relative_ordered_cons_result
          ordinary_seen
          (path_relative_ordered False rest)
          (path_relative_ordered True rest)
          (path_is_dot segment)
          (path_is_dotdot segment))
        True =
  match path_is_dot segment {
    True ↦ λnot_dot. λnot_dotdot. λtail_ordered. absurd not_dot;
    False ↦
      λnot_dot.
        match path_is_dotdot segment {
          True ↦ λnot_dotdot. λtail_ordered. absurd not_dotdot;
          False ↦ λnot_dotdot. λtail_ordered. tail_ordered
        }
  }

lemma path_ordinary_ordered
      (ordinary_seen : Bool) (segments : List PathOrdinarySegment)
    : Equal Bool (path_relative_ordered ordinary_seen (path_forget_ordinary segments)) True =
  match segments {
    Nil ↦ Proved;
    Cons segment rest ↦ path_ordered_ordinary_cons
      ordinary_seen
      (path_ordinary_bytes_of segment)
      (path_forget_ordinary rest)
      ((proof not_dot for path_ordinary_bytes_of) segment)
      ((proof not_dotdot for path_ordinary_bytes_of) segment)
      (path_ordinary_ordered True rest)
  }

lemma path_ordered_dotdot_result
      (segment : List UInt8) (rest : List (List UInt8))
    : Equal Bool (path_is_dot segment) False
      → Equal Bool (path_is_dotdot segment) True
      → Equal Bool (path_relative_ordered False rest) True
      → Equal Bool
        (path_relative_ordered_cons_result
          False
          (path_relative_ordered False rest)
          (path_relative_ordered True rest)
          (path_is_dot segment)
          (path_is_dotdot segment))
        True =
  match path_is_dot segment {
    True ↦ λnot_dot. λis_dotdot. λtail_ordered. absurd not_dot;
    False ↦
      λnot_dot.
        match path_is_dotdot segment {
          True ↦ λis_dotdot. λtail_ordered. tail_ordered;
          False ↦ λis_dotdot. λtail_ordered. absurd is_dotdot
        }
  }

lemma path_ordered_dotdot_cons
      (rest : List (List UInt8))
    : Equal Bool (path_relative_ordered False rest) True
      → Equal Bool
        (path_relative_ordered_cons_result
          False
          (path_relative_ordered False rest)
          (path_relative_ordered True rest)
          (path_is_dot path_dotdot_segment)
          (path_is_dotdot path_dotdot_segment))
        True =
  path_ordered_dotdot_result path_dotdot_segment rest path_dotdot_not_dot path_dotdot_is_dotdot

lemma path_relative_form_ordered
      (parents : Nat) (ordinary : List PathOrdinarySegment)
    : Equal Bool
        (path_relative_ordered
          False
          (list_append
            (List UInt8)
            (path_repeat_dotdot parents)
            (path_forget_ordinary ordinary)))
        True =
  match parents {
    Zero ↦ path_ordinary_ordered False ordinary;
    Suc rest ↦ path_ordered_dotdot_cons
      (list_append (List UInt8) (path_repeat_dotdot rest) (path_forget_ordinary ordinary))
      (path_relative_form_ordered rest ordinary)
  }

lemma path_forget_relative_no_dot
      (form : PathNormalForm)
    : Equal Bool (path_segments_no_dot (path_forget_relative form)) True =
  match form {
    MkPathNormalForm parents ordinary ↦ path_no_dot_append
      (path_repeat_dotdot parents)
      (path_forget_ordinary ordinary)
      (path_repeat_dotdot_no_dot parents)
      (path_ordinary_no_dot ordinary)
  }

lemma path_forget_absolute_no_dot
      (form : PathNormalForm)
    : Equal Bool (path_segments_no_dot (path_forget_absolute form)) True =
  match form {
    MkPathNormalForm parents ordinary ↦ path_ordinary_no_dot ordinary
  }

lemma path_absolute_form_no_dotdot
      (form : PathNormalForm)
    : Equal Bool (path_segments_no_dotdot (path_forget_absolute form)) True =
  match form {
    MkPathNormalForm parents ordinary ↦ path_ordinary_no_dotdot ordinary
  }

lemma path_absolute_form_normalized
      (form : PathNormalForm)
    : Equal Bool
        (bool_and
          (path_segments_no_dot (path_forget_absolute form))
          (path_segments_no_dotdot (path_forget_absolute form)))
        True =
  match form {
    MkPathNormalForm parents ordinary ↦ (proof intro for bool_and)
      (path_segments_no_dot (path_forget_ordinary ordinary))
      (path_segments_no_dotdot (path_forget_ordinary ordinary))
      (path_ordinary_no_dot ordinary)
      (path_ordinary_no_dotdot ordinary)
  }

lemma path_relative_form_normalized
      (form : PathNormalForm)
    : Equal Bool
        (bool_and
          (path_segments_no_dot (path_forget_relative form))
          (path_relative_ordered False (path_forget_relative form)))
        True =
  match form {
    MkPathNormalForm parents ordinary ↦ (proof intro for bool_and)
      (path_segments_no_dot
        (list_append (List UInt8) (path_repeat_dotdot parents) (path_forget_ordinary ordinary)))
      (path_relative_ordered
        False
        (list_append (List UInt8) (path_repeat_dotdot parents) (path_forget_ordinary ordinary)))
      (path_no_dot_append
        (path_repeat_dotdot parents)
        (path_forget_ordinary ordinary)
        (path_repeat_dotdot_no_dot parents)
        (path_ordinary_no_dot ordinary))
      (path_relative_form_ordered parents ordinary)
  }

lemma path_normalize_raw_normalized
      (path : Path)
    : Equal Bool (path_normalized (path_normalize_raw path)) True =
  match path {
    MkPath absolute segments ↦
      match absolute {
        True ↦ path_absolute_form_normalized (path_normal_form segments);
        False ↦ path_relative_form_normalized (path_normal_form segments)
      }
  }

lemma path_normalize_normalized_result
      (path : Path) (raw : Path) (already_normalized : Bool)
    : Equal Bool (path_normalized path) already_normalized
      → Equal Bool (path_normalized raw) True
      → Equal Bool (path_normalized (path_normalize_result path raw already_normalized)) True =
  match already_normalized {
    True ↦ λpath_decision. λraw_normalized. path_decision;
    False ↦ λpath_decision. λraw_normalized. raw_normalized
  }

lemma path_normalize_normalized
      (path : Path)
    : Equal Bool (path_normalized (path_normalize path)) True =
  path_normalize_normalized_result
    path
    (path_normalize_raw path)
    (path_normalized path)
    Refl
    (path_normalize_raw_normalized path)

lemma path_normalize_idempotent_result
      (path : Path) (raw : Path) (already_normalized : Bool)
    : Equal Bool already_normalized True
      → Eq Path (path_normalize_result path raw already_normalized) path =
  match already_normalized {
    True ↦ λis_normalized. Refl;
    False ↦ λis_normalized. absurd is_normalized
  }

lemma path_normalize_idempotent
      (path : Path)
    : Eq Path (path_normalize (path_normalize path)) (path_normalize path) =
  path_normalize_idempotent_result
    (path_normalize path)
    (path_normalize_raw (path_normalize path))
    (path_normalized (path_normalize path))
    (path_normalize_normalized path)

fn path_has_no_dot (path : Path) : Bool =
  match path {
    MkPath absolute segments ↦ path_segments_no_dot segments
  }

fn path_absolute_has_no_dotdot (path : Path) : Bool =
  match path {
    MkPath True segments ↦ path_segments_no_dotdot segments;
    MkPath False segments ↦ True
  }

lemma path_normalized_has_no_dot
      (path : Path)
    : Equal Bool (path_normalized path) True → Equal Bool (path_has_no_dot path) True =
  match path {
    MkPath absolute segments ↦
      match absolute {
        True ↦ (proof left for bool_and)
          (path_segments_no_dot segments)
          (path_segments_no_dotdot segments);
        False ↦ (proof left for bool_and)
          (path_segments_no_dot segments)
          (path_relative_ordered False segments)
      }
  }

lemma path_normalized_absolute_has_no_dotdot
      (path : Path)
    : Equal Bool (path_normalized path) True
      → Equal Bool (path_absolute_has_no_dotdot path) True =
  match path {
    MkPath absolute segments ↦
      match absolute {
        True ↦ (proof right for bool_and)
          (path_segments_no_dot segments)
          (path_segments_no_dotdot segments);
        False ↦ λnormalized. Proved
      }
  }

lemma path_normalize_has_no_dot
      (path : Path)
    : Equal Bool (path_has_no_dot (path_normalize path)) True =
  path_normalized_has_no_dot (path_normalize path) (path_normalize_normalized path)

lemma path_normalize_absolute_has_no_dotdot
      (path : Path)
    : Equal Bool (path_absolute_has_no_dotdot (path_normalize path)) True =
  path_normalized_absolute_has_no_dotdot (path_normalize path) (path_normalize_normalized path)
```

## 4. Trust and boundary

Every operation is an ordinary transparent definition over `List UInt8` and
the existing lawful `DecEq` instances. This package declares no primitive,
postulate, opaque constant, or `Axiom`; its `trusted_base()` delta is zero.
