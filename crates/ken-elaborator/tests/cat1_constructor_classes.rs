//! CAT-1 D1 acceptance tests for the bounded constructor-class elaborator
//! extension. These drive the production parser -> resolver -> class/instance
//! elaboration path, not helper APIs.

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

#[test]
fn explicit_class_param_kind_accepts_bare_indformer_instance_head() {
    let mut env = mk_env();

    env.elaborate_file(
        r#"
        class Pointed (f : Type -> Type) {
          empty : (a : Type) -> f a
        }

        instance Pointed List {
          empty = Nil
        }
        "#,
    )
    .expect("higher-kinded class and bare List instance should elaborate");

    assert!(
        env.class_env.instance_search("Pointed", "List").is_some(),
        "instance Functor-style bare indformer head should register under List"
    );
}

#[test]
fn parametric_instance_head_generalizes_free_type_variable() {
    let mut env = mk_env();

    env.elaborate_file(
        r#"
        class Empty A {
          empty : A
        }

        instance Empty (List a) {
          empty = Nil a
        }
        "#,
    )
    .expect("free `a` in instance head should be generalized at Type0");

    let instance_id = env
        .globals
        .get("Empty_instance_List")
        .copied()
        .expect("parametric List instance should be registered by head name");
    let (_, ty) = env
        .env
        .const_type(instance_id)
        .expect("registered instance should have a kernel type");
    assert!(
        matches!(ty, Term::Pi(_, _)),
        "parametric instance should elaborate as a Pi-typed dictionary, got {ty:?}"
    );
    assert!(
        env.class_env.instance_search("Empty", "List").is_some(),
        "parametric List instance should keep coherence key on the bare head"
    );
}
