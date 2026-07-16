//! ES3-build acceptance tests: modules/imports/visibility made real in the
//! elaborator (`spec/30-surface/33-declarations.md` §3-4), netted against
//! `conformance/surface/modules/seed-modules.md`'s 7 discriminating cases.
//!
//! Producer-grep discipline (the WP's load-bearing gate): every case here
//! drives the **real** `crates/ken-elaborator/src/modules.rs` expansion +
//! resolution path via `ElabEnv::elaborate_file`/`elaborate_decl` — never a
//! hand-constructed `M.foo -> GlobalId` binding.

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::env::Decl as KernelDecl;
use ken_kernel::{Level, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

// ─────────────────────────────────────────────────────────────────────────
// A. Modules elaborate away — zero TCB delta (AC1 ★)
// ─────────────────────────────────────────────────────────────────────────

/// `module-elaborates-to-identical-flat-sigma`: a `module`/`import` program
/// and its fully-qualified single-namespace equivalent produce the
/// **identical** flat `Σ` / `trusted_base()` — discriminating on that
/// identity, not "both type-check" (which would pass vacuously even if a
/// design leaked a kernel-level module/visibility primitive).
#[test]
fn module_elaborates_to_identical_flat_sigma() {
    let mut a = mk_env();
    a.elaborate_file(
        "module M { pub const foo : Int = 0 } \
         import M \
         const bar : Int = M.foo",
    )
    .expect("module program elaborates");

    let mut b = mk_env();
    b.elaborate_file(
        "const M_foo : Int = 0 \
         const bar : Int = M_foo",
    )
    .expect("flat equivalent elaborates");

    assert_eq!(
        a.env.decls().count(),
        b.env.decls().count(),
        "AC1: a module program must add exactly the same NUMBER of Σ decls \
         as its flat equivalent — no extra kernel-level module/visibility entry"
    );
    assert_eq!(
        a.env.trusted_base(),
        b.env.trusted_base(),
        "AC1: module wrapping must not perturb trusted_base() at all"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// B. Abstract export IS the opaque constant (AC2)
// ─────────────────────────────────────────────────────────────────────────

/// `abstract-export-is-the-opaque-constant`: a `pub data T = MkT` (ctors
/// never separately `pub`-able, so always withheld) must be kernel-
/// representation byte-identical to a hand-written opaque constant — no
/// new `Decl` variant, no kernel "abstract" flag.
#[test]
fn abstract_export_is_the_opaque_constant() {
    let mut env = mk_env();
    env.elaborate_file("module M { pub data T = MkT }").expect("module M elaborates");
    let t_id = env.globals["M.T"];

    let hand_id = env
        .declare_postulate_raw("HandT", Term::ty(Level::Zero))
        .expect("hand-written opaque constant declares");

    match (env.env.lookup(t_id), env.env.lookup(hand_id)) {
        (Some(KernelDecl::Opaque { level_params: lp1, ty: ty1, .. }),
         Some(KernelDecl::Opaque { level_params: lp2, ty: ty2, .. })) => {
            assert_eq!(lp1, lp2, "same (empty) level params");
            assert_eq!(ty1, ty2, "same Type-0 signature");
        }
        other => panic!(
            "AC2: abstractly-exported `T` must be Decl::Opaque, byte-identical \
             to a hand-written opaque constant; got {:?}",
            other
        ),
    }
    // No constructor is ever registered anywhere — not constructible, not
    // matchable, by any observer (kernel included).
    assert!(!env.globals.contains_key("M.MkT"));
    assert!(!env.globals.contains_key("MkT"));
}

/// Regression (language-qa, `evt_6pp9m18vp5bj6`): abstract export is a
/// `module { … }`-only concept — there is no "outside" to hide from at the
/// true file root. A top-level `pub data T = MkT` (no enclosing module)
/// must NOT be silently reinterpreted as an opaque constant; `MkT` stays a
/// real, constructible/matchable constructor, exactly as an unmarked
/// top-level `data` would (matching `pub`'s already-inert behavior on
/// top-level `View`/`Let`/`TypeAlias`).
#[test]
fn top_level_pub_data_is_not_abstract_exported() {
    let mut env = mk_env();
    env.elaborate_file("pub data T = MkT").expect("top-level pub data elaborates");

    let t_id = env.globals["T"];
    assert!(
        env.env.inductive(t_id).is_some(),
        "a top-level `pub data T` must stay a real inductive, not become an opaque constant"
    );
    assert!(env.globals.contains_key("MkT"), "the constructor must remain registered");

    // The constructor must still be constructible AND matchable in the
    // same compilation unit — the exact capability the defect silently
    // destroyed.
    env.elaborate_decl("const mk : T = MkT").expect("MkT must be constructible");
    env.elaborate_decl("fn unwrap (t : T) : Int = match t { MkT |-> 0 }")
        .expect("MkT must be matchable");
}

/// `client-match-hidden-ctor-rejected-at-surface`: a client that `import`s
/// `M` and attempts to `match` on the withheld constructor is rejected at
/// the surface — the constructor was never registered, so this fails
/// during surface elaboration, never reaching the kernel.
#[test]
fn client_match_hidden_ctor_rejected_at_surface() {
    let mut env = mk_env();
    env.elaborate_file("module M { pub data T = MkT }").expect("module M elaborates");

    let result = env.elaborate_decl(
        "fn bad (t : M.T) : Int = match t { MkT |-> 0 }",
    );
    assert!(result.is_err(), "AC2: matching a hidden constructor must be rejected");
    // Surface, not kernel: never a KernelRejected/TypeMismatch — the ctor
    // simply never entered scope.
    match result.unwrap_err() {
        ElabError::KernelRejected { .. } => {
            panic!("AC2: rejection must be a SURFACE diagnostic, not a kernel rejection")
        }
        _ => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────
// C. Visibility + resolution — surface-only, well-defined (AC3/AC4)
// ─────────────────────────────────────────────────────────────────────────

/// `private-name-access-rejected-at-surface` (+ AC4 witness): a non-`pub`
/// name is module-private; a client's qualified reference to it fails at
/// the surface, while the `pub` sibling resolves.
#[test]
fn private_name_access_rejected_at_surface() {
    let mut env = mk_env();
    env.elaborate_file(
        "module M { const secret : Int = 0 pub const api : Int = 1 } \
         import M",
    )
    .expect("module M elaborates");

    let ok = env.elaborate_decl("const getApi : Int = M.api");
    assert!(ok.is_ok(), "AC3/AC4: M.api (pub) must resolve");

    let bad = env.elaborate_decl("const getSecret : Int = M.secret");
    assert!(bad.is_err(), "AC3/AC4: M.secret (private) must be rejected");
    match bad.unwrap_err() {
        ElabError::KernelRejected { .. } => {
            panic!("AC3: private-name rejection must be surface, never kernel")
        }
        _ => {}
    }
}

/// Two selective imports binding the same bare name to different declarations
/// reject latently at the second binding (`33 §3.3`), even when no later
/// expression references the name.
#[test]
fn selective_import_ambiguity_rejected_naming_both() {
    let mut env = mk_env();
    let bad = env.elaborate_file(
        "module M { pub const foo : Int = 0 } \
         module N { pub const foo : Int = 1 } \
         import M (foo) \
         import N (foo)",
    );
    match bad {
        Err(ElabError::AmbiguousReference { name, sources, .. }) => {
            assert_eq!(name, "foo");
            assert!(sources.contains(&"M.foo".to_string()));
            assert!(sources.contains(&"N.foo".to_string()));
        }
        other => panic!("AC3: expected AmbiguousReference naming both M.foo and N.foo, got {:?}", other),
    }
}

/// N3 reversal: a TOP-LEVEL local and a selective import of the same bare name
/// clash even when the name is never referenced. This was ES3's local-wins
/// seed; N3 deliberately flips it while retaining narrower lexical shadowing.
#[test]
fn top_level_local_import_clash_is_rejected_latently() {
    let mut env = mk_env();
    let result = env.elaborate_file(
        "module M { pub const foo : Int = 0 } \
         import M (foo) \
         const foo : Int = 9",
    );
    match result {
        Err(ElabError::AmbiguousReference { name, sources, .. }) => {
            assert_eq!(name, "foo");
            assert!(sources.contains(&"foo".to_string()));
            assert!(sources.contains(&"M.foo".to_string()));
        }
        other => panic!("N3: expected latent top-level clash, got {other:?}"),
    }
}

/// `three-import-forms-resolve-to-one-binding`: qualified / aliased /
/// selective all resolve to the **same** underlying `GlobalId` — the
/// accept anchor confirming import is re-naming, not re-declaration.
#[test]
fn three_import_forms_resolve_to_one_binding() {
    let mut env = mk_env();
    env.elaborate_file("module M { pub const foo : Int = 0 }").expect("module M elaborates");
    let m_foo = env.globals["M.foo"];

    env.elaborate_file("import M").unwrap();
    let via_qualified = env
        .elaborate_decl("const c1 : Int = M.foo")
        .expect("import M / qualified M.foo");
    let (_, b1) = env.env.transparent_body(via_qualified).unwrap();

    env.elaborate_file("import M as N").unwrap();
    let via_aliased = env.elaborate_decl("const c2 : Int = N.foo").expect("import M as N");
    let (_, b2) = env.env.transparent_body(via_aliased).unwrap();

    env.elaborate_file("import M (foo)").unwrap();
    let via_selective = env.elaborate_decl("const c3 : Int = foo").expect("import M (foo)");
    let (_, b3) = env.env.transparent_body(via_selective).unwrap();

    for (label, body) in [("qualified", &b1), ("aliased", &b2), ("selective", &b3)] {
        assert!(
            matches!(body, Term::Const { id, .. } if *id == m_foo),
            "AC3/AC1: the {} import form must resolve to the SAME GlobalId as \
             `M.foo` (re-naming, not re-declaration); got {:?}",
            label, body
        );
    }
}

#[test]
fn retired_use_reports_the_migration_diagnostic() {
    let mut env = mk_env();
    let result = env.elaborate_file("use Capability.Parsing.Parsing");
    match result {
        Err(ElabError::ParseError { msg, span }) => {
            assert_eq!(
                msg,
                "`use` is retired (ADR-0015); use `import M`, `import M as N`, or \
                 `import M (…)` for a provenance-preserving import."
            );
            assert_eq!(span, ken_elaborator::Span::new(0, 3));
        }
        other => panic!("expected the specific retired-`use` ParseError, got {other:?}"),
    }
}
