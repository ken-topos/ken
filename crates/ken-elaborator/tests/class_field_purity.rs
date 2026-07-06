//! SURF-2 D1 acceptance: optional `const`/`fn`/`proc` purity markers on class
//! fields, enforced through the real parser -> resolver -> elaborator path.

use ken_elaborator::{ClassKind, ElabEnv, ElabError};

fn err_text<T: std::fmt::Debug>(res: Result<T, ElabError>) -> String {
    format!("{:?}", res.expect_err("expected elaboration to reject"))
}

#[test]
fn surf2_d1_proc_traverse_parses_elaborates_and_registers_metadata() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_file(
        "class Functor (f : (Type -> Type)) {
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
         }",
    )
    .expect("landed marked Traversable signature must parse and elaborate");

    let traversable = env
        .class_env
        .classes
        .get("Traversable")
        .expect("Traversable class metadata is registered");
    assert_eq!(
        traversable.field_names,
        vec![
            "functor".to_string(),
            "foldable".to_string(),
            "traverse".to_string()
        ]
    );
    assert_eq!(
        traversable.field_types.len(),
        3,
        "Sigma field types stay field-only"
    );
    assert_eq!(format!("{:?}", traversable.field_purities[0]), "None");
    assert_eq!(format!("{:?}", traversable.field_purities[1]), "None");
    assert_eq!(format!("{:?}", traversable.field_purities[2]), "Some(Proc)");
}

#[test]
fn surf2_d1_marked_instance_fields_and_projection_are_checked() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_file(
        "class Effectful A {
             proc step : A ->[FS] A
         }

         proc step_int (x : Int) : Int visits [FS] = x
         instance Effectful Int { step = step_int }

         proc ok_use (x : Int) : Int visits [FS] =
             (Effectful_instance_Int).step x",
    )
    .expect("proc class field, proc implementation, and proc use must accept");

    let bad_projection = err_text(env.elaborate_decl(
        "fn bad_use (x : Int) : Int =
                 (Effectful_instance_Int).step x",
    ));
    assert!(
        bad_projection.contains("false purity or effect escape")
            && bad_projection.contains("EffectEscapes")
            && bad_projection.contains("FS"),
        "projected proc field must be visible to SURF-1 escape checking: {bad_projection}"
    );

    let bad_where = err_text(env.elaborate_decl(
        "fn bad_where_use (x : Int) : Int where Effectful Int =
                 d.step x",
    ));
    assert!(
        bad_where.contains("false purity or effect escape")
            && bad_where.contains("EffectEscapes")
            && bad_where.contains("FS"),
        "where-dictionary proc projection must be visible to SURF-1 checking: {bad_where}"
    );

    let bad_bound = err_text(env.elaborate_decl(
        "fn bad_bound (d : Effectful Int) (x : Int) : Int =
                 d.step x",
    ));
    assert!(
        bad_bound.contains("false purity or effect escape")
            && bad_bound.contains("EffectEscapes")
            && bad_bound.contains("projected proc class field `Effectful.step`"),
        "bound dictionary proc projection must be visible to SURF-1 checking: {bad_bound}"
    );

    env.elaborate_decl("fn step_bool (x : Bool) : Bool = x")
        .expect("pure implementation helper itself is valid");
    let bad_instance = err_text(env.elaborate_decl("instance Effectful Bool { step = step_bool }"));
    assert!(
        bad_instance.contains("Effectful.step")
            && bad_instance.contains("requires `proc`")
            && bad_instance.contains("implementation is pure"),
        "proc-marked class field must reject a pure implementation: {bad_instance}"
    );
}

#[test]
fn surf2_d1_const_fn_and_malformed_markers_follow_field_signature() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_file(
        "class PureFields A {
             const seed : Int
             fn unary_fn : Int -> Int
         }

         const seed_ok : Int = 0
         fn unary_fn_ok (x : Int) : Int = x
         instance PureFields Int { seed = seed_ok ; unary_fn = unary_fn_ok }",
    )
    .expect("valid const/fn class fields and instance implementations accept");

    let bad_const = err_text(env.elaborate_decl(
        "class BadConstField A {
             const not_const : Int -> Int
         }",
    ));
    assert!(
        bad_const.contains("`const` class field `not_const`") && bad_const.contains("use `fn`"),
        "function-shaped const field must reject with should-be-fn wording: {bad_const}"
    );

    let bad_fn = err_text(env.elaborate_decl(
        "class BadFnField A {
             fn not_fn : Int
         }",
    ));
    assert!(
        bad_fn.contains("`fn` class field `not_fn`") && bad_fn.contains("use `const`"),
        "value-shaped fn field must reject with should-be-const wording: {bad_fn}"
    );

    let bad_proc = err_text(env.elaborate_decl(
        "class BadProcField A {
             proc not_proc : Int
         }",
    ));
    assert!(
        bad_proc.contains("`proc` class field `not_proc`") && bad_proc.contains("use `const`"),
        "value-shaped proc field must reject with should-be-const wording: {bad_proc}"
    );

    let bad_pure_proc = err_text(env.elaborate_decl(
        "class BadPureProcField A {
             proc pure_step : A -> A
         }",
    ));
    assert!(
        bad_pure_proc.contains("`proc` class field `pure_step`")
            && bad_pure_proc.contains("no latent or row-polymorphic effect"),
        "pure-arrow proc field must reject because the field type does not earn proc: {bad_pure_proc}"
    );

    let bad_effectful_fn = err_text(env.elaborate_decl(
        "class BadEffectfulFnField A {
             fn impure_fn : A ->[FS] A
         }",
    ));
    assert!(
        bad_effectful_fn.contains("class field `impure_fn`")
            && bad_effectful_fn.contains("declares a latent or row-polymorphic effect")
            && bad_effectful_fn.contains("use `proc`"),
        "fn field with a latent effect row must reject as should-be-proc: {bad_effectful_fn}"
    );
}

#[test]
fn surf2_d1_unmarked_classes_and_sort_discriminant_stay_status_quo() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_file(
        "class Endo A {
             apply : A -> A
         }

         fn id_int (x : Int) : Int = x
         instance Endo Int { apply = id_int }

         fn pure_projection (x : Int) : Int =
             (Endo_instance_Int).apply x",
    )
    .expect("unmarked class field, instance, and projection stay pure/status quo");

    env.elaborate_file(
        "class PlainStruct A {
             op : A -> A
         }

         class MarkedStruct A {
             fn op : A -> A
         }

         class PlainProp A {
             witness : Eq Bool True True
         }

         class MarkedProp A {
             const witness : Eq Bool True True
         }",
    )
    .expect("marked and unmarked sort-discriminant pairs elaborate");

    assert_eq!(
        env.class_env.classes["PlainStruct"].kind,
        ClassKind::Structure
    );
    assert_eq!(
        env.class_env.classes["MarkedStruct"].kind,
        ClassKind::Structure
    );
    assert_eq!(env.class_env.classes["PlainProp"].kind, ClassKind::Property);
    assert_eq!(
        env.class_env.classes["MarkedProp"].kind,
        ClassKind::Property
    );
    assert_eq!(env.class_env.classes["PlainStruct"].field_types.len(), 1);
    assert_eq!(env.class_env.classes["MarkedStruct"].field_types.len(), 1);
    assert_eq!(env.class_env.classes["PlainProp"].field_types.len(), 1);
    assert_eq!(env.class_env.classes["MarkedProp"].field_types.len(), 1);
}
