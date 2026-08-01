//! SEAL-2 — the closed, carrier-parameterized producer-closure oracle.
//!
//! This is the primary acceptance for SEAL-2. It exercises the closed oracle in
//! `seal2_support` along all three closure axes and pins the loud-failure
//! behavior. The starting-evidence property/gap tests (AC-8) live in the sibling
//! `adversary_seal2_repros.rs`, which shares the same oracle.
//!
//! SPAN-SEAL's own two alias discriminators
//! (`px8f_buffer_io_surface.rs::buffer_span_producer_closure_reduces_transparent_type_aliases`
//! and `…_resolves_public_constructors`) are retained **unchanged** by leaving
//! that file untouched (AC-3).

mod seal2_support;

use std::collections::BTreeSet;

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

use seal2_support::{
    assert_confined, carrier_closure, catalog_package_files, catalog_root_facts, closed_producers,
    conservative_deep_producers, head_only_producers, reaching_roots, result_type_produces,
    synthetic_facts, ALLOWED_SECTIONS,
};

const BUFFER_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/System/Buffer.ken.md");
const IO_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/System/IO.ken.md");

fn landed_surface() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("SEAL-2 prelude");
    env.elaborate_ken_md_file(BUFFER_KEN_MD)
        .expect("System.Buffer checked fences");
    env.elaborate_ken_md_file(IO_KEN_MD)
        .expect("System.IO checked fences");
    env
}

fn carrier_former(env: &ElabEnv, carrier_name: &str) -> Term {
    Term::IndFormer {
        id: env.globals[carrier_name],
        level_args: Vec::new(),
    }
}

// ===========================================================================
// AC-1 — one derivation, instantiated at every sealed carrier.
// ===========================================================================

/// The identical `closed_producers` derivation, applied to both sealed carriers
/// with no per-carrier walker copy. Adding a third sealed carrier is a one-line
/// call at this same shape.
#[test]
fn one_derivation_covers_both_sealed_carriers() {
    let env = landed_surface();
    for carrier in ["BufferSpan", "TransferCount"] {
        assert_eq!(
            closed_producers(&env, carrier),
            BTreeSet::new(),
            "landed surface must expose no public `{carrier}` producer"
        );
    }
}

// ===========================================================================
// AC-3 — closure over POSITION (carrier detected off-head, modulo defeq).
// ===========================================================================

/// A `Result ResourceError BufferSpan` producer: the carrier is present but not
/// at the head. The head-only oracle derives `{}` (its blind spot); the closed
/// oracle sees it. (AC-3, AC-6 — adversary S1 family A.)
#[test]
fn closed_oracle_sees_carrier_off_head_under_result() {
    let mut env = ElabEnv::empty().expect("prelude");
    env.elaborate_file(
        "fn escaped_wrapped (span : BufferSpan) : Result ResourceError BufferSpan = \
         Ok ResourceError BufferSpan span",
    )
    .expect("wrapped producer elaborates");

    assert!(
        !head_only_producers(&env, "BufferSpan").contains("escaped_wrapped"),
        "PREVIOUS oracle: the head-only walk derives {{}} here — that is the gap"
    );
    assert!(
        closed_producers(&env, "BufferSpan").contains("escaped_wrapped"),
        "CLOSED oracle: the off-head carrier under `Result` is now seen"
    );
}

/// A producer returning `Option BufferSpanAlias`, where `BufferSpanAlias` is a
/// transparent alias of `BufferSpan`. This is the case that distinguishes the
/// closed oracle from the adversary's conservative `mentions()` fixture: the
/// head-only oracle is blind (off-head), the conservative deep walk is *also*
/// blind (it does not unfold `Const`s in non-head positions), and only the
/// closed oracle — WHNF-normalizing at every node — unfolds the alias and sees
/// the carrier. This is precisely why adopting `mentions()` wholesale (the ⛔ in
/// the brief) would have been instance #6 of the same defect. (AC-3, AC-6.)
#[test]
fn closed_oracle_unfolds_a_transparent_alias_in_a_nonhead_position() {
    let mut env = ElabEnv::empty().expect("prelude");
    env.elaborate_file(
        "def BufferSpanAlias = BufferSpan\n\
         fn escaped_opt (span : BufferSpan) : Option BufferSpanAlias = \
         Some BufferSpanAlias span",
    )
    .expect("aliased wrapped producer elaborates");

    assert!(
        !head_only_producers(&env, "BufferSpan").contains("escaped_opt"),
        "PREVIOUS oracle (head-only): blind"
    );
    assert!(
        !conservative_deep_producers(&env, "BufferSpan").contains("escaped_opt"),
        "ADVERSARY fixture (conservative mentions()): ALSO blind — does not unfold \
         the alias in a non-head position; adopting it would be instance #6"
    );
    assert!(
        closed_producers(&env, "BufferSpan").contains("escaped_opt"),
        "CLOSED oracle: unfolds `BufferSpanAlias` and sees the carrier"
    );
}

/// Per-former position coverage, synthesized at the `Term` level so it is
/// independent of surface syntax: the carrier former wrapped by each structural
/// position must be detected, and a wrapper of a NON-carrier former must not
/// produce a false hit. The head-only oracle misses every one of these (the
/// carrier is never at the head). (AC-3.)
#[test]
fn position_closure_holds_under_every_structural_former() {
    let env = landed_surface();
    let bs = carrier_former(&env, "BufferSpan");
    let filler = Term::Var(0); // a non-carrier leaf used to occupy sibling slots
    let b = |t: Term| Box::new(t);

    let carrier_positions = [
        ("app-argument", Term::App(b(filler.clone()), b(bs.clone()))),
        ("sigma-codomain", Term::Sigma(b(filler.clone()), b(bs.clone()))),
        ("pair-second", Term::Pair(b(filler.clone()), b(bs.clone()))),
        (
            "eq-rhs",
            Term::Eq(b(filler.clone()), b(filler.clone()), b(bs.clone())),
        ),
        (
            "nested-app-under-sigma",
            Term::Sigma(
                b(filler.clone()),
                b(Term::App(b(filler.clone()), b(bs.clone()))),
            ),
        ),
    ];
    for (label, ty) in carrier_positions {
        assert!(
            result_type_produces(&env.env, &ty, env.globals["BufferSpan"]),
            "carrier at position `{label}` must be detected"
        );
    }

    // A wrapper containing NO carrier must not be a false positive.
    let no_carrier = Term::Sigma(b(filler.clone()), b(Term::App(b(filler.clone()), b(filler))));
    assert!(
        !result_type_produces(&env.env, &no_carrier, env.globals["BufferSpan"]),
        "a carrier-free result type must not be reported"
    );
}

// ===========================================================================
// AC-2 — namespace closure: class fields are enumerated (a second namespace).
// ===========================================================================

/// A class field `unbox : A → BufferSpan` produces the carrier by projection,
/// but lives in `class_env.classes[C].field_types`, never in `globals`. The
/// head-only oracle (globals-only) cannot see it at all; the closed oracle's
/// structure-derived enumeration reaches the class-field namespace. (AC-2, AC-6
/// — adversary S1 family B.)
#[test]
fn closed_oracle_reaches_the_class_field_namespace() {
    let mut env = ElabEnv::empty().expect("prelude");
    env.elaborate_decl("class SpanBox A { unbox : A → BufferSpan }")
        .expect("class with a BufferSpan-producing field elaborates");
    env.elaborate_decl("instance SpanBox BufferSpan { unbox = λs.s }")
        .expect("instance elaborates");

    // The class field is invisible to the globals-only namespace...
    assert!(
        head_only_producers(&env, "BufferSpan").is_empty(),
        "PREVIOUS oracle (globals-only): the class field never enters `globals`"
    );
    // ...but the closed enumeration reaches it.
    assert!(
        closed_producers(&env, "BufferSpan").contains("SpanBox.unbox"),
        "CLOSED oracle: the class-field namespace is enumerated and walked"
    );
}

// ===========================================================================
// AC-4 + AC-7 — closure over SOURCE ROOT via a DERIVED semantic certificate.
//
// A single flat ElabEnv over every catalog package is NOT achievable (the
// catalog is layered; arbitrary-order co-load collides — Transport dup `sym`,
// LawfulClasses overlapping instances) and is NOT required. The source-root
// closure is a fail-closed semantic certificate (seal2_support): parse each
// root's checked fences and mark a root NON-EXEMPT when any declaration
// references a carrier-closure name in any type OR body position (the body axis
// is load-bearing — Ken's bottom eliminator inhabits a carrier from an ex-falso
// body), OR uses a form the certificate cannot resolve (`module`/`import`/
// `export` → fail closed). Its
// soundness is the conjunction documented on `RootFacts`, NOT "every producer
// names the carrier" (which is false for a term forwarder `fn leak = makeSpan`).
// Every non-exempt root must be explicitly enumerated, loaded through its known
// dependency env, and checked by closed_producers; any other is named and fails.
// ===========================================================================

/// The non-exempt roots whose known dependency environment we load and run
/// through `closed_producers`. The certificate DERIVES the non-exempt set; this
/// names the ones we have an elaborated env for. Buffer/IO are non-exempt from
/// their real consumer signatures (they name the carriers), so this assertion is
/// exercised by production data — the certificate fails-and-names any OTHER
/// non-exempt root. Their canonical env is `prelude + Buffer + IO`
/// (`landed_surface`); a third would be one more entry + its loaded env.
const ENUMERATED_CARRIER_ROOTS: [&str; 2] = [
    "Capability/System/Buffer.ken.md",
    "Capability/System/IO.ken.md",
];

/// The glob ranges over EVERY catalog package and classifies each Section (an
/// unclassifiable root panics — pinned separately). Not a hardcoded list. (AC-4.)
#[test]
fn catalog_glob_ranges_over_every_package() {
    let packages = catalog_package_files(); // panics on an unclassifiable root
    assert!(
        packages.len() >= 30,
        "the glob must range over the whole catalog, found only {}",
        packages.len()
    );
    for (section, path) in &packages {
        assert!(
            ALLOWED_SECTIONS.contains(&section.as_str()),
            "package {} classified to a non-allowed Section `{section}`",
            path.display()
        );
    }
}

/// The derived source-root confinement certificate (AC-4) + AC-7. Over the real
/// catalog: every non-exempt root is admitted only if enumerated (Buffer/IO are
/// non-exempt from their consumer signatures — this exercises the admission
/// assertion against production data), else named and failed. The enumerated
/// roots' canonical env (`prelude + Buffer + IO`) derives `{}` for both carriers
/// — the AC-7 result and the conjunction's clause 1.
#[test]
fn catalog_source_root_confinement_certificate() {
    let facts = catalog_root_facts();
    assert!(
        facts.len() >= 30,
        "certificate must range over the whole catalog, parsed only {}",
        facts.len()
    );
    let closure = carrier_closure(&facts);
    println!("carrier closure: {closure:?}");
    println!("reaching roots: {:?}", reaching_roots(&facts, &closure));

    // Admission: every non-exempt root must be enumerated, else fail-named.
    assert_confined(&facts, &ENUMERATED_CARRIER_ROOTS);

    // Buffer/IO ARE non-exempt against real data — the enumerated assertion is
    // not a vacuous forward tripwire.
    let reaching: BTreeSet<String> = reaching_roots(&facts, &closure)
        .into_iter()
        .map(|(root, _, _)| root)
        .collect();
    for enumerated in ENUMERATED_CARRIER_ROOTS {
        assert!(
            reaching.contains(enumerated),
            "expected `{enumerated}` to be non-exempt from its real consumer \
             signatures, so the admission assertion is exercised by production data"
        );
    }

    // AC-7 / clause 1: the enumerated roots' canonical env exposes no public
    // producer for either carrier.
    let env = landed_surface();
    for carrier in ["BufferSpan", "TransferCount"] {
        assert_eq!(
            closed_producers(&env, carrier),
            BTreeSet::new(),
            "a public `{carrier}` producer exists in the carrier-bearing env — live \
             finding, route it before changing anything"
        );
    }
}

/// Discriminator (d) — the actual UNENUMERATED-ROOT REJECTION PATH, and the
/// ALIAS BYPASS the raw `source.contains()` guard missed. A pseudo-System root
/// aliases the carrier; a pseudo-external root builds a producer using ONLY the
/// alias (its source spells no carrier). `SpanAlias` joins the closure, the
/// external root becomes non-exempt, and `assert_confined` — with the external
/// root NOT enumerated — drives the real admission panic naming it.
#[test]
#[should_panic(expected = "Pseudo/External/Leak.ken.md")]
fn certificate_rejects_an_unenumerated_aliased_producer_root() {
    let external_src = "```ken\nfn leak (s : SpanAlias) : SpanAlias = s\n```";
    assert!(
        !external_src.contains("BufferSpan") && !external_src.contains("TransferCount"),
        "precondition: the external producer must not spell a carrier"
    );
    let facts = vec![
        synthetic_facts(
            "Pseudo/System/Alias.ken.md",
            "```ken\ndef SpanAlias = BufferSpan\n```",
        ),
        synthetic_facts("Pseudo/External/Leak.ken.md", external_src),
    ];
    // `SpanAlias` must have joined the closure via the alias for this to bite.
    assert!(carrier_closure(&facts).contains("SpanAlias"));
    // The System root names `BufferSpan` directly (like Buffer/IO) — enumerate
    // it; the EXTERNAL root spells only the alias, and must STILL be rejected by
    // name. This is the bypass the raw `source.contains()` guard passed.
    assert_confined(&facts, &["Pseudo/System/Alias.ken.md"]);
}

/// Discriminator (a) — INFERRED RESULT TYPE. `fn leak (span : BufferSpan) = span`
/// has no result annotation (the elaborator would infer `BufferSpan -> BufferSpan`).
/// The certificate does not infer; it flags the root non-exempt because the
/// parameter names the carrier — so an unannotated producer is never silently
/// omitted. `assert_confined` rejects the unenumerated root by name.
#[test]
#[should_panic(expected = "Pseudo/Inferred/Leak.ken.md")]
fn certificate_flags_an_unannotated_producer_via_its_carrier_parameter() {
    let facts = vec![synthetic_facts(
        "Pseudo/Inferred/Leak.ken.md",
        "```ken\nfn leak (span : BufferSpan) = span\n```",
    )];
    assert_confined(&facts, &[]);
}

/// Discriminator (c) — GENERIC CLASS + CONCRETE INSTANCE substitution. The class
/// row `class Echo A { echo : A -> A }` reaches no carrier (open `A`), but
/// `instance Echo BufferSpan { echo = \x. x }` specializes the projection to
/// `BufferSpan -> BufferSpan`. The certificate flags the root because the
/// instance head names the carrier — instances are not silently discarded.
#[test]
#[should_panic(expected = "Pseudo/Instance/Echo.ken.md")]
fn certificate_flags_a_concrete_instance_of_a_generic_class() {
    let facts = vec![synthetic_facts(
        "Pseudo/Instance/Echo.ken.md",
        "```ken\nclass Echo A { echo : A → A }\ninstance Echo BufferSpan { echo = λx. x }\n```",
    )];
    assert_confined(&facts, &[]);
}

/// Discriminator (e) — BOTTOM ELIMINATOR (ex-falso). Ken's `absurd_empty
/// (C : Type) (e : Empty) : C` inhabits any type from a proof of `Empty`, so
/// `fn leak (e : Empty) = absurd_empty BufferSpan e` is a carrier producer with
/// NO carrier in any TYPE position — the carrier is named as the eliminated
/// motive in the BODY. The certificate scans bodies, so it flags the root; the
/// unenumerated root is rejected by name. (Closes the Architect's `ef20cb13`
/// counterexample; the conjunction on `RootFacts` names this mechanism.)
#[test]
#[should_panic(expected = "Pseudo/ExFalso/Leak.ken.md")]
fn certificate_flags_an_ex_falso_bottom_path_producer() {
    let body_only = "```ken\nfn leak (e : Empty) = absurd_empty BufferSpan e\n```";
    // The only carrier mention is in the body — no type position spells it.
    assert!(
        body_only.contains("BufferSpan"),
        "precondition: the ex-falso body names the carrier motive"
    );
    let facts = vec![synthetic_facts("Pseudo/ExFalso/Leak.ken.md", body_only)];
    assert_confined(&facts, &[]);
}

/// Discriminator (b) — MODULE / IMPORT IDENTITY, fail closed. A `module` (or an
/// `import`) carries cross-module identity the certificate does not resolve; it
/// sets `requires_semantic_resolution`, making the root non-exempt regardless of
/// whether a carrier name is textually visible. `assert_confined` rejects it.
#[test]
#[should_panic(expected = "requires-semantic-resolution")]
fn certificate_fails_closed_on_an_unresolvable_module_form() {
    let facts = vec![synthetic_facts(
        "Pseudo/Module/Qualified.ken.md",
        "```ken\nmodule M { def SpanAlias = BufferSpan }\n\
         fn leak (s : M.SpanAlias) : M.SpanAlias = s\n```",
    )];
    assert_confined(&facts, &[]);
}

/// PROSE CONTROL. A carrier named only in Markdown prose (outside a checked
/// fence) must not classify a root: the extractor blanks prose, so the
/// certificate is structural, not a noisier grep. This root has no unresolvable
/// form and references nothing in the closure ⇒ exempt ⇒ `assert_confined` does
/// not fire even with no enumerated roots.
#[test]
fn certificate_ignores_prose_only_carrier_mentions() {
    let facts = vec![synthetic_facts(
        "Pseudo/Prose/Only.ken.md",
        "This prose paragraph mentions BufferSpan and TransferCount but declares \
         nothing.\n\n```ken\nfn f (n : Nat) : Nat = n\n```",
    )];
    let closure = carrier_closure(&facts);
    assert!(
        reaching_roots(&facts, &closure).is_empty(),
        "prose-only carrier mentions must not classify a root as carrier-bearing"
    );
    assert_confined(&facts, &[]); // exempt ⇒ no admission failure
}

/// The CONJUNCTION's load-bearing clause (proof note): a term-only forwarder
/// (`fn leak = makeSpan`) is not credited to the root scanner — its soundness
/// rests on the upstream closed-producer inventory rejecting any source-visible
/// carrier producer. Here a producer IS seeded into the env; `closed_producers`
/// flags it — so were a `makeSpan` ever source-visible, clause 1 would reject it
/// and no exempt forwarder could stand.
#[test]
fn a_source_visible_producer_is_rejected_by_the_upstream_inventory() {
    let mut env = ElabEnv::empty().expect("prelude");
    env.elaborate_file(
        "fn makeSpan (span : BufferSpan) : Result ResourceError BufferSpan = \
         Ok ResourceError BufferSpan span",
    )
    .expect("seeded producer elaborates");
    assert!(
        closed_producers(&env, "BufferSpan").contains("makeSpan"),
        "the upstream inventory must reject a source-visible carrier producer — the \
         clause that licenses exempting a term-only forwarder"
    );
}

// ===========================================================================
// AC-5 — loud failure on anything unhandled, per axis.
// ===========================================================================
//
// * type-former axis  — closed at COMPILE time by the wildcard-free `Term` match
//   in `seal2_support::occurs`; a new variant is a build break. There is no
//   runtime path to a "defaulted" former, so it is pinned by the match itself.
// * namespace-member axis — runtime panic (below).
// * source-root axis  — runtime panic (below).

/// A `globals` id that is neither a declaration nor a constructor fails loudly
/// rather than being silently filtered. (AC-5, namespace-member axis.)
#[test]
#[should_panic(expected = "is neither a declaration nor a constructor")]
fn unclassifiable_global_member_fails_loudly() {
    let mut env = ElabEnv::empty().expect("prelude");
    env.globals
        .insert("escaped_unknown_id".to_string(), ken_kernel::GlobalId(u32::MAX));
    let _ = closed_producers(&env, "BufferSpan");
}

/// A catalog package root outside the controlled Section allowlist fails loudly
/// rather than silently covering fewer roots than claimed. (AC-5, source-root
/// axis.)
#[test]
#[should_panic(expected = "is not an allowed Section")]
fn unclassifiable_source_root_fails_loudly() {
    seal2_support::classify_section("Misc");
}

/// Sanity: the controlled Section allowlist the catalog glob classifies against
/// matches `catalog_taxonomy.rs`'s controlled set.
#[test]
fn source_root_allowlist_is_the_controlled_taxonomy() {
    assert_eq!(
        ALLOWED_SECTIONS,
        [
            "Core",
            "Data",
            "Algorithm",
            "Capability",
            "Protocol",
            "Application",
            "Tooling",
        ]
    );
}
