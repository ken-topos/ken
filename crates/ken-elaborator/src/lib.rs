//! `ken-elaborator` — V0/V1 surface elaborator (`docs/program/wp/V0-elaborator.md`,
//! `spec/20-verification/21-spec-syntax.md`).
//!
//! Pipeline: `lex → parse → resolve → elaborate → kernel-check`.
//!
//! V1 extensions: requires/ensures, obligation holes, honesty guard.
//! Clean-room: built from `/spec` and `/conformance` only.

mod ast;
pub mod diagnostics;
pub mod elab;
pub mod effects;
pub mod export;
pub mod ifc;
pub mod protocol;
pub mod error;
pub mod extract;
mod lexer;
pub mod parser;
pub mod prover;
pub mod resolve;

use std::collections::HashMap;

use ken_kernel::{
    check as kernel_check, declare_postulate, Context, GlobalEnv, GlobalId, Level, Term,
};

pub use elab::{elaborate_rdecl, elaborate_rexpr, ElabResult, Obligation, ObligationKind};
pub use error::{ElabError, Span};
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
pub use protocol::{
    hole_id_string, obligation_id_string, project_obligation_status, project_wire_verdict,
    rollup_doc_status, round_trip, serialize_action, serialize_countermodel, serialize_decomposition,
    serialize_diagnostic, serialize_document, serialize_hole, serialize_obligation,
    serialize_slice, trusted_base_entry, validate_document, DocStatus, ObligationStatus,
    WireVerdict,
};
pub use resolve::{RDecl, RDeclKind, RExpr, RType};

/// The surface-level elaboration environment.
pub struct ElabEnv {
    pub env: GlobalEnv,
    pub globals: HashMap<String, GlobalId>,
}

impl ElabEnv {
    pub fn empty() -> Self {
        Self {
            env: GlobalEnv::new(),
            globals: HashMap::new(),
        }
    }

    /// Create an environment with `Nat : Type 0` and `Bool : Type 0`.
    pub fn new() -> Result<Self, ElabError> {
        let mut this = Self::empty();
        let nat_id = declare_postulate(&mut this.env, vec![], Term::ty(Level::Zero))
            .map_err(|e| ElabError::Internal(format!("Nat predeclaration failed: {}", e)))?;
        this.globals.insert("Nat".into(), nat_id);
        let bool_id = declare_postulate(&mut this.env, vec![], Term::ty(Level::Zero))
            .map_err(|e| ElabError::Internal(format!("Bool predeclaration failed: {}", e)))?;
        this.globals.insert("Bool".into(), bool_id);
        Ok(this)
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

    /// Elaborate a single V0/V1 declaration from source.
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
        let rdecl = resolve::resolve_decl(&decls[0])?;
        elaborate_rdecl(&mut self.env, &mut self.globals, &rdecl)
    }

    /// Elaborate a V1 declaration, returning obligations alongside the id.
    pub fn elaborate_decl_v1(&mut self, src: &str) -> Result<ElabResult, ElabError> {
        let decls = parser::parse_decls(src)?;
        if decls.len() != 1 {
            return Err(ElabError::ParseError {
                msg: format!("expected exactly one declaration, found {}", decls.len()),
                span: Span::zero(),
            });
        }
        let rdecl = resolve::resolve_decl(&decls[0])?;
        elab::elaborate_rdecl_v1(&mut self.env, &mut self.globals, &rdecl)
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
        elaborate_rexpr(&mut self.env, &self.globals, &rexpr)
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
