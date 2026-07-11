//! Module namespacing, import resolution, and visibility (`33 §3-4`,
//! ES3-build) — a pure surface/elaboration-time layer.
//!
//! `module`/`import`/`use`/`pub` add **no kernel feature**: a `module M { … }`
//! block is an environment fragment whose declarations are renamed to their
//! fully-qualified surface names (`M.foo`) and elaborated through the exact
//! same `resolve::resolve_decl` → `elab::elaborate_rdecl_v1` pipeline as a
//! flat, unqualified program. The kernel `GlobalEnv`/`Σ` never sees a name —
//! only `GlobalId`s — so qualification is bookkeeping entirely local to the
//! `globals: HashMap<String, GlobalId>` surface layer plus the bookkeeping
//! in `ModuleState` below. Abstract export (`§4.2`) requires zero additional
//! mechanism: a `pub data T = MkT` registers `T` in the module's export
//! table but never `MkT` (constructors are never auto-exported), which IS
//! the existing opaque-constant discipline at the surface layer — a client
//! that never gets `MkT` into scope can't build or match it, exactly as if
//! `T` had been declared as a hand-written opaque constant.
//!
//! Pipeline per compilation unit (one `elaborate_*` call's `Vec<Decl>`):
//! rename (qualify decl-level names) → `resolve_decl` (unchanged, purely
//! lexical) → rewrite (qualify free `RCon`/pattern-ctor references via the
//! active import scope) → `elaborate_rdecl_v1` (unchanged).

use std::collections::HashMap;

use crate::ast::{CtorDecl, Decl, ExplicitDataCtor, ImportKind};
use crate::error::{ElabError, Span};
use crate::resolve::{
    self, RCtorDecl, RDecl, RDeclKind, RExplicitCtorDecl, RExpr, RMatchArm, RPatKind, RPattern,
    RPropIntro, RTelescopeEntry, RType,
};
use crate::ElabEnv;

/// Persistent cross-call module bookkeeping (lives on `ElabEnv`).
#[derive(Default, Clone)]
pub struct ModuleState {
    /// The root (unqualified, file-level) scope: accumulates `import`/`use`
    /// bindings and top-level local names seen across separate
    /// `elaborate_decl`/`elaborate_file` calls, so a later call's bare
    /// references still see earlier imports/locals (a "file" is an implicit
    /// module, `33 §3.1`).
    root_scope: Scope,
    /// Qualified module path (`"M"`, `"M.N"`) → {bare `pub` name → canonical
    /// qualified name}. Populated whenever a `module { … }` block elaborates.
    /// Only `pub` names are recorded here — the export table IS the
    /// enforcement point for private-by-default (`§4.1`) and abstract
    /// export (`§4.2`): a name simply isn't here if it wasn't exported.
    exports: HashMap<String, HashMap<String, String>>,
}

#[derive(Clone, Debug)]
enum Binding {
    One(String),
    Ambiguous(Vec<String>),
}

/// Per-scope bare-name resolution: import bindings (qualified/aliased/
/// selective/open) plus this scope's own local declarations, which always
/// take precedence regardless of import order (`33 §3.3`, local-over-
/// imported).
#[derive(Default, Clone)]
struct Scope {
    bindings: HashMap<String, Binding>,
    /// Bare names bound by a LOCAL declaration in this scope — these are
    /// permanently immune to import-driven ambiguity (`bind_import` on a
    /// local name is a no-op): local always wins, regardless of import
    /// order (`33 §3.3`).
    locals: std::collections::HashSet<String>,
    /// Alias prefixes from `import M as N` — `N` resolves to `M` when used
    /// as a qualifying prefix (`N.foo`).
    prefixes: HashMap<String, String>,
}

impl Scope {
    fn bind_import(&mut self, bare: &str, qualified: &str) {
        if self.locals.contains(bare) {
            return;
        }
        match self.bindings.get(bare) {
            None => {
                self.bindings
                    .insert(bare.to_string(), Binding::One(qualified.to_string()));
            }
            Some(Binding::One(existing)) if existing == qualified => {}
            Some(Binding::One(existing)) => {
                let existing = existing.clone();
                self.bindings.insert(
                    bare.to_string(),
                    Binding::Ambiguous(vec![existing, qualified.to_string()]),
                );
            }
            Some(Binding::Ambiguous(v)) => {
                let mut v = v.clone();
                if !v.iter().any(|e| e == qualified) {
                    v.push(qualified.to_string());
                }
                self.bindings
                    .insert(bare.to_string(), Binding::Ambiguous(v));
            }
        }
    }

    /// A local declaration always wins outright, discarding any prior
    /// import binding (ambiguous or not) — `33 §3.3`.
    fn bind_local(&mut self, bare: &str, qualified: &str) {
        self.locals.insert(bare.to_string());
        self.bindings
            .insert(bare.to_string(), Binding::One(qualified.to_string()));
    }
}

fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", prefix, name)
    }
}

/// Resolve a (possibly dotted) surface name reference to its canonical
/// qualified form, using the active `scope` for bare names and `exports`
/// for qualified (`M.foo`) references. Returns the name **unchanged** if it
/// isn't module-tracked at all (not imported, not locally shadowed) — this
/// is what keeps every non-module program byte-for-byte backward
/// compatible: `scope`/`exports` are empty unless `module`/`import`/`use`
/// actually appear, so every lookup here is a no-op passthrough to the
/// pre-existing flat `cx.globals` resolution.
fn resolve_ref(
    scope: &Scope,
    exports: &HashMap<String, HashMap<String, String>>,
    name: &str,
    span: &Span,
) -> Result<String, ElabError> {
    if let Some(dot) = name.rfind('.') {
        let (prefix_part, leaf) = (&name[..dot], &name[dot + 1..]);
        if let Some(binding) = scope.bindings.get(prefix_part) {
            return match binding {
                Binding::One(q) => Ok(format!("{q}.{leaf}")),
                Binding::Ambiguous(sources) => Err(ElabError::AmbiguousReference {
                    name: name.to_string(),
                    sources: sources.clone(),
                    span: span.clone(),
                }),
            };
        }
        if prefix_part.contains('.') {
            let resolved_prefix = resolve_ref(scope, exports, prefix_part, span)?;
            return Ok(format!("{resolved_prefix}.{leaf}"));
        }
        let canonical_module = scope
            .prefixes
            .get(prefix_part)
            .cloned()
            .unwrap_or_else(|| prefix_part.to_string());
        match exports.get(&canonical_module) {
            Some(pubmap) => match pubmap.get(leaf) {
                Some(q) => Ok(q.clone()),
                // Either private (module-private by default, `§4.1`) or
                // simply not a declared name — both are the identical
                // surface diagnostic: not in scope, never reaching the
                // kernel.
                None => Err(ElabError::UnboundName {
                    name: name.to_string(),
                    span: span.clone(),
                }),
            },
            None => Err(ElabError::UnboundName {
                name: name.to_string(),
                span: span.clone(),
            }),
        }
    } else {
        match scope.bindings.get(name) {
            Some(Binding::One(q)) => Ok(q.clone()),
            Some(Binding::Ambiguous(sources)) => Err(ElabError::AmbiguousReference {
                name: name.to_string(),
                sources: sources.clone(),
                span: span.clone(),
            }),
            None => Ok(name.to_string()),
        }
    }
}

fn resolve_attached_ref(
    scope: &Scope,
    exports: &HashMap<String, HashMap<String, String>>,
    subject: &str,
    proof_name: &str,
    span: &Span,
) -> Result<String, ElabError> {
    let subject_is_local = !subject.contains('.') && scope.locals.contains(subject);
    let subject = resolve_ref(scope, exports, subject, span)?;
    if !subject_is_local {
        if let Some(dot) = subject.rfind('.') {
            let (module, leaf) = (&subject[..dot], &subject[dot + 1..]);
            if let Some(pubmap) = exports.get(module) {
                let attached_key = format!("{leaf}::{proof_name}");
                return pubmap
                    .get(&attached_key)
                    .cloned()
                    .ok_or_else(|| ElabError::UnboundName {
                        name: format!("{subject}::{proof_name}"),
                        span: span.clone(),
                    });
            }
        }
    }
    Ok(format!("{subject}::{proof_name}"))
}

fn apply_import(
    scope: &mut Scope,
    exports: &HashMap<String, HashMap<String, String>>,
    module: &str,
    kind: &ImportKind,
    span: &Span,
) -> Result<(), ElabError> {
    let pubmap = exports.get(module).ok_or_else(|| ElabError::UnboundName {
        name: module.to_string(),
        span: span.clone(),
    })?;
    match kind {
        ImportKind::Qualified => {
            scope
                .prefixes
                .insert(module.to_string(), module.to_string());
        }
        ImportKind::Aliased(alias) => {
            scope.prefixes.insert(alias.clone(), module.to_string());
        }
        ImportKind::Selective(names) => {
            for n in names {
                let q = pubmap.get(n).ok_or_else(|| ElabError::UnboundName {
                    name: format!("{}.{}", module, n),
                    span: span.clone(),
                })?;
                scope.bind_import(n, q);
            }
        }
        ImportKind::Open => {
            for (n, q) in pubmap.iter() {
                scope.bind_import(n, q);
            }
        }
    }
    Ok(())
}

/// Rename the declared name(s) of a raw surface `Decl` to their fully
/// qualified form (`prefix.name`), leaving every reference *inside* the
/// decl's body/type/etc untouched (those are qualified later, post-resolve,
/// by `rewrite_rdecl`). Only decl kinds with a single ordinary declared
/// name participate in module qualification (`view`/`let`/`data`/`type`);
/// classes/instances/laws/foreign/temporal/prove decls are elaborated
/// unqualified even inside a module (out of this WP's scope — no seed case
/// exercises them nested).
fn qualify_decl_name(decl: &Decl, prefix: &str) -> Decl {
    match decl {
        Decl::ViewDecl {
            keyword,
            name,
            params,
            ret_ty,
            requires,
            ensures,
            constraints,
            visits,
            body,
            is_space_op,
            span,
        } => Decl::ViewDecl {
            keyword: *keyword,
            name: qualify(prefix, name),
            params: params.clone(),
            ret_ty: ret_ty.clone(),
            requires: requires.clone(),
            ensures: ensures.clone(),
            constraints: constraints.clone(),
            visits: visits.clone(),
            body: body.clone(),
            is_space_op: *is_space_op,
            span: span.clone(),
        },
        Decl::LetDecl {
            name,
            ty,
            val,
            span,
        } => Decl::LetDecl {
            name: qualify(prefix, name),
            ty: ty.clone(),
            val: val.clone(),
            span: span.clone(),
        },
        Decl::DataDecl {
            name,
            type_params,
            ctors,
            span,
        } => Decl::DataDecl {
            name: qualify(prefix, name),
            type_params: type_params.clone(),
            ctors: ctors
                .iter()
                .map(|c| CtorDecl {
                    name: qualify(prefix, &c.name),
                    args: c.args.clone(),
                    field_labels: c.field_labels.clone(),
                    span: c.span.clone(),
                })
                .collect(),
            span: span.clone(),
        },
        Decl::ExplicitDataDecl {
            name,
            params,
            family,
            ctors,
            span,
        } => Decl::ExplicitDataDecl {
            name: qualify(prefix, name),
            params: params.clone(),
            family: family.clone(),
            ctors: ctors
                .iter()
                .map(|c| match c {
                    ExplicitDataCtor::Simple(c) => ExplicitDataCtor::Simple(CtorDecl {
                        name: qualify(prefix, &c.name),
                        args: c.args.clone(),
                        field_labels: c.field_labels.clone(),
                        span: c.span.clone(),
                    }),
                    ExplicitDataCtor::Signature {
                        name,
                        signature,
                        span,
                    } => ExplicitDataCtor::Signature {
                        name: qualify(prefix, name),
                        signature: signature.clone(),
                        span: span.clone(),
                    },
                })
                .collect(),
            span: span.clone(),
        },
        Decl::TypeAlias { name, ty, span } => Decl::TypeAlias {
            name: qualify(prefix, name),
            ty: ty.clone(),
            span: span.clone(),
        },
        Decl::PropDecl {
            name,
            params,
            ret_ty,
            intros,
            span,
        } => Decl::PropDecl {
            name: qualify(prefix, name),
            params: params.clone(),
            ret_ty: ret_ty.clone(),
            intros: intros.clone(),
            span: span.clone(),
        },
        Decl::LemmaDecl {
            name,
            params,
            theorem,
            body,
            span,
        } => Decl::LemmaDecl {
            name: qualify(prefix, name),
            params: params.clone(),
            theorem: theorem.clone(),
            body: body.clone(),
            span: span.clone(),
        },
        Decl::AttachedProofDecl {
            proof_name,
            subject,
            params,
            theorem,
            body,
            span,
        } => Decl::AttachedProofDecl {
            proof_name: proof_name.clone(),
            subject: subject.clone(),
            params: params.clone(),
            theorem: theorem.clone(),
            body: body.clone(),
            span: span.clone(),
        },
        other => other.clone(),
    }
}

fn rewrite_rtype(
    scope: &Scope,
    exports: &HashMap<String, HashMap<String, String>>,
    ty: RType,
) -> Result<RType, ElabError> {
    Ok(match ty {
        RType::RCon(name, span) => {
            let n = resolve_ref(scope, exports, &name, &span)?;
            RType::RCon(n, span)
        }
        RType::RVarTy(i, n, s) => RType::RVarTy(i, n, s),
        RType::RUniv(l, s) => RType::RUniv(l, s),
        RType::RPi(x, a, b, s) => RType::RPi(
            x,
            Box::new(rewrite_rtype(scope, exports, *a)?),
            Box::new(rewrite_rtype(scope, exports, *b)?),
            s,
        ),
        RType::RArr(a, b, s) => RType::RArr(
            Box::new(rewrite_rtype(scope, exports, *a)?),
            Box::new(rewrite_rtype(scope, exports, *b)?),
            s,
        ),
        RType::REffectArr(a, row, b, s) => RType::REffectArr(
            Box::new(rewrite_rtype(scope, exports, *a)?),
            row,
            Box::new(rewrite_rtype(scope, exports, *b)?),
            s,
        ),
        RType::RRefine(x, a, phi, s) => RType::RRefine(
            x,
            Box::new(rewrite_rtype(scope, exports, *a)?),
            Box::new(rewrite_rexpr(scope, exports, *phi)?),
            s,
        ),
        RType::RApp(f, a, s) => RType::RApp(
            Box::new(rewrite_rtype(scope, exports, *f)?),
            Box::new(rewrite_rtype(scope, exports, *a)?),
            s,
        ),
    })
}

fn rewrite_rexpr(
    scope: &Scope,
    exports: &HashMap<String, HashMap<String, String>>,
    e: RExpr,
) -> Result<RExpr, ElabError> {
    Ok(match e {
        RExpr::RCon(name, span) => {
            let n = resolve_ref(scope, exports, &name, &span)?;
            RExpr::RCon(n, span)
        }
        RExpr::RVar(i, n, s) => RExpr::RVar(i, n, s),
        RExpr::RUniv(l, s) => RExpr::RUniv(l, s),
        RExpr::RApp(f, a, s) => RExpr::RApp(
            Box::new(rewrite_rexpr(scope, exports, *f)?),
            Box::new(rewrite_rexpr(scope, exports, *a)?),
            s,
        ),
        RExpr::RLam(n, b, s) => RExpr::RLam(n, Box::new(rewrite_rexpr(scope, exports, *b)?), s),
        RExpr::RLet(x, ty, rhs, body, s) => RExpr::RLet(
            x,
            ty.map(|t| rewrite_rtype(scope, exports, t)).transpose()?,
            Box::new(rewrite_rexpr(scope, exports, *rhs)?),
            Box::new(rewrite_rexpr(scope, exports, *body)?),
            s,
        ),
        RExpr::RAsc(e, t, s) => RExpr::RAsc(
            Box::new(rewrite_rexpr(scope, exports, *e)?),
            Box::new(rewrite_rtype(scope, exports, *t)?),
            s,
        ),
        RExpr::ROld(e, s) => RExpr::ROld(Box::new(rewrite_rexpr(scope, exports, *e)?), s),
        RExpr::RNumLit(l, s) => RExpr::RNumLit(l, s),
        RExpr::RStr(v, s) => RExpr::RStr(v, s),
        RExpr::RBinOp(op, l, r, s) => RExpr::RBinOp(
            op,
            Box::new(rewrite_rexpr(scope, exports, *l)?),
            Box::new(rewrite_rexpr(scope, exports, *r)?),
            s,
        ),
        RExpr::RMatch {
            scrut,
            equation,
            arms,
            span,
        } => {
            let scrut = Box::new(rewrite_rexpr(scope, exports, *scrut)?);
            let arms = arms
                .into_iter()
                .map(|a| {
                    Ok(RMatchArm {
                        pat: rewrite_rpattern(scope, exports, a.pat)?,
                        body: rewrite_rexpr(scope, exports, a.body)?,
                        span: a.span,
                    })
                })
                .collect::<Result<Vec<_>, ElabError>>()?;
            RExpr::RMatch {
                scrut,
                equation,
                arms,
                span,
            }
        }
        RExpr::RProj(e, field, s) => {
            RExpr::RProj(Box::new(rewrite_rexpr(scope, exports, *e)?), field, s)
        }
        RExpr::RPi(x, a, b, s) => RExpr::RPi(
            x,
            Box::new(rewrite_rtype(scope, exports, *a)?),
            Box::new(rewrite_rexpr(scope, exports, *b)?),
            s,
        ),
        RExpr::RArrow(a, b, s) => RExpr::RArrow(
            Box::new(rewrite_rexpr(scope, exports, *a)?),
            Box::new(rewrite_rexpr(scope, exports, *b)?),
            s,
        ),
        RExpr::RAttachedProofRef {
            subject,
            proof_name,
            span,
        } => RExpr::RCon(
            resolve_attached_ref(scope, exports, &subject, &proof_name, &span)?,
            span,
        ),
    })
}

fn rewrite_rpattern(
    scope: &Scope,
    exports: &HashMap<String, HashMap<String, String>>,
    p: RPattern,
) -> Result<RPattern, ElabError> {
    let kind = match p.kind {
        RPatKind::Wild => RPatKind::Wild,
        RPatKind::Var(n) => RPatKind::Var(n),
        RPatKind::Ctor(name, subs) => {
            let n = resolve_ref(scope, exports, &name, &p.span)?;
            let subs = subs
                .into_iter()
                .map(|s| rewrite_rpattern(scope, exports, s))
                .collect::<Result<Vec<_>, ElabError>>()?;
            RPatKind::Ctor(n, subs)
        }
    };
    Ok(RPattern { kind, span: p.span })
}

fn rewrite_rdecl(
    scope: &Scope,
    exports: &HashMap<String, HashMap<String, String>>,
    rdecl: RDecl,
) -> Result<RDecl, ElabError> {
    let ty = rdecl
        .ty
        .map(|t| rewrite_rtype(scope, exports, t))
        .transpose()?;
    let body = rewrite_rexpr(scope, exports, rdecl.body)?;
    let requires = rdecl
        .requires
        .into_iter()
        .map(|e| rewrite_rexpr(scope, exports, e))
        .collect::<Result<Vec<_>, ElabError>>()?;
    let ensures = rdecl
        .ensures
        .into_iter()
        .map(|e| rewrite_rexpr(scope, exports, e))
        .collect::<Result<Vec<_>, ElabError>>()?;
    let kind = match rdecl.kind {
        RDeclKind::View {
            keyword,
            is_space_op,
            constraints,
            visits,
        } => RDeclKind::View {
            keyword,
            is_space_op,
            constraints: constraints
                .into_iter()
                .map(|constraint| {
                    Ok(crate::resolve::RInstanceConstraint {
                        class_name: constraint.class_name,
                        head_type: rewrite_rtype(scope, exports, constraint.head_type)?,
                        binder: constraint.binder,
                    })
                })
                .collect::<Result<Vec<_>, ElabError>>()?,
            visits,
        },
        RDeclKind::Let => RDeclKind::Let,
        RDeclKind::Prove => RDeclKind::Prove,
        RDeclKind::Prop { intros } => RDeclKind::Prop {
            intros: intros
                .into_iter()
                .map(|intro| {
                    Ok(RPropIntro {
                        name: intro.name,
                        ty: rewrite_rtype(scope, exports, intro.ty)?,
                        span: intro.span,
                    })
                })
                .collect::<Result<Vec<_>, ElabError>>()?,
        },
        RDeclKind::Lemma => RDeclKind::Lemma,
        RDeclKind::AttachedProof {
            subject,
            proof_name,
        } => RDeclKind::AttachedProof {
            subject: resolve_ref(scope, exports, &subject, &rdecl.span)?,
            proof_name,
        },
        RDeclKind::Law { param, fields } => RDeclKind::Law {
            param,
            fields: fields
                .into_iter()
                .map(|(n, e)| Ok((n, rewrite_rexpr(scope, exports, e)?)))
                .collect::<Result<Vec<_>, ElabError>>()?,
        },
        RDeclKind::DataDecl { type_params, ctors } => RDeclKind::DataDecl {
            type_params,
            ctors: ctors
                .into_iter()
                .map(|c| {
                    Ok(RCtorDecl {
                        name: c.name,
                        args: c
                            .args
                            .into_iter()
                            .map(|t| rewrite_rtype(scope, exports, t))
                            .collect::<Result<Vec<_>, ElabError>>()?,
                        field_labels: c.field_labels,
                        span: c.span,
                    })
                })
                .collect::<Result<Vec<_>, ElabError>>()?,
        },
        RDeclKind::ExplicitDataDecl {
            params,
            indices,
            level,
            ctors,
        } => {
            let rewrite_entry = |entry: RTelescopeEntry| -> Result<RTelescopeEntry, ElabError> {
                Ok(RTelescopeEntry {
                    name: entry.name,
                    ty: rewrite_rtype(scope, exports, entry.ty)?,
                    span: entry.span,
                })
            };
            RDeclKind::ExplicitDataDecl {
                params: params
                    .into_iter()
                    .map(rewrite_entry)
                    .collect::<Result<Vec<_>, ElabError>>()?,
                indices: indices
                    .into_iter()
                    .map(rewrite_entry)
                    .collect::<Result<Vec<_>, ElabError>>()?,
                level,
                ctors: ctors
                    .into_iter()
                    .map(|c| {
                        Ok(RExplicitCtorDecl {
                            name: c.name,
                            args: c
                                .args
                                .into_iter()
                                .map(rewrite_entry)
                                .collect::<Result<Vec<_>, ElabError>>()?,
                            result: c
                                .result
                                .map(|t| rewrite_rtype(scope, exports, t))
                                .transpose()?,
                            span: c.span,
                        })
                    })
                    .collect::<Result<Vec<_>, ElabError>>()?,
            }
        }
        RDeclKind::TypeAlias { ty } => RDeclKind::TypeAlias {
            ty: rewrite_rtype(scope, exports, ty)?,
        },
        RDeclKind::Foreign {
            symbol,
            library,
            is_pure,
            visits,
        } => RDeclKind::Foreign {
            symbol,
            library,
            is_pure,
            visits,
        },
        RDeclKind::Temporal { formula, source } => RDeclKind::Temporal { formula, source },
        RDeclKind::ClassDecl {
            param,
            param_kind,
            fields,
        } => RDeclKind::ClassDecl {
            param,
            param_kind: param_kind
                .map(|t| rewrite_rtype(scope, exports, t))
                .transpose()?,
            fields: fields
                .into_iter()
                .map(|f| {
                    Ok(crate::resolve::RClassField {
                        purity: f.purity,
                        name: f.name,
                        ty: rewrite_rtype(scope, exports, f.ty)?,
                    })
                })
                .collect::<Result<Vec<_>, ElabError>>()?,
        },
        RDeclKind::InstanceDecl {
            head_params,
            head_type,
            constraints,
            fields,
        } => RDeclKind::InstanceDecl {
            head_params,
            head_type: rewrite_rtype(scope, exports, head_type)?,
            constraints: constraints
                .into_iter()
                .map(|constraint| {
                    Ok(crate::resolve::RInstanceConstraint {
                        class_name: constraint.class_name,
                        head_type: rewrite_rtype(scope, exports, constraint.head_type)?,
                        binder: constraint.binder,
                    })
                })
                .collect::<Result<Vec<_>, ElabError>>()?,
            fields: fields
                .into_iter()
                .map(|(n, e)| Ok((n, rewrite_rexpr(scope, exports, e)?)))
                .collect::<Result<Vec<_>, ElabError>>()?,
        },
        RDeclKind::DeriveDecl { data_name } => RDeclKind::DeriveDecl { data_name },
    };
    let name = match &kind {
        RDeclKind::AttachedProof {
            subject,
            proof_name,
        } => format!("{subject}::{proof_name}"),
        _ => rdecl.name,
    };
    Ok(RDecl {
        name,
        ty,
        body,
        requires,
        ensures,
        span: rdecl.span,
        kind,
    })
}

/// Does this (unwrapped) decl kind participate in module-local-name
/// shadowing / qualification (`view`/`let`/`data`/`type`)? Classes,
/// instances, laws, foreign bindings, temporal obligations, and `prove`
/// obligations are elaborated unqualified even inside a `module { … }`
/// block (out of this WP's scope).
fn is_qualifiable(decl: &Decl) -> bool {
    matches!(
        decl,
        Decl::ViewDecl { .. }
            | Decl::LetDecl { .. }
            | Decl::PropDecl { .. }
            | Decl::LemmaDecl { .. }
            | Decl::AttachedProofDecl { .. }
            | Decl::DataDecl { .. }
            | Decl::ExplicitDataDecl { .. }
            | Decl::TypeAlias { .. }
    )
}

fn is_recursive_candidate(decl: &Decl) -> bool {
    matches!(
        decl,
        Decl::ViewDecl { .. }
            | Decl::LetDecl { .. }
            | Decl::LemmaDecl { .. }
            | Decl::AttachedProofDecl { .. }
    )
}

fn register_effect_row(elab: &mut ElabEnv, result: &crate::elab::ElabResult) {
    if let Some(row) = &result.effect_row_type {
        elab.effect_rows.insert(result.name.clone(), row.clone());
    }
    if let Some(fb) = &result.foreign_binding {
        elab.foreign_env.register(result.name.clone(), fb.clone());
        if !fb.effect_row.is_empty() {
            elab.effect_rows.insert(
                result.name.clone(),
                crate::effects::RowType::Concrete(fb.effect_row.clone()),
            );
        }
    }
}

fn register_declared_effect_row(
    elab: &mut ElabEnv,
    rdecl: &crate::resolve::RDecl,
) -> Result<(), ElabError> {
    if let Some(row) = crate::elab::surface_declared_row_type(rdecl)? {
        elab.effect_rows.insert(rdecl.name.clone(), row);
    }
    Ok(())
}

fn elaborate_checked(
    elab: &mut ElabEnv,
    rdecl: &crate::resolve::RDecl,
) -> Result<crate::elab::ElabResult, ElabError> {
    crate::elab::check_surface_purity(rdecl, &elab.effect_rows, &elab.globals, &elab.class_env)?;
    let result = crate::elab::elaborate_rdecl_v1_with_effect_rows(
        &mut elab.env,
        &mut elab.globals,
        &mut elab.num_values,
        &elab.numeric_env,
        &mut elab.class_env,
        &elab.effect_rows,
        rdecl,
    )?;
    register_effect_row(elab, &result);
    Ok(result)
}

/// Expand and elaborate a compilation unit's raw decls (one `elaborate_*`
/// call's `Vec<Decl>`) at nesting `prefix` ("" at the file root), threading
/// `scope` (built fresh for a `module { … }` block; the persisted root
/// scope at the top level) and returning every produced `GlobalId` in
/// order, plus this scope's own `pub` export table.
fn expand_scope(
    elab: &mut ElabEnv,
    decls: &[Decl],
    prefix: &str,
    scope: &mut Scope,
) -> Result<(Vec<crate::elab::ElabResult>, HashMap<String, String>), ElabError> {
    // Pre-pass: collect this scope's own local declared names FIRST, so
    // they unconditionally shadow any import processed below regardless of
    // textual order (`33 §3.3`, local-over-imported).
    for decl in decls {
        let inner = decl.unwrap_pub();
        if is_qualifiable(inner) {
            if matches!(inner, Decl::AttachedProofDecl { .. }) {
                continue;
            }
            let bare = inner.name().to_string();
            scope.bind_local(&bare, &qualify(prefix, &bare));
        }
    }

    let mut ids = Vec::new();
    let mut exports_here: HashMap<String, String> = HashMap::new();
    let mut i = 0;
    while i < decls.len() {
        let decl = &decls[i];
        match decl {
            // Imports are applied HERE, in textual order, so `import M`
            // sees `M`'s export table only once `module M { … }` has
            // actually been expanded — which happens earlier in this same
            // ordered pass if `M` is a sibling defined above (the normal
            // case; a module must be declared before it's imported).
            Decl::ImportDecl { module, kind, span } => {
                apply_import(scope, &elab.module_state.exports, module, kind, span)?;
                i += 1;
            }
            Decl::ModuleDecl {
                name,
                decls: inner,
                span: _,
            } => {
                let child_prefix = qualify(prefix, name);
                let mut child_scope = Scope::default();
                let (child_ids, child_exports) =
                    expand_scope(elab, inner, &child_prefix, &mut child_scope)?;
                ids.extend(child_ids);
                elab.module_state
                    .exports
                    .insert(child_prefix, child_exports);
                i += 1;
            }
            // A maximal run of non-`pub` definitions — auto-grouped by
            // call-graph SCC (`33 §1`: "All
            // top-level definitions are mutually recursive within a module
            // if the SCT check accepts the group"). A run with no actual
            // cycle degenerates to today's one-decl-at-a-time path, member
            // by member, byte-identical (AC3).
            _ if is_recursive_candidate(decl.unwrap_pub()) => {
                let run_end = {
                    let mut e = i;
                    while e < decls.len() && is_recursive_candidate(decls[e].unwrap_pub()) {
                        e += 1;
                    }
                    e
                };
                let run = &decls[i..run_end];

                // Resolve + rewrite every run member up front — safe because
                // a run contains no import/module, so `scope`/`exports`
                // don't change across it; each member sees exactly the
                // state it would have seen processed alone at its position.
                let mut bare_names: Vec<String> = Vec::with_capacity(run.len());
                let mut rdecls: Vec<crate::resolve::RDecl> = Vec::with_capacity(run.len());
                for d in run {
                    let inner = d.unwrap_pub();
                    let renamed = qualify_decl_name(inner, prefix);
                    let rdecl = resolve::resolve_decl(&renamed)?;
                    let rdecl = rewrite_rdecl(scope, &elab.module_state.exports, rdecl)?;
                    bare_names.push(rdecl.name.clone());
                    rdecls.push(rdecl);
                }

                // Call graph: edge a -> b iff a's body mentions b's bare
                // name (over-approximates on shadowing — safe, only ever
                // makes an SCC too LARGE, never misses a real cycle).
                let n = rdecls.len();
                let adj: Vec<Vec<usize>> = (0..n)
                    .map(|a| {
                        (0..n)
                            .filter(|&b| {
                                crate::elab::rexpr_mentions_name(&rdecls[a].body, &bare_names[b])
                                    || rdecls[a].ty.as_ref().is_some_and(|ty| {
                                        crate::elab::rtype_mentions_name(ty, &bare_names[b])
                                    })
                            })
                            .collect()
                    })
                    .collect();
                let sccs = scc_membership(&adj);

                // Process the SCC condensation dependency-first: a caller's
                // body is checked only after every acyclic callee body is
                // available for delta reduction.  The signature pre-pass in
                // a recursive SCC still admits every member before any body.
                let mut consumed = vec![false; n];
                for k in scc_dependency_order(&adj, &sccs) {
                    if consumed[k] {
                        continue;
                    }
                    let scc = &sccs[k];
                    for &m in scc {
                        consumed[m] = true;
                    }
                    // Existing singleton view/let recursion has its own
                    // spec-aware elaboration path.  Self edges are newly
                    // routed through the group/SCT seam only for proof
                    // declarations; multi-member SCCs remain shared.
                    let recursive = scc.len() > 1
                        || (adj[k].contains(&k)
                            && matches!(
                                rdecls[k].kind,
                                RDeclKind::Lemma | RDeclKind::AttachedProof { .. }
                            ));
                    if !recursive {
                        let rdecl = &rdecls[k];
                        let result = elaborate_checked(elab, rdecl)?;
                        ids.push(result);
                    } else {
                        let members: Vec<crate::resolve::RDecl> =
                            scc.iter().map(|&m| rdecls[m].clone()).collect();
                        let has_proof = members.iter().any(|rdecl| {
                            matches!(
                                rdecl.kind,
                                RDeclKind::Lemma | RDeclKind::AttachedProof { .. }
                            )
                        });
                        let has_computational = members.iter().any(|rdecl| {
                            matches!(rdecl.kind, RDeclKind::Let | RDeclKind::View { .. })
                        });
                        if has_proof && has_computational {
                            return Err(ElabError::TypeMismatch {
                                span: members[0].span.clone(),
                                reason: "mixed fn/const and proof recursive cycle is not supported"
                                    .to_string(),
                            });
                        }
                        let mut group_effect_rows = elab.effect_rows.clone();
                        for rdecl in &members {
                            if let Some(row) = crate::elab::surface_declared_row_type(rdecl)? {
                                group_effect_rows.insert(rdecl.name.clone(), row);
                            }
                        }
                        // Eligibility guard: the new group path only covers
                        // the plain V0 view/let shape (matches the existing
                        // singleton recursive-const rule) — a mutual member
                        // needing requires/ensures/where/refinement-return
                        // is out of this WP's scope; fail clearly rather
                        // than silently dropping its obligation.
                        for rdecl in &members {
                            let simple_kind = matches!(
                                &rdecl.kind,
                                RDeclKind::Let | RDeclKind::Lemma | RDeclKind::AttachedProof { .. }
                            ) || matches!(
                                &rdecl.kind,
                                RDeclKind::View { constraints, is_space_op, .. }
                                    if constraints.is_empty() && !is_space_op
                            );
                            let has_refine_return = rdecl
                                .ty
                                .as_ref()
                                .and_then(|ty| crate::elab::innermost_refine_pred(ty))
                                .is_some();
                            if !simple_kind
                                || !rdecl.requires.is_empty()
                                || !rdecl.ensures.is_empty()
                                || has_refine_return
                            {
                                return Err(ElabError::Internal(format!(
                                    "mutual recursion is only supported for plain recursive \
                                     definitions (no requires/ensures/where-constraints/\
                                     refinement-return); '{}' does not qualify",
                                    rdecl.name
                                )));
                            }
                            crate::elab::check_surface_purity(
                                rdecl,
                                &group_effect_rows,
                                &elab.globals,
                                &elab.class_env,
                            )?;
                        }
                        let results = crate::elab::elaborate_mutual_group(
                            &mut elab.env,
                            &mut elab.globals,
                            &mut elab.num_values,
                            &elab.numeric_env,
                            &elab.class_env,
                            &members,
                        )?;
                        for (rdecl, result) in members.iter().zip(results) {
                            register_effect_row(elab, &result);
                            register_declared_effect_row(elab, rdecl)?;
                            ids.push(result);
                        }
                    }
                }
                // Public definitions participate in the same scope-wide
                // admission run; publish their already-elaborated canonical
                // names only after the run succeeds, preserving the module
                // export boundary while allowing forward references.
                for (d, rdecl) in run.iter().zip(&rdecls) {
                    if !d.is_pub() {
                        continue;
                    }
                    let inner = d.unwrap_pub();
                    if let Decl::AttachedProofDecl {
                        subject,
                        proof_name,
                        ..
                    } = inner
                    {
                        let subject_is_public = exports_here.contains_key(subject)
                            || run.iter().any(|candidate| {
                                candidate.is_pub() && candidate.unwrap_pub().name() == subject
                            });
                        if !subject_is_public {
                            return Err(ElabError::UnboundName {
                                name: subject.clone(),
                                span: inner.span().clone(),
                            });
                        }
                        exports_here.insert(format!("{subject}::{proof_name}"), rdecl.name.clone());
                    } else {
                        exports_here.insert(inner.name().to_string(), rdecl.name.clone());
                    }
                }
                i = run_end;
            }
            other => {
                let is_pub = other.is_pub();
                let inner = other.unwrap_pub();
                if is_pub && !prefix.is_empty() {
                    if let Decl::DataDecl { name, span, .. } = inner {
                        // Abstract export (`33 §4.2`) applies only INSIDE a
                        // real `module { … }` (`prefix` non-empty) — there
                        // is no "outside" to hide from at the true file
                        // root (`prefix == ""`), exactly as a root-level
                        // `pub` on `View`/`Let`/`TypeAlias` is already
                        // inert there (its `exports_here` entry is
                        // produced but discarded by `expand_and_elaborate`
                        // as `_root_exports`). A `pub data T = MkT` at the
                        // top level must fall through to ordinary `data`
                        // elaboration below — `MkT` stays a real,
                        // constructible/matchable constructor, not a
                        // silently-stripped opaque constant with no
                        // client to protect.
                        //
                        // A `pub data T = …` exports the type name only —
                        // constructors are never `pub`-able in this
                        // surface, so the whole ctor set is always
                        // withheld. Rather than a real `Decl::Inductive`
                        // with hidden-but-present ctors, declare `T` as
                        // the kernel's EXISTING opaque constant (`11 §4`)
                        // directly: byte-identical to a hand-written
                        // `T : Type` postulate, no new `Decl` variant, no
                        // kernel "abstract" flag. The constructors are
                        // simply never registered anywhere (not in
                        // `globals`, not in any export table) —
                        // unconstructible and unmatchable, by every
                        // observer, kernel included.
                        let qualified = qualify(prefix, name);
                        let ty = ken_kernel::Term::ty(ken_kernel::Level::Zero);
                        let id = ken_kernel::declare_postulate(&mut elab.env, vec![], ty).map_err(
                            |e| ElabError::KernelRejected {
                                error: e,
                                span: span.clone(),
                            },
                        )?;
                        elab.globals.insert(qualified.clone(), id);
                        exports_here.insert(name.clone(), qualified.clone());
                        ids.push(crate::elab::ElabResult {
                            name: qualified,
                            def_id: id,
                            obligations: vec![],
                            foreign_binding: None,
                            temporal_obligations: vec![],
                            effect_row_type: None,
                        });
                        i += 1;
                        continue;
                    }
                }
                if is_qualifiable(inner) {
                    let bare = inner.name().to_string();
                    if is_pub {
                        if let Decl::AttachedProofDecl {
                            subject,
                            proof_name,
                            ..
                        } = inner
                        {
                            if !exports_here.contains_key(subject) {
                                return Err(ElabError::UnboundName {
                                    name: subject.clone(),
                                    span: inner.span().clone(),
                                });
                            }
                            if exports_here.contains_key(&format!("{subject}::{proof_name}")) {
                                return Err(ElabError::TypeMismatch {
                                    span: inner.span().clone(),
                                    reason: format!(
                                        "duplicate public attached proof '{}::{}'",
                                        subject, proof_name
                                    ),
                                });
                            }
                        }
                    }
                    let renamed = qualify_decl_name(inner, prefix);
                    let rdecl = resolve::resolve_decl(&renamed)?;
                    let rdecl = rewrite_rdecl(scope, &elab.module_state.exports, rdecl)?;
                    let result = elaborate_checked(elab, &rdecl)?;
                    if is_pub {
                        if let Decl::AttachedProofDecl {
                            subject,
                            proof_name,
                            ..
                        } = inner
                        {
                            exports_here
                                .insert(format!("{subject}::{proof_name}"), result.name.clone());
                        } else {
                            // Only the decl's own qualified name is exported —
                            // never a `DataDecl`'s constructors (`33 §4.2`,
                            // abstract export: ctors are simply never entered
                            // into any export table, so a client can't bring
                            // them into scope by any import form).
                            exports_here.insert(bare, result.name.clone());
                        }
                    }
                    ids.push(result);
                } else {
                    // Not module-qualifiable (class/instance/law/foreign/
                    // temporal/prove) — elaborate unchanged, unqualified.
                    let rdecl = resolve::resolve_decl(inner)?;
                    let result = elaborate_checked(elab, &rdecl)?;
                    ids.push(result);
                }
                i += 1;
            }
        }
    }
    Ok((ids, exports_here))
}

/// Strongly-connected-component membership for a small directed call graph
/// (`adj[i]` = out-edges from `i`, i.e. "`i`'s body mentions `j`"). Returns,
/// per node, the sorted list of node indices in its SCC (always includes the
/// node itself). O(n^3) — fine for a same-scope call graph (one source
/// file's mutually-recursive group), not sized for a whole-program graph.
fn scc_membership(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let mut reach: Vec<Vec<bool>> = vec![vec![false; n]; n];
    for (i, reach_i) in reach.iter_mut().enumerate() {
        let mut stack = adj[i].clone();
        let mut seen = vec![false; n];
        while let Some(j) = stack.pop() {
            if seen[j] {
                continue;
            }
            seen[j] = true;
            reach_i[j] = true;
            for &k in &adj[j] {
                if !seen[k] {
                    stack.push(k);
                }
            }
        }
    }
    (0..n)
        .map(|i| {
            let mut members: Vec<usize> = (0..n)
                .filter(|&j| j == i || (reach[i][j] && reach[j][i]))
                .collect();
            members.sort_unstable();
            members
        })
        .collect()
}

/// Return one representative per SCC in dependency-first order.  An edge
/// `a -> b` means that `a`'s body uses `b`, so `b` must be elaborated first.
/// Members of an SCC are still elaborated together by the SCT path.
fn scc_dependency_order(adj: &[Vec<usize>], sccs: &[Vec<usize>]) -> Vec<usize> {
    let mut representatives = Vec::new();
    for (node, scc) in sccs.iter().enumerate() {
        if scc[0] == node {
            representatives.push(node);
        }
    }
    let mut order = Vec::new();
    let mut seen = vec![false; adj.len()];
    fn visit(
        node: usize,
        adj: &[Vec<usize>],
        sccs: &[Vec<usize>],
        seen: &mut [bool],
        order: &mut Vec<usize>,
    ) {
        let rep = sccs[node][0];
        if seen[rep] {
            return;
        }
        seen[rep] = true;
        for &dep in &adj[node] {
            visit(dep, adj, sccs, seen, order);
        }
        order.push(rep);
    }
    for node in representatives {
        visit(node, adj, sccs, &mut seen, &mut order);
    }
    order
}

/// Entry point: expand + elaborate one `elaborate_*` call's raw decls
/// against the persisted root scope (the file-level implicit module,
/// `33 §3.1`), returning every produced `ElabResult` in order.
pub fn expand_and_elaborate(
    elab: &mut ElabEnv,
    decls: &[Decl],
) -> Result<Vec<crate::elab::ElabResult>, ElabError> {
    let mut scope = elab.module_state.root_scope.clone();
    let (results, _root_exports) = expand_scope(elab, decls, "", &mut scope)?;
    elab.module_state.root_scope = scope;
    Ok(results)
}
