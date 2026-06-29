//! `ken-elaborator` — V0 minimal surface elaborator (`docs/program/wp/V0-elaborator.md`).
//!
//! Pipeline: `lex → parse → resolve → elaborate → kernel-check`.
//!
//! Clean-room: built from `/spec` and `/conformance` only. Never reads `local/refs/`.

mod ast;
pub mod elab;
pub mod error;
mod lexer;
pub mod parser;
pub mod resolve;

use std::collections::HashMap;

use ken_kernel::{declare_postulate, GlobalEnv, GlobalId, Level, Term};

pub use elab::{elaborate_rdecl, elaborate_rexpr};
pub use error::{ElabError, Span};
pub use resolve::{RExpr, RType};

/// The surface-level elaboration environment.
///
/// Wraps the kernel `GlobalEnv` and a name→id map for top-level declarations.
/// Call `ElabEnv::new()` to get a pre-populated environment with the V0 base
/// types (`Nat : Type 0`, `Bool : Type 0`) that the conformance suite expects.
pub struct ElabEnv {
    pub env: GlobalEnv,
    pub globals: HashMap<String, GlobalId>,
}

impl ElabEnv {
    /// Create an environment with no pre-declared types.
    pub fn empty() -> Self {
        Self {
            env: GlobalEnv::new(),
            globals: HashMap::new(),
        }
    }

    /// Create an environment with `Nat : Type 0` and `Bool : Type 0`
    /// pre-declared as postulates (required by several conformance cases).
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

    /// Elaborate and kernel-check a single top-level declaration from source.
    ///
    /// On success the declaration is registered in `self.env` and can be
    /// referenced by subsequent calls.
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

    /// Elaborate and kernel-check a standalone expression from source.
    ///
    /// Returns `(core_term, inferred_type)` — fully explicit, no metas.
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

/// Stub API from the original scaffold.
pub fn kernel_version() -> &'static str {
    ken_kernel::version()
}
