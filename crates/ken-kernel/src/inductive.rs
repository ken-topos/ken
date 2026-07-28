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

use crate::check::{infer_motive_level, Sort};
use crate::conv::normalize;
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
/// [`method_type`] and [`iota_reduct`] consume this descriptor as one atomic
/// semantic unit: every generated structured binder has a matching lifted
/// ι-term.
#[derive(Clone, Debug)]
pub struct RecursiveArgumentShape {
    pub position: usize,
    pub shape: RecursiveShape,
}

/// Structural recipe for lifting a motive through one recursive argument.
///
/// The variants exhaust the positive recursive shapes in the core grammar:
/// direct occurrences, Π-bound/W-style occurrences, primitive dependent Σ,
/// and applications of an admitted former through checked positive parameter
/// positions. Field terms are normalized through the kernel's established
/// terminating δ+β semantics before this structural classification.
/// Retained [`Term`] and [`Level`] values are semantic payloads, not Rust
/// identity: normalization is not a canonical representative of conversion,
/// so the descriptor deliberately does not implement `PartialEq`/`Eq`.
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

    /// Project the legacy direct/Π-bound class for consumers which have not
    /// moved to structured lifts. Structured Sigma/former recipes return
    /// `None`.
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
/// Unlike [`recursive_args`], this API represents primitive Sigma and
/// checked-positive former nesting for the atomic method/ι consumers.
pub fn recursive_shapes(
    env: &GlobalEnv,
    c: &ConstructorDecl,
    d: GlobalId,
    parameter_count: usize,
) -> KernelResult<Vec<RecursiveArgumentShape>> {
    let mut shapes = Vec::new();
    for (position, argument) in c.args.iter().enumerate() {
        if let ShapeDerivation::Recursive(shape) =
            derive_recursive_shape(env, argument, d, parameter_count)?
        {
            shapes.push(RecursiveArgumentShape { position, shape });
        }
    }
    Ok(shapes)
}

fn apply_motive(motive: &Term, indices: &[Term], value: Term) -> Term {
    let mut result = motive.clone();
    for index in indices {
        result = Term::app(result, index.clone());
    }
    Term::app(result, value)
}

fn evidence_sigma(mut components: Vec<Term>, terminal: Term) -> Term {
    let mut result = terminal;
    while let Some(component) = components.pop() {
        result = Term::sigma(component, weaken(&result, 1));
    }
    result
}

fn evidence_pair(mut components: Vec<Term>, terminal: Term) -> Term {
    let mut result = terminal;
    while let Some(component) = components.pop() {
        result = Term::pair(component, result);
    }
    result
}

fn terminal_type(
    ind: &InductiveDecl,
    level_args: &[Level],
    params: &[Term],
    source_type: &Term,
    motive: &Term,
) -> Term {
    let index_count = ind.indices.len();
    let params_under_source = params
        .iter()
        .map(|parameter| weaken(parameter, 1))
        .collect::<Vec<_>>();
    let instantiated_indices = ind
        .indices
        .iter()
        .map(|index| subst_levels(index, &ind.level_params, level_args))
        .collect::<Vec<_>>();
    let index_types = instantiated_indices
        .iter()
        .enumerate()
        .map(|(position, index)| {
            subst_outer(index, ind.params.len(), &params_under_source, position)
        })
        .collect::<Vec<_>>();

    let mut family = Term::indformer(ind.id, level_args.to_vec());
    for parameter in &params_under_source {
        family = Term::app(family, weaken(parameter, index_count as i64));
    }
    for position in 0..index_count {
        family = Term::app(family, Term::var(index_count - 1 - position));
    }

    let mut motive_at_value = weaken(motive, (index_count + 2) as i64);
    for position in 0..index_count {
        motive_at_value = Term::app(motive_at_value, Term::var(index_count - position));
    }
    motive_at_value = Term::app(motive_at_value, Term::var(0));
    let evidence_type = motive_at_value;
    let mut result = Term::pi(
        family,
        Term::pi(evidence_type.clone(), weaken(&evidence_type, 1)),
    );
    for index_type in index_types.into_iter().rev() {
        result = Term::pi(index_type, result);
    }
    Term::pi(source_type.clone(), result)
}

fn terminal_term(env: &GlobalEnv, ctx: &Context, terminal_type: &Term) -> KernelResult<Term> {
    let normalized = crate::whnf(env, ctx, terminal_type);
    let (domains, _) = peel_pi(&normalized);
    Ok(domains
        .into_iter()
        .rev()
        .fold(Term::var(0), |body, domain| Term::lam(domain, body)))
}

fn pack_checked(
    env: &GlobalEnv,
    ctx: &Context,
    expected: Term,
    values: &[Term],
) -> KernelResult<Term> {
    match values.split_first() {
        Some((value, rest)) => match crate::whnf(env, ctx, &expected) {
            Term::Sigma(domain, codomain) => {
                crate::check::check(env, ctx, value, &domain)?;
                let tail_expected = crate::subst::subst0(&codomain, value);
                let tail = pack_checked(env, ctx, tail_expected, rest)?;
                Ok(Term::pair(value.clone(), tail))
            }
            _ => Err(unsupported_recursive_shape(
                "evidence component has no expected Sigma domain",
            )),
        },
        None => {
            let terminal = terminal_term(env, ctx, &expected)?;
            crate::check::check(env, ctx, &terminal, &expected)?;
            Ok(terminal)
        }
    }
}

fn extract_host_ih(shape: &RecursiveShape, ih: Term, as_types: bool) -> KernelResult<Vec<Term>> {
    match shape {
        RecursiveShape::Direct { .. } => Ok(vec![ih]),
        RecursiveShape::Pi { domains, body } => {
            let binder_count = domains.len();
            let mut applied = weaken(&ih, binder_count as i64);
            for binder in 0..binder_count {
                applied = Term::app(applied, Term::var(binder_count - 1 - binder));
            }
            let mut components = extract_host_ih(body, applied, as_types)?;
            for component in &mut components {
                for domain in domains.iter().rev() {
                    *component = if as_types {
                        Term::pi(domain.clone(), component.clone())
                    } else {
                        Term::lam(domain.clone(), component.clone())
                    };
                }
            }
            Ok(components)
        }
        RecursiveShape::Sigma { domain, codomain } => {
            let both = domain.is_some() && codomain.is_some();
            let mut components = Vec::new();
            if let Some(shape) = domain {
                let value = if both {
                    Term::proj1(ih.clone())
                } else {
                    ih.clone()
                };
                components.extend(extract_host_ih(shape, value, as_types)?);
            }
            if let Some(shape) = codomain {
                let value = if both { Term::proj2(ih) } else { ih };
                components.extend(extract_host_ih(shape, value, as_types)?);
            }
            Ok(components)
        }
        RecursiveShape::Former { .. } => Err(unsupported_recursive_shape(
            "nested former evidence inside a host recursive IH is unsupported",
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn structured_former_lift_type(
    env: &GlobalEnv,
    former: GlobalId,
    former_level_args: &[Level],
    arguments: &[RecursiveFormerArgument],
    value: &Term,
    motive: &Term,
    d: GlobalId,
    parameter_count: usize,
) -> KernelResult<Term> {
    let former_decl = env.inductive(former).ok_or_else(|| {
        unsupported_recursive_shape("declared-former lift lost its admitted declaration")
    })?;
    let guest_decl = env
        .inductive(d)
        .ok_or_else(|| unsupported_recursive_shape("nested evidence lost its guest declaration"))?;
    if arguments.len() < former_decl.params.len() {
        return Err(unsupported_recursive_shape(
            "declared-former lift is under-saturated",
        ));
    }

    let mut source_type = Term::indformer(former, former_level_args.to_vec());
    for argument in arguments {
        source_type = Term::app(source_type, argument.term.clone());
    }
    let source_params = arguments[..former_decl.params.len()]
        .iter()
        .map(|argument| argument.term.clone())
        .collect::<Vec<_>>();
    let guest_params = {
        let first_shape = arguments
            .iter()
            .find_map(|argument| argument.shape.as_deref())
            .ok_or_else(|| unsupported_recursive_shape("former lift has no recursive path"))?;
        let field = arguments
            .iter()
            .find(|argument| argument.shape.is_some())
            .expect("shape found");
        let normalized = normalize(env, &Context::new(), &field.term);
        let mut cursor = (&normalized, first_shape);
        while let RecursiveShape::Pi { body, .. } = cursor.1 {
            cursor = (cursor.0, body);
        }
        let (_, application) = peel_app(cursor.0);
        application
            .into_iter()
            .take(parameter_count)
            .collect::<Vec<_>>()
    };
    let motive_sort =
        infer_motive_level(env, &Context::new(), guest_decl, &[], &guest_params, motive)?;
    let host_level = subst_levels(
        &Term::Type(former_decl.level.clone()),
        &former_decl.level_params,
        former_level_args,
    );
    let Term::Type(host_level) = host_level else {
        unreachable!("inductive levels instantiate to Type")
    };
    let evidence_level = host_level.max(motive_sort.level().clone()).normalize();
    let type_motive = Term::Ascript(
        Box::new(Term::lam(
            source_type.clone(),
            Term::Type(evidence_level.clone()),
        )),
        Box::new(Term::pi(
            source_type.clone(),
            Term::Type(evidence_level.clone().suc()),
        )),
    );

    let mut host_methods = Vec::with_capacity(former_decl.constructors.len());
    for (constructor_index, constructor) in former_decl.constructors.iter().enumerate() {
        let host_recursive = recursive_shapes(env, constructor, former, former_decl.params.len())?;
        let host_method_type = method_type(
            env,
            former_decl,
            constructor_index,
            &type_motive,
            &source_params,
            former_level_args,
        )?;
        let (method_domains, _) = peel_pi(&host_method_type);
        let field_count = constructor.args.len();
        let ih_count = host_recursive.len();
        let binder_count = method_domains.len();
        let mut components = Vec::new();

        for position in 0..field_count {
            let field_type = shift(
                &subst_levels(
                    &subst_outer(
                        &constructor.args[position],
                        former_decl.params.len(),
                        &source_params,
                        position,
                    ),
                    &former_decl.level_params,
                    former_level_args,
                ),
                (field_count - position + ih_count) as i64,
                0,
            );
            if let Some((ordinal, recursive)) = host_recursive
                .iter()
                .enumerate()
                .find(|(_, recursive)| recursive.position == position)
            {
                components.extend(extract_host_ih(
                    &recursive.shape,
                    Term::var(ih_count - 1 - ordinal),
                    true,
                )?);
            } else if let ShapeDerivation::Recursive(shape) =
                derive_recursive_shape(env, &field_type, d, parameter_count)?
            {
                components.push(structured_lift_type(
                    env,
                    &shape,
                    &field_type,
                    &Term::var(ih_count + field_count - 1 - position),
                    &weaken(motive, binder_count as i64),
                    d,
                    parameter_count,
                )?);
            }
        }
        let terminal = terminal_type(
            guest_decl,
            &[],
            &guest_params
                .iter()
                .map(|parameter| weaken(parameter, binder_count as i64))
                .collect::<Vec<_>>(),
            &weaken(&source_type, binder_count as i64),
            &weaken(motive, binder_count as i64),
        );
        let mut body = evidence_sigma(components, terminal);
        for domain in method_domains.into_iter().rev() {
            body = Term::lam(domain, body);
        }
        host_methods.push(body);
    }

    Ok(Term::Elim {
        fam: former,
        level_args: former_level_args.to_vec(),
        params: source_params,
        motive: Box::new(type_motive),
        methods: host_methods,
        indices: arguments[former_decl.params.len()..]
            .iter()
            .map(|argument| argument.term.clone())
            .collect(),
        scrut: Box::new(value.clone()),
    })
}

/// Build `Lift_D(M, A, a)` from the D3a skeleton (`14 §3.2`).
///
/// `field_type`, `value`, and `motive` are already instantiated in one common
/// caller context.  Reading indices back from `field_type` is intentional: the
/// retained terms in [`RecursiveShape`] are semantic payloads rather than a
/// canonical representative of conversion.
fn structured_lift_type(
    env: &GlobalEnv,
    shape: &RecursiveShape,
    field_type: &Term,
    value: &Term,
    motive: &Term,
    d: GlobalId,
    parameter_count: usize,
) -> KernelResult<Term> {
    let field_type = normalize(env, &Context::new(), field_type);
    match shape {
        RecursiveShape::Direct { .. } => {
            let (head, arguments) = peel_app(&field_type);
            match head {
                Term::IndFormer { id, .. } if id == d && arguments.len() >= parameter_count => Ok(
                    apply_motive(motive, &arguments[parameter_count..], value.clone()),
                ),
                _ => Err(unsupported_recursive_shape(
                    "direct lift no longer has the recursive family at its head",
                )),
            }
        }
        RecursiveShape::Pi { domains, body } => {
            let (actual_domains, actual_body) = peel_pi(&field_type);
            if actual_domains.len() != domains.len() {
                return Err(unsupported_recursive_shape(
                    "Pi lift skeleton and normalized field arity disagree",
                ));
            }
            let binder_count = actual_domains.len();
            let mut applied_value = weaken(value, binder_count as i64);
            for binder in 0..binder_count {
                applied_value = Term::app(applied_value, Term::var(binder_count - 1 - binder));
            }
            let mut lifted = structured_lift_type(
                env,
                body,
                &actual_body,
                &applied_value,
                &weaken(motive, binder_count as i64),
                d,
                parameter_count,
            )?;
            for domain in actual_domains.into_iter().rev() {
                lifted = Term::pi(domain, lifted);
            }
            Ok(lifted)
        }
        RecursiveShape::Sigma { domain, codomain } => {
            let Term::Sigma(actual_domain, actual_codomain) = field_type else {
                return Err(unsupported_recursive_shape(
                    "Sigma lift skeleton no longer has a normalized Sigma field",
                ));
            };
            let first_value = normalize(env, &Context::new(), &Term::proj1(value.clone()));
            let second_value = normalize(env, &Context::new(), &Term::proj2(value.clone()));
            let first = domain
                .as_deref()
                .map(|shape| {
                    structured_lift_type(
                        env,
                        shape,
                        &actual_domain,
                        &first_value,
                        motive,
                        d,
                        parameter_count,
                    )
                })
                .transpose()?;
            let second_type = crate::subst::subst0(&actual_codomain, &first_value);
            let second = codomain
                .as_deref()
                .map(|shape| {
                    structured_lift_type(
                        env,
                        shape,
                        &second_type,
                        &second_value,
                        motive,
                        d,
                        parameter_count,
                    )
                })
                .transpose()?;
            match (first, second) {
                (Some(first), Some(second)) => Ok(Term::sigma(first, weaken(&second, 1))),
                (Some(only), None) | (None, Some(only)) => Ok(only),
                (None, None) => Err(unsupported_recursive_shape(
                    "Sigma lift skeleton contains no recursive component",
                )),
            }
        }
        RecursiveShape::Former {
            former,
            level_args: former_level_args,
            arguments,
        } => {
            let (actual_head, actual_arguments) = peel_app(&field_type);
            if !matches!(actual_head, Term::IndFormer { id, .. } if id == *former)
                || actual_arguments.len() != arguments.len()
            {
                return Err(unsupported_recursive_shape(
                    "declared-former lift skeleton and normalized field disagree",
                ));
            }
            let actual_arguments = arguments
                .iter()
                .zip(actual_arguments)
                .map(|(shape_argument, term)| RecursiveFormerArgument {
                    term,
                    shape: shape_argument.shape.clone(),
                })
                .collect::<Vec<_>>();
            structured_former_lift_type(
                env,
                *former,
                former_level_args,
                &actual_arguments,
                value,
                motive,
                d,
                parameter_count,
            )
        }
    }
}

fn structured_lift_term(
    env: &GlobalEnv,
    shape: &RecursiveShape,
    field_type: &Term,
    value: &Term,
    motive: &Term,
    methods: &[Term],
    d: GlobalId,
    parameter_count: usize,
    level_args: &[Level],
    params: &[Term],
) -> KernelResult<Term> {
    let field_type = normalize(env, &Context::new(), field_type);
    match shape {
        RecursiveShape::Direct { .. } => {
            let (head, arguments) = peel_app(&field_type);
            match head {
                Term::IndFormer { id, .. } if id == d && arguments.len() >= parameter_count => {
                    Ok(Term::Elim {
                        fam: d,
                        level_args: level_args.to_vec(),
                        params: params.to_vec(),
                        motive: Box::new(motive.clone()),
                        methods: methods.to_vec(),
                        indices: arguments[parameter_count..].to_vec(),
                        scrut: Box::new(value.clone()),
                    })
                }
                _ => Err(unsupported_recursive_shape(
                    "direct lifted term no longer has the recursive family at its head",
                )),
            }
        }
        RecursiveShape::Pi { domains, body } => {
            let (actual_domains, actual_body) = peel_pi(&field_type);
            if actual_domains.len() != domains.len() {
                return Err(unsupported_recursive_shape(
                    "Pi lifted term skeleton and normalized field arity disagree",
                ));
            }
            let binder_count = actual_domains.len();
            let mut applied_value = weaken(value, binder_count as i64);
            for binder in 0..binder_count {
                applied_value = Term::app(applied_value, Term::var(binder_count - 1 - binder));
            }
            let mut lifted = structured_lift_term(
                env,
                body,
                &actual_body,
                &applied_value,
                &weaken(motive, binder_count as i64),
                &methods
                    .iter()
                    .map(|method| weaken(method, binder_count as i64))
                    .collect::<Vec<_>>(),
                d,
                parameter_count,
                level_args,
                &params
                    .iter()
                    .map(|param| weaken(param, binder_count as i64))
                    .collect::<Vec<_>>(),
            )?;
            for domain in actual_domains.into_iter().rev() {
                lifted = Term::lam(domain, lifted);
            }
            Ok(lifted)
        }
        RecursiveShape::Sigma { domain, codomain } => {
            let Term::Sigma(actual_domain, actual_codomain) = field_type else {
                return Err(unsupported_recursive_shape(
                    "Sigma lifted term skeleton no longer has a normalized Sigma field",
                ));
            };
            let first_value = normalize(env, &Context::new(), &Term::proj1(value.clone()));
            let second_value = normalize(env, &Context::new(), &Term::proj2(value.clone()));
            let first = domain
                .as_deref()
                .map(|shape| {
                    structured_lift_term(
                        env,
                        shape,
                        &actual_domain,
                        &first_value,
                        motive,
                        methods,
                        d,
                        parameter_count,
                        level_args,
                        params,
                    )
                })
                .transpose()?;
            let second_type = crate::subst::subst0(&actual_codomain, &first_value);
            let second = codomain
                .as_deref()
                .map(|shape| {
                    structured_lift_term(
                        env,
                        shape,
                        &second_type,
                        &second_value,
                        motive,
                        methods,
                        d,
                        parameter_count,
                        level_args,
                        params,
                    )
                })
                .transpose()?;
            match (first, second) {
                (Some(first), Some(second)) => Ok(Term::pair(first, second)),
                (Some(only), None) | (None, Some(only)) => Ok(only),
                (None, None) => Err(unsupported_recursive_shape(
                    "Sigma lifted term skeleton contains no recursive component",
                )),
            }
        }
        RecursiveShape::Former {
            former,
            level_args: former_level_args,
            arguments,
        } => {
            let (actual_head, actual_arguments) = peel_app(&field_type);
            if !matches!(actual_head, Term::IndFormer { id, .. } if id == *former)
                || actual_arguments.len() != arguments.len()
            {
                return Err(unsupported_recursive_shape(
                    "declared-former lifted term skeleton and normalized field disagree",
                ));
            }
            let actual_arguments = arguments
                .iter()
                .zip(actual_arguments)
                .map(|(shape_argument, term)| RecursiveFormerArgument {
                    term,
                    shape: shape_argument.shape.clone(),
                })
                .collect::<Vec<_>>();
            structured_former_lift_term(
                env,
                *former,
                former_level_args,
                &actual_arguments,
                value,
                motive,
                methods,
                d,
                parameter_count,
                level_args,
                params,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn structured_former_lift_term(
    env: &GlobalEnv,
    former: GlobalId,
    former_level_args: &[Level],
    arguments: &[RecursiveFormerArgument],
    value: &Term,
    motive: &Term,
    methods: &[Term],
    d: GlobalId,
    parameter_count: usize,
    d_level_args: &[Level],
    d_params: &[Term],
) -> KernelResult<Term> {
    let former_decl = env.inductive(former).ok_or_else(|| {
        unsupported_recursive_shape("declared-former lift lost its admitted declaration")
    })?;
    let guest_decl = env
        .inductive(d)
        .ok_or_else(|| unsupported_recursive_shape("nested evidence lost its guest declaration"))?;
    if arguments.len() < former_decl.params.len() {
        return Err(unsupported_recursive_shape(
            "declared-former lift is under-saturated",
        ));
    }

    let mut source_type = Term::indformer(former, former_level_args.to_vec());
    for argument in arguments {
        source_type = Term::app(source_type, argument.term.clone());
    }
    let source_params = arguments[..former_decl.params.len()]
        .iter()
        .map(|argument| argument.term.clone())
        .collect::<Vec<_>>();
    let motive_sort = infer_motive_level(
        env,
        &Context::new(),
        guest_decl,
        d_level_args,
        d_params,
        motive,
    )?;
    let host_level = subst_levels(
        &Term::Type(former_decl.level.clone()),
        &former_decl.level_params,
        former_level_args,
    );
    let Term::Type(host_level) = host_level else {
        unreachable!("inductive levels instantiate to Type")
    };
    let evidence_level = host_level.max(motive_sort.level().clone()).normalize();
    let evidence_sort = match motive_sort {
        Sort::Type(_) => Term::Type(evidence_level),
        Sort::Omega(_) => Term::Omega(evidence_level),
    };
    let lifted_arguments = arguments
        .iter()
        .map(|argument| RecursiveFormerArgument {
            term: weaken(&argument.term, 1),
            shape: argument.shape.clone(),
        })
        .collect::<Vec<_>>();
    let body_type = structured_former_lift_type(
        env,
        former,
        former_level_args,
        &lifted_arguments,
        &Term::var(0),
        &weaken(motive, 1),
        d,
        parameter_count,
    )?;
    let host_motive = Term::Ascript(
        Box::new(Term::lam(source_type.clone(), body_type)),
        Box::new(Term::pi(source_type.clone(), evidence_sort)),
    );

    let mut host_methods = Vec::with_capacity(former_decl.constructors.len());
    for (constructor_index, constructor) in former_decl.constructors.iter().enumerate() {
        let host_recursive = recursive_shapes(env, constructor, former, former_decl.params.len())?;
        let host_method_type = method_type(
            env,
            former_decl,
            constructor_index,
            &host_motive,
            &source_params,
            former_level_args,
        )?;
        let (method_domains, expected_body) = peel_pi(&host_method_type);
        let mut host_ctx = Context::new();
        host_ctx.extend_tel(&method_domains);
        let field_count = constructor.args.len();
        let ih_count = host_recursive.len();
        if method_domains.len() != field_count + ih_count {
            return Err(unsupported_recursive_shape(
                "host former method binder count disagrees with its recursive skeleton",
            ));
        }
        let binder_count = method_domains.len();
        let mut components = Vec::new();
        for position in 0..field_count {
            let field_type = shift(
                &subst_levels(
                    &subst_outer(
                        &constructor.args[position],
                        former_decl.params.len(),
                        &source_params,
                        position,
                    ),
                    &former_decl.level_params,
                    former_level_args,
                ),
                (field_count - position + ih_count) as i64,
                0,
            );
            if let Some((ih_ordinal, recursive)) = host_recursive
                .iter()
                .enumerate()
                .find(|(_, recursive)| recursive.position == position)
            {
                components.extend(extract_host_ih(
                    &recursive.shape,
                    Term::var(ih_count - 1 - ih_ordinal),
                    false,
                )?);
            } else if let ShapeDerivation::Recursive(shape) =
                derive_recursive_shape(env, &field_type, d, parameter_count)?
            {
                components.push(structured_lift_term(
                    env,
                    &shape,
                    &field_type,
                    &Term::var(ih_count + field_count - 1 - position),
                    &weaken(motive, binder_count as i64),
                    &methods
                        .iter()
                        .map(|method| weaken(method, binder_count as i64))
                        .collect::<Vec<_>>(),
                    d,
                    parameter_count,
                    d_level_args,
                    &d_params
                        .iter()
                        .map(|parameter| weaken(parameter, binder_count as i64))
                        .collect::<Vec<_>>(),
                )?);
            }
        }
        let mut body = pack_checked(env, &host_ctx, expected_body.clone(), &components)?;
        for domain in method_domains.into_iter().rev() {
            body = Term::lam(domain, body);
        }
        crate::check::check(env, &Context::new(), &body, &host_method_type)?;
        host_methods.push(body);
    }

    Ok(Term::Elim {
        fam: former,
        level_args: former_level_args.to_vec(),
        params: source_params,
        motive: Box::new(host_motive),
        methods: host_methods,
        indices: arguments[former_decl.params.len()..]
            .iter()
            .map(|argument| argument.term.clone())
            .collect(),
        scrut: Box::new(value.clone()),
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
    env: &GlobalEnv,
    ind: &InductiveDecl,
    k: usize,
    motive: &Term,
    params: &[Term],
    level_args: &[Level],
) -> KernelResult<Term> {
    let c = &ind.constructors[k];
    let m = ind.params.len();
    let n = c.args.len();
    let rec = recursive_shapes(env, c, ind.id, m)?;
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
        let pos = rec[j].position;
        let field_type = shift(
            &subst_levels(
                &subst_outer(&c.args[pos], m, params, pos),
                &ind.level_params,
                level_args,
            ),
            (n - pos + j) as i64,
            0,
        );
        let field_value = Term::var(n - 1 - pos + j);
        let ih_ty = structured_lift_type(
            env,
            &rec[j].shape,
            &field_type,
            &field_value,
            &weaken(motive, (n + j) as i64),
            ind.id,
            m,
        )?;
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
    Ok(ty)
}

/// The ι-reduct of an eliminator applied to a constructor-headed scrutinee
/// (`14 §7.3`): `elim_D p̄ M m̄ i̅ (cₖ p̄ ā) ⇝ mₖ ā [IHs]`.
///
/// `ctor_all_args` is the constructor's full argument spine `p̄ ++ ā` (params
/// then args), already peeled from the scrutinee. Returns the reduct, or an
/// error if the spine does not match the constructor's arity.
pub fn iota_reduct(
    env: &GlobalEnv,
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

    let rec = recursive_shapes(env, c, ind.id, m)?;
    // Induction hypotheses for each recursive arg (`14 §7.3`, `14 §7.7`):
    //   - Direct (nb=0):    `elim_D p̄ M m̄ idx(a_j) a_j`
    //   - W-style (nb≥1):  `λ(b₁:B₁)...(b_{nb}:B_{nb}). elim_D p̄ M m̄ idx(a_j b₁..b_{nb}) (a_j b₁..b_{nb})`
    let mut ihs: Vec<Term> = Vec::new();
    for argument in &rec {
        let pos = argument.position;
        let field_type = subst_levels(
            &subst_tel(
                &subst_outer(&c.args[pos], m, params, pos),
                &ctor_args[..pos],
            ),
            &ind.level_params,
            level_args,
        );
        ihs.push(structured_lift_term(
            env,
            &argument.shape,
            &field_type,
            &ctor_args[pos],
            motive,
            methods,
            ind.id,
            m,
            level_args,
            params,
        )?);
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
            &GlobalEnv::new(),
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
            &GlobalEnv::new(),
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
            &GlobalEnv::new(),
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
