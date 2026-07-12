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

/// `use-open-ambiguity-rejected-naming-both`: two `use`-opened modules
/// exporting the same bare name → an unqualified reference is rejected,
/// naming both sources; the qualified forms still resolve unambiguously.
#[test]
fn use_open_ambiguity_rejected_naming_both() {
    let mut env = mk_env();
    env.elaborate_file(
        "module M { pub const foo : Int = 0 } \
         module N { pub const foo : Int = 1 } \
         use M \
         use N",
    )
    .expect("both modules + opens elaborate");

    // Qualified references disambiguate regardless of the open-collision.
    assert!(env.elaborate_decl("const viaM : Int = M.foo").is_ok());
    assert!(env.elaborate_decl("const viaN : Int = N.foo").is_ok());

    // The bare, unqualified reference is ambiguous — must name both sources.
    let bad = env.elaborate_decl("const bad : Int = foo");
    match bad {
        Err(ElabError::AmbiguousReference { name, sources, .. }) => {
            assert_eq!(name, "foo");
            assert!(sources.contains(&"M.foo".to_string()));
            assert!(sources.contains(&"N.foo".to_string()));
        }
        other => panic!("AC3: expected AmbiguousReference naming both M.foo and N.foo, got {:?}", other),
    }
}

/// `local-shadows-imported-lexically`: a local declaration shadows a
/// `use`-opened import of the same bare name — lexical, innermost wins,
/// never an ambiguity error (the discriminating pair against the case
/// above: same shape, but a LOCAL is present so it must NOT error).
#[test]
fn local_shadows_imported_lexically() {
    let mut env = mk_env();
    env.elaborate_file(
        "module M { pub const foo : Int = 0 } \
         use M \
         const foo : Int = 9 \
         const getFoo : Int = foo",
    )
    .expect("local-over-import must elaborate without any ambiguity error");

    let root_foo = env.globals["foo"];
    let m_foo = env.globals["M.foo"];
    assert_ne!(root_foo, m_foo, "the root `foo` and `M.foo` are distinct decls");

    let getfoo_id = env.globals["getFoo"];
    let (_, body) = env.env.transparent_body(getfoo_id).expect("getFoo is transparent");
    assert!(
        matches!(body, Term::Const { id, .. } if id == root_foo),
        "AC3: `getFoo` must resolve `foo` to the LOCAL binding (innermost wins), \
         not the `use M`-imported `M.foo`; got {:?}",
        body
    );
}

/// `four-import-forms-resolve-to-one-binding`: qualified / aliased /
/// selective / open all resolve to the **same** underlying `GlobalId` — the
/// accept anchor confirming import is re-naming, not re-declaration.
#[test]
fn four_import_forms_resolve_to_one_binding() {
    let mut env = mk_env();
    env.elaborate_file("module M { pub const foo : Int = 0 }").expect("module M elaborates");
    let m_foo = env.globals["M.foo"];

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

    // `use M` (open) brings the SAME `M.foo` binding under bare `foo` — since
    // it's the identical qualified origin as the prior selective import, this
    // merges (not an ambiguity: `33 §3.3`'s "same declaration" exception).
    env.elaborate_file("use M").unwrap();
    let via_open = env.elaborate_decl("const c4 : Int = foo").expect("use M");
    let (_, b4) = env.env.transparent_body(via_open).unwrap();

    for (label, body) in [("qualified", &b1), ("aliased", &b2), ("selective", &b3), ("open", &b4)] {
        assert!(
            matches!(body, Term::Const { id, .. } if *id == m_foo),
            "AC3/AC1: the {} import form must resolve to the SAME GlobalId as \
             `M.foo` (re-naming, not re-declaration); got {:?}",
            label, body
        );
    }
}
