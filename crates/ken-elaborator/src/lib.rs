//! `ken-elaborator` — V0/V1/L1 surface elaborator (`docs/program/wp/V0-elaborator.md`,
//! `spec/20-verification/21-spec-syntax.md`, `spec/30-surface/35-numbers.md`).
//!
//! Pipeline: `lex → parse → resolve → elaborate → kernel-check`.
//!
//! V1 extensions: requires/ensures, obligation holes, honesty guard.
//! L1 extensions: numeric tower, literal defaulting, overflow obligations.
//! Clean-room: built from `/spec` and `/conformance` only.

mod ast;
pub mod bytes;
pub mod checked_core;
pub mod classes;
pub mod capabilities;
pub mod compiler_driver;
pub mod conversions;
pub mod data;
pub mod decimal_char;
pub mod diagnostics;
pub mod elab;
pub mod effects;
pub mod erasure;
pub mod export;
pub mod format;
pub mod foreign;
pub mod ifc;
pub mod literate;
pub mod trace;
pub mod protocol;
pub mod error;
pub mod extract;
pub mod lexer;
pub mod modules;
pub mod numbers;
pub mod parser;
pub mod prelude;
pub mod prover;
pub mod resolve;
pub mod temporal;

use std::collections::HashMap;
use std::path::PathBuf;

use ken_kernel::{
    check as kernel_check, declare_postulate, Context, GlobalEnv, GlobalId, Term,
};

pub use elab::{elaborate_rdecl, elaborate_rexpr, ElabResult, Obligation, ObligationKind};
pub use error::{ElabError, Span};
pub use ast::{
    BinOp, ConstructorSignature, ConstructorSignatureArg, Decl, ExplicitDataCtor, Expr,
    ImportKind, Type,
};
pub use extract::{
    v2_extract, ExtractionResult, ObligationId, ObligationTriple, ProvKind, Provenance,
};
pub use diagnostics::{
    project_all, project_diagnostic, Diagnostic, DiagnosticTag, FailureWitness, FormRef, HoleId,
    KripkeCountermodel, Region, SuggestedAction, ThirdValue, TypedHole, WorldId, tv_and, tv_not,
    tv_or, tv_strict,
};
pub use prover::{
    attempt_obligation, attempt_with_cert, classify, Countermodel, ProverResult, Route, Verdict,
};
pub use export::{
    emit_export, serialize_export, BehavioralExport, ExportError, GEntry, PEntry, PStatus, QEntry,
    TEntry,
};
pub use temporal::{
    closed, elaborate_temporal_expr, temporal_hoas_inductive_spec, temporal_inductive_spec,
    Pred, Temporal, TemporalExpr, TemporalObligation, Var,
};
pub use trace::{
    emit_trace_contract, serialize_trace_contract,
    AssertionPoint, MonitorProjection, TraceContract, TraceEvent,
};
pub use protocol::{
    hole_id_string, obligation_id_string, project_obligation_status, project_wire_verdict,
    rollup_doc_status, round_trip, serialize_action, serialize_countermodel, serialize_decomposition,
    serialize_diagnostic, serialize_document, serialize_hole, serialize_obligation,
    serialize_slice, trusted_base_entry, validate_document, DocStatus, ObligationStatus,
    WireVerdict,
};
pub use resolve::{RDecl, RDeclKind, RExpr, RType};
pub use numbers::{NumericEnv, NumericLitVal};
pub use bytes::BytesEnv;
pub use prelude::PreludeEnv;
pub use foreign::{
    trusted_base_delta, FfiRuntimeCheck, ForeignBinding, ForeignEnv, MarshalKind, MarshalSig,
};
pub use literate::{extract_ken_md, validate_ken_md_fences, KenMdExtraction};
pub use classes::{ClassEnv, ClassInfo, ClassKind, InstanceInfo};

/// The surface-level elaboration environment.
pub struct ElabEnv {
    pub env: GlobalEnv,
    pub globals: HashMap<String, GlobalId>,
    /// Numeric literal values keyed by their opaque-postulate GlobalId.
    /// Accumulated during elaboration; copied to `EvalStore.num_values` for eval.
    pub num_values: HashMap<GlobalId, NumericLitVal>,
    /// The numeric tower (registered op ids, dispatch tables).
    pub numeric_env: NumericEnv,
    /// The Bytes layer (L6): type ids, I/O effect row registry (`38 §1`, `41`).
    pub bytes_env: BytesEnv,
    /// The foreign FFI layer (L7): binding registry (`38 §2–§4`).
    pub foreign_env: ForeignEnv,
    /// Surface effect rows for already-elaborated definitions. SURF-1 D2 uses
    /// this to release a callee's declared row at a resolved call site.
    pub effect_rows: HashMap<String, effects::RowType>,
    /// The L3 prelude: collection inductives + Ω constants (`37`).
    pub prelude_env: PreludeEnv,
    /// The Lc typeclass environment: class/instance registry + structural
    /// postulates (`RecordNil`, `record_nil_val`). Initialized in `empty()`.
    pub class_env: ClassEnv,
    /// Module/import/visibility bookkeeping (`33 §3-4`, ES3-build) —
    /// persists the file-level (root) import scope and every elaborated
    /// module's `pub` export table across separate `elaborate_*` calls.
    /// Purely a surface-layer concern: never touches `env`/`Σ`.
    pub module_state: modules::ModuleState,
}

impl ElabEnv {
    pub fn empty() -> Result<Self, ElabError> {
        let mut env = GlobalEnv::new();
        let mut globals = HashMap::new();
        // `Bool` is pre-registered here (real `data Bool = True | False`, ES2 —
        // demotes the former opaque `declare_postulate` so `Bool` is
        // matchable data; `reg_ty!("Bool")` in `register_numeric_env` reuses
        // this GlobalId) so downstream code using `ElabEnv::empty` gets a
        // consistent GlobalId. Declared via the raw inductive machinery
        // (`data.rs::elab_data_decl`, not `elaborate_decl`) since the full
        // `ElabEnv` doesn't exist yet at this point in construction.
        let true_ctor = resolve::RCtorDecl {
            name: "True".into(),
            args: vec![],
            field_labels: None,
            span: Span::zero(),
        };
        let false_ctor = resolve::RCtorDecl {
            name: "False".into(),
            args: vec![],
            field_labels: None,
            span: Span::zero(),
        };
        data::elab_data_decl(
            &mut env,
            &mut globals,
            "Bool",
            &[],
            &[true_ctor, false_ctor],
            &Span::zero(),
        )?;
        let numeric_env = numbers::register_numeric_env(&mut env, &mut globals)
            .map_err(|e| ElabError::Internal(format!("numeric tower init failed: {}", e)))?;
        let bytes_env = bytes::register_bytes_env(&mut env, &mut globals)
            .map_err(|e| ElabError::Internal(format!("bytes layer init failed: {}", e)))?;
        let effect_rows = bytes_env
            .io_effect_rows
            .iter()
            .map(|(name, row)| (name.clone(), effects::RowType::Concrete(row.clone())))
            .collect();
        let mut elab = Self {
            env,
            globals,
            num_values: HashMap::new(),
            numeric_env,
            bytes_env,
            foreign_env: foreign::ForeignEnv::empty(),
            effect_rows,
            // placeholder; `register_prelude` fills it (and needs `&mut self`).
            prelude_env: prelude::empty_prelude_env(),
            // placeholder; replaced after prelude registration below.
            class_env: classes::ClassEnv::sentinel(),
            module_state: modules::ModuleState::default(),
        };
        // L3 prelude: Peano `Nat` (replaces the placeholder postulate) + the
        // collection inductives + Ω constants (`37`). Registered via the landed
        // `data` / postulate machinery — no new kernel rule.
        elab.prelude_env = prelude::register_prelude(&mut elab)?;
        // Lc typeclass env: pre-declare RecordNil + record_nil_val (`33 §5`).
        elab.class_env =
            elab::init_class_env(&mut elab.env, &mut elab.globals)?;
        Ok(elab)
    }

    /// Create an environment with pre-declared `Nat`, `Bool`, and the full numeric tower.
    pub fn new() -> Result<Self, ElabError> {
        Self::empty()
    }

    /// Declare a postulate `name : ty_term` in the environment.
    ///
    /// Used by tests to pre-declare types, predicates, and propositions needed
    /// for conformance test setup.
    pub fn declare_postulate_raw(
        &mut self,
        name: &str,
        ty: Term,
    ) -> Result<GlobalId, ElabError> {
        let id = declare_postulate(&mut self.env, vec![], ty)
            .map_err(|e| ElabError::Internal(format!("declare_postulate failed: {}", e)))?;
        self.globals.insert(name.to_string(), id);
        Ok(id)
    }

    /// Elaborate a single V0/V1/L1 declaration from source.
    ///
    /// On success the declaration is registered in `self.env`.
    pub fn elaborate_decl(&mut self, src: &str) -> Result<GlobalId, ElabError> {
        let decls = parser::parse_decls(src)?;
        if decls.len() != 1 {
            return Err(ElabError::ParseError {
                msg: format!("expected exactly one declaration, found {}", decls.len()),
                span: Span::zero(),
            });
        }
        let results = modules::expand_and_elaborate(self, &decls)?;
        results
            .into_iter()
            .last()
            .map(|r| r.def_id)
            .ok_or_else(|| ElabError::Internal("declaration produced no definition (bare import?)".into()))
    }

    /// Elaborate a V1/L1 declaration, returning obligations alongside the id.
    pub fn elaborate_decl_v1(&mut self, src: &str) -> Result<ElabResult, ElabError> {
        let decls = parser::parse_decls(src)?;
        if decls.len() != 1 {
            return Err(ElabError::ParseError {
                msg: format!("expected exactly one declaration, found {}", decls.len()),
                span: Span::zero(),
            });
        }
        let results = modules::expand_and_elaborate(self, &decls)?;
        results
            .into_iter()
            .last()
            .ok_or_else(|| ElabError::Internal("declaration produced no definition (bare import?)".into()))
    }

    /// Elaborate zero or more declarations from source, in order.
    ///
    /// Each declaration is elaborated and registered in `self.env` before the
    /// next is processed, so later declarations may refer to earlier ones.
    /// `module`/`import`/`use`/`pub` (`33 §3-4`) are resolved away here —
    /// they contribute zero or more `GlobalId`s (a bare `import` contributes
    /// none; a `module { … }` block contributes one per inner decl) but
    /// never a kernel-visible module concept. Returns the `GlobalId` of
    /// every successfully elaborated declaration.
    pub fn elaborate_file(&mut self, src: &str) -> Result<Vec<GlobalId>, ElabError> {
        let decls = parser::parse_decls(src)?;
        let results = modules::expand_and_elaborate(self, &decls)?;
        Ok(results.into_iter().map(|r| r.def_id).collect())
    }

    /// Elaborate the in-repo compilation unit named by `entry` under the
    /// plural catalog-root input (`33 §3.2`, ADR 0014 MRES-1/2/3a).
    ///
    /// N2 populates exactly one root. The plural slice is the stable API shape;
    /// multi-root precedence remains deliberately deferred.
    pub fn elaborate_module_from_roots(
        &mut self,
        roots: &[PathBuf],
        entry: &str,
    ) -> Result<Vec<GlobalId>, ElabError> {
        modules::elaborate_module_from_roots(self, roots, entry)
    }

    /// Number of successfully loaded cross-file units in this elaboration run.
    /// Exposed so acceptance tests and drivers can verify at-most-once loading.
    pub fn loaded_module_count(&self) -> usize {
        self.module_state.loaded_unit_count()
    }

    /// Elaborate a single `.ken.md` source artifact.
    ///
    /// The Markdown extractor is a read-boundary transform only: it preserves
    /// byte offsets into the original artifact by blanking prose, validates
    /// each compiled fence independently, then reuses the ordinary file
    /// parser/elaborator path on the full blank-preserved buffer.
    ///
    /// After the module elaborates, every `` ```ken reject `` block is
    /// checked to still fail to elaborate (an unexpected success means the
    /// negative example has gone stale) and every `` ```ken example `` block
    /// is checked to elaborate (`catalog-literate-fence-roles` §4.6). Both
    /// checks run **in document order against this same, module-seeded
    /// `self`** — a deliberate V1 simplification: a later checked block may
    /// observe declarations an earlier one introduced, and neither role
    /// forks/rolls back env state.
    pub fn elaborate_ken_md_file(&mut self, src: &str) -> Result<Vec<GlobalId>, ElabError> {
        let extracted = literate::extract_ken_md(src)?;
        literate::validate_ken_md_fences(&extracted)?;
        let decls = parser::parse_decls(&extracted.source)?;
        let results = modules::expand_and_elaborate(self, &decls)?;
        let ids = results.into_iter().map(|r| r.def_id).collect();

        for range in &extracted.reject_ranges {
            if self.elaborate_file(&src[range.clone()]).is_ok() {
                return Err(ElabError::ParseError {
                    msg: "a 'ken reject' block unexpectedly elaborated: the negative example \
                          is stale and no longer demonstrates a rejection"
                        .to_string(),
                    span: Span::new(range.start, range.end),
                });
            }
        }
        for range in &extracted.example_ranges {
            self.elaborate_file(&src[range.clone()]).map_err(|_| {
                ElabError::ParseError {
                    msg: "a 'ken example' block failed to elaborate".to_string(),
                    span: Span::new(range.start, range.end),
                }
            })?;
        }

        Ok(ids)
    }

    /// Try to discharge an obligation hole with a certificate term.
    ///
    /// `cert` is a CLOSED term (no free variables) of type `closed_goal`.
    /// If `check(env, [], cert, closed_goal)` succeeds, the postulate is
    /// upgraded to a transparent definition (`trusted_base()` membership removed).
    /// Returns `true` if the discharge succeeded.
    pub fn discharge_hole(&mut self, obl: &Obligation, cert: Term) -> bool {
        // Kernel-check the certificate against the closed goal
        if kernel_check(&self.env, &Context::new(), &cert, &obl.goal_closed).is_err() {
            return false;
        }
        // Retire the hole postulate by upgrading to transparent
        self.env.upgrade_to_transparent(obl.hole_id, cert)
    }

    /// Returns `true` if `hole_id` is still in `trusted_base()` (status = `unknown`).
    pub fn is_open_hole(&self, hole_id: GlobalId) -> bool {
        self.env.trusted_base().contains(&hole_id)
    }

    /// Elaborate a standalone expression from source.
    pub fn elaborate_expr(&mut self, src: &str) -> Result<(Term, Term), ElabError> {
        let expr = parser::parse_expr(src)?;
        let rexpr = resolve::resolve_expr_standalone(&expr)?;
        elaborate_rexpr(
            &mut self.env,
            &self.globals,
            &mut self.num_values,
            &self.numeric_env,
            &rexpr,
        )
    }

    pub fn kernel_version(&self) -> &'static str {
        ken_kernel::version()
    }
}

impl Default for ElabEnv {
    fn default() -> Self {
        Self::new().expect("base environment predeclaration failed")
    }
}

pub fn kernel_version() -> &'static str {
    ken_kernel::version()
}
