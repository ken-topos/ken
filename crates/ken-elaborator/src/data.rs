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

use ken_kernel::{
    declare_inductive, CtorSpec, GlobalEnv, GlobalId, InductiveSpec, Level, Term,
};
use ken_kernel::subst::weaken;

use crate::error::{ElabError, Span};
use crate::resolve::{RCtorDecl, RType};

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

    // Params: m params each of type `Type 0` (left-to-right, innermost-last in the Π-chain).
    let params: Vec<Term> = (0..m).map(|_| Term::ty(Level::Zero)).collect();

    // Clone ctors so the closure can take ownership.
    let ctors_owned = ctors.to_vec();
    let d_name_owned = d_name.to_string();

    let d_id =
        declare_inductive(env, |d_id| {
            let ctor_specs = ctors_owned
                .iter()
                .map(|c| {
                    let args = c
                        .args
                        .iter()
                        .enumerate()
                        .map(|(j, rty)| {
                            // rtype_to_kernel gives the type in context [Δ_p] only.
                            // Arg j is in context [Δ_p, arg₀…argⱼ₋₁], so weaken by j.
                            let raw = rtype_to_kernel(
                                rty,
                                &d_name_owned,
                                d_id,
                                &global_info,
                                &ind_id_set,
                            );
                            weaken(&raw, j as i64)
                        })
                        .collect();
                    CtorSpec {
                        args,
                        // Non-indexed families: all constructors target the empty index.
                        target_indices: vec![],
                    }
                })
                .collect();
            InductiveSpec {
                level_params: vec![],
                params,
                indices: vec![],
                level: Level::Zero,
                constructors: ctor_specs,
            }
        })
        .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;

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

/// Convert a resolved type to a kernel `Term` for use inside `declare_inductive`.
///
/// `d_name` / `d_id`: the type being declared — self-references become `IndFormer(d_id)`.
/// `globals` / `ind_id_set`: snapshot of the environment before declaration.
fn rtype_to_kernel(
    rty: &RType,
    d_name: &str,
    d_id: GlobalId,
    globals: &HashMap<String, GlobalId>,
    ind_id_set: &HashSet<GlobalId>,
) -> Term {
    match rty {
        RType::RCon(name, _) => {
            if name == d_name {
                Term::IndFormer { id: d_id, level_args: vec![] }
            } else if let Some(&id) = globals.get(name) {
                if ind_id_set.contains(&id) {
                    Term::IndFormer { id, level_args: vec![] }
                } else {
                    Term::const_(id, vec![])
                }
            } else {
                // Unknown name — produce a type-level placeholder.
                Term::ty(Level::Zero)
            }
        }

        RType::RVarTy(i, _, _) => Term::var(*i),

        RType::RArr(a, b, _) | RType::REffectArr(a, _, b, _) => {
            let a_k = rtype_to_kernel(a, d_name, d_id, globals, ind_id_set);
            let b_k = rtype_to_kernel(b, d_name, d_id, globals, ind_id_set);
            // Non-dependent arrow A → B: in kernel Π representation, B lives
            // under one binder, so weaken b_k by 1.
            Term::pi(a_k, weaken(&b_k, 1))
        }

        RType::RPi(_, a, b, _) => {
            let a_k = rtype_to_kernel(a, d_name, d_id, globals, ind_id_set);
            let b_k = rtype_to_kernel(b, d_name, d_id, globals, ind_id_set);
            Term::pi(a_k, b_k)
        }

        RType::RApp(f, a, _) => {
            let f_k = rtype_to_kernel(f, d_name, d_id, globals, ind_id_set);
            let a_k = rtype_to_kernel(a, d_name, d_id, globals, ind_id_set);
            Term::app(f_k, a_k)
        }

        RType::RUniv(None, _) => Term::ty(Level::Zero),
        RType::RUniv(Some(n), _) => {
            let mut l = Level::Zero;
            for _ in 0..*n {
                l = Level::Suc(Box::new(l));
            }
            Term::ty(l)
        }

        // Refinement in a ctor arg position: lower to carrier (`34 §7`).
        RType::RRefine(_, carrier, _, _) => {
            rtype_to_kernel(carrier, d_name, d_id, globals, ind_id_set)
        }
    }
}
