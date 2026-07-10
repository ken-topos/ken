//! DS-8b (`docs/program/wp/ds-8b-pure-into-proc-widening.md`) acceptance —
//! a pure (`∅`-row) witness now satisfies a `proc`-marked class field
//! (covariant subsumption `∅ ⊆ open row`, SURF-1 §1.6 do-not-optimize).
//! This unblocks `class Traversable`'s `proc traverse` field, which is
//! inherently row-polymorphic but every lawful witness (`list_traverse`) is
//! genuinely pure.
//!
//! AC8 discriminator: the positive (pure witness on a proc field now
//! succeeds) and the dangerous direction (an effectful witness on a
//! fn/const field still rejects) are both exercised here, plus the
//! zero-sort-delta and AC6 (field still classifies proc) hard bars.

use ken_elaborator::{ClassKind, ElabEnv};

const TRAVERSABLE_CLASSES: &str = "
class Functor (f : (Type -> Type)) {
    map : (a : Type) -> (b : Type) -> (a -> b) -> f a -> f b
}

class Foldable (f : (Type -> Type)) {
    foldr : (a : Type) -> (b : Type) -> (a -> b -> b) -> b -> f a -> b
}

class Applicative (g : (Type -> Type)) {
    functor : Functor g ;
    pure : (a : Type) -> a -> g a ;
    ap : (a : Type) -> (b : Type) -> g (a -> b) -> g a -> g b
}

class Traversable (f : (Type -> Type)) {
    functor : Functor f ;
    foldable : Foldable f ;
    proc traverse :
      (g : (Type -> Type)) -> Applicative g -> (a : Type) -> (b : Type) ->
      (a -> g b) -> f a -> g (f b)
}
";

// The exact shape this WP exists to unblock: a genuinely pure witness for
// Traversable's row-polymorphic `proc traverse` field. Not a synthetic
// stand-in -- this is the real class shape and a real pure witness (a
// trivial but honestly pure traverse over Option, calling only pure
// Functor/Applicative operations, matching list_traverse/option_traverse's
// own purity story).
#[test]
fn ac8_positive_pure_witness_satisfies_traversable_proc_field() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(TRAVERSABLE_CLASSES)
        .expect("Traversable class family must elaborate");

    env.elaborate_file(
        "fn map_option (a : Type) (b : Type) (f : a -> b) (o : Option a) : Option b =
             match o { None => None b ; Some x => Some b (f x) }

         fn foldr_option (a : Type) (b : Type) (f : a -> b -> b) (z : b) (o : Option a) : b =
             match o { None => z ; Some x => f x z }

         instance Functor Option { map = map_option }
         instance Foldable Option { foldr = foldr_option }

         fn option_traverse
           (g : (Type -> Type)) (apg : Applicative g) (a : Type) (b : Type)
           (f : a -> g b) (o : Option a) : g (Option b) =
             match o {
               None    => apg.pure (Option b) (None b) ;
               Some x  => apg.ap b (Option b)
                            (apg.pure (b -> Option b) (Some b))
                            (f x)
             }",
    )
    .expect("Functor/Foldable Option instances + the pure option_traverse witness must elaborate");

    // Before DS-8b this instance elaboration rejected outright ("class field
    // `Traversable.traverse` requires `proc` but instance implementation is
    // pure") -- that's the exact bug this WP fixes.
    env.elaborate_decl(
        "instance Traversable Option {
             functor = Functor_instance_Option ;
             foldable = Foldable_instance_Option ;
             traverse = option_traverse
         }",
    )
    .expect(
        "DS-8b: a pure traverse witness must now satisfy Traversable's proc-marked \
         traverse field (covariant ∅ ⊆ open-row subsumption)",
    );
}

// AC6: accepting a pure witness must not weaken the FIELD's own
// classification -- `d.traverse` still projects as `proc` at a use site, so
// a pure caller still can't call it directly through the class dictionary,
// exactly as before DS-8b. The widening only concerns which witnesses can
// INHABIT the field, never the field's declared purity.
#[test]
fn ac6_proc_field_still_classifies_proc_at_projection_after_pure_instance() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(
        "class Effectful A {
             proc step : A ->[FS] A
         }

         fn step_pure (x : Bool) : Bool = x
         instance Effectful Bool { step = step_pure }",
    )
    .expect("class decl + pure instance for a proc field must elaborate (DS-8b)");

    let err = env
        .elaborate_decl(
            "fn bad_use (x : Bool) : Bool =
                 (Effectful_instance_Bool).step x",
        )
        .expect_err("AC6: proc field projection must still be rejected from a pure fn");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("false purity or effect escape") && msg.contains("EffectEscapes"),
        "the proc field must still classify as proc at the use site, regardless of \
         which witness inhabits it for this instance: {msg}"
    );
}

// AC8 dangerous-direction discriminator, unchanged: an actually-effectful
// witness assigned to a fn/const (pure) field still rejects, on the
// SPECIFIC error variant/message -- this is the arm (elab.rs:3189,
// Const|Fn if impure) DS-8b must not touch.
#[test]
fn ac8_negative_effectful_witness_still_rejected_for_fn_field() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(
        "class Pure A {
             fn compute : A -> A
         }

         proc compute_effectful (x : Bool) : Bool visits [FS] = x",
    )
    .expect("class decl + effectful helper must elaborate on their own");
    let err = env
        .elaborate_decl("instance Pure Bool { compute = compute_effectful }")
        .expect_err("an effectful witness for an fn class field must still be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Pure.compute")
            && msg.contains("requires")
            && msg.contains("effectful"),
        "must reject via the specific effectful-witness-for-pure-field message, \
         not a generic error: {msg}"
    );
}

#[test]
fn ac8_negative_effectful_witness_still_rejected_for_const_field() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(
        "class HasSeed A {
             const seed : A
         }

         proc effectful_seed : Bool visits [FS] = True",
    )
    .expect("class decl + effectful helper must elaborate on their own");
    let err = env
        .elaborate_decl("instance HasSeed Bool { seed = effectful_seed }")
        .expect_err("an effectful witness for a const class field must still be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("HasSeed.seed") && msg.contains("effectful"),
        "must reject via the specific effectful-witness-for-pure-field message: {msg}"
    );
}

// Zero-TCB/sort delta: the class-record's Type/Ω sort discriminant
// (ClassKind, computed at declaration time from field TYPES alone, kernel's
// sort_sigma) is captured immediately after the class declares -- BEFORE
// any instance exists -- then re-checked after registering the pure
// instance DS-8b newly accepts. Identical before/after proves the
// instance-purity widening has zero effect on the class's own kernel-level
// Σ-structure, exactly as the WP requires (not just "no ken-kernel files
// touched" -- an executable assertion on the actual discriminant).
#[test]
fn zero_sort_delta_class_record_kind_unchanged_by_pure_instance_registration() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(TRAVERSABLE_CLASSES)
        .expect("Traversable class family must elaborate");

    let kind_before = env.class_env.classes["Traversable"].kind.clone();
    let field_types_len_before = env.class_env.classes["Traversable"].field_types.len();
    let type_id_before = env.class_env.classes["Traversable"].type_id;
    assert_eq!(
        kind_before,
        ClassKind::Structure,
        "Traversable carries computational content (Functor/Foldable dicts + \
         the traverse function), so it must be Type-sorted Structure"
    );

    env.elaborate_file(
        "fn map_option (a : Type) (b : Type) (f : a -> b) (o : Option a) : Option b =
             match o { None => None b ; Some x => Some b (f x) }

         fn foldr_option (a : Type) (b : Type) (f : a -> b -> b) (z : b) (o : Option a) : b =
             match o { None => z ; Some x => f x z }

         instance Functor Option { map = map_option }
         instance Foldable Option { foldr = foldr_option }

         fn option_traverse
           (g : (Type -> Type)) (apg : Applicative g) (a : Type) (b : Type)
           (f : a -> g b) (o : Option a) : g (Option b) =
             match o {
               None    => apg.pure (Option b) (None b) ;
               Some x  => apg.ap b (Option b)
                            (apg.pure (b -> Option b) (Some b))
                            (f x)
             }

         instance Traversable Option {
             functor = Functor_instance_Option ;
             foldable = Foldable_instance_Option ;
             traverse = option_traverse
         }",
    )
    .expect("the pure Traversable Option instance (DS-8b) must elaborate");

    let class_after = &env.class_env.classes["Traversable"];
    assert_eq!(
        class_after.kind, kind_before,
        "zero sort delta: registering a pure instance for the proc traverse \
         field must not change the class-record's Type/Ω sort discriminant"
    );
    assert_eq!(
        class_after.field_types.len(),
        field_types_len_before,
        "zero sort delta: the Sigma-telescope field-type count is unchanged"
    );
    assert_eq!(
        class_after.type_id, type_id_before,
        "zero sort delta: the class's own kernel GlobalId is unchanged \
         (registering an instance never re-declares the class)"
    );
}

// Non-regression: the malformed-marker rejections (a value-shaped field
// wrongly marked `proc`, etc.) are untouched -- these reject at CLASS-DECL
// time (field-type-vs-marker mismatch), an entirely different check from
// check_instance_field_purity, and DS-8b must not touch them.
#[test]
fn non_regression_malformed_proc_field_marker_still_rejected_at_class_decl() {
    let mut env = ElabEnv::new().expect("base env");
    let err = env
        .elaborate_decl(
            "class BadPureProcField A {
                 proc pure_step : A -> A
             }",
        )
        .expect_err("a value-shaped/no-latent-effect proc field must still reject at class-decl");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("`proc` class field `pure_step`") && msg.contains("no latent or row-polymorphic effect"),
        "DS-8b must not affect the class-decl-time marker/shape check: {msg}"
    );
}
