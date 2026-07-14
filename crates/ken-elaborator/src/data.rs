//! Inductive type declaration elaboration (`34 §1`, `14 §8`).
//!
//! `elab_data_decl` translates a resolved `data D p₁…pₙ = C₁ τ… | …`
//! declaration into a kernel `InductiveSpec`, calls `declare_inductive`,
//! and registers the type former + all constructors in `globals`.
//!
//! ## De Bruijn conventions
//! - The kernel stores ctor arg types in context `[Δ_p]` (params innermost-first):
//!   `Var(0)` = last param, `Var(m-1)` = first param.
//! - The resolver has params in scope left-to-right: pushing them in order
//!   gives last_param at `Var(0)`, first_param at `Var(m-1)` — same convention.
//! - Ctor arg `j` is in context `[Δ_p, arg₀…argⱼ₋₁]`; raw kernel storage uses
//!   context `[Δ_p]` only, so arg `j` is weakened by `j` before being stored.

use std::collections::{HashMap, HashSet};

use ken_kernel::subst::weaken;
use ken_kernel::{
    declare_inductive, infer, level_eq, whnf, Context, CtorSpec, GlobalEnv, GlobalId,
    InductiveSpec, KernelError, Level, Term,
};

use crate::error::{ElabError, Span};
use crate::resolve::{RCtorDecl, RExplicitCtorDecl, RTelescopeEntry, RType};

/// Elaborate `data D p₁…pₙ = C₁ τ… | …` (`34 §1`).
///
/// Registers D (the type former) and every constructor Cₖ in `globals`.
/// Returns the `GlobalId` of the type former.
pub fn elab_data_decl(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    d_name: &str,
    type_params: &[String],
    ctors: &[RCtorDecl],
    span: &Span,
) -> Result<GlobalId, ElabError> {
    let m = type_params.len();

    // Snapshot globals + inductive-id set BEFORE the `declare_inductive` closure borrows `env`.
    let global_info: HashMap<String, GlobalId> = globals.clone();
    let ind_id_set: HashSet<GlobalId> = globals
        .values()
        .copied()
        .filter(|&id| env.inductive(id).is_some())
        .collect();
    let ctor_id_set: HashSet<GlobalId> = globals
        .values()
        .copied()
        .filter(|&id| env.constructor(id).is_some())
        .collect();

    // Params: m params each of type `Type 0` (left-to-right, innermost-last in the Π-chain).
    let params: Vec<Term> = (0..m).map(|_| Term::ty(Level::Zero)).collect();

    // Clone ctors so the closure can take ownership.
    let ctors_owned = ctors.to_vec();
    let d_name_owned = d_name.to_string();
    let mut build_error: Option<ElabError> = None;

    let d_id = declare_inductive(env, |d_id| {
        match build_legacy_inductive_spec(
            d_id,
            &d_name_owned,
            &params,
            &ctors_owned,
            &global_info,
            &ind_id_set,
            &ctor_id_set,
        ) {
            Ok(spec) => spec,
            Err(err) => {
                build_error = Some(err);
                empty_inductive_spec()
            }
        }
    })
    .map_err(|error| {
        translate_legacy_kernel_error(
            env,
            error,
            d_name,
            &params,
            ctors,
            &global_info,
            &ind_id_set,
            &ctor_id_set,
            span,
        )
    })?;

    if let Some(err) = build_error {
        env.remove_last();
        return Err(err);
    }

    // Register the type former.
    globals.insert(d_name.to_string(), d_id);

    // Register constructors in declaration order.
    // `env.inductive` is available again now that `declare_inductive` returned.
    let ctor_ids: Vec<GlobalId> = env
        .inductive(d_id)
        .ok_or_else(|| ElabError::Internal("inductive not found after declare".into()))?
        .constructors
        .iter()
        .map(|c| c.id)
        .collect();

    for (i, c) in ctors.iter().enumerate() {
        globals.insert(c.name.clone(), ctor_ids[i]);
    }

    Ok(d_id)
}

fn build_legacy_inductive_spec(
    d_id: GlobalId,
    d_name: &str,
    params: &[Term],
    ctors: &[RCtorDecl],
    globals: &HashMap<String, GlobalId>,
    ind_ids: &HashSet<GlobalId>,
    ctor_ids: &HashSet<GlobalId>,
) -> Result<InductiveSpec, ElabError> {
    let constructors = ctors
        .iter()
        .map(|ctor| {
            let args = ctor
                .args
                .iter()
                .enumerate()
                .map(|(j, rty)| {
                    // The resolved legacy type is based in [Δ_p]. Argument j
                    // is checked in [Δ_p, arg₀…argⱼ₋₁], so weaken it by j.
                    rtype_to_kernel_checked(rty, d_name, d_id, globals, ind_ids, ctor_ids)
                        .map(|raw| weaken(&raw, j as i64))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CtorSpec {
                args,
                target_indices: vec![],
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(InductiveSpec {
        level_params: vec![],
        params: params.to_vec(),
        indices: vec![],
        level: Level::Zero,
        constructors,
    })
}

fn empty_inductive_spec() -> InductiveSpec {
    InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::Zero,
        constructors: vec![],
    }
}

/// Elaborate `data D (Δp) : (Δi) -> Type where { C : (Δk) -> D Δp t̄ }`
/// (`34 §2`, `39 §2.2`) through the same kernel inductive-family admission path
/// used by legacy simple data.
pub fn elab_explicit_data_decl(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    d_name: &str,
    params: &[RTelescopeEntry],
    indices: &[RTelescopeEntry],
    level: Option<u32>,
    ctors: &[RExplicitCtorDecl],
    span: &Span,
) -> Result<GlobalId, ElabError> {
    let m = params.len();

    let global_info: HashMap<String, GlobalId> = globals.clone();
    let ind_id_set: HashSet<GlobalId> = globals
        .values()
        .copied()
        .filter(|&id| env.inductive(id).is_some())
        .collect();
    let ctor_id_set: HashSet<GlobalId> = globals
        .values()
        .copied()
        .filter(|&id| env.constructor(id).is_some())
        .collect();

    let param_terms = params
        .iter()
        .map(|p| {
            rtype_to_kernel_checked(
                &p.ty,
                d_name,
                GlobalId(u32::MAX),
                &global_info,
                &ind_id_set,
                &ctor_id_set,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let index_terms = indices
        .iter()
        .map(|i| {
            rtype_to_kernel_checked(
                &i.ty,
                d_name,
                GlobalId(u32::MAX),
                &global_info,
                &ind_id_set,
                &ctor_id_set,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let level = level.map(level_from_nat).unwrap_or(Level::Zero);
    let ctors_owned = ctors.to_vec();
    let d_name_owned = d_name.to_string();
    let mut build_error: Option<ElabError> = None;

    let d_id = declare_inductive(env, |d_id| {
        match build_explicit_inductive_spec(
            d_id,
            &d_name_owned,
            m,
            indices.len(),
            &param_terms,
            &index_terms,
            level.clone(),
            &ctors_owned,
            &global_info,
            &ind_id_set,
            &ctor_id_set,
        ) {
            Ok(spec) => spec,
            Err(err) => {
                build_error = Some(err);
                empty_inductive_spec()
            }
        }
    })
    .map_err(|error| {
        translate_explicit_kernel_error(
            env,
            error,
            d_name,
            &param_terms,
            &index_terms,
            &level,
            ctors,
            &global_info,
            &ind_id_set,
            &ctor_id_set,
            span,
        )
    })?;

    if let Some(err) = build_error {
        env.remove_last();
        return Err(err);
    }

    globals.insert(d_name.to_string(), d_id);

    let ctor_ids: Vec<GlobalId> = env
        .inductive(d_id)
        .ok_or_else(|| ElabError::Internal("inductive not found after declare".into()))?
        .constructors
        .iter()
        .map(|c| c.id)
        .collect();

    for (i, c) in ctors.iter().enumerate() {
        globals.insert(c.name.clone(), ctor_ids[i]);
    }

    Ok(d_id)
}

#[allow(clippy::too_many_arguments)]
fn build_explicit_inductive_spec(
    d_id: GlobalId,
    d_name: &str,
    param_count: usize,
    index_count: usize,
    params: &[Term],
    indices: &[Term],
    level: Level,
    ctors: &[RExplicitCtorDecl],
    globals: &HashMap<String, GlobalId>,
    ind_ids: &HashSet<GlobalId>,
    ctor_ids: &HashSet<GlobalId>,
) -> Result<InductiveSpec, ElabError> {
    let constructors = ctors
        .iter()
        .map(|c| {
            let args = c
                .args
                .iter()
                .map(|arg| {
                    rtype_to_kernel_checked(&arg.ty, d_name, d_id, globals, ind_ids, ctor_ids)
                })
                .collect::<Result<Vec<_>, _>>()?;

            let target_indices = match &c.result {
                Some(result) => {
                    let result_term =
                        rtype_to_kernel_checked(result, d_name, d_id, globals, ind_ids, ctor_ids)?;
                    validate_ctor_result_target(
                        &result_term,
                        d_id,
                        d_name,
                        param_count,
                        index_count,
                        &args,
                        c,
                    )?
                }
                None => {
                    if index_count == 0 {
                        vec![]
                    } else {
                        return Err(bad_ctor_target(
                            c,
                            d_name,
                            "simple constructor sugar cannot target an indexed family",
                        ));
                    }
                }
            };

            Ok(CtorSpec {
                args,
                target_indices,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(InductiveSpec {
        level_params: vec![],
        params: params.to_vec(),
        indices: indices.to_vec(),
        level,
        constructors,
    })
}

fn validate_ctor_result_target(
    result: &Term,
    d_id: GlobalId,
    d_name: &str,
    param_count: usize,
    index_count: usize,
    ctor_args: &[Term],
    ctor: &RExplicitCtorDecl,
) -> Result<Vec<Term>, ElabError> {
    let (head, args) = peel_app(result);
    match head {
        Term::IndFormer { id, .. } if id == d_id => {}
        _ => {
            return Err(bad_ctor_target(
                ctor,
                d_name,
                "result target does not expose the declared family head",
            ))
        }
    }

    let expected_args = param_count + index_count;
    if args.len() != expected_args {
        return Err(bad_ctor_target(
            ctor,
            d_name,
            "result target has the wrong number of parameters or indices",
        ));
    }

    let ctor_arg_count = ctor_args.len();
    for j in 0..param_count {
        let expected = Term::var(ctor_arg_count + param_count - 1 - j);
        if args[j] != expected {
            return Err(bad_ctor_target(
                ctor,
                d_name,
                "result target changes a data-head parameter",
            ));
        }
    }

    Ok(args[param_count..].to_vec())
}

fn bad_ctor_target(ctor: &RExplicitCtorDecl, d_name: &str, reason: &str) -> ElabError {
    ElabError::TypeMismatch {
        span: ctor.span.clone(),
        reason: format!(
            "bad constructor result target for '{}': expected family head '{}'; {}",
            ctor.name, d_name, reason
        ),
    }
}

fn peel_app(term: &Term) -> (Term, Vec<Term>) {
    let mut head = term.clone();
    let mut args = Vec::new();
    while let Term::App(f, a) = head {
        args.push(*a);
        head = *f;
    }
    args.reverse();
    (head, args)
}

fn level_from_nat(n: u32) -> Level {
    let mut level = Level::Zero;
    for _ in 0..n {
        level = Level::Suc(Box::new(level));
    }
    level
}

#[derive(Clone)]
struct UniverseArgument {
    constructor: String,
    name: Option<String>,
    index: usize,
    span: Span,
}

#[allow(clippy::too_many_arguments)]
fn translate_legacy_kernel_error(
    env: &GlobalEnv,
    error: KernelError,
    d_name: &str,
    params: &[Term],
    ctors: &[RCtorDecl],
    globals: &HashMap<String, GlobalId>,
    ind_ids: &HashSet<GlobalId>,
    ctor_ids: &HashSet<GlobalId>,
    decl_span: &Span,
) -> ElabError {
    match error {
        KernelError::ConstructorUniverseViolation { argument, family } => {
            let located = locate_legacy_universe_argument(
                env, d_name, params, ctors, globals, ind_ids, ctor_ids, &argument,
            );
            surface_universe_error(d_name, located, argument, family, decl_span)
        }
        error => ElabError::KernelRejected {
            error,
            span: decl_span.clone(),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn translate_explicit_kernel_error(
    env: &GlobalEnv,
    error: KernelError,
    d_name: &str,
    params: &[Term],
    indices: &[Term],
    family_level: &Level,
    ctors: &[RExplicitCtorDecl],
    globals: &HashMap<String, GlobalId>,
    ind_ids: &HashSet<GlobalId>,
    ctor_ids: &HashSet<GlobalId>,
    decl_span: &Span,
) -> ElabError {
    match error {
        KernelError::ConstructorUniverseViolation { argument, family } => {
            let located = locate_explicit_universe_argument(
                env,
                d_name,
                params,
                indices,
                family_level,
                ctors,
                globals,
                ind_ids,
                ctor_ids,
                &argument,
            );
            surface_universe_error(d_name, located, argument, family, decl_span)
        }
        error => ElabError::KernelRejected {
            error,
            span: decl_span.clone(),
        },
    }
}

fn surface_universe_error(
    d_name: &str,
    located: Option<UniverseArgument>,
    argument_level: Level,
    family_level: Level,
    decl_span: &Span,
) -> ElabError {
    match located {
        Some(located) => ElabError::ConstructorUniverseViolation {
            data: d_name.to_string(),
            constructor: located.constructor,
            argument_name: located.name,
            argument_index: located.index,
            argument_level,
            family_level,
            span: located.span,
        },
        None => ElabError::KernelRejected {
            error: KernelError::ConstructorUniverseViolation {
                argument: argument_level,
                family: family_level,
            },
            span: decl_span.clone(),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn locate_legacy_universe_argument(
    env: &GlobalEnv,
    d_name: &str,
    params: &[Term],
    ctors: &[RCtorDecl],
    globals: &HashMap<String, GlobalId>,
    ind_ids: &HashSet<GlobalId>,
    ctor_ids: &HashSet<GlobalId>,
    target_level: &Level,
) -> Option<UniverseArgument> {
    let (probe_env, d_id) = family_probe(env, params, &[], Level::Zero)?;
    let mut fallback = None;
    for ctor in ctors {
        let mut ctx = Context::new();
        ctx.extend_tel(params);
        for (index, rty) in ctor.args.iter().enumerate() {
            let located = UniverseArgument {
                constructor: ctor.name.clone(),
                name: ctor
                    .field_labels
                    .as_ref()
                    .and_then(|names| names.get(index).cloned()),
                index,
                span: rty.span().clone(),
            };
            fallback.get_or_insert_with(|| located.clone());
            let term = rtype_to_kernel_checked(rty, d_name, d_id, globals, ind_ids, ctor_ids)
                .ok()
                .map(|raw| weaken(&raw, index as i64));
            if let Some(term) = term {
                if inferred_sort_level(&probe_env, &ctx, &term)
                    .is_some_and(|level| level_eq(&level, target_level))
                {
                    return Some(located);
                }
                ctx.push(term);
            }
        }
    }
    fallback
}

#[allow(clippy::too_many_arguments)]
fn locate_explicit_universe_argument(
    env: &GlobalEnv,
    d_name: &str,
    params: &[Term],
    indices: &[Term],
    family_level: &Level,
    ctors: &[RExplicitCtorDecl],
    globals: &HashMap<String, GlobalId>,
    ind_ids: &HashSet<GlobalId>,
    ctor_ids: &HashSet<GlobalId>,
    target_level: &Level,
) -> Option<UniverseArgument> {
    let (probe_env, d_id) = family_probe(env, params, indices, family_level.clone())?;
    let mut fallback = None;
    for ctor in ctors {
        let mut ctx = Context::new();
        ctx.extend_tel(params);
        for (index, arg) in ctor.args.iter().enumerate() {
            let located = UniverseArgument {
                constructor: ctor.name.clone(),
                name: arg.name.clone(),
                index,
                span: arg.span.clone(),
            };
            fallback.get_or_insert_with(|| located.clone());
            if let Ok(term) =
                rtype_to_kernel_checked(&arg.ty, d_name, d_id, globals, ind_ids, ctor_ids)
            {
                if inferred_sort_level(&probe_env, &ctx, &term)
                    .is_some_and(|level| level_eq(&level, target_level))
                {
                    return Some(located);
                }
                ctx.push(term);
            }
        }
    }
    fallback
}

/// Recreate only the already-accepted family head in a cloned environment so
/// diagnostic attribution can classify each surface argument in the same
/// context as the kernel gate. This runs only after the real admission has
/// rejected and rolled back; it neither decides acceptance nor mutates `env`.
fn family_probe(
    env: &GlobalEnv,
    params: &[Term],
    indices: &[Term],
    level: Level,
) -> Option<(GlobalEnv, GlobalId)> {
    let mut probe = env.clone();
    let params = params.to_vec();
    let indices = indices.to_vec();
    let id = declare_inductive(&mut probe, |_| InductiveSpec {
        level_params: vec![],
        params,
        indices,
        level,
        constructors: vec![],
    })
    .ok()?;
    Some((probe, id))
}

fn inferred_sort_level(env: &GlobalEnv, ctx: &Context, term: &Term) -> Option<Level> {
    match whnf(env, ctx, &infer(env, ctx, term).ok()?) {
        Term::Type(level) | Term::Omega(level) => Some(level),
        _ => None,
    }
}

fn rtype_to_kernel_checked(
    rty: &RType,
    d_name: &str,
    d_id: GlobalId,
    globals: &HashMap<String, GlobalId>,
    ind_id_set: &HashSet<GlobalId>,
    ctor_id_set: &HashSet<GlobalId>,
) -> Result<Term, ElabError> {
    match rty {
        RType::RCon(name, span) => {
            if name == d_name {
                Ok(Term::IndFormer {
                    id: d_id,
                    level_args: vec![],
                })
            } else if let Some(&id) = globals.get(name) {
                if ind_id_set.contains(&id) {
                    Ok(Term::IndFormer {
                        id,
                        level_args: vec![],
                    })
                } else if ctor_id_set.contains(&id) {
                    Ok(Term::Constructor {
                        id,
                        level_args: vec![],
                    })
                } else {
                    Ok(Term::const_(id, vec![]))
                }
            } else {
                Err(ElabError::UnresolvedCon {
                    name: name.clone(),
                    span: span.clone(),
                })
            }
        }

        RType::RVarTy(i, _, _) => Ok(Term::var(*i)),

        RType::RArr(a, b, _) | RType::REffectArr(a, _, b, _) => {
            let a_k = rtype_to_kernel_checked(a, d_name, d_id, globals, ind_id_set, ctor_id_set)?;
            let b_k = rtype_to_kernel_checked(b, d_name, d_id, globals, ind_id_set, ctor_id_set)?;
            Ok(Term::pi(a_k, weaken(&b_k, 1)))
        }

        RType::RPi(_, a, b, _) => {
            let a_k = rtype_to_kernel_checked(a, d_name, d_id, globals, ind_id_set, ctor_id_set)?;
            let b_k = rtype_to_kernel_checked(b, d_name, d_id, globals, ind_id_set, ctor_id_set)?;
            Ok(Term::pi(a_k, b_k))
        }

        RType::RApp(f, a, _) => {
            let f_k = rtype_to_kernel_checked(f, d_name, d_id, globals, ind_id_set, ctor_id_set)?;
            let a_k = rtype_to_kernel_checked(a, d_name, d_id, globals, ind_id_set, ctor_id_set)?;
            Ok(Term::app(f_k, a_k))
        }

        RType::RUniv(None, _) => Ok(Term::ty(Level::Zero)),
        RType::RUniv(Some(n), _) => Ok(Term::ty(level_from_nat(*n))),

        RType::RRefine(_, carrier, _, _) => {
            rtype_to_kernel_checked(carrier, d_name, d_id, globals, ind_id_set, ctor_id_set)
        }
    }
}
