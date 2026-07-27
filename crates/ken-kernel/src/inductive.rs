//! Inductive families — strict-positivity admission and the dependent
//! eliminator (`14-inductive.md`).
//!
//! Three loads:
//! - [`check_positivity`] — the strict-positivity check (`14 §8`), the
//!   **fixed** algorithm with `occurs`-guards on every position that could
//!   discard a subterm (application arguments `C u`, recursive-occurrence
//!   indices `D Δ_p t̄`, type parameters `X`). This is the soundness hole the
//!   Architect's review caught (`Bad3`/`Bad4`); the guards conservatively
//!   reject what K1 cannot prove strictly positive (`14 §8.4`).
//! - [`method_type`] — the dependent eliminator's per-constructor method type
//!   `Π Δₖ. Π (IHs). M t̄ₖ (cₖ p̄ Δₖ)`, computed from the family declaration and
//!   the concrete motive/params at a use site (`14 §3`, `14 §3.1`). W-style
//!   recursive args (`(b:B) → D Δ_p t̄[b]`) get a Π-abstracted IH
//!   `(b:B) → M t̄[b] (k b)` (K1.5).
//! - [`iota_reduct`] — the algorithmic ι-step `elim_D … (cₖ p̄ ā) ⇝ mₖ ā [IHs]`
//!   (`14 §7.3`, `14 §7.7`), capture-avoiding, with induction hypotheses on
//!   structurally smaller recursive arguments. W-style args produce a
//!   λ-abstracted IH `λb. elim_D … (k b)` (K1.5).
//!
//! **K1.5**: W-style (Π-bound) recursive arguments `(b:B) → D Δ_p t̄[b]` are
//! now **admitted** (`14 §2.1`, `14 §8.4`). The separate blanket gate
//! `check_no_pi_bound_recursive` is retired; strict positivity (`14 §8.2`) is
//! the sole structural admission test. The eliminator and ι handle the
//! Π-abstracted IH and the λ-threaded recursive call (`14 §3.1`, `14 §7.7`).

use crate::conv::{convert_type, normalize};
use crate::env::{ConstructorDecl, Context, GlobalEnv, InductiveDecl, ParameterPolarity};
use crate::error::{KernelError, KernelResult};
use crate::subst::{apply_args, shift, subst_levels, subst_outer, subst_tel, weaken};
use crate::term::{GlobalId, Level, Term};

/// Does the inductive former `d` occur anywhere in `t` (syntactic sub-term)?
/// Used by the positivity guards (`14 §8`). de Bruijn indices make this
/// unambiguous: a former is a `Term::IndFormer { id, .. }` node.
pub fn occurs(d: GlobalId, t: &Term) -> bool {
    match t {
        Term::IndFormer { id, .. } => *id == d,
        _ => t.children().iter().any(|c| occurs(d, c)),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pol {
    Plus,
    Minus,
    Unknown,
}

impl Pol {
    fn flip(self) -> Pol {
        match self {
            Pol::Plus => Pol::Minus,
            Pol::Minus => Pol::Plus,
            Pol::Unknown => Pol::Unknown,
        }
    }
}

/// Peel a left-nested `App` spine into `(head, args)` left-to-right.
pub fn peel_app(t: &Term) -> (Term, Vec<Term>) {
    let mut args = Vec::new();
    let mut cur = t.clone();
    while let Term::App(f, a) = cur {
        args.push((*a).clone());
        cur = (*f).clone();
    }
    args.reverse();
    (cur, args)
}

/// Peel leading `Pi` binders into `(binder_domain_types, body)`.
pub fn peel_pi(t: &Term) -> (Vec<Term>, Term) {
    let mut doms = Vec::new();
    let mut cur = t.clone();
    while let Term::Pi(a, b) = cur {
        doms.push((*a).clone());
        cur = (*b).clone();
    }
    (doms, cur)
}

/// `check-pos-arg(D, pol, A)` — the strict-positivity judgment (`14 §8.2`).
///
/// Returns `true` if `A` is strictly positive in `D` at polarisation `pol`.
/// Every position that would discard a subterm without inspection is guarded
/// by an `occurs` check (the fixed algorithm): application arguments, the
/// indices of a recursive occurrence, and bare type parameters.
fn check_pos_arg(d: GlobalId, pol: Pol, a: &Term) -> bool {
    match a {
        Term::Pi(dom, cod) => check_pos_arg(d, pol.flip(), dom) && check_pos_arg(d, pol, cod),
        Term::Sigma(dom, cod) => check_pos_arg(d, pol, dom) && check_pos_arg(d, pol, cod),
        Term::App(_, _) => {
            // `C u` (or `D Δ_p t̄` if the head is `D`).
            let (head, args) = peel_app(a);
            match head {
                Term::IndFormer { id, .. } if id == d => {
                    // Recursive occurrence `D Δ_p t̄`: positive polarity, and
                    // `D` must not occur in the (index) arguments.
                    pol == Pol::Plus && args.iter().all(|x| !occurs(d, x))
                }
                Term::IndFormer { .. }
                | Term::Const { .. }
                | Term::Constructor { .. }
                | Term::Var(_) => {
                    // `C u` with a non-`D` head: recurse into the (atomic) head
                    // and `occurs`-guard every argument.
                    check_pos_arg(d, pol, &head) && args.iter().all(|x| !occurs(d, x))
                }
                Term::Type(_) => {
                    // `Type ℓ` applied is ill-formed as a type; conservatively
                    // reject if `D` lurks anywhere.
                    args.is_empty() || !occurs(d, a)
                }
                _ => {
                    // Pi/Sigma/Lam/... applied: ill-formed; conservative reject.
                    !occurs(d, a)
                }
            }
        }
        Term::Type(_) => true, // `Type ℓ`; `D` is a type, not a level.
        Term::IndFormer { id, .. } if *id == d => {
            // Bare `D` (no arguments) — recursive occurrence with empty indices.
            pol == Pol::Plus
        }
        Term::IndFormer { .. } | Term::Const { .. } | Term::Constructor { .. } | Term::Var(_) => {
            // Bare `X` — a parameter or other type; reject if `D` occurs within.
            !occurs(d, a)
        }
        // Anything else as a type is ill-formed; conservatively reject if D hides.
        _ => !occurs(d, a),
    }
}

fn parameter_index(
    var: usize,
    local_depth: usize,
    prior_constructor_args: usize,
    parameter_count: usize,
) -> Option<usize> {
    let cutoff = local_depth.checked_add(prior_constructor_args)?;
    let relative = var.checked_sub(cutoff)?;
    if relative >= parameter_count {
        return None;
    }
    Some(parameter_count - 1 - relative)
}

struct ParameterPolarityDeriver<'a> {
    d: GlobalId,
    parameter_count: usize,
    prior_constructor_args: usize,
    positive: &'a mut [bool],
}

impl ParameterPolarityDeriver<'_> {
    fn visit(&mut self, local_depth: usize, pol: Pol, term: &Term) {
        match term {
            Term::Pi(dom, cod) => {
                self.visit(local_depth, pol.flip(), dom);
                self.visit(local_depth + 1, pol, cod);
            }
            Term::Sigma(dom, cod) => {
                self.visit(local_depth, pol, dom);
                self.visit(local_depth + 1, pol, cod);
            }
            Term::Lam(dom, body) => {
                self.visit(local_depth, Pol::Unknown, dom);
                self.visit(local_depth + 1, Pol::Unknown, body);
            }
            Term::Let { ty, val, body } => {
                self.visit(local_depth, Pol::Unknown, ty);
                self.visit(local_depth, Pol::Unknown, val);
                self.visit(local_depth + 1, Pol::Unknown, body);
            }
            Term::App(_, _) => {
                let (head, args) = peel_app(term);
                if matches!(head, Term::IndFormer { id, .. } if id == self.d) {
                    for arg in &args {
                        self.visit(local_depth, pol, arg);
                    }
                } else {
                    // D1a does not open structural traversal through another
                    // former. Until D1b supplies that rule, every such
                    // argument is unknown and therefore fails closed.
                    self.visit(local_depth, Pol::Unknown, &head);
                    for arg in &args {
                        self.visit(local_depth, Pol::Unknown, arg);
                    }
                }
            }
            Term::Var(var) => {
                let parameter = parameter_index(
                    *var,
                    local_depth,
                    self.prior_constructor_args,
                    self.parameter_count,
                );
                if let Some(parameter) = parameter {
                    if pol != Pol::Plus {
                        self.positive[parameter] = false;
                    }
                }
            }
            Term::Type(_)
            | Term::Omega(_)
            | Term::IndFormer { .. }
            | Term::Const { .. }
            | Term::Constructor { .. } => {}
            _ => {
                // Pi, Sigma, Lam, and Let are Term's complete binder set and
                // have depth-aware arms above. Every remaining child stays at
                // the current depth.
                // Any parameter below a type form not covered by the D1a
                // grammar is unknown, hence not declared strictly positive.
                for child in term.children() {
                    self.visit(local_depth, Pol::Unknown, child);
                }
            }
        }
    }
}

/// Derive the fail-closed polarity record for an inductive family's parameters.
///
/// D1a deliberately does not traverse applications of other type formers:
/// those positions remain unknown until the D1b structural rule lands.
pub fn derive_parameter_polarities(ind: &InductiveDecl) -> Vec<ParameterPolarity> {
    let mut positive = vec![true; ind.params.len()];
    for constructor in &ind.constructors {
        for (argument, term) in constructor.args.iter().enumerate() {
            ParameterPolarityDeriver {
                d: ind.id,
                parameter_count: ind.params.len(),
                prior_constructor_args: argument,
                positive: &mut positive,
            }
            .visit(0, Pol::Plus, term);
        }
    }
    positive
        .into_iter()
        .map(|is_positive| {
            if is_positive {
                ParameterPolarity::StrictlyPositive
            } else {
                ParameterPolarity::NonPositive
            }
        })
        .collect()
}

/// Run the strict-positivity check on a family declaration (`14 §8`): every
/// constructor argument type must be strictly positive in `D`. The family's
/// own parameters, indices, and each constructor's result target indices are
/// also `occurs`-checked (K1 rejects `D` appearing in its own indices, `Bad4`,
/// and nested parameter occurrences).
pub fn check_positivity(ind: &InductiveDecl) -> KernelResult<()> {
    let d = ind.id;
    if ind.parameter_polarities.len() != ind.params.len() {
        return Err(KernelError::PositivityViolation(
            "parameter polarity record does not match the parameter telescope".into(),
        ));
    }
    let derived = derive_parameter_polarities(ind);
    if ind
        .parameter_polarities
        .iter()
        .zip(derived)
        .any(|(recorded, actual)| recorded != &actual)
    {
        return Err(KernelError::PositivityViolation(
            "recorded parameter polarity does not match the declaration".into(),
        ));
    }
    for p in &ind.params {
        if occurs(d, p) {
            return Err(KernelError::PositivityViolation(
                "D occurs in its own parameter telescope".into(),
            ));
        }
    }
    for ix in &ind.indices {
        if occurs(d, ix) {
            return Err(KernelError::PositivityViolation(
                "D occurs in its own index telescope".into(),
            ));
        }
    }
    for c in &ind.constructors {
        for (j, a) in c.args.iter().enumerate() {
            if !check_pos_arg(d, Pol::Plus, a) {
                return Err(KernelError::PositivityViolation(format!(
                    "non-strictly-positive occurrence of D in constructor {:?} arg {j}",
                    c.id
                )));
            }
        }
        for (j, ix) in c.target_indices.iter().enumerate() {
            if occurs(d, ix) {
                return Err(KernelError::PositivityViolation(format!(
                    "D occurs in constructor {:?} target index {j}",
                    c.id
                )));
            }
        }
    }
    Ok(())
}

/// The recursive arguments of a constructor: `(arg_position, branching_tel,
/// index_exprs)` for each arg whose type peels to `(b₁:B₁)...(b_{nb}:B_{nb})
/// → D Δ_p t̄` (K1.5, `14 §2.1`).
///
/// - `branching_tel` — the leading Π-binder domains `[B₁, B₂[b₁], ...]`
///   (empty for a direct `D Δ_p t̄`); each `B_k` is in context
///   `[Δ_p, args_before_pos, b₁..b_{k-1}]`.
/// - `index_exprs` — the index expressions after the family's `m` params, in
///   context `[Δ_p, args_before_pos, b₁..b_{nb}]` (under the branching binders).
pub fn recursive_args(
    c: &ConstructorDecl,
    d: GlobalId,
    m: usize,
) -> Vec<(usize, Vec<Term>, Vec<Term>)> {
    let mut out = Vec::new();
    for (j, a) in c.args.iter().enumerate() {
        let (pis, body) = peel_pi(a);
        let (head, args) = peel_app(&body);
        if let Term::IndFormer { id, .. } = head {
            if id == d && args.len() >= m {
                out.push((j, pis, args[m..].to_vec()));
            }
        }
    }
    out
}

/// One constructor argument whose type contains structurally recursive
/// occurrences of the family being described.
///
/// This is preparation for the nested eliminator of `14 §3.2`. It is
/// deliberately inert: [`method_type`] and [`iota_reduct`] continue to consume
/// [`recursive_args`], whose direct/Π-bound result is unchanged. The first
/// semantic consumer of this descriptor must land atomically with nested ι.
#[derive(Clone, Debug)]
pub struct RecursiveArgumentShape {
    pub position: usize,
    /// The complete constructor field type from which `shape` was derived.
    ///
    /// Descriptor equivalence compares this term with kernel conversion, so
    /// retained subterms are interpreted in their original dependent context
    /// rather than compared as Rust syntax.
    pub field_type: Term,
    /// Constructor fields preceding this one, in telescope order.
    ///
    /// These extend the caller-supplied parameter context when comparing the
    /// dependent `field_type`.
    pub prior_field_types: Vec<Term>,
    pub shape: RecursiveShape,
}

/// Structural recipe for lifting a motive through one recursive argument.
///
/// The variants exhaust the positive recursive shapes in the core grammar:
/// direct occurrences, Π-bound/W-style occurrences, primitive dependent Σ,
/// and applications of an admitted former through checked positive parameter
/// positions. Field terms are normalized through the kernel's established
/// terminating δ+β semantics before this structural classification.
/// Definitional identity is exposed by [`recursive_shapes_equivalent`], not
/// Rust structural equality: conversion also includes semantic level equality
/// and type-directed rules that do not have a unique [`Term`] representative.
/// A D-free field contributes no [`RecursiveArgumentShape`].
#[derive(Clone, Debug)]
pub enum RecursiveShape {
    /// `D Δ_p t̄`: one motive leaf, indexed by `t̄`.
    Direct { index_exprs: Vec<Term> },
    /// `(b₁:B₁)…(bₙ:Bₙ) → A`: preserve the branching telescope and lift `A`.
    Pi {
        domains: Vec<Term>,
        body: Box<RecursiveShape>,
    },
    /// `(x:A) × B`: preserve dependent Sigma topology. A D-free component is
    /// `None`; a recursive component retains its complete nested shape.
    Sigma {
        domain: Option<Box<RecursiveShape>>,
        codomain: Option<Box<RecursiveShape>>,
    },
    /// `F a₁…aₙ`: preserve the former and application spine. Each argument
    /// records its original term plus a lift only when its checked parameter
    /// position contains recursive content.
    Former {
        former: GlobalId,
        level_args: Vec<Level>,
        arguments: Vec<RecursiveFormerArgument>,
    },
}

/// One argument in a [`RecursiveShape::Former`] application spine.
#[derive(Clone, Debug)]
pub struct RecursiveFormerArgument {
    pub term: Term,
    pub shape: Option<Box<RecursiveShape>>,
}

impl RecursiveShape {
    fn same_topology(&self, other: &RecursiveShape) -> bool {
        match (self, other) {
            (
                RecursiveShape::Direct { index_exprs: left },
                RecursiveShape::Direct { index_exprs: right },
            ) => left.len() == right.len(),
            (
                RecursiveShape::Pi {
                    domains: left_domains,
                    body: left_body,
                },
                RecursiveShape::Pi {
                    domains: right_domains,
                    body: right_body,
                },
            ) => left_domains.len() == right_domains.len() && left_body.same_topology(right_body),
            (
                RecursiveShape::Sigma {
                    domain: left_domain,
                    codomain: left_codomain,
                },
                RecursiveShape::Sigma {
                    domain: right_domain,
                    codomain: right_codomain,
                },
            ) => {
                optional_topology_eq(left_domain, right_domain)
                    && optional_topology_eq(left_codomain, right_codomain)
            }
            (
                RecursiveShape::Former {
                    former: left_former,
                    level_args: left_levels,
                    arguments: left_arguments,
                },
                RecursiveShape::Former {
                    former: right_former,
                    level_args: right_levels,
                    arguments: right_arguments,
                },
            ) => {
                left_former == right_former
                    && level_spines_equivalent(left_levels, right_levels)
                    && left_arguments.len() == right_arguments.len()
                    && left_arguments.iter().zip(right_arguments).all(
                        |(left_argument, right_argument)| {
                            optional_topology_eq(&left_argument.shape, &right_argument.shape)
                        },
                    )
            }
            _ => false,
        }
    }

    /// Number of syntactic motive leaves represented by this recipe.
    ///
    /// Runtime multiplicity is supplied by the containing value's topology:
    /// e.g. the single leaf recipe under `List` occurs once per list element.
    pub fn leaf_count(&self) -> usize {
        match self {
            RecursiveShape::Direct { .. } => 1,
            RecursiveShape::Pi { body, .. } => body.leaf_count(),
            RecursiveShape::Sigma { domain, codomain } => domain
                .iter()
                .chain(codomain)
                .map(|shape| shape.leaf_count())
                .sum(),
            RecursiveShape::Former { arguments, .. } => arguments
                .iter()
                .filter_map(|argument| argument.shape.as_deref())
                .map(RecursiveShape::leaf_count)
                .sum(),
        }
    }

    /// Project the legacy direct/Π-bound class consumed by the landed
    /// eliminator. Structured Sigma/former recipes intentionally return
    /// `None` until method generation and ι land together.
    pub fn as_legacy(&self) -> Option<(Vec<Term>, Vec<Term>)> {
        match self {
            RecursiveShape::Direct { index_exprs } => Some((Vec::new(), index_exprs.clone())),
            RecursiveShape::Pi { domains, body } => {
                let (mut inner_domains, index_exprs) = body.as_legacy()?;
                let mut all_domains = domains.clone();
                all_domains.append(&mut inner_domains);
                Some((all_domains, index_exprs))
            }
            RecursiveShape::Sigma { .. } | RecursiveShape::Former { .. } => None,
        }
    }
}

fn level_spines_equivalent(left: &[Level], right: &[Level]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.equiv(right))
}

fn optional_topology_eq(
    left: &Option<Box<RecursiveShape>>,
    right: &Option<Box<RecursiveShape>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left.same_topology(right),
        _ => false,
    }
}

enum ShapeDerivation {
    DFree,
    Recursive(RecursiveShape),
}

fn unsupported_recursive_shape(message: impl Into<String>) -> KernelError {
    KernelError::PositivityViolation(message.into())
}

fn derive_recursive_shape(
    env: &GlobalEnv,
    term: &Term,
    d: GlobalId,
    parameter_count: usize,
) -> KernelResult<ShapeDerivation> {
    let normalized = normalize(env, &Context::new(), term);
    let term = &normalized;
    if !occurs(d, term) {
        return Ok(ShapeDerivation::DFree);
    }

    match term {
        Term::Pi(_, _) => {
            let (domains, body) = peel_pi(term);
            if domains.iter().any(|domain| occurs(d, domain)) {
                return Err(unsupported_recursive_shape(
                    "recursive occurrence in a Pi domain is not positive",
                ));
            }
            match derive_recursive_shape(env, &body, d, parameter_count)? {
                ShapeDerivation::DFree => Err(unsupported_recursive_shape(
                    "Pi field contains an unclassified recursive occurrence",
                )),
                ShapeDerivation::Recursive(body) => {
                    Ok(ShapeDerivation::Recursive(RecursiveShape::Pi {
                        domains,
                        body: Box::new(body),
                    }))
                }
            }
        }
        Term::Sigma(domain, codomain) => {
            let domain = match derive_recursive_shape(env, domain, d, parameter_count)? {
                ShapeDerivation::DFree => None,
                ShapeDerivation::Recursive(shape) => Some(Box::new(shape)),
            };
            let codomain = match derive_recursive_shape(env, codomain, d, parameter_count)? {
                ShapeDerivation::DFree => None,
                ShapeDerivation::Recursive(shape) => Some(Box::new(shape)),
            };
            Ok(ShapeDerivation::Recursive(RecursiveShape::Sigma {
                domain,
                codomain,
            }))
        }
        Term::App(_, _) | Term::IndFormer { .. } => {
            let (head, arguments) = peel_app(term);
            match head {
                Term::IndFormer { id, level_args: _ } if id == d => {
                    if arguments.len() < parameter_count
                        || arguments.iter().any(|argument| occurs(d, argument))
                    {
                        return Err(unsupported_recursive_shape(
                            "recursive family parameters or indices contain the family",
                        ));
                    }
                    Ok(ShapeDerivation::Recursive(RecursiveShape::Direct {
                        index_exprs: arguments[parameter_count..].to_vec(),
                    }))
                }
                Term::IndFormer { id, level_args } => {
                    let former = env.inductive(id).ok_or_else(|| {
                        unsupported_recursive_shape(
                            "nested occurrence has no admitted former metadata",
                        )
                    })?;
                    if arguments.len() < former.params.len() {
                        return Err(unsupported_recursive_shape(
                            "nested former application is under-saturated",
                        ));
                    }

                    let mut shaped_arguments = Vec::with_capacity(arguments.len());
                    for (position, argument) in arguments.into_iter().enumerate() {
                        let shape = if occurs(d, &argument) {
                            if position >= former.params.len()
                                || former.parameter_polarities.get(position)
                                    != Some(&ParameterPolarity::StrictlyPositive)
                            {
                                return Err(unsupported_recursive_shape(
                                    "recursive occurrence is not in a checked positive parameter",
                                ));
                            }
                            match derive_recursive_shape(env, &argument, d, parameter_count)? {
                                ShapeDerivation::DFree => {
                                    return Err(unsupported_recursive_shape(
                                        "positive parameter lost a recursive occurrence",
                                    ))
                                }
                                ShapeDerivation::Recursive(shape) => Some(Box::new(shape)),
                            }
                        } else {
                            None
                        };
                        shaped_arguments.push(RecursiveFormerArgument {
                            term: argument,
                            shape,
                        });
                    }
                    Ok(ShapeDerivation::Recursive(RecursiveShape::Former {
                        former: id,
                        level_args,
                        arguments: shaped_arguments,
                    }))
                }
                Term::Const { .. } => Err(unsupported_recursive_shape(
                    "recursive occurrence has an opaque or unresolved application head",
                )),
                _ => Err(unsupported_recursive_shape(
                    "recursive occurrence has an unresolved application head",
                )),
            }
        }
        _ => Err(unsupported_recursive_shape(
            "recursive occurrence is in an unsupported type form",
        )),
    }
}

/// Describe every positive recursive shape in a constructor telescope.
///
/// Unlike [`recursive_args`], this preparatory API represents primitive Sigma
/// and checked-positive former nesting. It does not alter admission or
/// eliminator behavior; semantic consumers remain on the legacy projection
/// until method generation and ι are landed atomically.
pub fn recursive_shapes(
    env: &GlobalEnv,
    c: &ConstructorDecl,
    d: GlobalId,
    parameter_count: usize,
) -> KernelResult<Vec<RecursiveArgumentShape>> {
    let mut shapes = Vec::new();
    let mut prior_field_types = Vec::new();
    for (position, argument) in c.args.iter().enumerate() {
        if let ShapeDerivation::Recursive(shape) =
            derive_recursive_shape(env, argument, d, parameter_count)?
        {
            shapes.push(RecursiveArgumentShape {
                position,
                field_type: argument.clone(),
                prior_field_types: prior_field_types.clone(),
                shape,
            });
        }
        prior_field_types.push(argument.clone());
    }
    Ok(shapes)
}

/// Compare recursive descriptors in the kernel's definitional-equality
/// quotient.
///
/// Topology is compared structurally, including semantic [`Level::equiv`] for
/// former instantiations. Each complete source field is compared with
/// [`convert_type`], which checks every retained term in its original binder
/// structure using the context- and type-aware conversion relation. Callers
/// supply the context in which the constructor field telescope is interpreted.
pub fn recursive_shapes_equivalent(
    env: &GlobalEnv,
    ctx: &Context,
    left: &[RecursiveArgumentShape],
    right: &[RecursiveArgumentShape],
) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            let mut field_ctx = ctx.clone();
            field_ctx.extend_tel(&left.prior_field_types);
            left.position == right.position
                && left.shape.same_topology(&right.shape)
                && convert_type(env, &field_ctx, &left.field_type, &right.field_type)
        })
}

/// The dependent eliminator's method type for constructor `k`:
/// `Π Δₖ. Π (IH₁…IH_p). M t̄ₖ (cₖ p̄ ā)` (`14 §3`, `14 §3.1`), in the
/// caller's context Γ.
///
/// W-style recursive args `(b:B) → D Δ_p t̄[b]` get a Π-abstracted IH
/// `(b:B) → M t̄[b] (k b)` (K1.5, `14 §3.1`).
///
/// `motive` (`M`) and `params` (`p̄`) are the concrete motive and param
/// instance at the use site (terms in Γ); `level_args` instantiate the
/// family's level parameters (used in the constructor reference).
pub fn method_type(
    ind: &InductiveDecl,
    k: usize,
    motive: &Term,
    params: &[Term],
    level_args: &[Level],
) -> Term {
    let c = &ind.constructors[k];
    let m = ind.params.len();
    let n = c.args.len();
    let rec = recursive_args(c, ind.id, m);
    let p = rec.len();

    // Conclusion `M t̄ₖ (cₖ p̄ ā')` in context [Γ, a₁'..aₙ', ih₁..ih_p]
    // (depth ctx_depth + n + p, but ctx_depth is implicit — we build relative
    // to Γ by weakening Γ-terms past the n+p new binders).
    let np = (n + p) as i64;
    let m_w = weaken(motive, np);
    let tgt: Vec<Term> = c
        .target_indices
        .iter()
        .map(|t| {
            weaken(
                &subst_levels(&subst_outer(t, m, params, n), &ind.level_params, level_args),
                p as i64,
            )
        })
        .collect();
    let mut capp = Term::Constructor {
        id: c.id,
        level_args: level_args.to_vec(),
    };
    for p in params {
        capp = Term::app(capp, weaken(p, np)); // p̄ weakened past args+IHs
    }
    for j in 0..n {
        // a_{j+1}' is at index (p + n - 1 - j) in [Γ, args, ihs].
        capp = Term::app(capp, Term::var(p + n - 1 - j));
    }
    let mut conclusion = m_w;
    for t in &tgt {
        conclusion = Term::app(conclusion, t.clone());
    }
    conclusion = Term::app(conclusion, capp);

    // Wrap IH binders innermost-first (ih_p … ih_1).
    // Each IH may be:
    //   - Direct (nb=0): `M idxs a_pos` — a plain type.
    //   - W-style (nb≥1): `Π(b₁:B₁)...(b_{nb}:B_{nb}). M idxs (a_pos b₁..b_{nb})`
    //     — a Π-type over the branching telescope (`14 §3.1`).
    let mut ty = conclusion;
    for j in (0..p).rev() {
        let (pos, branching_tel, idxs) = &rec[j];
        let nb = branching_tel.len();
        // Context when building IH_j: [Γ, args, ih₁..ih_{j-1}] (depth n+j from Γ).
        // Inside the nb Π-binders of the IH: [Γ, args, ih₁..ih_{j-1}, b₁..b_{nb}].
        let m_w_body = weaken(motive, (n + j + nb) as i64);
        // Index exprs are in [Δ_p, args_before_pos, b₁..b_{nb}].
        // subst_outer replaces m params (inner_depth = pos+nb).
        // shift with cutoff=nb lifts args_before_pos and Γ vars past the n-pos
        // remaining args and j preceding IHs, while PRESERVING the branch binders
        // b₁..b_{nb} at indices 0..nb-1 (they are bound by the Π-wrap below).
        let idxs_in_body: Vec<Term> = idxs
            .iter()
            .map(|t| {
                shift(
                    &subst_levels(
                        &subst_outer(t, m, params, *pos + nb),
                        &ind.level_params,
                        level_args,
                    ),
                    (n - pos + j) as i64,
                    nb,
                )
            })
            .collect();
        // a_pos under nb extra binders: index n - 1 - pos + j + nb.
        // Apply to b₁..b_{nb}: Var(nb-1)=b₁, ..., Var(0)=b_{nb}.
        let mut scrut_body = Term::var(n - 1 - pos + j + nb);
        for bk in 0..nb {
            scrut_body = Term::app(scrut_body, Term::var(nb - 1 - bk));
        }
        // Assemble IH body: M idxs (a_pos b₁..b_{nb}).
        let mut ih_inner = m_w_body;
        for ix in &idxs_in_body {
            ih_inner = Term::app(ih_inner, ix.clone());
        }
        ih_inner = Term::app(ih_inner, scrut_body);
        // Wrap in Π-binders from innermost (B_{nb}) to outermost (B₁).
        // B_k is in [Δ_p, args_before_pos, b₁..b_{k-1}] (under bk extra binders;
        // branching_tel[bk] has bk Pi-binders above it from peel_pi).
        let mut ih_ty = ih_inner;
        for bk in (0..nb).rev() {
            // branching_tel[bk] is in context [Δ_p, args_before_pos, b₁..b_{bk}].
            // shift with cutoff=bk lifts args_before_pos and Γ vars while PRESERVING
            // b₁..b_{bk} at indices 0..bk-1 (bound by the Π-binders already added).
            let b_dom = shift(
                &subst_levels(
                    &subst_outer(&branching_tel[bk], m, params, *pos + bk),
                    &ind.level_params,
                    level_args,
                ),
                (n - pos + j) as i64,
                bk,
            );
            ih_ty = Term::pi(b_dom, ih_ty);
        }
        ty = Term::pi(ih_ty, ty);
    }

    // Wrap arg binders innermost-first (aₙ' … a₁').
    for j in (0..n).rev() {
        let a_ty = subst_levels(
            &subst_outer(&c.args[j], m, params, j),
            &ind.level_params,
            level_args,
        ); // in [Γ, a₁'..a_j']
        ty = Term::pi(a_ty, ty);
    }
    ty
}

/// The ι-reduct of an eliminator applied to a constructor-headed scrutinee
/// (`14 §7.3`): `elim_D p̄ M m̄ i̅ (cₖ p̄ ā) ⇝ mₖ ā [IHs]`.
///
/// `ctor_all_args` is the constructor's full argument spine `p̄ ++ ā` (params
/// then args), already peeled from the scrutinee. Returns the reduct, or an
/// error if the spine does not match the constructor's arity.
pub fn iota_reduct(
    ind: &InductiveDecl,
    k: usize,
    level_args: &[Level],
    params: &[Term],
    motive: &Term,
    methods: &[Term],
    ctor_all_args: &[Term],
) -> KernelResult<Term> {
    let c = &ind.constructors[k];
    let m = ind.params.len();
    let n = c.args.len();
    // Arity guards: `raw_wf` checks only scoping for an `Elim`, but `whnf` calls
    // `iota_reduct` on any constructor-headed scrutinee. A raw-well-formed
    // `Elim` with too few params/methods/level-args would index out of bounds
    // here — the kernel contract is yes/no, never a crash (`18 §4`).
    if params.len() != m {
        return Err(KernelError::BadEliminator(format!(
            "expected {m} params, got {}",
            params.len()
        )));
    }
    if methods.len() != ind.constructors.len() {
        return Err(KernelError::BadEliminator(format!(
            "expected {} methods, got {}",
            ind.constructors.len(),
            methods.len()
        )));
    }
    if level_args.len() != ind.level_params.len() {
        return Err(KernelError::BadEliminator(format!(
            "expected {} level args, got {}",
            ind.level_params.len(),
            level_args.len()
        )));
    }
    if ctor_all_args.len() != m + n {
        return Err(KernelError::BadEliminator(format!(
            "constructor {:?} arity mismatch: expected {} args, got {}",
            c.id,
            m + n,
            ctor_all_args.len()
        )));
    }
    let ctor_args = &ctor_all_args[m..]; // ā (the actual constructor args)
    let method = &methods[k];

    let rec = recursive_args(c, ind.id, m);
    // Induction hypotheses for each recursive arg (`14 §7.3`, `14 §7.7`):
    //   - Direct (nb=0):    `elim_D p̄ M m̄ idx(a_j) a_j`
    //   - W-style (nb≥1):  `λ(b₁:B₁)...(b_{nb}:B_{nb}). elim_D p̄ M m̄ idx(a_j b₁..b_{nb}) (a_j b₁..b_{nb})`
    let mut ihs: Vec<Term> = Vec::new();
    for (pos, branching_tel, idxs) in &rec {
        let a_j = &ctor_args[*pos];
        let nb = branching_tel.len();
        if nb == 0 {
            // Direct case: elim applied to a_j itself.
            let idx_vals: Vec<Term> = idxs
                .iter()
                .map(|t| {
                    subst_levels(
                        &subst_tel(&subst_outer(t, m, params, *pos), &ctor_args[..*pos]),
                        &ind.level_params,
                        level_args,
                    )
                })
                .collect();
            ihs.push(Term::Elim {
                fam: ind.id,
                level_args: level_args.to_vec(),
                params: params.to_vec(),
                motive: Box::new(motive.clone()),
                methods: methods.to_vec(),
                indices: idx_vals,
                scrut: Box::new(a_j.clone()),
            });
        } else {
            // W-style case: build λ(b₁:B₁)...(b_{nb}:B_{nb}). elim_D … (a_j b₁..b_{nb}).
            // Inside nb lambda binders, context extends by b₁..b_{nb}.
            // a_j weakened by nb to sit inside the binders.
            let a_j_inner = weaken(a_j, nb as i64);
            // a_j b₁ b₂ ... b_{nb}: b_k = Var(nb-1-k) under the lambdas.
            let mut scrut_inner = a_j_inner;
            for bk in 0..nb {
                scrut_inner = Term::app(scrut_inner, Term::var(nb - 1 - bk));
            }
            // Index vals in [Γ, b₁..b_{nb}]:
            // idxs[i] in [Δ_p, args_before_pos, b₁..b_{nb}]; subst_outer replaces
            // m params (inner_depth=pos+nb), then subst_tel substitutes pos args
            // (weakened by nb to sit inside the binders).
            let ctor_args_inner: Vec<Term> = ctor_args[..*pos]
                .iter()
                .map(|t| weaken(t, nb as i64))
                .collect();
            let idx_vals_inner: Vec<Term> = idxs
                .iter()
                .map(|t| {
                    subst_levels(
                        &subst_tel(&subst_outer(t, m, params, *pos + nb), &ctor_args_inner),
                        &ind.level_params,
                        level_args,
                    )
                })
                .collect();
            // Build the elim call inside the lambdas (all Γ-terms weakened by nb).
            let elim_inner = Term::Elim {
                fam: ind.id,
                level_args: level_args.to_vec(),
                params: params.iter().map(|p| weaken(p, nb as i64)).collect(),
                motive: Box::new(weaken(motive, nb as i64)),
                methods: methods.iter().map(|mth| weaken(mth, nb as i64)).collect(),
                indices: idx_vals_inner,
                scrut: Box::new(scrut_inner),
            };
            // Wrap in λ-binders from innermost (B_{nb}) to outermost (B₁).
            // B_k (branching_tel[bk]) in [Δ_p, args_before_pos, b₁..b_{bk}].
            // subst_outer with inner_depth=pos+bk, then subst_tel with ctor_args
            // weakened by bk → result in [Γ, b₁..b_{bk}].
            let mut ih_term = elim_inner;
            for bk in (0..nb).rev() {
                let ctor_args_k: Vec<Term> = ctor_args[..*pos]
                    .iter()
                    .map(|t| weaken(t, bk as i64))
                    .collect();
                let b_dom = subst_levels(
                    &subst_tel(
                        &subst_outer(&branching_tel[bk], m, params, *pos + bk),
                        &ctor_args_k,
                    ),
                    &ind.level_params,
                    level_args,
                );
                ih_term = Term::lam(b_dom, ih_term);
            }
            ihs.push(ih_term);
        }
    }

    // `mₖ ā [IHs]` — method applied to the constructor args then the IHs.
    let mut full_args = ctor_args.to_vec();
    full_args.extend(ihs);
    Ok(apply_args(method.clone(), &full_args))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::telescope_to_pi;
    use crate::term::{Level, LevelVar};

    fn d(id: u32) -> GlobalId {
        GlobalId(id)
    }

    #[test]
    fn occurs_finds_former() {
        // D applied to something containing D.
        let t = Term::app(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        assert!(occurs(d(0), &t));
        assert!(!occurs(d(1), &t));
    }

    #[test]
    fn positivity_nat_accepted() {
        // data Nat : Type 0 where zero : Nat ; suc : Nat → Nat
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            parameter_polarities: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![
                ConstructorDecl {
                    id: d(1),
                    args: vec![],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
                ConstructorDecl {
                    id: d(2),
                    args: vec![Term::indformer(d(0), vec![])],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
            ],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(check_positivity(&ind).is_ok());
    }

    #[test]
    fn positivity_bad_rejected() {
        // data Bad : Type 0 where mk : (Bad → Bool) → Bad
        let bool_ = Term::indformer(d(9), vec![]); // some other type `Bool`
        let arg = Term::pi(Term::indformer(d(0), vec![]), bool_); // Bad → Bool
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            parameter_polarities: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![arg],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(check_positivity(&ind).is_err());
    }

    #[test]
    fn positivity_bad3_in_application_rejected() {
        // data Bad3 : Type 0 where mk : Pair (Bad3 → Empty) Unit → Bad3
        // `Pair` is an inductive former (id 7); arg = Pair (Bad3→Empty) Unit.
        let empty = Term::indformer(d(8), vec![]);
        let bad3 = Term::indformer(d(0), vec![]);
        let unit = Term::indformer(d(6), vec![]);
        let pair_ty = Term::app(
            Term::app(Term::indformer(d(7), vec![]), Term::pi(bad3.clone(), empty)),
            unit,
        );
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            parameter_polarities: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![pair_ty],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(
            check_positivity(&ind).is_err(),
            "Bad3 nested-negative-in-application must be rejected"
        );
    }

    #[test]
    fn positivity_bad4_in_own_indices_rejected() {
        // data Bad4 : (Bad4 → Empty) → Type 0 where mk : Bad4 Empty
        let empty = Term::indformer(d(8), vec![]);
        let bad4 = Term::indformer(d(0), vec![]);
        let idx = Term::pi(bad4, empty); // Bad4 → Empty as an index
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            parameter_polarities: vec![],
            indices: vec![idx], // D in its own index telescope
            level: Level::zero(),
            constructors: vec![],
            former_type: Term::Type(Level::zero()),
        };
        let _ = telescope_to_pi; // keep import
        ind.build_types();
        assert!(
            check_positivity(&ind).is_err(),
            "Bad4 D-in-own-indices must be rejected"
        );
    }

    #[test]
    fn w_style_pi_bound_admitted_in_k1p5() {
        // data W : Type 0 where mk : (Nat → W) → W   (strictly positive W-style;
        // K1.5 admits it, `14 §2.1`, `14 §8.4`).
        let nat = Term::indformer(d(5), vec![]);
        let w = Term::indformer(d(0), vec![]);
        let arg = Term::pi(nat, w); // Nat → W
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            parameter_polarities: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![arg],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(
            check_positivity(&ind).is_ok(),
            "W-style is strictly positive"
        );
        // K1.5: recursive_args now includes the W-style arg.
        let rec = recursive_args(&ind.constructors[0], d(0), 0);
        assert_eq!(rec.len(), 1);
        let (pos, branching_tel, _idxs) = &rec[0];
        assert_eq!(*pos, 0);
        assert_eq!(branching_tel.len(), 1, "one Π-binder (Nat)");
    }

    #[test]
    fn w_style_branching_domain_not_d_free_rejected() {
        // data Bad5 : Type 0 where mk : (Bad5 → Bad5) → Bad5
        // The branching domain `Bad5` is not D-free: §8.2 checks the domain at
        // flipped (−) polarity and finds D there, so it rejects.
        // `14 §2.1` "B contains no occurrence of D"; conformance `wstyle-branching-
        // domain-not-d-free-rejected`. Soundness guard: gate-removal must not
        // relax the polarity check on the branching domain.
        let bad5 = Term::indformer(d(0), vec![]);
        // (Bad5 → Bad5) → Bad5: Pi(Pi(Bad5, Bad5), Bad5)
        let neg_arg = Term::pi(Term::pi(bad5.clone(), bad5.clone()), bad5);
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            parameter_polarities: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![neg_arg],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(
            check_positivity(&ind).is_err(),
            "branching domain not D-free must be rejected by §8.2 polarity check"
        );
    }

    // --- B3a regression: iota_reduct must not panic on arity mismatch ---
    // (Architect review on dec_2hnhhdb7mrxze.) `raw_wf` checks only scoping for
    // an `Elim`; `whnf` calls `iota_reduct` on any constructor-headed scrutinee.
    // A raw-well-formed `Elim` with too few params/methods/level-args must
    // return `KernelError::BadEliminator`, never panic.

    fn nat_decl() -> InductiveDecl {
        // data Nat : Type 0 where zero : Nat ; suc : Nat → Nat
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            parameter_polarities: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![
                ConstructorDecl {
                    id: d(1),
                    args: vec![],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
                ConstructorDecl {
                    id: d(2),
                    args: vec![Term::indformer(d(0), vec![])],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
            ],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        ind
    }

    #[test]
    fn iota_reduct_wrong_methods_arity_errors_not_panics() {
        let ind = nat_decl();
        // `zero` (k=0) has no args; ctor_all_args = [] (m=0, n=0). But supply
        // only ONE method (Nat has two constructors) → must error, not panic.
        let motive = Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        let res = iota_reduct(
            &ind,
            0,
            &[],
            &[],
            &motive,
            std::slice::from_ref(&motive), // 1 method, expected 2
            &[],
        );
        assert!(matches!(res, Err(KernelError::BadEliminator(_))));
    }

    #[test]
    fn iota_reduct_wrong_ctor_arity_errors_not_panics() {
        let ind = nat_decl();
        // `suc` (k=1) expects 1 ctor arg; supply 0 → must error, not panic.
        let motive = Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        let m1 = Term::lam(
            Term::indformer(d(0), vec![]),
            Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![])),
        );
        let res = iota_reduct(
            &ind,
            1, // suc
            &[],
            &[],
            &motive,
            &[motive.clone(), m1],
            &[], // 0 ctor args, expected 1
        );
        assert!(matches!(res, Err(KernelError::BadEliminator(_))));
    }

    #[test]
    fn iota_reduct_wrong_level_arity_errors_not_panics() {
        // A level-polymorphic family: supply the wrong number of level args.
        let mut ind = nat_decl();
        ind.level_params = vec![LevelVar(0)]; // one level param
        let motive = Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        let res = iota_reduct(
            &ind,
            0,
            &[Level::zero(), Level::zero()], // 2 level args, expected 1
            &[],
            &motive,
            &[motive.clone(), motive.clone()],
            &[],
        );
        assert!(matches!(res, Err(KernelError::BadEliminator(_))));
    }
}
