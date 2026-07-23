//! SEAL-2 — the closed producer-closure oracle, shared by the two SEAL-2 test
//! binaries (`seal2_producer_closure.rs` and `adversary_seal2_repros.rs`).
//!
//! # What this repairs
//!
//! SPAN-SEAL landed a producer-closure oracle for `BufferSpan`
//! (`px8f_buffer_io_surface.rs::public_buffer_span_producers`). The *property*
//! it checks holds — but the *gate* guarantees materially less than the sentence
//! it carries, along three independent axes:
//!
//!   * **namespace** — it iterates `env.globals` only. A class field
//!     (`class_env.classes[C].field_types`) is source-reachable by projection
//!     yet never enters `globals`, so it is invisible.
//!   * **position** — it matches only the *head* of a result type. A
//!     `Result E BufferSpan` producer has the carrier off-head, so it is skipped.
//!   * **source root** — it elaborates only `{prelude, Buffer, IO}`. A producer
//!     in any other catalog package is outside its environment entirely.
//!
//! Each earlier repair *named a mechanism* (a Pi-codomain walk, then
//! `GlobalEnv::lookup`, then "declarations and constructors") and inherited that
//! mechanism's blind spot. The lesson
//! (`an-enumeration-needs-a-proven-closure-not-a-better-grep`) is that a bigger
//! grep is never the fix: you find the *gate every member must pass through* and
//! bind the enumeration to it, so the population cannot silently omit a member.
//!
//! # How each axis is CLOSED here (not merely widened)
//!
//!   * **namespace (AC-2)** — [`enumerate_producer_types`] is bound to
//!     `ElabEnv`'s own fields by an exhaustive struct destructuring with **no
//!     `..`**. Adding a field to `ElabEnv` is a *build break* in this file,
//!     forcing whoever adds it to classify the new namespace. The known-today
//!     producer sources are *fixtures* the discriminators test the walker
//!     against, never the walker's definition.
//!   * **position (AC-3)** — [`result_type_produces`] descends into *every*
//!     sub-position of the result type via an **exhaustive `match` over `Term`**
//!     (no wildcard), WHNF-normalizing at each node so transparent aliases
//!     unfold. A new `Term` variant is a build break; no compound former is ever
//!     skipped or defaulted.
//!   * **source root (AC-4)** — [`catalog_package_files`] globs *every* catalog
//!     package (a root outside the controlled Section allowlist is a loud
//!     failure), and a **derived semantic confinement certificate**
//!     ([`root_facts`] + [`carrier_closure`] + [`assert_confined`]) parses each
//!     root's checked fences and marks a root non-exempt when any declaration
//!     references a carrier-closure name in any type **or body** position (the
//!     body axis is load-bearing — Ken's bottom eliminator can inhabit a carrier
//!     from an ex-falso body), or uses a form the certificate cannot resolve
//!     (`module`/`import`/`export` → fail closed). Its
//!     soundness is the conjunction on [`RootFacts`], not "every producer names
//!     the carrier". A single flat all-catalog `ElabEnv` is not achievable (the
//!     catalog is layered and co-load collides) and is not required; each
//!     non-exempt root is instead enumerated and checked by [`closed_producers`].
//!
//! # AC-5 — loud failure on anything unhandled, per axis
//!
//! The three axes fail closed by the strongest mechanism each admits:
//!   * type-former axis → **compile time** (the wildcard-free `Term` match);
//!   * namespace-member axis → **runtime panic** in [`type_of_global`] when a
//!     `globals` id is neither a declaration nor a constructor;
//!   * source-root axis → **runtime panic** in [`classify_section`] on a root
//!     outside the allowlist.
//! The two runtime panics are pinned by `#[should_panic]` tests; the compile-time
//! one is pinned by the match being exhaustive.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::{
    extract_ken_md, parser, validate_ken_md_fences, ConstructorSignatureArg, Decl as SurfaceDecl,
    ElabEnv, ExplicitDataCtor, Expr, Type as SurfaceType,
};
use ken_kernel::{infer, whnf, Context, Decl, GlobalEnv, GlobalId, Term};

// ---------------------------------------------------------------------------
// Namespace enumeration (AC-2) — DERIVED from ElabEnv's own field structure.
// ---------------------------------------------------------------------------

/// A source-reachable producer surface: a named entity whose declared type is
/// walked for the carrier. `namespace` records which structural namespace of
/// `ElabEnv` it came from — [`enumerate_producer_types`] is the sole authority
/// on that set.
pub struct Producer {
    pub namespace: &'static str,
    pub name: String,
    pub ty: Term,
}

/// Enumerate **every** namespace of `ElabEnv` in which a source-reachable
/// producer can live.
///
/// The walk is bound to `ElabEnv`'s own fields by an exhaustive struct
/// destructuring with **no `..`**. Adding a field to `ElabEnv` fails to compile
/// here, forcing whoever adds it to classify the new namespace as either a
/// producer source (walked below) or a justified non-source (added to the
/// discard with its reason). This is the AC-2 gate: a new namespace can never be
/// a silent pass, only a build break.
///
/// ⛔ Do **not** rewrite this as field access (`env.globals`, `env.class_env`,
/// …). Naming every field with no `..` is the entire point — it is what makes
/// the enumeration closed rather than a hand list waiting for its next omission.
pub fn enumerate_producer_types(env: &ElabEnv) -> Vec<Producer> {
    let ElabEnv {
        // --- namespaces that hold carrier-bearing `Term`s of their own ---
        env: global_env,
        globals,
        class_env,
        // --- fields that hold NO carrier-bearing `Term` of their own ---
        // Each is either an index of `GlobalId`s into `global_env` (its producer
        // types are therefore already reached through `globals` below) or an
        // alias/value table carrying no `Term`. Named explicitly, with a reason,
        // so the build breaks the instant one of these stops being true.
        num_values,   // literal VALUES keyed by GlobalId — no type of their own
        numeric_env,  // GlobalId op / dispatch tables — types live in global_env
        bytes_env,    // GlobalId type / op ids — types live in global_env
        foreign_env,  // FFI postulate GlobalIds — types live in global_env
        effect_rows,  // effect-row algebra — carries no Term
        prelude_env,  // GlobalIds for prelude decls — types live in global_env
        module_state, // surface-name -> canonical-name aliases into global_env
    } = env;
    let _ = (
        num_values,
        numeric_env,
        bytes_env,
        foreign_env,
        effect_rows,
        prelude_env,
        module_state,
    );

    let mut producers = Vec::new();

    // NAMESPACE — public globals: kernel declarations AND constructors, indexed
    // by source-visible name. `type_of_global` classifies each; an id that is
    // neither a declaration nor a constructor is a LOUD failure (AC-5).
    for (name, id) in globals.iter() {
        producers.push(Producer {
            namespace: "globals",
            name: name.clone(),
            ty: type_of_global(global_env, name, *id),
        });
    }

    // NAMESPACE — class field types, source-reachable by projection `d.field`.
    // These never enter `globals` (the S1-family-B blind spot the head-only
    // oracle inherited).
    for (class_name, info) in class_env.classes.iter() {
        for (field_name, field_ty) in info.field_names.iter().zip(info.field_types.iter()) {
            producers.push(Producer {
                namespace: "class_field",
                name: format!("{class_name}.{field_name}"),
                ty: field_ty.clone(),
            });
        }
    }

    producers
}

/// The declared type of a `globals` member. A member is a kernel declaration
/// (transparent / opaque / primitive / inductive former) or a constructor of an
/// inductive; anything else is unclassifiable and fails loudly (AC-5).
pub fn type_of_global(env: &GlobalEnv, name: &str, id: GlobalId) -> Term {
    match env.lookup(id) {
        Some(decl) => match decl {
            Decl::Transparent { ty, .. }
            | Decl::Opaque { ty, .. }
            | Decl::Primitive { ty, .. } => ty.clone(),
            Decl::Inductive(inductive) => inductive.former_type.clone(),
        },
        None => match env.constructor(id) {
            Some((inductive, index)) => inductive.constructors[index].type_.clone(),
            None => panic!(
                "SEAL-2 producer enumeration: public global `{name}` is neither a \
                 declaration nor a constructor"
            ),
        },
    }
}

// ---------------------------------------------------------------------------
// Position closure (AC-3) — carrier detected ANYWHERE in a result type.
// ---------------------------------------------------------------------------

/// Does `carrier` occur anywhere in the result type of `ty`, modulo defeq?
///
/// Two phases:
///  1. **Π-telescope strip** — keeps SPAN-SEAL's WHNF discipline verbatim
///     (reduce before the Pi decision and after every codomain step, carrying a
///     `Context`), so a result type that is itself a transparent alias for a
///     function type is unfolded before the Pi decision. This is exactly the
///     landed `result_head` walk and, like it, only ever WHNFs the head.
///  2. **nested descent** — [`occurs`] walks *every* sub-position of the result
///     type (the AC-3 extension), detecting the carrier former modulo defeq by
///     delta-unfolding transparent aliases. It does **not** re-run full WHNF
///     (which would β-expand transparent-def applications explosively across the
///     hundreds of law/proposition types in `globals`); instead it unfolds each
///     transparent const's body at most once (a `scanned` set), which is
///     sufficient because an occurrence of the carrier *former* in a const's
///     body is context-independent. Bounded, and it still closes the
///     `Option BufferSpanAlias` blind spot the conservative `mentions()` fixture
///     leaves open.
pub fn result_type_produces(env: &GlobalEnv, ty: &Term, carrier: GlobalId) -> bool {
    let mut ctx = Context::new();
    let mut result = whnf(env, &ctx, ty);
    while let Term::Pi(domain, codomain) = result {
        ctx.push(*domain);
        result = whnf(env, &ctx, &codomain);
    }
    // A producer yields a carrier *value* only when its result type is a data
    // type (`Type ℓ`-sorted). A proposition (`Ω`-sorted result — a law or proof
    // obligation) yields a PROOF: an occurrence of the carrier former inside a
    // proof is proof-irrelevant and not source-projectable to a value, so such a
    // global is not a producer. (This is what separates a real
    // `… → Option BufferSpanAlias` value producer from a law like
    // `write_all_preserves_exact_prefix` that merely mentions the carrier in its
    // statement.) If the sort cannot be inferred — e.g. an open class-field type
    // carrying a free class parameter — err toward flagging rather than silently
    // dropping a candidate.
    if let Ok(sort) = infer(env, &ctx, &result) {
        if matches!(whnf(env, &ctx, &sort), Term::Omega(_)) {
            return false;
        }
    }
    let mut scanned = std::collections::HashSet::new();
    occurs(env, &result, carrier, &mut scanned)
}

/// Structural occurrence of the `carrier` former in `t`, modulo alias defeq.
///
/// The `match` is **exhaustive** (no `_`): a new `Term` variant is a build
/// break, and every compound former is fully descended — nothing is skipped or
/// defaulted (AC-3, AC-5, type-former axis). Transparent consts are unfolded
/// (once each, tracked in `scanned`) so an alias like `BufferSpanAlias` is seen;
/// a const already scanned did not contain the carrier former on its first
/// visit, so re-descending it is unnecessary.
fn occurs(
    env: &GlobalEnv,
    t: &Term,
    carrier: GlobalId,
    scanned: &mut std::collections::HashSet<GlobalId>,
) -> bool {
    match t {
        // Carrier leaves — the inductive former is the carrier; a constructor of
        // the carrier is also a hit.
        Term::IndFormer { id, .. } | Term::Constructor { id, .. } => *id == carrier,
        // A const may BE the carrier name, or a transparent alias whose body
        // reaches it — unfold once and descend.
        Term::Const { id, .. } => {
            if *id == carrier {
                return true;
            }
            if let Some((_level_params, body)) = env.transparent_body(*id) {
                if scanned.insert(*id) {
                    return occurs(env, &body, carrier, scanned);
                }
            }
            false
        }
        // Atomic non-carrier leaves — cannot contain the carrier.
        Term::Type(_) | Term::Omega(_) | Term::Var(_) | Term::IntLit(_) => false,
        // Compounds — descend every subterm structurally (no β, no full WHNF).
        Term::Pi(a, b)
        | Term::Lam(a, b)
        | Term::Sigma(a, b)
        | Term::App(a, b)
        | Term::Pair(a, b)
        | Term::Ascript(a, b)
        | Term::Quot(a, b)
        | Term::Absurd(a, b) => {
            occurs(env, a, carrier, scanned) || occurs(env, b, carrier, scanned)
        }
        Term::Let { ty, val, body } => {
            occurs(env, ty, carrier, scanned)
                || occurs(env, val, carrier, scanned)
                || occurs(env, body, carrier, scanned)
        }
        Term::Proj1(a)
        | Term::Proj2(a)
        | Term::Refl(a)
        | Term::QuotClass(a)
        | Term::Trunc(a)
        | Term::TruncProj(a) => occurs(env, a, carrier, scanned),
        Term::Eq(a, b, c) | Term::J(a, b, c) => {
            occurs(env, a, carrier, scanned)
                || occurs(env, b, carrier, scanned)
                || occurs(env, c, carrier, scanned)
        }
        Term::Cast(a, b, c, d) => {
            occurs(env, a, carrier, scanned)
                || occurs(env, b, carrier, scanned)
                || occurs(env, c, carrier, scanned)
                || occurs(env, d, carrier, scanned)
        }
        Term::Elim {
            params,
            motive,
            methods,
            indices,
            scrut,
            ..
        } => {
            params.iter().any(|p| occurs(env, p, carrier, scanned))
                || occurs(env, motive, carrier, scanned)
                || methods.iter().any(|m| occurs(env, m, carrier, scanned))
                || indices.iter().any(|i| occurs(env, i, carrier, scanned))
                || occurs(env, scrut, carrier, scanned)
        }
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => {
            occurs(env, motive, carrier, scanned)
                || occurs(env, method, carrier, scanned)
                || occurs(env, respect, carrier, scanned)
                || occurs(env, scrut, carrier, scanned)
        }
    }
}

// ---------------------------------------------------------------------------
// The closed oracle — carrier-parameterized (AC-1).
// ---------------------------------------------------------------------------

/// The closed producer set for `carrier_name` over `env`: every namespace (via
/// [`enumerate_producer_types`]) crossed with position-closure (via
/// [`result_type_produces`]). One derivation, applied per carrier — adding a
/// third sealed carrier is a one-line call, no walker copy (AC-1).
pub fn closed_producers(env: &ElabEnv, carrier_name: &str) -> BTreeSet<String> {
    let carrier = env.globals[carrier_name];
    enumerate_producer_types(env)
        .into_iter()
        .filter(|p| result_type_produces(&env.env, &p.ty, carrier))
        .map(|p| p.name)
        .collect()
}

// ---------------------------------------------------------------------------
// The PREVIOUS (cd4184b8) oracle — kept ONLY as the AC-6 contrast baseline.
// ---------------------------------------------------------------------------

/// The head-only, `globals`-only oracle, behaviorally identical to SPAN-SEAL's
/// landed `public_buffer_span_producers`. It is **not** the closure — it is the
/// baseline each discriminator measures against ("the head-only oracle derives
/// `{}` here; the closed oracle sees it").
pub fn head_only_producers(env: &ElabEnv, carrier_name: &str) -> BTreeSet<String> {
    let carrier = env.globals[carrier_name];
    env.globals
        .iter()
        .filter_map(|(name, id)| {
            let ty = type_of_global(&env.env, name, *id);
            matches!(head_of(&env.env, &ty), Term::IndFormer { id, .. } if id == carrier)
                .then(|| name.clone())
        })
        .collect()
}

/// SPAN-SEAL's `result_head`, verbatim: walk the Π-telescope, return the head of
/// the result type after WHNF.
fn head_of(env: &GlobalEnv, ty: &Term) -> Term {
    let mut context = Context::new();
    let mut head = whnf(env, &context, ty);
    loop {
        match head {
            Term::Pi(domain, codomain) => {
                context.push(*domain);
                head = whnf(env, &context, &codomain);
            }
            result => return result,
        }
    }
}

/// The adversary's `mentions()` walker (the SEAL-2 starting evidence),
/// **deliberately conservative** and marked in-file as *not a proposed fix*: it
/// recurses structurally but does **not** unfold `Const`s in non-head positions,
/// so `Option BufferSpanAlias` slips past it. Kept as an AC-6 contrast fixture —
/// the closed oracle beats it, which is exactly why adopting it wholesale (the
/// ⛔ in the brief) would have been instance #6 of the same defect.
pub fn conservative_mentions(t: &Term, target: GlobalId) -> bool {
    match t {
        Term::IndFormer { id, .. } | Term::Constructor { id, .. } | Term::Const { id, .. } => {
            *id == target
        }
        Term::Pi(a, b)
        | Term::Lam(a, b)
        | Term::App(a, b)
        | Term::Sigma(a, b)
        | Term::Pair(a, b)
        | Term::Ascript(a, b)
        | Term::Quot(a, b)
        | Term::Absurd(a, b) => conservative_mentions(a, target) || conservative_mentions(b, target),
        Term::Eq(a, b, c) | Term::J(a, b, c) => {
            conservative_mentions(a, target)
                || conservative_mentions(b, target)
                || conservative_mentions(c, target)
        }
        Term::Cast(a, b, c, d) => {
            conservative_mentions(a, target)
                || conservative_mentions(b, target)
                || conservative_mentions(c, target)
                || conservative_mentions(d, target)
        }
        Term::Refl(a) | Term::Trunc(a) | Term::TruncProj(a) | Term::QuotClass(a) => {
            conservative_mentions(a, target)
        }
        _ => false,
    }
}

/// The adversary's `producers(deep=true)`: `globals`-only, head after Π-strip,
/// then the conservative `mentions()` structural recursion. Finds an off-head
/// carrier under an inductive (`Result E BufferSpan`) but misses one behind a
/// transparent alias (`Option BufferSpanAlias`). Contrast fixture only.
pub fn conservative_deep_producers(env: &ElabEnv, carrier_name: &str) -> BTreeSet<String> {
    let carrier = env.globals[carrier_name];
    env.globals
        .iter()
        .filter_map(|(name, id)| {
            let ty = type_of_global(&env.env, name, *id);
            conservative_mentions(&head_of(&env.env, &ty), carrier).then(|| name.clone())
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Source-root closure (AC-4) — range over EVERY catalog package.
// ---------------------------------------------------------------------------

/// The controlled top-level catalog Sections (mirrors `catalog_taxonomy.rs`).
/// Any package root outside this set is unclassifiable.
pub const ALLOWED_SECTIONS: [&str; 7] = [
    "Core",
    "Data",
    "Algorithm",
    "Capability",
    "Protocol",
    "Application",
    "Tooling",
];

/// Classify a catalog package's top-level Section, failing loudly (AC-5,
/// source-root axis) on a root outside the controlled allowlist rather than
/// silently covering fewer roots than claimed.
pub fn classify_section(section: &str) -> &'static str {
    ALLOWED_SECTIONS
        .into_iter()
        .find(|allowed| *allowed == section)
        .unwrap_or_else(|| {
            panic!("SEAL-2 catalog glob: package root `{section}` is not an allowed Section")
        })
}

/// The `catalog/packages` directory, relative to this crate.
pub fn catalog_packages_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("catalog/packages")
}

/// Every `*.ken.md` package file under `catalog/packages`, each paired with its
/// classified top-level Section (unclassifiable roots fail loudly). Sorted for
/// determinism.
pub fn catalog_package_files() -> Vec<(String, PathBuf)> {
    let root = catalog_packages_dir();
    let mut out = Vec::new();
    collect_ken_md(&root, &root, &mut out);
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

fn collect_ken_md(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    for entry in fs::read_dir(dir).expect("read catalog dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let file_type = entry.file_type().expect("file type");
        if file_type.is_dir() {
            collect_ken_md(root, &path, out);
        } else if path.to_string_lossy().ends_with(".ken.md") {
            // The top-level Section is the first path component under `root`.
            let rel = path.strip_prefix(root).expect("under catalog root");
            let section = rel
                .components()
                .next()
                .expect("package has a Section root")
                .as_os_str()
                .to_string_lossy()
                .into_owned();
            let section = classify_section(&section);
            out.push((section.to_string(), path));
        }
    }
}

/// The sealed carrier type names, for the source-root confinement guard.
pub const CARRIER_NAMES: [&str; 2] = ["BufferSpan", "TransferCount"];

/// A package's path relative to `catalog/packages`, `/`-joined.
pub fn package_rel(path: &Path) -> String {
    path.strip_prefix(catalog_packages_dir())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Read a catalog package's raw `.ken.md` source.
pub fn read_package_source(path: &Path) -> String {
    fs::read_to_string(path).expect("read catalog package source")
}

// ---------------------------------------------------------------------------
// Fail-closed semantic confinement certificate (AC-4) — the source-root closure.
// ---------------------------------------------------------------------------
//
// A single flat `ElabEnv` over all catalog packages is not achievable (the
// catalog is layered with boundary headers + cross-package name reuse; an
// arbitrary-order co-load collides — Transport duplicate `sym`, LawfulClasses
// OverlappingInstances DecEq Bool). So the source-root closure is established
// semantically, not by loading every package:
//
//   1. Parse each globbed package's CHECKED Ken fences through the production
//      literate extractor + parser (prose and comments are blanked by the
//      extractor, so they are not evidence). Parsing is purely syntactic — it
//      needs none of the package's dependencies loaded.
//   2. Compute the transitive set of type NAMES that reach a sealed carrier,
//      seeded by the carrier declarations and closed through transparent type
//      aliases and carrier-reaching class/data names (to a fixpoint, across all
//      roots).
//   3. A root is NON-EXEMPT when any declaration references a carrier-closure
//      name in any type OR body position (`walk_decl` scans both — the bottom
//      eliminator makes the body load-bearing), OR uses a form the certificate
//      cannot resolve (`module`/`import`/`export` → `requires_semantic_resolution`,
//      FAIL CLOSED — no surface-string resolver graph). Every non-exempt root is
//      named (root + declaration/form + carrier/reason) and must be explicitly
//      enumerated, loaded through its known dependency environment, and run
//      through `closed_producers` — never silently accepted. The soundness of
//      exempting the rest is the conjunction on `RootFacts`.

/// A parsed catalog root: its `/`-joined rel-path and the surface declarations
/// of its checked fences.
pub struct ParsedRoot {
    pub rel: String,
    pub decls: Vec<SurfaceDecl>,
}

/// Parse every globbed catalog package's checked Ken fences (syntactic only,
/// no dependency loading). The glob classifies each Section and fails loudly on
/// an unclassifiable root (via [`catalog_package_files`]).
pub fn parse_catalog_roots() -> Vec<ParsedRoot> {
    catalog_package_files()
        .into_iter()
        .map(|(_section, path)| {
            let rel = package_rel(&path);
            let src = read_package_source(&path);
            let extracted =
                extract_ken_md(&src).unwrap_or_else(|e| panic!("extract fences of {rel}: {e:?}"));
            validate_ken_md_fences(&extracted)
                .unwrap_or_else(|e| panic!("validate fences of {rel}: {e:?}"));
            let decls = parser::parse_decls(&extracted.source)
                .unwrap_or_else(|e| panic!("parse checked fences of {rel}: {e:?}"));
            ParsedRoot { rel, decls }
        })
        .collect()
}

/// Every type-NAME (`TCon`) referenced in a surface type. Recurses every
/// sub-type; a refinement predicate is walked through [`type_names_in_expr`].
/// The `match` is exhaustive (no `_`): a new `Type` variant is a build break.
pub fn type_names_in_type(ty: &SurfaceType, out: &mut BTreeSet<String>) {
    match ty {
        // A bare identifier reference in a type position parses as `TCon` or,
        // depending on syntactic context (e.g. a type-alias RHS), as `TVar` —
        // the con/var distinction is resolved later. Collect BOTH names: a
        // genuine type parameter is lowercase and can never equal an (uppercase)
        // carrier or alias name, so this is sound and catches `def X = Carrier`,
        // whose RHS the parser emits as `TVar("Carrier")`.
        SurfaceType::TCon(name, _) | SurfaceType::TVar(name, _) => {
            out.insert(name.clone());
        }
        SurfaceType::TUniv(_, _) => {}
        SurfaceType::TPi(_binder, dom, cod, _) => {
            type_names_in_type(dom, out);
            type_names_in_type(cod, out);
        }
        SurfaceType::TArr(a, b, _) => {
            type_names_in_type(a, out);
            type_names_in_type(b, out);
        }
        SurfaceType::TEffectArr(a, _row, b, _) => {
            type_names_in_type(a, out);
            type_names_in_type(b, out);
        }
        SurfaceType::TApp(head, arg, _) => {
            type_names_in_type(head, out);
            type_names_in_type(arg, out);
        }
        SurfaceType::TRefine(_binder, base, pred, _) => {
            type_names_in_type(base, out);
            type_names_in_expr(pred, out);
        }
    }
}

/// Every carrier-relevant NAME referenced by a surface expression — the general
/// expression/body name walk. Used both for expressions that sit in a type
/// position (a refinement predicate, an explicit-data constructor arg/result)
/// AND for declaration BODIES (`ViewDecl`/`LetDecl`/proof/instance-field values),
/// where the body axis is load-bearing: an ex-falso body names the carrier as
/// the eliminated motive with no carrier in any type position. It collects every
/// `ECon`/`EVar` identifier (a `ConId`/name used as a term is a potential
/// type-name reference); a lowercase value name simply never matches an
/// uppercase carrier/alias, so this is sound. The `match` is exhaustive (no
/// `_`): a new `Expr` variant is a build break.
pub fn type_names_in_expr(e: &Expr, out: &mut BTreeSet<String>) {
    match e {
        // As in `type_names_in_type`, an identifier used as a term-level type
        // reference may be `ECon` or `EVar`; collect both (a lowercase term var
        // can never equal an uppercase carrier/alias name).
        Expr::ECon(name, _) | Expr::EVar(name, _) => {
            out.insert(name.clone());
        }
        Expr::EUniv(_, _)
        | Expr::ENumLit(_, _)
        | Expr::EStr(_, _)
        | Expr::EAttachedProofRef { .. } => {}
        Expr::EApp(a, b, _) | Expr::EBinOp(_, a, b, _) | Expr::EArrow(a, b, _) => {
            type_names_in_expr(a, out);
            type_names_in_expr(b, out);
        }
        Expr::ELam(_, body, _) | Expr::EOld(body, _) | Expr::EProj(body, _, _) => {
            type_names_in_expr(body, out);
        }
        Expr::EAsc(inner, ty, _) => {
            type_names_in_expr(inner, out);
            type_names_in_type(ty, out);
        }
        Expr::EPi(_binder, dom, cod, _) => {
            type_names_in_type(dom, out);
            type_names_in_expr(cod, out);
        }
        Expr::ELet(bindings, body, _) => {
            for binding in bindings {
                if let Some(ty) = &binding.annotation {
                    type_names_in_type(ty, out);
                }
                type_names_in_expr(&binding.value, out);
            }
            type_names_in_expr(body, out);
        }
        Expr::EMatch { scrut, arms, .. } => {
            type_names_in_expr(scrut, out);
            for arm in arms {
                type_names_in_expr(&arm.body, out);
            }
        }
    }
}

/// Facts extracted from one catalog root's surface declarations, sufficient to
/// decide — soundly and fail-closed — whether the root could host a
/// source-reachable carrier producer.
///
/// # Why a conservative scan is sound (the conjunction)
///
/// The certificate does NOT identify producer *positions* (that would require
/// replicating the elaborator's inference / instance / import resolution — where
/// a syntactic approximation leaks). It answers only the coarser question it can
/// decide soundly: **does a declaration reference any carrier-closure name in any
/// type OR body position, or use a form the certificate cannot resolve?** Its
/// soundness is a conjunction, not the false claim "every producer names the
/// carrier in a type position":
///
///   1. AC-2/AC-3 first prove the source-visible *starting* environment
///      (`prelude + Buffer + IO`) contains no carrier producer — every namespace
///      and class field — i.e. `closed_producers` derives `{}` there.
///   2. The carrier constructors are prelude-private (not source-visible). Ken
///      HAS a bottom eliminator — `absurd_empty (C : Type) (e : Empty) : C =
///      match e {}` (`Core/Logic/EmptyDec.ken.md`) inhabits *any* type from a
///      proof of `Empty`, so `fn leak (e : Empty) = absurd_empty BufferSpan e`
///      is a carrier producer with no carrier in any *type* position. But
///      exercising that path to yield a carrier requires naming the carrier (or
///      an alias) as the eliminated motive, which appears as a carrier-closure
///      reference in the declaration's TYPE or BODY. There is no other synthesis
///      path (no cast/forge that manufactures a value of a type it does not name).
///   3. Therefore the *first* producer a catalog root introduces must reference a
///      carrier-closure NAME in some type OR body position — parameter, result,
///      field, constructor argument, alias RHS, class/instance head, a bottom
///      motive in a body, or a closure member reached through those. [`walk_decl`]
///      scans every such position, including declaration bodies. A term-only
///      forwarder (`fn leak = makeSpan`) can exist only *after* an earlier
///      producer, which clause 1 or an earlier admitted root already rejects.
///
/// That conjunction — not body inference — is what licenses an unannotated body
/// with no carrier-closure reference to stay exempt. A root that DOES reference a
/// closure name (in any type or body position), or uses an unresolvable form, is
/// non-exempt: it must be elaborated in its known dependency environment and run
/// through `closed_producers`, or named as a live finding.
pub struct RootFacts {
    pub rel: String,
    /// `(declaration, carrier-relevant names it references)` for every named
    /// declaration — collected from every type position AND the declaration body
    /// (so body identifiers, e.g. an ex-falso motive, enter this set too).
    pub decl_refs: Vec<(String, BTreeSet<String>)>,
    /// `(name-to-add, trigger-refs)`: the name joins the carrier closure when any
    /// trigger reference is already in it — transparent aliases, and class / data
    /// names whose field / constructor-argument types reach the closure (so USING
    /// such a class or data type is itself a carrier reference).
    pub propagators: Vec<(String, BTreeSet<String>)>,
    /// A binding form the certificate cannot resolve without the production
    /// resolver — any `module`, `import`, or `export`. Such a root is
    /// conservatively non-exempt regardless of its references — FAIL CLOSED on
    /// the undecidable form rather than recreate resolver identity with a surface
    /// string graph. (No catalog package uses these today; this guards the form.)
    pub requires_semantic_resolution: Option<String>,
}

/// Extract [`RootFacts`] from a parsed root.
pub fn root_facts(root: &ParsedRoot) -> RootFacts {
    let mut facts = RootFacts {
        rel: root.rel.clone(),
        decl_refs: Vec::new(),
        propagators: Vec::new(),
        requires_semantic_resolution: None,
    };
    for decl in &root.decls {
        walk_decl(decl, &mut facts);
    }
    facts
}

/// Walk one surface declaration, accumulating references / propagators into
/// `facts`. The `match` is EXHAUSTIVE (no `_`): a new declaration/binding form is
/// a build break, forcing it to be classified (AC-5 at the declaration level).
/// Every type-bearing position of every form is collected; a form the walk
/// cannot resolve sets `requires_semantic_resolution` (fail closed), never a
/// silent skip.
fn walk_decl(decl: &SurfaceDecl, facts: &mut RootFacts) {
    let mut refs = BTreeSet::new();
    match decl {
        SurfaceDecl::ViewDecl {
            name,
            params,
            ret_ty,
            requires,
            ensures,
            constraints,
            body,
            ..
        } => {
            for b in params {
                type_names_in_type(&b.ty, &mut refs);
            }
            if let Some(ty) = ret_ty {
                type_names_in_type(ty, &mut refs);
            }
            for c in constraints {
                type_names_in_type(&c.head_type, &mut refs);
            }
            // Scan the BODY (and contracts) too. The result annotation may be
            // omitted and inferred — and Ken's bottom eliminator lets an
            // ex-falso body inhabit a carrier from a proof of `Empty`
            // (`fn leak (e : Empty) = absurd_empty BufferSpan e`): the carrier is
            // named as the eliminated motive in the body, in no type position.
            type_names_in_expr(body, &mut refs);
            for e in requires.iter().chain(ensures) {
                type_names_in_expr(e, &mut refs);
            }
            facts.decl_refs.push((name.clone(), refs));
        }
        SurfaceDecl::LetDecl { name, ty, val, .. } => {
            if let Some(ty) = ty {
                type_names_in_type(ty, &mut refs);
            }
            // Scan the value expression too — same inferred-result / ex-falso
            // reasoning as `ViewDecl`.
            type_names_in_expr(val, &mut refs);
            facts.decl_refs.push((name.clone(), refs));
        }
        SurfaceDecl::ForeignDecl { name, ty, .. } => {
            type_names_in_type(ty, &mut refs);
            facts.decl_refs.push((name.clone(), refs));
        }
        // A bare postulate mints a value of its declared type out of nothing —
        // `axiom x : BufferSpan` is a forgery, so its type position counts.
        SurfaceDecl::AxiomDecl { name, theorem, .. } => {
            type_names_in_type(theorem, &mut refs);
            facts.decl_refs.push((name.clone(), refs));
        }
        // Proof / proposition forms are not value producers, but a type position
        // naming a carrier still makes the root non-exempt (conservative: the root
        // is elaborated and precisely checked by closed_producers).
        SurfaceDecl::ProveDecl { name, prop, .. } => {
            type_names_in_expr(prop, &mut refs);
            facts.decl_refs.push((name.clone(), refs));
        }
        SurfaceDecl::PropDecl {
            name,
            params,
            ret_ty,
            intros,
            ..
        } => {
            for b in params {
                type_names_in_type(&b.ty, &mut refs);
            }
            type_names_in_type(ret_ty, &mut refs);
            for i in intros {
                type_names_in_type(&i.ty, &mut refs);
            }
            facts.decl_refs.push((name.clone(), refs));
        }
        SurfaceDecl::LemmaDecl {
            name,
            params,
            theorem,
            body,
            ..
        } => {
            for b in params {
                type_names_in_type(&b.ty, &mut refs);
            }
            type_names_in_type(theorem, &mut refs);
            type_names_in_expr(body, &mut refs);
            facts.decl_refs.push((name.clone(), refs));
        }
        SurfaceDecl::AttachedProofDecl {
            proof_name,
            params,
            theorem,
            body,
            ..
        } => {
            for b in params {
                type_names_in_type(&b.ty, &mut refs);
            }
            type_names_in_type(theorem, &mut refs);
            type_names_in_expr(body, &mut refs);
            facts.decl_refs.push((proof_name.clone(), refs));
        }
        SurfaceDecl::LawDecl { name, fields, .. } => {
            for (_, e) in fields {
                type_names_in_expr(e, &mut refs);
            }
            facts.decl_refs.push((name.clone(), refs));
        }
        SurfaceDecl::DataDecl { name, ctors, .. } => {
            let mut ctor_refs = BTreeSet::new();
            for c in ctors {
                let mut cr = BTreeSet::new();
                for a in &c.args {
                    type_names_in_type(a, &mut cr);
                }
                ctor_refs.extend(cr.iter().cloned());
                facts.decl_refs.push((c.name.clone(), cr));
            }
            // destructuring the data type yields its ctor args: the data name
            // reaches the closure when any ctor arg does.
            facts.propagators.push((name.clone(), ctor_refs));
        }
        SurfaceDecl::ExplicitDataDecl {
            name,
            params,
            family,
            ctors,
            ..
        } => {
            let mut all = BTreeSet::new();
            for b in params {
                type_names_in_type(&b.ty, &mut all);
            }
            type_names_in_type(family, &mut all);
            for c in ctors {
                let mut cr = BTreeSet::new();
                let cname = match c {
                    ExplicitDataCtor::Simple(s) => {
                        for a in &s.args {
                            type_names_in_type(a, &mut cr);
                        }
                        s.name.clone()
                    }
                    ExplicitDataCtor::Signature {
                        name: cn,
                        signature,
                        ..
                    } => {
                        for arg in &signature.args {
                            match arg {
                                ConstructorSignatureArg::Explicit(b)
                                | ConstructorSignatureArg::Implicit(b) => {
                                    type_names_in_type(&b.ty, &mut cr)
                                }
                                ConstructorSignatureArg::Anonymous(e) => {
                                    type_names_in_expr(e, &mut cr)
                                }
                            }
                        }
                        type_names_in_expr(&signature.result, &mut cr);
                        cn.clone()
                    }
                };
                all.extend(cr.iter().cloned());
                facts.decl_refs.push((cname, cr));
            }
            facts.propagators.push((name.clone(), all));
        }
        SurfaceDecl::TypeAlias { name, ty, .. } => {
            type_names_in_type(ty, &mut refs);
            facts.decl_refs.push((name.clone(), refs.clone()));
            facts.propagators.push((name.clone(), refs));
        }
        SurfaceDecl::ClassDecl {
            name,
            param_kind,
            fields,
            ..
        } => {
            let mut field_refs = BTreeSet::new();
            if let Some(k) = param_kind {
                type_names_in_type(k, &mut field_refs);
            }
            for f in fields {
                let mut fr = BTreeSet::new();
                type_names_in_type(&f.ty, &mut fr);
                field_refs.extend(fr.iter().cloned());
                facts
                    .decl_refs
                    .push((format!("{name}.{}", f.name), fr));
            }
            // projecting a field yields the field type: using the class reaches
            // the closure when any field does.
            facts.propagators.push((name.clone(), field_refs));
        }
        SurfaceDecl::InstanceDecl {
            class_name,
            head_type,
            constraints,
            fields,
            ..
        } => {
            type_names_in_type(head_type, &mut refs);
            for c in constraints {
                type_names_in_type(&c.head_type, &mut refs);
            }
            // Scan field value bodies too (ex-falso defense-in-depth).
            for (_, e) in fields {
                type_names_in_expr(e, &mut refs);
            }
            facts
                .decl_refs
                .push((format!("instance {class_name}"), refs));
        }
        SurfaceDecl::DeriveDecl {
            class_name,
            data_name,
            ..
        } => {
            refs.insert(class_name.clone());
            refs.insert(data_name.clone());
            facts
                .decl_refs
                .push((format!("derive {class_name} {data_name}"), refs));
        }
        SurfaceDecl::TemporalDecl { name, .. } => {
            facts.decl_refs.push((name.clone(), refs));
        }
        SurfaceDecl::BoundaryDecl { .. } => {}
        // `module` / `import` / `export` carry cross-module identity the
        // certificate does not resolve — fail closed rather than recreate the
        // resolver with a surface string graph. (Zero catalog packages use these.)
        SurfaceDecl::ModuleDecl { name, .. } => {
            facts.requires_semantic_resolution = Some(format!("module `{name}`"));
        }
        SurfaceDecl::ImportDecl { module, .. } => {
            facts.requires_semantic_resolution = Some(format!("import of `{module}`"));
        }
        SurfaceDecl::ExportDecl { .. } => {
            facts.requires_semantic_resolution = Some("re-export".to_string());
        }
        SurfaceDecl::Pub(inner) => walk_decl(inner, facts),
    }
}

/// The transitive closure of type NAMES that reach a sealed carrier: seeded by
/// the prelude carriers and closed to a fixpoint through transparent aliases and
/// carrier-reaching class/data names across ALL roots.
pub fn carrier_closure(facts: &[RootFacts]) -> BTreeSet<String> {
    let mut closure: BTreeSet<String> = CARRIER_NAMES.iter().map(|s| s.to_string()).collect();
    loop {
        let before = closure.len();
        for f in facts {
            for (name, trigger) in &f.propagators {
                if trigger.iter().any(|n| closure.contains(n)) {
                    closure.insert(name.clone());
                }
            }
        }
        if closure.len() == before {
            break;
        }
    }
    closure
}

/// Every root that reaches the carrier closure, with the triggering
/// `(root, declaration/form, carrier/reason)` — an unresolvable form reaches
/// unconditionally (fail closed). A root may appear more than once.
pub fn reaching_roots(
    facts: &[RootFacts],
    closure: &BTreeSet<String>,
) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for f in facts {
        if let Some(reason) = &f.requires_semantic_resolution {
            out.push((
                f.rel.clone(),
                reason.clone(),
                "requires-semantic-resolution".to_string(),
            ));
        }
        for (decl, decl_refs) in &f.decl_refs {
            for name in decl_refs {
                if closure.contains(name) {
                    out.push((f.rel.clone(), decl.clone(), name.clone()));
                }
            }
        }
    }
    out
}

/// The AC-4 admission assertion: every root the certificate finds reaching the
/// carrier closure must be one under elaborated enumeration; any other is a live
/// finding, PANICKING with `root + declaration/form + carrier/reason`. This is
/// the unenumerated-root rejection path, factored out so the self-discriminators
/// can drive it directly.
pub fn assert_confined(facts: &[RootFacts], enumerated: &[&str]) {
    let closure = carrier_closure(facts);
    for (root, decl, reason) in reaching_roots(facts, &closure) {
        assert!(
            enumerated.contains(&root.as_str()),
            "SEAL-2 AC-4: catalog root `{root}` is non-exempt via `{decl}` ({reason}) but \
             is not under elaborated enumeration — add its known dependency environment \
             and run closed_producers there (do not auto-accept)"
        );
    }
}

/// [`RootFacts`] for every globbed catalog package.
pub fn catalog_root_facts() -> Vec<RootFacts> {
    parse_catalog_roots().iter().map(root_facts).collect()
}

/// [`RootFacts`] for a synthetic `.ken.md` source (the self-discriminators).
pub fn synthetic_facts(rel: &str, src: &str) -> RootFacts {
    root_facts(&parse_synthetic_root(rel, src))
}

/// Parse a synthetic `.ken.md`-style source into a single [`ParsedRoot`], for
/// the certificate's own self-discriminators (alias-bypass, prose control).
pub fn parse_synthetic_root(rel: &str, src: &str) -> ParsedRoot {
    let extracted = extract_ken_md(src).expect("extract synthetic fences");
    validate_ken_md_fences(&extracted).expect("validate synthetic fences");
    let decls = parser::parse_decls(&extracted.source).expect("parse synthetic fences");
    ParsedRoot {
        rel: rel.to_string(),
        decls,
    }
}
