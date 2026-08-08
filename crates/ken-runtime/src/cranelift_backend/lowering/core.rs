//! The indivisible lowering SCC (RT-SPLIT §10.1/§10.2).
//!
//! Moved verbatim from `cranelift_backend.rs` in RT-SPLIT slice 4; the
//! 29-method SCC plus `compile_expr_into_module`. Imports come only from
//! this module's parent, per §10.5, so slice 5 need not touch this file.

// Re-exported at facade scope so this module's `tests` subtree inherits the
// same names; a private `use` cannot be re-globbed by a descendant.
pub(in crate::cranelift_backend) use super::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
thread_local! {
    static C2_UNIT_EMISSION_EPOCH: std::cell::Cell<Option<u64>> =
        const { std::cell::Cell::new(None) };
    static RECURSIVE_POSITION_UNIT_CALLS: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

/// **`RT-SEED-CALL-PORT` `D1` — the residual set observed at the PRODUCTION
/// selector site, per compilation.**
///
/// The durable enumerator landed in `RT-SRCBODY-BIND-ORDER` (`7ca5cfc0`).
/// Before `D1`, all of its call sites used hand-built `RuntimeExpr` witnesses.
/// That answers *"can the instrument see variant V?"* and **not** the question
/// every node in this campaign actually asks: *"which variants fire on the
/// programs this repository really compiles?"* A witness is authored to exhibit
/// the variant it is named for, so a walk with a gap still reports it.
///
/// This cell closes that distance by recording the enumeration where
/// `compile_expr_into_module_with_root_projection` selects the authority — the
/// one gate every compiled program passes. The population is therefore defined
/// by the gate, not by the set of programs someone thought to enumerate.
///
/// **Domain, stated so it can be checked:** this observes every program compiled
/// **within `ken-runtime`'s own test profile**. Programs reached only from
/// `ken-cli` integration tests are outside it, because `cfg(test)` is not set
/// for this crate when it is built as a dependency. `D1` reports a NON-empty
/// population, so widening the domain can only add members — it cannot change
/// the answer. A close-on-absence would have needed the wider domain first.
#[cfg(test)]
thread_local! {
    static OBSERVED_RESIDUALS: std::cell::RefCell<Option<BTreeSet<RecursiveDescentResidual>>> =
        const { std::cell::RefCell::new(None) };
    static RESIDUAL_ENUMERATION_MUTATION: std::cell::Cell<ResidualEnumerationMutation> =
        const { std::cell::Cell::new(ResidualEnumerationMutation::None) };
}

/// **`D2` reachability — how many times the ported seed-callee arm reached its
/// HANDOFF POINT: arity checked, every capture resolved, inputs handed to the
/// existing typed call path.**
///
/// Without this the `AC-6` controls cannot discriminate. The canonical seed
/// returns `7` on the `RecursiveDescent` lane and `7` through the new port, so
/// a green observation is consistent with the port never running. Counting the
/// arm's own handoff is what separates "the program still works" from "the
/// program went through the mechanism `D2` built".
///
/// **This is NOT an emission oracle, and the distinction is load-bearing.** The
/// increment precedes `call_declared_unit`, and the actual call instruction is
/// emitted later in the unchanged transport (`lowering/mod.rs`, at the
/// `builder.ins().call`). Between the two, target lookup, descriptor and input
/// checks, carrier transfer and host-context resolution can all still refuse.
/// So `(Err(_), count == 1)` is reachable **without any unit call existing**.
///
/// ⇒ The count alone proves the **handoff**. Evidence for a *completed* typed
/// unit call is the pair **successful outcome AND count 1**, which is how every
/// positive row below reads it. A count of **0** proves refusal *before* the
/// handoff, which is the distinct thing `AC-6.2` asserts.
#[cfg(test)]
thread_local! {
    static SEED_CALLEE_UNIT_PORTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_seed_callee_unit_ports() {
    SEED_CALLEE_UNIT_PORTS.with(|cell| cell.set(0));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn seed_callee_unit_ports() -> usize {
    SEED_CALLEE_UNIT_PORTS.with(std::cell::Cell::get)
}

/// **`RT-PRODUCER-MATCH-PORT` `D2` — a HANDOFF counter, and named so.**
///
/// Incremented once the composed ordinary frame has been checked for the three
/// pieces of state the carried elimination cannot express, immediately before
/// `lower_carried_match`. **That call can still refuse**, so `(Err(_), count ==
/// 1)` is reachable with no elimination emitted.
///
/// ⇒ The count alone proves the arm **took the port** rather than refusing ahead
/// of it. Evidence for a *completed* carried elimination is the pair **successful
/// outcome AND count 1**; a count of **0** proves a refusal strictly before the
/// handoff. Every row below reads it as that pair, never alone.
#[cfg(test)]
thread_local! {
    static PRODUCER_MATCH_UNIT_PORTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_producer_match_unit_ports() {
    PRODUCER_MATCH_UNIT_PORTS.with(|cell| cell.set(0));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn producer_match_unit_ports() -> usize {
    PRODUCER_MATCH_UNIT_PORTS.with(std::cell::Cell::get)
}


/// The mutation `D1a` proves its exact-set control against.
///
/// `ShortCircuitLikeTheSelector` is not a synthetic perturbation: it makes the
/// enumerator return exactly what its short-circuiting twin returns, which is
/// the one regression the instrument exists to prevent and the one a
/// reachability control cannot see.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::cranelift_backend) enum ResidualEnumerationMutation {
    None,
    ShortCircuitLikeTheSelector,
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_residual_enumeration_mutation(
    mutation: ResidualEnumerationMutation,
) {
    RESIDUAL_ENUMERATION_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
fn residual_enumeration_mutation() -> ResidualEnumerationMutation {
    RESIDUAL_ENUMERATION_MUTATION.with(std::cell::Cell::get)
}

/// The residual set the last compilation on this thread observed, or `None` if
/// no compilation has run since the last reset.
///
/// `None` and `Some(empty)` are deliberately distinct: *"the instrument never
/// ran"* and *"the instrument ran and found nothing"* are the two readings a
/// close-on-absence must never conflate, and this node's whole `D1a` gate exists
/// because the second is the predicted answer.
#[cfg(test)]
pub(in crate::cranelift_backend) fn observed_recursive_descent_residuals(
) -> Option<BTreeSet<RecursiveDescentResidual>> {
    OBSERVED_RESIDUALS.with(|cell| cell.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_observed_recursive_descent_residuals() {
    OBSERVED_RESIDUALS.with(|cell| *cell.borrow_mut() = None);
}

/// **`RT-CARRIER-BYTESPAN-OBSERVE` `D2` — the reply byte span, MASKED at the
/// typed producer** (Architect `dec_12s3j2gj67c66`).
///
/// ⭐ **Why the masking lives HERE and not in the carrier.** Since `D2`,
/// [`Lowered::ResponseBytes`] means *a byte span that will be dereferenced and
/// copied*, so **every instance must independently be a valid span** — being
/// the arm a runtime discriminant does not select cannot legalize a pointer
/// nobody established. The carrier materializes both `HostResult` children
/// eagerly and B2V controls pin that, so the obligation cannot be discharged by
/// making transfer lazy. It is discharged where `success`, the operation and the
/// byte source are all already in scope: right here.
///
/// ⛔ **Not a placeholder lane and not a new representation.** The unselected
/// arm is the canonical EMPTY span `{null, 0}` — an ordinary, lawful
/// `ResponseBytes` whose copy loop runs zero times. Nothing downstream learns a
/// new shape.
///
/// ⚠ **This deliberately does NOT rest on the dispatcher clearing
/// `reply.bytes`.** That is a true implementation fact today and it is not an
/// ABI-authority contract; building on it would silently promote an
/// implementation detail into one. The mask makes the span self-validating
/// regardless of what the reply buffer happens to hold.
///
/// **`D4b` / `AC-10` — the `select` pair moved INTO
/// [`SafeByteSpan::masked_at_producer`].** This function is no longer the place
/// the invariant is kept; it is a caller that cannot avoid it. Reading the two
/// reply fields is all that is left here, because the only route from a
/// `{span, success}` triple to a `ResponseBytes` now emits the mask itself.
fn masked_reply_response_bytes(
    builder: &mut FunctionBuilder<'_>,
    pointer_type: cranelift_codegen::ir::Type,
    reply: cranelift_codegen::ir::StackSlot,
    data_offset: u32,
    len_offset: u32,
    success: cranelift_codegen::ir::Value,
) -> Lowered {
    let pointer = builder.ins().stack_load(
        pointer_type,
        reply,
        i32::try_from(data_offset).expect("reply bytes data offset is u32"),
    );
    let len = builder.ins().stack_load(
        types::I64,
        reply,
        i32::try_from(len_offset).expect("reply bytes len offset is u32"),
    );
    Lowered::ResponseBytes(SafeByteSpan::masked_at_producer(
        builder,
        pointer_type,
        pointer,
        len,
        success,
    ))
}

#[cfg(test)]
fn c2_unit_emission_epoch() -> Option<u64> {
    C2_UNIT_EMISSION_EPOCH.with(std::cell::Cell::get)
}

#[cfg(test)]
fn recursive_position_unit_calls() -> usize {
    RECURSIVE_POSITION_UNIT_CALLS.with(std::cell::Cell::get)
}

type ConsumedSubcontinuationFrame = (u64, u64);

/// **`D8m`** — withhold the tuple the checked bridge transports, restoring the
/// pre-`D8m` shape at that one site.
#[cfg(test)]
thread_local! {
    static D8M_SUPPRESS_TRANSPORTED_TUPLE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8m_suppress_transported_tuple(armed: bool) {
    D8M_SUPPRESS_TRANSPORTED_TUPLE.with(|cell| cell.set(armed));
}

#[cfg(test)]
fn d8m_suppress_transported_tuple() -> bool {
    D8M_SUPPRESS_TRANSPORTED_TUPLE.with(std::cell::Cell::get)
}

/// **`D8m`** — give the bridge the WRAPPER's own occurrence instead of the
/// wrapped match's, which is child 0 of it.
///
/// The marker names the frame; the match IS the frame. Every origin-keyed
/// lookup downstream -- case bodies, the planner's continuation origin -- has to
/// land on the match, and substituting the wrapper is the one-node-off error a
/// single checked occurrence cannot see: with one occurrence any origin that
/// resolves at all resolves to the only candidate.
#[cfg(test)]
thread_local! {
    static D8M_WRAPPER_ORIGIN_SUBSTITUTION: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8m_wrapper_origin_substitution(armed: bool) {
    D8M_WRAPPER_ORIGIN_SUBSTITUTION.with(|cell| cell.set(armed));
}

#[cfg(test)]
fn d8m_wrapper_origin_substitution() -> bool {
    D8M_WRAPPER_ORIGIN_SUBSTITUTION.with(std::cell::Cell::get)
}

/// **`D8m`** — consume the marker with a default the source match does not
/// carry.
///
/// The bridge consumes through the existing pair, and that pair holds the
/// marker to the shape the plan transported for it. Withholding the tuple proves
/// the identity is carried; this proves the identity is carried FOR THE MATCH
/// THE MARKER WRAPPED, rather than for whatever shape the bridge happens to hold
/// by the time it consumes.
#[cfg(test)]
thread_local! {
    static D8M_FOREIGN_CONSUMED_SHAPE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8m_foreign_consumed_shape(armed: bool) {
    D8M_FOREIGN_CONSUMED_SHAPE.with(|cell| cell.set(armed));
}

#[cfg(test)]
fn d8m_foreign_consumed_shape() -> bool {
    D8M_FOREIGN_CONSUMED_SHAPE.with(std::cell::Cell::get)
}

/// **`D8f`** — let the DECLINED call answer for the checked application's
/// composed causal identity, which is what it did before this checkpoint.
///
/// The call itself is unchanged either way; only the claim moves. That is what
/// makes the duplicate-discharge refusal a difference this checkpoint removed
/// rather than one it routed around.
#[cfg(test)]
thread_local! {
    static D8F_DECLINED_CALL_CLAIMS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8f_declined_call_claims(armed: bool) {
    D8F_DECLINED_CALL_CLAIMS.with(|cell| cell.set(armed));
}

#[cfg(test)]
fn d8f_declined_call_claims() -> bool {
    D8F_DECLINED_CALL_CLAIMS.with(std::cell::Cell::get)
}

/// **`D8n`** — restore the compile-wide consumed-frame lifetime.
#[cfg(test)]
thread_local! {
    static D8N_COMPILE_WIDE_LIFECYCLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8n_compile_wide_lifecycle(armed: bool) {
    D8N_COMPILE_WIDE_LIFECYCLE.with(|cell| cell.set(armed));
}

#[cfg(test)]
fn d8n_compile_wide_lifecycle() -> bool {
    D8N_COMPILE_WIDE_LIFECYCLE.with(std::cell::Cell::get)
}

/// Transactions checked-frame consumption across mutually exclusive lowering
/// successors. A successor begins at the common predecessor baseline, while
/// the union remains authoritative after their join.
struct CheckedFrameBranchScope {
    baseline: BTreeSet<ConsumedSubcontinuationFrame>,
    union: BTreeSet<ConsumedSubcontinuationFrame>,
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8o` — the ambient body-authority binding.**
///
/// ⭐⭐ **Both ambient facts are bound for the lifetime of ONE emitted body and
/// cleared afterwards, so a later `Function` cannot inherit residue.** Before
/// `D8o` only two of the three source-bearing body kinds wrote them: an
/// ordinary unit body and a generated-context body did, and a **specialization
/// body did not** -- so throughout it both fields held whatever the previously
/// defined body left behind.
///
/// ⛔ **Why that was quiet, which is the census's real finding.** Of the eight
/// production readers of `defining_emission_owner`, two fail closed and two
/// compare -- residue there becomes a refusal, and someone sees it. The other
/// **four DECLINE**: `composed_recursive_argument_binding` keeps the ordinary
/// route, and the three synthesized-aggregate sites return `None`/`Ok(())`. A
/// declining reader cannot tell "no body is being defined" from "a body is
/// being defined and left the wrong value here", so a stale owner simply
/// changes the answer. See `docs/notes/rt-contsrc-d8o-ambient-body-authority-
/// census.md`.
///
/// ⛔ **The owner is supplied, never inferred.** Not from the `FuncId`, not
/// from the raw body's origin, not from the prior ambient value: each body kind
/// passes the exact fact the planner issued for it. A specialization body's
/// owner is exactly `ContinuationEmissionOwner::Specialization(unit.id)`.
///
/// ⛔ On exit both fields are RESTORED to what the enclosing scope held, which
/// is `None` at the top level. Clearing unconditionally would be wrong for a
/// nested pass; restoring is correct for both and does not have to know which
/// it is in.
pub(super) struct AmbientBodyAuthority {
    enclosing_owner: Option<ContinuationEmissionOwner>,
    enclosing_unit: Option<PredeclaredFunctionId>,
}

impl AmbientBodyAuthority {
    /// Bind one emitted body's authority, before any source lowering.
    pub(super) fn bind(
        compiler: &mut Lowering<'_>,
        owner: ContinuationEmissionOwner,
        unit: PredeclaredFunctionId,
    ) -> Self {
        let scope = Self {
            enclosing_owner: compiler.defining_emission_owner,
            enclosing_unit: compiler.defining_unit,
        };
        compiler.defining_emission_owner = Some(owner);
        compiler.defining_unit = Some(unit);
        // `D8o` — the observation, taken AFTER installation and read back out of
        // the LIVE fields, under the Function currently being defined.
        //
        // ⛔ Not the bind arguments. Recording those would say only that this
        // function was called with them; reading the fields says what any
        // reader in this body will actually see, which is the property the
        // census is about. The two differ exactly when the install is wrong.
        //
        // ⛔ The inherited pair is the load-bearing half -- with the release in
        // place it is `None` at every body, and a release that failed to restore
        // shows up here as the previous body's facts arriving as this one's
        // enclosing scope.
        #[cfg(test)]
        crate::cranelift_backend::lowering::record_d8o_body_authority(
            compiler.defining_function_id,
            compiler
                .defining_emission_owner
                .expect("just installed above"),
            compiler.defining_unit.expect("just installed above"),
            scope.enclosing_owner,
            scope.enclosing_unit,
        );
        scope
    }

    /// Release it, restoring the enclosing scope's facts.
    pub(super) fn release(self, compiler: &mut Lowering<'_>) {
        // ⛔ `D8o` — the pre-repair behaviour, restored under test: the body's
        // facts are LEFT in place instead of being rolled back, so the next
        // body inherits them. That is exactly the residue this binding removes.
        #[cfg(test)]
        if crate::cranelift_backend::lowering::d8o_inherit_residue() {
            return;
        }
        compiler.defining_emission_owner = self.enclosing_owner;
        compiler.defining_unit = self.enclosing_unit;
    }
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8n` — the per-`Function` checked-frame
/// transaction.**
///
/// ⭐⭐ **Consumption is a fact about ONE emitted function, not about a
/// compile.** `consumed_subcontinuation_frames` was a single compile-wide set,
/// so a source body lowered into two generated `Function`s -- an ordinary
/// declaration body and the specialization body derived from the same text --
/// consumed the same `(invocation_id, frame_id)` twice and refused. That is not
/// a double consumption: it is one consumption in each of two functions, which
/// is exactly what a split body means.
///
/// ⛔ **The identity key is untouched.** No emission-owner, `FuncId` or
/// `PredeclaredFunctionId` salt: salting the key would make one source frame
/// two identities and quietly permit a real double consumption inside a single
/// function. What was wrong was the ledger's LIFETIME, not what it counts.
///
/// ⛔ **Distinct from [`CheckedFrameBranchScope`], and they nest.** Branch
/// successors are mutually exclusive paths through ONE function that rejoin, so
/// their consumption unions at the join. Separate emitted functions never
/// rejoin, so theirs must not: each starts empty and the enclosing set is
/// restored afterwards. Using the branch scope here would union two functions'
/// consumption into one set and reintroduce the collision one level up.
///
/// ⛔ **No active marker may cross the boundary in either direction.** A
/// function body that begins with a marker pending would consume a frame its
/// caller entered; one that ends with a marker pending has entered a frame
/// nobody consumed, and the enclosing restore would then hide it.
pub(super) struct CheckedFrameFunctionScope {
    enclosing_consumed: BTreeSet<ConsumedSubcontinuationFrame>,
}

impl CheckedFrameFunctionScope {
    /// Begin one generated `Function`'s consumption transaction.
    pub(super) fn open(compiler: &mut Lowering<'_>) -> Result<Self, CraneliftBackendError> {
        if compiler.active_subcontinuation_frame.is_some() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "a checked subcontinuation marker was still active when a generated function                  body began, so that body would consume a frame its caller entered",
            ));
        }
        // ⛔ `D8n` — the OLD lifecycle, restored under test: the set is shared
        // compile-wide instead of starting empty per function. It is the exact
        // pre-`D8n` behaviour, not an invented corruption, so the refusal it
        // brings back is the one this checkpoint repaired.
        #[cfg(test)]
        if d8n_compile_wide_lifecycle() {
            return Ok(Self {
                enclosing_consumed: compiler.consumed_subcontinuation_frames.clone(),
            });
        }
        Ok(Self {
            enclosing_consumed: std::mem::take(&mut compiler.consumed_subcontinuation_frames),
        })
    }

    /// End it, restoring the enclosing function's own consumption.
    pub(super) fn close(self, compiler: &mut Lowering<'_>) -> Result<(), CraneliftBackendError> {
        let dangling = compiler.active_subcontinuation_frame.take();
        // ⛔ Under the compile-wide switch the body's consumption is LEFT in
        // place instead of being rolled back, which is the other half of the
        // pre-`D8n` behaviour. Restoring here while sharing at `open` would
        // still hand the second function an empty set and reproduce nothing --
        // the mutation has to be faithful at both ends or it measures neither.
        #[cfg(test)]
        if d8n_compile_wide_lifecycle() {
            if dangling.is_some() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "a generated function body ended with a checked subcontinuation marker still                      active, so it entered a frame nothing consumed",
                ));
            }
            return Ok(());
        }
        compiler.consumed_subcontinuation_frames = self.enclosing_consumed;
        if dangling.is_some() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "a generated function body ended with a checked subcontinuation marker still                  active, so it entered a frame nothing consumed",
            ));
        }
        Ok(())
    }
}

impl CheckedFrameBranchScope {
    fn capture(consumed: &BTreeSet<ConsumedSubcontinuationFrame>) -> Self {
        Self {
            baseline: consumed.clone(),
            union: consumed.clone(),
        }
    }

    fn start_successor(&self) -> BTreeSet<ConsumedSubcontinuationFrame> {
        self.baseline.clone()
    }

    fn merge_successor(&mut self, consumed: &BTreeSet<ConsumedSubcontinuationFrame>) {
        self.union.extend(consumed.iter().copied());
    }

    fn finish(self) -> BTreeSet<ConsumedSubcontinuationFrame> {
        self.union
    }

    #[cfg(any(test, feature = "px8-ds-test-support"))]
    fn harness(
        consumed: &mut BTreeSet<ConsumedSubcontinuationFrame>,
        mutation: FrameScopeHarnessMutation,
    ) -> FrameScopeHarnessWitness {
        let key = (71, 23);
        let mut scope = Self::capture(consumed);

        let mut first_successor = scope.start_successor();
        let first_consume_succeeds = first_successor.insert(key);
        let same_successor_duplicate_rejected = !first_successor.insert(key);
        scope.merge_successor(&first_successor);

        let mut second_successor = match mutation {
            FrameScopeHarnessMutation::SharedLedger => first_successor.clone(),
            FrameScopeHarnessMutation::Exact | FrameScopeHarnessMutation::DropUnion => {
                scope.start_successor()
            }
        };
        let second_successor_first_consume_succeeds = second_successor.insert(key);
        scope.merge_successor(&second_successor);

        *consumed = match mutation {
            FrameScopeHarnessMutation::DropUnion => scope.baseline,
            FrameScopeHarnessMutation::Exact | FrameScopeHarnessMutation::SharedLedger => {
                scope.finish()
            }
        };
        let post_join_duplicate_rejected = !consumed.insert(key);

        FrameScopeHarnessWitness {
            first_consume_succeeds,
            same_successor_duplicate_rejected,
            second_successor_first_consume_succeeds,
            post_join_duplicate_rejected,
        }
    }
}

#[cfg(any(test, feature = "px8-ds-test-support"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameScopeHarnessWitness {
    first_consume_succeeds: bool,
    same_successor_duplicate_rejected: bool,
    second_successor_first_consume_succeeds: bool,
    post_join_duplicate_rejected: bool,
}

#[cfg(any(test, feature = "px8-ds-test-support"))]
#[derive(Clone, Copy)]
enum FrameScopeHarnessMutation {
    Exact,
    SharedLedger,
    DropUnion,
}

/// The closed production routes that still require retained recursive descent.
///
/// This type is the D5 accounting: the selector produces one of these reasons
/// rather than consulting a second spelling list. D1/D2/D3/D6/D8 ported and
/// admitted recursive positions, trap terminals, carried host-effect seats, and
/// result-directed joins; D7/S4 exercise their corrected governed composition.
/// S4's completed-emission rows establish collection capability only; they are
/// not an asymptotic verdict about those rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RecursiveDescentResidual {
    /// An ordinary match consuming an active computational recursor.
    MatchScrutineeRecursor,
    /// A lexical unit call whose argument is an active computational recursor.
    ///
    /// The recursive result still carries invocation-local scope/return-hole
    /// state. Passing it through a separately declared lexical unit is not one
    /// of the completed functionized ports, so the established recursive
    /// descent lane retains the whole call.
    LexicalCallArgumentRecursor,
    // ⭐⭐ **`RT-DECL-CLOSURE-PORT` `D6` RETIRED `TransparentDeclarationClosure`.**
    //
    // A transparent declaration whose body is a closure seed is now reached as a
    // separately owned callable unit, so it is no longer a reason to retain the
    // monolithic `RecursiveDescent` root. That is the whole node: `D1`-`D5a`
    // built the planner-owned declaration units, the typed transport, the
    // `DeclarationRef` calls and the complete owner/phase validation the ruling
    // required **before** this variant could be removed.
    //
    // ⛔ The variants above are untouched and the classifier below is still
    // exhaustive and fail-closed. This is a retirement, not a relaxation: the
    // selector is unchanged in kind and still refuses to select
    // `FunctionizedUnits` while any remaining variant fires.
    //
    // (That sentence read "the four variants above" until `RT-SEED-CALL-PORT`
    // `D3` removed one. A count in prose next to the thing it counts goes stale
    // the first time the population moves, which is why it is now stated as a
    // relation instead.)

    // **`RT-PRODUCER-MATCH-PORT` `D3` RETIRED `ProducerMatchCall`.**
    //
    // An ordinary producer `Match` whose scrutinee is directly a `Call` is now
    // lowered as a separately owned callable unit whose typed result crosses the
    // unit boundary into the match. `D2` built that port, and it is a delegation
    // to `lower_carried_match` rather than a second transport: the same
    // elimination the direct `RuntimeExpr::Match` route already used for a
    // carried scrutinee.
    //
    // The `D2` selector witness is gone with the variant. It existed only to
    // make the ported arm reachable while this classification still fired, so
    // every `D2` control now reaches that arm exactly as production does.
    //
    // ⚠ **The port carries three conservative refusals**, for frame states the
    // carried elimination cannot express: a retained scrutinee index, a deferred
    // constructor case, and a trailing composed eliminator. Retiring the class
    // makes those refusals **live in production for the first time**. They fail
    // closed, so the direction is over-strict rather than unsound, and they have
    // no shape-reaching control — stated at the `D2` control block rather than
    // implied here.

    // **`RT-SEED-CALL-PORT` `D3` RETIRED `SeedClosureCall`.**
    //
    // A `Call` whose callee is the retained non-lexical closure form is now
    // lowered as that callee's planner-owned body unit, reached through the
    // existing typed `call_declared_unit` transport with exactly
    // `Parameter ++ Capture` inputs. `D2` built that port; this is its
    // activation, and the port arm stops being dead code at the moment this
    // variant disappears.
    //
    // The capability is PORTED, not deleted: `RuntimeExpr::Closure` remains a
    // live member of the public backend-neutral IR with its own evaluator
    // semantics, and a call to one is now handled rather than made unreachable.
    // Deleting the shape instead would have made the same closure value callable
    // after a `DeclarationRef` but unlawful written directly, purely by source
    // position (Architect `evt_7p8dmg1rez02c`).
    //
    // The `D2` selector witness is gone with the variant. It existed only to
    // make the port arm reachable while this classification still fired, so it
    // has no remaining purpose and every `D2` control now runs unhooked.
}

/// Produce the retained reason, if any, from the exhaustive source walk.
///
/// Wrapper and child-producing forms propagate a reason from their children.
/// The exhaustive match is the fail-closed default: a new `RuntimeExpr` form
/// cannot compile until this production classifier assigns it to the
/// functionized population or to a typed retained reason.
fn recursive_descent_residual(expr: &RuntimeExpr) -> Option<RecursiveDescentResidual> {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
        | RuntimeExpr::Closure { body, .. } => recursive_descent_residual(body),
        RuntimeExpr::LexicalClosure { captures, body, .. } => captures
            .iter()
            .find_map(recursive_descent_residual)
            .or_else(|| recursive_descent_residual(body)),
        RuntimeExpr::Let { value, body } => {
            recursive_descent_residual(value).or_else(|| recursive_descent_residual(body))
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => recursive_descent_residual(scrutinee)
            .or_else(|| recursive_descent_residual(then_expr))
            .or_else(|| recursive_descent_residual(else_expr)),
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            args.iter().find_map(recursive_descent_residual)
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => matches!(
            scrutinee.as_ref(),
            RuntimeExpr::ComputationalMatch { cases, .. }
                if cases
                    .iter()
                    .any(|case| !case.recursive_positions.is_empty())
        )
        .then_some(RecursiveDescentResidual::MatchScrutineeRecursor)
        .or_else(|| recursive_descent_residual(scrutinee))
        .or_else(|| {
            cases
                .iter()
                .find_map(|case| recursive_descent_residual(&case.body))
        }),
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => recursive_descent_residual(scrutinee).or_else(|| {
            cases
                .iter()
                .find_map(|case| recursive_descent_residual(&case.body))
        }),
        RuntimeExpr::Record { fields } => fields
            .iter()
            .find_map(|(_, value)| recursive_descent_residual(value)),
        RuntimeExpr::Project { record, .. } => recursive_descent_residual(record),
        RuntimeExpr::Call { callee, args } => {
            (matches!(callee.as_ref(), RuntimeExpr::LexicalClosure { .. })
                && args.iter().any(|argument| {
                    matches!(
                        argument,
                        RuntimeExpr::ComputationalMatch { cases, .. }
                            if cases
                                .iter()
                                .any(|case| !case.recursive_positions.is_empty())
                    )
                }))
            .then_some(RecursiveDescentResidual::LexicalCallArgumentRecursor)
            .or_else(|| recursive_descent_residual(callee))
            .or_else(|| args.iter().find_map(recursive_descent_residual))
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => capability
            .as_ref()
            .and_then(|capability| recursive_descent_residual(&capability.value))
            .or_else(|| args.iter().find_map(recursive_descent_residual)),
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => None,
    }
}

/// **`RT-DECL-CLOSURE-PORT` `D1` -- report EVERY residual variant present, not
/// the first.**
///
/// [`recursive_descent_residual`] answers the selector's question: *is there at
/// least one residual?* It is built from `.or_else(..)` and `find_map`, so it
/// stops at the first hit and is **correct for that question and useless for
/// this one**. ⛔ This walk never short-circuits: every classification is
/// recorded and every child is visited regardless of what a sibling produced.
///
/// ⚠ **The reason this instrument owes a compound control rather than a
/// plausibility read.** If it silently kept the short-circuit it would report
/// exactly one variant on the governed fixture -- which is precisely the answer
/// the frame leads a reader to expect, so nothing would look wrong. Only a
/// program that fires two or more variants can tell the two behaviours apart.
///
/// ⭐ The campaign reuses this instrument (`RT-SEED-CALL-PORT`,
/// `RT-PRODUCER-MATCH-PORT`, `RT-RECURSOR-TRANSPORT`, `RT-DESCENT-RETIRE`), and
/// `RT-DESCENT-RETIRE`'s "no residual fires anywhere" becomes vacuous at exactly
/// the moment it authorizes deleting the lane if this walk has a gap. Re-prove
/// it cheaply at each point of use -- `D2`-`D6` rewrite this file underneath it.
fn enumerate_recursive_descent_residuals(
    expr: &RuntimeExpr,
    declarations: &BTreeMap<&str, &RuntimeDeclaration>,
) -> BTreeSet<RecursiveDescentResidual> {
    // `RT-SEED-CALL-PORT` `D1a`: revert to the short-circuiting twin's answer.
    // This is the regression the instrument exists to prevent, injected at the
    // instrument itself rather than at a convenient downstream point, so a
    // control that stays green under it is measuring something else.
    #[cfg(test)]
    if residual_enumeration_mutation() == ResidualEnumerationMutation::ShortCircuitLikeTheSelector {
        return recursive_descent_residual(expr)
            .or_else(|| {
                declarations
                    .values()
                    .find_map(|declaration| declaration_recursive_descent_residual(declaration))
            })
            .into_iter()
            .collect();
    }
    let mut found = BTreeSet::new();
    collect_recursive_descent_residuals(expr, &mut found);
    for declaration in declarations.values() {
        collect_declaration_recursive_descent_residuals(declaration, &mut found);
    }
    found
}

/// The non-short-circuiting twin of [`recursive_descent_residual`].
///
/// ⛔ The `match` is exhaustive with no wildcard arm, exactly as its twin is
/// (`AC-5`): a new `RuntimeExpr` form must still be unable to compile until
/// someone classifies it. A wildcard here would make the instrument silently
/// under-report the moment the IR grows.
fn collect_recursive_descent_residuals(
    expr: &RuntimeExpr,
    found: &mut BTreeSet<RecursiveDescentResidual>,
) {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
        | RuntimeExpr::Closure { body, .. } => {
            collect_recursive_descent_residuals(body, found);
        }
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for capture in captures {
                collect_recursive_descent_residuals(capture, found);
            }
            collect_recursive_descent_residuals(body, found);
        }
        RuntimeExpr::Let { value, body } => {
            collect_recursive_descent_residuals(value, found);
            collect_recursive_descent_residuals(body, found);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_recursive_descent_residuals(scrutinee, found);
            collect_recursive_descent_residuals(then_expr, found);
            collect_recursive_descent_residuals(else_expr, found);
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for argument in args {
                collect_recursive_descent_residuals(argument, found);
            }
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            // The classification, then BOTH walks. The twin stops after
            // whichever fires first; this one records and keeps walking.
            if matches!(
                scrutinee.as_ref(),
                RuntimeExpr::ComputationalMatch { cases, .. }
                    if cases
                        .iter()
                        .any(|case| !case.recursive_positions.is_empty())
            ) {
                found.insert(RecursiveDescentResidual::MatchScrutineeRecursor);
            }
            collect_recursive_descent_residuals(scrutinee, found);
            for case in cases {
                collect_recursive_descent_residuals(&case.body, found);
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_recursive_descent_residuals(scrutinee, found);
            for case in cases {
                collect_recursive_descent_residuals(&case.body, found);
            }
        }
        RuntimeExpr::Record { fields } => {
            for (_, value) in fields {
                collect_recursive_descent_residuals(value, found);
            }
        }
        RuntimeExpr::Project { record, .. } => {
            collect_recursive_descent_residuals(record, found);
        }
        RuntimeExpr::Call { callee, args } => {
            if matches!(callee.as_ref(), RuntimeExpr::LexicalClosure { .. })
                && args.iter().any(|argument| {
                    matches!(
                        argument,
                        RuntimeExpr::ComputationalMatch { cases, .. }
                            if cases
                                .iter()
                                .any(|case| !case.recursive_positions.is_empty())
                    )
                })
            {
                found.insert(RecursiveDescentResidual::LexicalCallArgumentRecursor);
            }
            collect_recursive_descent_residuals(callee, found);
            for argument in args {
                collect_recursive_descent_residuals(argument, found);
            }
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability.as_ref() {
                collect_recursive_descent_residuals(&capability.value, found);
            }
            for argument in args {
                collect_recursive_descent_residuals(argument, found);
            }
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
    }
}

/// The non-short-circuiting twin of
/// [`declaration_recursive_descent_residual`].
///
/// ⛔ The twin uses `.or_else(..)` and stops at its first hit; this walk records
/// every classification and visits every child regardless of what a sibling
/// produced. ⚠ `D6` retired the declaration-head variant, so a transparent
/// declaration now contributes exactly what its **body** contributes — but the
/// distinction between the two functions is unchanged and still load-bearing for
/// the rest of the campaign.
fn collect_declaration_recursive_descent_residuals(
    declaration: &RuntimeDeclaration,
    found: &mut BTreeSet<RecursiveDescentResidual>,
) {
    match &declaration.kind {
        RuntimeDeclarationKind::Transparent { body } => {
            collect_recursive_descent_residuals(body, found);
        }
        RuntimeDeclarationKind::Primitive { .. }
        | RuntimeDeclarationKind::Data { .. }
        | RuntimeDeclarationKind::Record { .. }
        | RuntimeDeclarationKind::RecursiveGroup { .. }
        | RuntimeDeclarationKind::EffectBoundary { .. }
        | RuntimeDeclarationKind::MetadataOnly => {}
    }
}

/// Produce the retained reason from the exhaustive declaration-kind route.
fn declaration_recursive_descent_residual(
    declaration: &RuntimeDeclaration,
) -> Option<RecursiveDescentResidual> {
    match &declaration.kind {
        // ⭐⭐ **`RT-DECL-CLOSURE-PORT` `D6` — THE ACTIVATION.**
        //
        // A transparent declaration's own head no longer contributes a retained
        // reason. Its body is classified exactly as any other expression is, so
        // a closure-seed declaration selects `FunctionizedUnits` unless
        // something in the body genuinely retains the lane.
        //
        // ⚠ **The `cfg(test)` selector witness that used to sit here is gone,
        // and its removal is half the deliverable.** It existed to break a
        // measured circularity: `D5` had to demonstrate its validator
        // fail-closed on an *accepted* input, a checked recursive declaration
        // call requires a closure-seed body, and this arm was exactly what kept
        // such a program on `RecursiveDescent`. With the variant retired the
        // route is reachable in production, so every control it governed now
        // runs **unhooked** — which is the evidence `D6` owes and the reason the
        // frame ordered activation after acceptance rather than with it.
        RuntimeDeclarationKind::Transparent { body } => recursive_descent_residual(body),
        RuntimeDeclarationKind::Primitive { .. }
        | RuntimeDeclarationKind::Data { .. }
        | RuntimeDeclarationKind::Record { .. }
        | RuntimeDeclarationKind::RecursiveGroup { .. }
        | RuntimeDeclarationKind::EffectBoundary { .. }
        | RuntimeDeclarationKind::MetadataOnly => None,
    }
}

/// The one temporary B2F migration selector, evaluated once at compilation
/// entry from source syntax and declaration kinds only.
///
/// `FunctionizedUnits` is selected only after both exhaustive production
/// classifiers produce no typed retained reason. No runtime value, carrier
/// class, walk result, or emission failure can change this answer after it is
/// chosen.
fn select_body_emission_authority(
    expr: &RuntimeExpr,
    declarations: &BTreeMap<&str, &RuntimeDeclaration>,
) -> BodyEmissionAuthority {
    if recursive_descent_residual(expr)
        .or_else(|| {
            declarations
                .values()
                .find_map(|declaration| declaration_recursive_descent_residual(declaration))
        })
        .is_some()
    {
        BodyEmissionAuthority::RecursiveDescent
    } else {
        BodyEmissionAuthority::FunctionizedUnits
    }
}

pub(in crate::cranelift_backend) fn compile_expr_into_module<'a, M: Module>(
    module: M,
    function_name: &str,
    linkage: Linkage,
    // `'a`, not an anonymous borrow: the plan files each planned occurrence's term
    // by reference, so the source tree must outlive the lowering that resolves
    // tags against it. Nothing borrowed reaches `CompiledModule`, which has no
    // lifetime parameter — see `Lowering::static_transition_plan`.
    expr: &'a RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    compile_expr_into_module_with_root_projection(
        module,
        function_name,
        linkage,
        expr,
        seed_env,
        declarations,
        staged_process_input,
        process_mode,
        process_symbols,
        native_join_plan,
        oriented_subcontinuation_plan,
        false,
        false,
    )
}

/// Compile an object entry whose public scalar launcher consumes a scalar,
/// while generated-unit calls continue to exchange their planner-selected
/// carrier words internally.
pub(in crate::cranelift_backend) fn compile_expr_into_object_module<'a, M: Module>(
    module: M,
    function_name: &str,
    linkage: Linkage,
    expr: &'a RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    compile_expr_into_module_with_root_projection(
        module,
        function_name,
        linkage,
        expr,
        seed_env,
        declarations,
        staged_process_input,
        process_mode,
        process_symbols,
        native_join_plan,
        oriented_subcontinuation_plan,
        !process_mode,
        process_mode,
    )
}

#[allow(clippy::too_many_arguments)]
fn compile_expr_into_module_with_root_projection<'a, M: Module>(
    mut module: M,
    function_name: &str,
    linkage: Linkage,
    expr: &'a RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
    project_public_scalar_root: bool,
    root_trap_process_sentinel: bool,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    #[cfg(test)]
    {
        scale_b_reset_emission_attempt();
        C2_UNIT_EMISSION_EPOCH.with(|epoch| epoch.set(Some(0)));
        RECURSIVE_POSITION_UNIT_CALLS.with(|calls| calls.set(0));
        reset_d8_join_conversion_counts();
    }
    validate_oriented_subcontinuation_transport(
        expr,
        &declarations,
        oriented_subcontinuation_plan.as_ref(),
    )?;
    // `RT-SEED-CALL-PORT` `D1` — observe the FULL residual set on the real
    // program, at the same site and from the same inputs the selector consumes.
    // Recording it anywhere else would measure a reconstruction of the program
    // rather than the one about to be compiled.
    #[cfg(test)]
    {
        let observed = enumerate_recursive_descent_residuals(expr, &declarations);
        OBSERVED_RESIDUALS.with(|cell| *cell.borrow_mut() = Some(observed));
    }
    let body_emission_authority = select_body_emission_authority(expr, &declarations);
    // Boundary A of RT-NATIVE-FNSPLIT: close and validate the factored static
    // graph before Cranelift sees any semantic body. The plan's positional
    // child-origin table is reachable from the lowering, so
    // every occurrence carries the static name the planner already gave it.
    //
    // ⚠ The plan also outlives this call's borrow of `expr` and holds each planned
    // occurrence BY REFERENCE, because a retained closure body is now selected by
    // its origin rather than carried as a clone. The emitter is otherwise
    // unchanged, and nothing borrowed reaches `CompiledModule`.
    let process_symbols = process_symbols
        .cloned()
        .unwrap_or_else(crate::NativeProcessSymbols::legacy_prelude);
    let static_transition_plan = plan_static_transition_graph_with_symbols(
        expr,
        &declarations,
        &process_symbols,
        if process_mode {
            AbiRootIngress::Process
        } else {
            AbiRootIngress::Value
        },
        matches!(
            body_emission_authority,
            BodyEmissionAuthority::FunctionizedUnits
        ),
    )?;
    #[cfg(test)]
    scale_b_begin_emission_attempt(
        &static_transition_plan,
        matches!(
            body_emission_authority,
            BodyEmissionAuthority::FunctionizedUnits
        ),
    );
    let mut sig = module.make_signature();
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.returns.push(AbiParam::new(types::I64));

    #[cfg(test)]
    C2_UNIT_EMISSION_EPOCH.with(|epoch| {
        epoch.set(Some(
            epoch
                .get()
                .expect("the C2 compilation epoch was initialized")
                .checked_add(1)
                .expect("the C2 compilation epoch fits u64"),
        ));
    });
    let func_id = module
        .declare_function(function_name, linkage, &sig)
        .map_err(|err| backend_module(err.to_string()))?;
    let native_int_wrapping_mutation = {
        #[cfg(test)]
        {
            NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get)
                == NativeIntLoweringMutation::Wrapping
        }
        #[cfg(not(test))]
        {
            false
        }
    };
    let native_int = crate::native_int_clif::emit_native_int_local_graph(
        &mut module,
        native_int_wrapping_mutation,
    )?;
    // `RT-FNSPLIT-B2V` `D3` — the boundary-value interface is declared and
    // defined in EVERY module, a fixed Θ(1) population, so a unit can project a
    // transfer wherever it is emitted rather than only where some caller
    // arranged a decoder.
    //
    // ⛔ `D6` INERT: nothing calls these yet. This adds no generated function
    // for any semantic origin, no cross-owner call and no second body-emission
    // authority — `RT-FNSPLIT-B2F` performs the switch-over that consumes them.
    // The population is emitted unconditionally so that B2F's switch-over is a
    // change of caller, never a change of what a module contains.
    // ⛔ `RECUT 2` — the emission plan is DERIVED from the representation
    // authority here, at the single-owner seam, and passed into the emitter.
    // The emitter consumes it to build the helper bodies' legal class sets; it
    // does not restate the authority and cannot reach it (`BoundaryInput` is
    // private to `cranelift_backend::lowering`). Ruled in scope and required by
    // the Architect: production codegen consumption is not `B2F` activation.
    let boundary_plan = crate::boundary_value::BoundaryEmissionPlan::derive();
    // ⭐ `RT-FNSPLIT-C1` `AC-C8` — the emitted graph's result is **consumed**,
    // not bound to `_`. ⚠ Labelled honestly: this is *necessary, not
    // sufficient*. Consuming the handle only proves the helpers are reachable;
    // `AC-C7`'s three per-eliminator executable-edge tests are what make it
    // evidence that the carrier is live. ⛔ Do not report this line alone.
    let boundary_value_abi = crate::boundary_value_clif::emit_boundary_value_local_graph(
        &mut module,
        &native_int,
        &boundary_plan,
    )?;
    let host_dispatch = if process_mode {
        let mut host_sig = module.make_signature();
        host_sig
            .params
            .push(AbiParam::new(module.target_config().pointer_type()));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig
            .params
            .push(AbiParam::new(module.target_config().pointer_type()));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig.returns.push(AbiParam::new(types::I64));
        Some(
            module
                .declare_function("ken_host_dispatch_v1", Linkage::Import, &host_sig)
                .map_err(|err| backend_module(err.to_string()))?,
        )
    } else {
        None
    };
    // ⭐ `RT-FNSPLIT-B2F` `D1` — forward-declare the WHOLE target-unit bundle
    // before any body (root or unit) is defined. A unit body may call any other
    // unit, so declaring every signature first is what makes the call graph
    // order-independent; a declare-and-define-in-one-pass loop could not emit a
    // call to a unit it had not reached yet.
    //
    // ⛔ The population is `B2O`'s validated owner partition as `B2R` described
    // it. This call does not derive it and must never be made to.
    // ⭐ `RT-FNSPLIT-B2F` `AC-11` clause 3 — the per-transfer representability
    // proof runs HERE, before a single unit is declared, defined or called.
    //
    // ⛔ Its position is the discharge. Moving this call below
    // `declare_unit_bundle` would satisfy everything the check asserts and prove
    // nothing about emission, and ⛔ no path may substitute `AbiPlane::validate`,
    // `C4`, or descriptor existence for it.
    // ⭐ The attempt epoch is stamped HERE, on the last statement before the
    // proof, so that "this compile reached emission and declared zero units" is
    // observable as a distinct outcome from "this compile never got here".
    // ⛔ Not inside `declare_unit_bundle`: stamping there would make the zero
    // reading unreachable, because observing the epoch would require declaring
    // the unit whose absence is the thing being measured.
    let functionized_bundle = match body_emission_authority {
        BodyEmissionAuthority::RecursiveDescent => None,
        BodyEmissionAuthority::FunctionizedUnits => {
            #[cfg(test)]
            super::units::b2f_reached_emission_seam();
            static_transition_plan.validate_emitted_transfers_are_representable()?;
            let units = super::units::declare_unit_bundle(&mut module, &static_transition_plan)?;
            // ⭐ `RT-FNSPLIT-B2F` `D4` — resolve every cross-owner call edge
            // against the bundle before a single body is defined.
            let calls = super::units::resolve_call_edges(&static_transition_plan, &units)?;
            Some((units, calls))
        }
    };
    // ⭐ `RT-FNSPLIT-B2F` `D3` — mint the artifact-static seed material before
    // any function context exists. `B2R` declared `GroundValueCarrier` as
    // `BorrowedForActivation` from `ArtifactStatic` and deliberately minted
    // nothing; this is the counterpart that gives the borrow an owner which
    // outlives every activation.
    //
    // ⛔ Minted from the environment, never from the plan: resolving which
    // symbols a unit captures would add an `origin -> expression` lookup, and
    // `AC-4` holds that count at exactly one.
    let seed_material = super::seed_material::mint_seed_material(&mut module, seed_env)?;
    let mut ctx = module.make_context();
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);
    // ⭐ `RT-FNSPLIT-B2F` `S6` — the module-level identities, gathered under one
    // name so the root and every future unit body resolve them through **one**
    // operation. What stood here was twenty inline `declare_*_in_func` calls;
    // the point of the move is that a helper cannot be present in the root's
    // function and absent from a unit's by someone forgetting to copy a line.
    //
    // ⚠ `seed_material` is `D3`'s minted artifact-static material: a `DataId` is
    // a module-level identity and cannot be addressed from inside a body, so it
    // is resolved into this `Function` exactly as the native-int and
    // boundary-carrier helpers are.
    let helpers = ArtifactHelpers {
        seed_material: &seed_material,
        host_dispatch,
        native_int: &native_int,
        boundary_value_abi: &boundary_value_abi,
    };
    let root_trap_exit = match body_emission_authority {
        BodyEmissionAuthority::RecursiveDescent => Some(TrapExitAuthority::Root {
            process_sentinel: root_trap_process_sentinel,
            source_authorized: true,
        }),
        BodyEmissionAuthority::FunctionizedUnits => None,
    };
    let root_function_local =
        helpers.declare_in_func(&mut module, &mut ctx.func, root_trap_exit);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        continuation_claims: None,
        checked_call_ledger: None,
        defining_unit: None,
        defining_emission_owner: None,
        defining_function_id: None,
        aggregate_allocations: None,
        host_effect_seats: None,
        seed_env,
        declarations,
        static_transition_plan,
        declaration_stack: Vec::new(),
        active_recursive_declarations: Vec::new(),
        result_table: BTreeMap::new(),
        next_token: 0,
        next_recursor_frame_provenance: 0,
        next_recursor_producer_origin: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        next_source_predecessor: 0,
        live_source_continuations: 0,
        source_control_root: None,
        active_oriented_semantic_regions: 0,
        active_carried_computational_eliminations: Vec::new(),
        native_join_plan,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan,
        consumed_subcontinuation_frames: BTreeSet::new(),
        active_subcontinuation_frame: None,
        consumed_recursive_call_templates: BTreeSet::new(),
        pending_recursive_call: None,
        pending_computational_ih_call: None,
        active_recursive_invocations: Vec::new(),
        next_recursive_invocation_instance: 1,
        dynamic_splice_edges: BTreeMap::new(),
        next_dynamic_splice_edge: 1,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        body_emission_authority,
        process_object: process_mode,
        process_symbols,
        #[cfg(test)]
        native_int_mutation: NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get),
        #[cfg(test)]
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
        function_local: root_function_local,
    };
    let root_result = match body_emission_authority {
        BodyEmissionAuthority::FunctionizedUnits => {
            let (unit_bundle, call_edges) = functionized_bundle
                .as_ref()
                .expect("the functionized selector arm owns its bundle");
            // `RT-DECL-CLOSURE-PORT` `D5a` checkpoint 2 — THE ONE LEDGER'S
            // LIFETIME, opened and closed HERE rather than inside any single
            // definition pass.
            //
            // ⛔ It used to open and close inside `define_unit_bodies`, which
            // is the FIRST of the passes that declare, claim and emit causal
            // calls. Its exact-set equality therefore ran while the
            // generated-context pass had not yet had a chance to declare
            // anything, and would report a planned token absent that was about
            // to be discharged. ⇒ The defect was the *lifetime*, not the
            // equality.
            //
            // ⭐ The position is the whole deliverable. Both endpoints sit in
            // this one block, around every pass that can own a causal token, so
            // "one global equality" is visible in a single place instead of
            // being a property a reader must reconstruct from three files.
            // There is deliberately no per-pass partial close and no second
            // mirrored ledger — the passes accumulate into this one and it is
            // checked once.
            super::units::open_continuation_claim_ledger(&mut compiler, unit_bundle)?;
            let root_result = super::units::define_unit_bodies(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
                call_edges,
                staged_process_input,
            )?;
            // `D5a` checkpoint 4 step 3 — the reaching mutation for the ONE
            // ledger's lifetime, and it is the checkpoint-2 defect itself:
            // close after the FIRST definition pass, before any generated
            // `Function` exists. ⛔ Nothing about the equality moves; only the
            // window it is taken over.
            #[cfg(test)]
            if d5a_route_mutation() == D5aRouteMutation::CloseLedgerAfterTheFirstPass {
                record_d5a_route_application();
                super::units::close_continuation_claim_ledger(&mut compiler)?;
                // `D7` — the relation's enforced laws close here: every event
                // maps to exactly one record, every committed pair is unique,
                // every related record is in `P`, no body is built twice, and
                // no event set is left open.
                //
                // ⚠ `image(R) ⊆ P`, deliberately not equality. `P` is a closed
                // AUTHORIZATION population: it plans a record for every
                // allocation-reachable node of every seat's tree under every
                // emission owner the seat may be lowered by, while one
                // compilation emits only the bodies it has. An unused record is
                // lawful, and requiring equality refused ordinary programs by
                // 1 to 132 records when measured.
                let _aggregate_relation =
                    super::units::close_aggregate_allocation_ledger(&mut compiler)?;
                let _effect_seats = super::units::close_host_effect_seat_ledger(&mut compiler)?;
            }
            // `RT-CONTSPEC-ACTIVATE` `D2` — define each declared continuation
            // target from its own projected contract, after the ordinary
            // bodies and before the root adapter.
            super::units::define_continuation_bodies(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
            )?;
            // `RT-DECL-CLOSURE-PORT` `D5a` — define each generated producer
            // execution context, after the specializations that call them.
            // Declaration already happened in the one up-front bundle pass, so
            // this ordering is a readability choice, not a linking constraint.
            super::units::define_continuation_context_bodies(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
                call_edges,
            )?;
            compiler.require_complete_join_plan_consumption()?;
            compiler.require_complete_dynamic_splice_edge_consumption()?;
            super::units::define_root_adapter(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
                func_id,
                process_mode,
                project_public_scalar_root,
            )?;
            // Every generated `Function` that can own a causal token now exists
            // and has recorded itself, so the ONE global exact-set equality
            // runs here.
            //
            // ⚠ Closing right after the last definition pass and before the
            // root adapter is the tempting spot. It is closed after the adapter
            // instead: the adapter is itself a generated `Function`, and
            // closing before it would make a causal ref declared there
            // invisible to the equality rather than caught by it. It declares
            // none today — that is a fact about the adapter, not a reason to
            // narrow the window.
            super::units::close_continuation_claim_ledger(&mut compiler)?;
                // `D7` — the relation's enforced laws close here: every event
                // maps to exactly one record, every committed pair is unique,
                // every related record is in `P`, no body is built twice, and
                // no event set is left open.
                //
                // ⚠ `image(R) ⊆ P`, deliberately not equality. `P` is a closed
                // AUTHORIZATION population: it plans a record for every
                // allocation-reachable node of every seat's tree under every
                // emission owner the seat may be lowered by, while one
                // compilation emits only the bodies it has. An unused record is
                // lawful, and requiring equality refused ordinary programs by
                // 1 to 132 records when measured.
                let _aggregate_relation =
                    super::units::close_aggregate_allocation_ledger(&mut compiler)?;
            // `D7` — planned seats against consumed seats, exactly.
            let _effect_seats = super::units::close_host_effect_seat_ledger(&mut compiler)?;
            root_result
        }
        BodyEmissionAuthority::RecursiveDescent => {
            let mut maybe_trap = None;
            let mut decoder = None;
            {
                let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
                let block = builder.create_block();
                builder.append_block_params_for_function_params(block);
                builder.switch_to_block(block);
                let ingress = builder.block_params(block)[0];
                let services = builder.block_params(block)[1];
                let pointer_type = module.target_config().pointer_type();
                let native_int_arena = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    services,
                    crate::activation_services::SERVICES_NATIVE_INT_ARENA,
                );
                Lowering::require_nonzero(&mut builder, native_int_arena);
                let boundary_arena = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    services,
                    crate::activation_services::SERVICES_BOUNDARY_ARENA,
                );
                Lowering::require_nonzero(&mut builder, boundary_arena);
                compiler.function_local.services_pointer = Some(services);
                compiler.function_local.native_int_arena = Some(native_int_arena);
                compiler.function_local.boundary_arena = Some(boundary_arena);

                let mut initial_env = Vec::new();
                if process_mode {
                    let process_input = builder.ins().load(
                        pointer_type,
                        MemFlags::trusted(),
                        ingress,
                        crate::boundary_activation::ROOT_INGRESS_PROCESS_INPUT,
                    );
                    Lowering::require_nonzero(&mut builder, process_input);
                    let host_dispatch_context = builder.ins().load(
                        pointer_type,
                        MemFlags::trusted(),
                        ingress,
                        crate::boundary_activation::ROOT_INGRESS_HOST_DISPATCH_CONTEXT,
                    );
                    Lowering::require_nonzero(&mut builder, host_dispatch_context);
                    let capability = builder.ins().load(
                        types::I64,
                        MemFlags::trusted(),
                        ingress,
                        crate::boundary_activation::ROOT_INGRESS_CAPABILITY,
                    );
                    compiler.function_local.host_dispatch_context = Some(host_dispatch_context);
                    initial_env.push(LoweringEnvironmentBinding::Value(
                        LoweringOperand::Specialized(Lowered::BorrowedNativeValue {
                            pointer: process_input,
                        }),
                    ));
                    initial_env.push(LoweringEnvironmentBinding::Value(
                        LoweringOperand::Specialized(Lowered::CapabilityToken { value: capability }),
                    ));
                } else {
                    compiler.function_local.host_dispatch_context =
                        Some(builder.ins().iconst(pointer_type, 0));
                }
                if let Some(value) = staged_process_input {
                    initial_env.push(LoweringEnvironmentBinding::Value(
                        LoweringOperand::Specialized(compiler.lower_value(&mut builder, value)?),
                    ));
                }
                compiler.root_terminal_authority =
                    compiler.take_distinguished_root_answer_authority()?;
                let root_origin = compiler.static_transition_plan.root_static_origin()?;
                let root = compiler.retained_body_occurrence(root_origin)?;
                compiler.select_terminal_result_origins(root_origin, root.expr)?;
                let lowered = compiler.lower_expr(&mut builder, root, &initial_env)?;
                // RecursiveDescent still owns the explicit active-recursor
                // residual. It inlines across generated-unit owner boundaries,
                // so the function-owner equality used by FunctionizedUnits is
                // inapplicable here. Static Match reachability is nevertheless
                // closed at this generated root boundary: otherwise a recursive
                // source-machine revisit can emit one case and later classify
                // that same subtree as dead.
                compiler.validate_recursive_descent_join_disposition()?;
                compiler.require_complete_join_plan_consumption()?;
                compiler.require_complete_dynamic_splice_edge_consumption()?;
                match lowered {
                    LoweringOperand::Carried(word) if process_mode => {
                        let tag = builder
                            .ins()
                            .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
                        Lowering::require_i64(
                            &mut builder,
                            tag,
                            BoundaryTag::ImmediateExitStatus as i64,
                        );
                        let status = compiler.emit_carrier_scalar(&mut builder, word)?;
                        builder.ins().return_(&[status]);
                        decoder = Some(ResultDecoder::ProcessStatus);
                    }
                    LoweringOperand::Carried(word) => {
                        builder.ins().return_(&[word.word]);
                        decoder = Some(ResultDecoder::Boundary);
                    }
                    LoweringOperand::Specialized(Lowered::Trap(trap)) => {
                        #[cfg(test)]
                        if process_mode {
                            px8tr_record_trap_provenance(
                                Px8trTrapProvenanceEvent::FinalProcessObjectTrap {
                                    trap: trap.clone(),
                                },
                            );
                        }
                        let status = builder
                            .ins()
                            .iconst(types::I64, if process_mode { -4 } else { 0 });
                        builder.ins().return_(&[status]);
                        maybe_trap = Some(trap);
                    }
                    LoweringOperand::Specialized(value) => {
                        let (token, result_decoder) = compiler.emit_result(&mut builder, value)?;
                        builder.ins().return_(&[token]);
                        decoder = Some(result_decoder);
                    }
                }
                builder.seal_all_blocks();
                builder.finalize();
            }
            compiler.validate_recursive_descent_materialized_dead_join_cfg(&ctx.func)?;
            verify_cranelift_function(&ctx.func, module.isa())?;
            #[cfg(test)]
            scale_b_record_recursive_descent_root(&ctx.func);
            module
                .define_function(func_id, &mut ctx)
                .map_err(|err| backend_module(err.to_string()))?;
            super::units::RootUnitResult {
                decoder,
                trap: maybe_trap,
            }
        }
    };
    let trap_catalog = compiler.static_transition_plan.trap_catalog();
    let carrier_identity_catalog = compiler
        .static_transition_plan
        .carrier_identity_catalog()?;
    let compiled = CompiledModule::from_parts(
        module,
        func_id,
        root_result.decoder,
        compiler.result_table,
        root_result.trap,
        trap_catalog,
        carrier_identity_catalog,
        true,
        compiler.assumptions,
        compiler.unsupported,
    );
    #[cfg(test)]
    scale_b_finish_emission_attempt();
    Ok(compiled)
}

/// `RT-CONTSRC-PRODUCER-LOCAL` `AC-1` -- the source-carried CONTROL mutation
/// family, for the activation-gate controls of families 5 and 2a.
///
/// Test-only, and deliberately NOT a widening of `D7`'s
/// `EffectSeatDispatchMutation`: that family perturbs effect-seat dispatch and
/// these two perturb the carried `Match` route. Sharing one enum would let a
/// control claim a mutation it did not apply.
///
/// Each variant refuses **after the real production decision has already been
/// taken**, and does nothing else. It never manufactures a carrier, terminal or
/// planner fact, never rewrites a join target, never duplicates the dispatch and
/// never lowers an alternative. That is what makes an application evidence that
/// the production path reached that exact point, rather than evidence about the
/// mutation's own machinery.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceCarriedControlMutation {
    /// Production behaviour.
    Exact,
    /// Family 5 -- refuse an ALREADY-CLASSIFIED `LoweringOperand::Carried` with
    /// the exact refusal the pre-repair seat produced.
    RefuseClassifiedCarried,
    /// Family 2a -- refuse an ALREADY-SPLIT `SourcePrefixTerminal::Join` before
    /// its inherited target is used.
    RefuseSplitInheritedJoin,
}

#[cfg(test)]
thread_local! {
    static SOURCE_CARRIED_CONTROL_MUTATION: std::cell::Cell<SourceCarriedControlMutation> =
        const { std::cell::Cell::new(SourceCarriedControlMutation::Exact) };
    /// How many times the active mutation actually fired.
    ///
    /// This is the anti-vacuity instrument. A mutated run that refuses with
    /// the right message but a count of zero refused for some OTHER reason, and
    /// is not evidence.
    static SOURCE_CARRIED_CONTROL_APPLICATIONS: std::cell::Cell<u32> =
        const { std::cell::Cell::new(0) };
}

/// Restores `Exact` on drop, so a panicking control cannot leak a mutation into
/// the next test on this thread.
#[cfg(test)]
struct SourceCarriedControlMutationGuard;

#[cfg(test)]
impl Drop for SourceCarriedControlMutationGuard {
    fn drop(&mut self) {
        SOURCE_CARRIED_CONTROL_MUTATION
            .with(|cell| cell.set(SourceCarriedControlMutation::Exact));
    }
}

/// Run `body` under `mutation`, returning its value and the number of times the
/// mutation fired. The counter is reset on entry, and `Exact` is restored on
/// exit even if `body` panics.
#[cfg(test)]
fn with_source_carried_control_mutation<R>(
    mutation: SourceCarriedControlMutation,
    body: impl FnOnce() -> R,
) -> (R, u32) {
    let _guard = SourceCarriedControlMutationGuard;
    SOURCE_CARRIED_CONTROL_APPLICATIONS.with(|cell| cell.set(0));
    SOURCE_CARRIED_CONTROL_MUTATION.with(|cell| cell.set(mutation));
    let value = body();
    let applications = SOURCE_CARRIED_CONTROL_APPLICATIONS.with(std::cell::Cell::get);
    (value, applications)
}

/// `Some(refusal)` when `mutation` is the active one; the caller returns it
/// unchanged. Counting happens here so a hook cannot fire without being counted.
#[cfg(test)]
fn source_carried_control_refusal(
    mutation: SourceCarriedControlMutation,
    construct: &'static str,
    reason: &'static str,
) -> Option<CraneliftBackendError> {
    if SOURCE_CARRIED_CONTROL_MUTATION.with(std::cell::Cell::get) != mutation {
        return None;
    }
    SOURCE_CARRIED_CONTROL_APPLICATIONS.with(|cell| cell.set(cell.get() + 1));
    Some(unsupported(construct, reason))
}

/// The status the carried source-machine `Match` returns when the boundary
/// word's CLASS is one this case set never decoded.
///
/// Named locally and deliberately: this is NOT claimed to be a canonical
/// carrier-wide failure word. It reuses the value the dynamic-constructor
/// emitter already returns for a malformed represented value, because both mean
/// *"this word is not the representation this chain decodes"*.
///
/// NOTHING PINS THIS VALUE, and an earlier draft of this comment claimed a
/// wrong-class control did. It does not exist and cannot be written today:
/// `mismatch_block` is emitted on the residual arm of the class chain, and a
/// sentinel sweep of the whole `ken-runtime` lib suite measured that arm as
/// reached ZERO times. `lower_source_carried_match` is entered exactly once in
/// the crate -- by
/// `constructors::ac1_source_machine_match_classifies_a_carried_scrutinee_by_phase`
/// -- and that entry refuses at join acquisition, before any selector is
/// emitted. So a divergence in this value is a SILENT DRIFT, not a test
/// failure. Retiring this paragraph needs a fixture that reaches the arm, not
/// a re-reading of the code.
const CARRIED_REPRESENTATION_MISMATCH_STATUS: i64 = MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS;

/// One emitted-case descriptor for the carried source-machine `Match`.
struct SourceCarriedCase {
    index: usize,
    emitted: bool,
    identity: u64,
    binders: i64,
    borrowed: Option<(i64, usize)>,
}

impl<'a> Lowering<'a> {
    fn resume_active_continuation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: LoweringOperand,
        active: ActiveContinuationFrame<'_>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let Some((head, tail)) = active.pending.split_first() else {
            return Ok(value);
        };
        let cursor = self.mint_continuation_cursor();
        let successor = EliminatorFrame::Active(ActiveContinuationFrame {
            activation: active.activation,
            cursor,
            parent: Some(&active),
            pending: tail,
            selected_ancestry: active.selected_ancestry,
            source_lineage: active.source_lineage,
            source_selected_cursor: active.source_selected_cursor,
            selected_scope: active.selected_scope,
        });
        self.lower_computational_match_value_composed(builder, RoutedAnswer::direct(value), &[*head, successor])
    }

    /// ⛔⛔ **`AC-C4` clause 3 — a carried residual is a transferred VALUE, never
    /// a transferred callable.** So an induction-hypothesis invocation against
    /// one may not carry source arguments, and this refuses **before** any
    /// invocation segment is installed or any semantic region entered.
    ///
    /// ⭐ **The zero-argument structural IH route is the admitted carried
    /// route**, and it is the only one. A function-valued recursive field would
    /// need the residual to *be* a closure over the carrier — the durable
    /// closure lane the ruling explicitly withholds — so it stays excluded by
    /// the existing closure-transfer prohibition rather than by a new check.
    ///
    /// ⚠ **Why this is a shared associated fn and not four inline `if`s.** Each
    /// of the four residual consumers reaches its carried arm by a different
    /// route, and a refusal that drifted at one of them would be a hole with
    /// three green siblings — the shape `AC-C7` already caught once on this
    /// node. One body, one message, one place to mutate.
    ///
    /// ⚠ Takes a **count**, not a slice: the four consumers hold their argument
    /// lists in two different forms (source `RuntimeExpr`s on three routes, an
    /// already-lowered operand vector on the source-machine route), and the
    /// property is about arity in both. A slice parameter would have forced one
    /// of them to spell its own refusal.
    fn reject_carried_residual_arguments(arguments: usize) -> Result<(), CraneliftBackendError> {
        if arguments == 0 {
            return Ok(());
        }
        Err(unsupported(
            "BoundaryCarrier",
            format!(
                "a carried recursive hypothesis is an eliminated value, not a callable, \
                 so it takes no arguments, but the call provides {arguments}"
            ),
        ))
    }

    /// `call_origin` is the origin of the `Call` occurrence `args` belong to.
    #[allow(clippy::too_many_arguments)]
    fn lower_recursor_residual_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        residual: &LoweringOperand,
        args: &[RuntimeExpr],
        call_origin: StaticOriginId,
        argument_env: &[LoweringEnvironmentBinding],
        saved_producer_env: &[LoweringEnvironmentBinding],
        outer_eliminators: &[EliminatorFrame<'_>],
        recursive_unit_body: Option<StaticOriginId>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⭐⭐ `AC-C4` — the carried residual, taken BEFORE the specialized
        // shapes so a carried word never reaches a template probe.
        if let LoweringOperand::Carried(word) = residual {
            if let Some(body) = recursive_unit_body.filter(|_| {
                matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::FunctionizedUnits
                )
            }) {
                let inputs = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg = self.child_occurrence(call_origin, 1 + position, arg)?;
                        self.lower_expr(builder, arg, argument_env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                // ⚠ No invocation segment is in scope on the pending-`Let`
                // resumption, so no coordinates can be supplied. The callee
                // fails closed if this body has a generated context.
                let returned =
                    self.call_declared_recursive_position_unit(builder, body, &inputs, None)?;
                return self.lower_computational_match_value_composed(
                    builder,
                    RoutedAnswer::direct(returned),
                    outer_eliminators,
                );
            }
            Self::reject_carried_residual_arguments(args.len())?;
            return self.lower_computational_match_value_composed(
                builder,
                RoutedAnswer::direct(LoweringOperand::Carried(*word)),
                outer_eliminators,
            );
        }
        let residual = residual.specialized_ref_at("a pending-let recursor residual")?;
        if let Lowered::BoundedNat(predecessor) = residual {
            if !args.is_empty() {
                return Err(unsupported(
                    "BoundedNat",
                    "structural Nat recursive hypothesis takes no arguments",
                ));
            }
            return self.lower_bounded_nat_computational(
                builder,
                *predecessor,
                false,
                outer_eliminators,
            );
        }
        let Lowered::Closure {
            captures,
            params,
            body,
        } = residual
        else {
            return Err(unsupported(
                "ComputationalMatch",
                "recursive constructor field is not a closure",
            ));
        };
        let mut call_env = args
            .iter()
            .enumerate()
            .map(|(position, arg)| {
                let arg = self.child_occurrence(call_origin, 1 + position, arg)?;
                self.lower_expr(builder, arg, argument_env)
                    .map(LoweringEnvironmentBinding::Value)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if params.len() != call_env.len() {
            return Err(unsupported(
                "ComputationalMatch",
                format!(
                    "recursive field expects {} args but call provides {}",
                    params.len(),
                    call_env.len()
                ),
            ));
        }
        extend_captures(&mut call_env, captures.iter().cloned());
        call_env.extend_from_slice(saved_producer_env);
        self.lower_computational_producer_expr(
            builder,
            self.retained_body_occurrence(*body)?,
            &call_env,
            outer_eliminators,
        )
    }

    /// `static_origin` is the `ComputationalMatch` occurrence's own origin, so
    /// `scrutinee` is its child `0` and case *i*'s body is its child `1 + i`.
    #[allow(clippy::too_many_arguments)]
    fn lower_computational_match_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: SourceOccurrence<'_>,
        cases: &[crate::RuntimeComputationalMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        producer_env: &[LoweringEnvironmentBinding],
        eliminator_env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // `D8m` — the shared derivation. ⛔ Not re-spelled here: the checked
        // bridge must carry this exact tuple, and two spellings is how they part.
        let checked = self.checked_computational_frame(cases, default)?;
        let provenance = self.mint_recursor_frame_provenance();
        self.lower_computational_producer_expr(
            builder,
            scrutinee,
            producer_env,
            &[EliminatorFrame::Computational(
                ComputationalEliminatorFrame {
                    cases,
                    default,
                    env: eliminator_env,
                    static_origin,
                    retained_scrutinee_index: None,
                    deferred_constructor_case: None,
                    provenance,
                    checked_frame_id: checked.id,
                    checked_invocation_id: checked.invocation_id,
                    checked_invocation_source: checked.invocation_source,
                    checked_invocation_depth: checked.invocation_depth,
                    answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                },
            )],
        )
    }

    /// Lowers one source occurrence as a *producer* under a stack of eliminator
    /// frames.
    ///
    /// This is the second traversal of the same source population — the one that
    /// reaches occurrences the direct descent does not — so it threads origins by
    /// exactly the same table as `lower_expr`: no guessed subset, both routes or
    /// neither.
    fn lower_computational_producer_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        producer_env: &[LoweringEnvironmentBinding],
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let SourceOccurrence {
            expr: scrutinee,
            static_origin,
        } = occurrence;
        if eliminators.is_empty() {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        }
        if matches!(eliminators[0], EliminatorFrame::InvocationReturn) {
            return self.lower_expr(builder, occurrence, producer_env);
        }
        if let EliminatorFrame::PendingLet(continuation) = eliminators[0] {
            let value = self.lower_expr(builder, occurrence, producer_env)?;
            if matches!(
                value,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
            }
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = value {
                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
            }
            let mut continuation_env = vec![LoweringEnvironmentBinding::Value(value)];
            continuation_env.extend_from_slice(continuation.env);
            return self.lower_recursor_residual_call(
                builder,
                continuation.residual,
                continuation.args,
                continuation.call_origin,
                &continuation_env,
                continuation.env,
                &eliminators[1..],
                continuation.recursive_unit_body,
            );
        }
        if let EliminatorFrame::Active(active) = eliminators[0] {
            if !matches!(
                scrutinee,
                RuntimeExpr::Let { .. }
                    | RuntimeExpr::Call { .. }
                    | RuntimeExpr::Match { .. }
                    | RuntimeExpr::ComputationalMatch { .. }
                    | RuntimeExpr::If { .. }
            ) {
                let value = self.lower_expr(builder, occurrence, producer_env)?;
                return self.resume_active_continuation(builder, value, active);
            }
        }
        self.enter_source_occurrence_plan(static_origin)?;
        match scrutinee {
            RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                self.enter_checked_subcontinuation_frame(*frame_id)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let result = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                );
                if self.active_subcontinuation_frame.take().is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "checked subcontinuation marker was not consumed by its frame",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                body,
                ..
            } => {
                let instance = self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let result = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                );
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.lower_computational_producer_expr(builder, body, producer_env, eliminators)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                // `D8f` — the marker's application occurrence, derived once and
                // used both to enter the marker and to lower the body, so the
                // occupancy check compares against the occurrence the lowering
                // actually visits.
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.enter_checked_computational_ih_invocation(
                    *call_template_id,
                    body.expr,
                    body.static_origin,
                )?;
                let value = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                )?;
                self.finish_checked_computational_ih_marker(value)
            }
            RuntimeExpr::Let { value, body } => {
                // The `Let`'s own children: value `0`, body `1`. When the body
                // is itself the `Call` below, that `Call` occurrence's origin is
                // this body child — which is what the pending-let frame carries
                // so its arguments stay positionally derivable.
                let body_origin = self
                    .static_transition_plan
                    .child_static_origin(static_origin, 1)?;
                if reaches_environment_computational_recursor(body, producer_env, 1) {
                    if let RuntimeExpr::Call { callee, args } = body.as_ref() {
                        if let RuntimeExpr::Var(index) = callee.as_ref() {
                            if let Some(index) = (*index as usize).checked_sub(1) {
                                if let Some(LoweringEnvironmentBinding::Value(
                                    LoweringOperand::Specialized(
                                        callee @ Lowered::ComputationalRecursorClosure { .. },
                                    ),
                                )) = producer_env.get(index)
                                {
                                    let (residual, boundary) = decompose_computational_recursor(
                                        LoweringOperand::Specialized(callee.clone()),
                                    );
                                    let (activation, invocation) = boundary.expect(
                                        "recursor closure carries a continuation delimiter",
                                    );
                                    let recursive_unit_body = invocation.recursive_unit_body;
                                    let resume_cursor = invocation.resume_cursor;
                                    let current =
                                        active_recursor_frame(eliminators).ok_or_else(|| {
                                            unsupported(
                                                "ComputationalRecursor",
                                                "recursive invocation has no active continuation",
                                            )
                                        })?;
                                    let _resume = find_continuation_cursor(current, resume_cursor)
                                        .ok_or_else(|| {
                                            unsupported(
                                                "ComputationalRecursor",
                                                "recursive invocation cursor is not active",
                                            )
                                        })?;
                                    if !recursor_invocation_is_checked(&invocation) {
                                        validate_recursor_invocation_segment(&invocation)?;
                                    }
                                    let dynamic_splice_edges =
                                        self.take_dynamic_splice_edges(&invocation)?;
                                    let installed = compose_oriented_subcontinuation(
                                        self.oriented_subcontinuation_plan.as_ref(),
                                        self.active_recursive_invocations.last().copied(),
                                        activation,
                                        invocation,
                                        dynamic_splice_edges,
                                    )?;
                                    let frames = installed_oriented_eliminator_frames(&installed);
                                    let mut composed = Vec::with_capacity(frames.len() + 2);
                                    composed.push(EliminatorFrame::PendingLet(
                                        PendingLetContinuationFrame {
                                            residual: &residual,
                                            args,
                                            call_origin: body_origin,
                                            env: producer_env,
                                            recursive_unit_body,
                                        },
                                    ));
                                    composed.extend(frames);
                                    composed.push(EliminatorFrame::InvocationReturn);
                                    self.enter_oriented_semantic_region(installed.checked);
                                    let value = self.child_occurrence(static_origin, 0, value)?;
                                    let returned = self.lower_computational_producer_expr(
                                        builder,
                                        value,
                                        producer_env,
                                        &composed,
                                    );
                                    self.leave_oriented_semantic_region(installed.checked);
                                    let returned = returned?;
                                    return self.lower_computational_match_value_composed(
                                        builder,
                                        RoutedAnswer::direct(returned),
                                        eliminators,
                                    );
                                }
                            }
                        }
                    }
                }
                let value_occurrence = self.child_occurrence(static_origin, 0, value)?;
                let body_occurrence = SourceOccurrence {
                    expr: body,
                    static_origin: body_origin,
                };
                let value = self.lower_expr(builder, value_occurrence, producer_env)?;
                if let LoweringOperand::Specialized(Lowered::Trap(trap)) = value {
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                }
                let mut body_env = vec![LoweringEnvironmentBinding::Value(value)];
                body_env.extend_from_slice(producer_env);
                self.lower_computational_producer_expr(
                    builder,
                    body_occurrence,
                    &body_env,
                    eliminators,
                )
            }
            RuntimeExpr::Call { callee, args } => {
                let join_plan = self.consumed_join_plan_token(static_origin)?;
                let callee = self.child_occurrence(static_origin, 0, callee)?;
                let callee = self.lower_expr(builder, callee, producer_env)?;
                match callee {
                    LoweringOperand::Specialized(Lowered::DeclarationClosure {
                        reference,
                        symbol,
                        captures,
                        params,
                        body,
                    }) => {
                        // `RT-DECL-CLOSURE-PORT` `D4`, consumer 2 of 3.
                        if self.body_emission_authority
                            == BodyEmissionAuthority::FunctionizedUnits
                        {
                            let args = args
                                .iter()
                                .enumerate()
                                .map(|(position, argument)| {
                                    let argument = self.child_occurrence(
                                        static_origin,
                                        1 + position,
                                        argument,
                                    )?;
                                    self.lower_expr(builder, argument, producer_env)
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            return self.call_declaration_closure_unit(
                                builder, reference, &symbol, &params, captures, args,
                            );
                        }
                        self.lower_recursive_declaration_call(
                            builder,
                            &symbol,
                            &captures,
                            &params,
                            self.retained_body_occurrence(body)?,
                            args,
                            static_origin,
                            producer_env,
                            Some(eliminators),
                            join_plan,
                        )
                    }
                    LoweringOperand::Specialized(Lowered::Closure {
                        captures,
                        params,
                        body,
                    }) => {
                        if matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::RecursiveDescent
                        ) {
                            let retained = self.retained_body_occurrence(body)?;
                            if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                                if let Some((cases, default)) =
                                    ordinary_match_continuation(&params, retained.expr)
                                {
                                    let argument =
                                        self.child_occurrence(static_origin, 1, &args[0])?;
                                    let frame_env =
                                        env_with_operands(captures.clone(), producer_env);
                                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                                    composed.push(EliminatorFrame::Ordinary(
                                        OrdinaryEliminatorFrame {
                                            cases,
                                            default,
                                            env: &frame_env,
                                            static_origin: retained.static_origin,
                                            retained_scrutinee_index: Some(0),
                                            deferred_constructor_case: None,
                                        },
                                    ));
                                    composed.extend_from_slice(eliminators);
                                    return self.lower_computational_producer_expr(
                                        builder,
                                        argument,
                                        producer_env,
                                        &composed,
                                    );
                                }
                            }
                        }
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "tree producer expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        let mut call_inputs = args
                            .iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                let lowered = self.lower_expr(builder, arg, producer_env)?;
                                match self.body_emission_authority {
                                    BodyEmissionAuthority::RecursiveDescent => Ok(lowered),
                                    BodyEmissionAuthority::FunctionizedUnits => Ok(match lowered {
                                        LoweringOperand::Carried(word) => {
                                            LoweringOperand::Carried(word)
                                        }
                                        LoweringOperand::Specialized(value) => {
                                            LoweringOperand::Carried(self.transfer_into_carrier(
                                                builder,
                                                arg.static_origin,
                                                &value,
                                            )?)
                                        }
                                    }),
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        // These operands serve two different roles below: a
                        // unit call's ordered inputs, and the prefix of a
                        // lexical environment. They stay operands here, and
                        // only the environment role crosses the binding
                        // authority -- there is no route back the other way.
                        call_inputs.extend(captures);
                        match self.body_emission_authority {
                            BodyEmissionAuthority::RecursiveDescent => {
                                let call_env = env_with_operands(call_inputs, producer_env);
                                let body = self.retained_body_occurrence(body)?;
                                self.lower_computational_producer_expr(
                                    builder,
                                    body,
                                    &call_env,
                                    eliminators,
                                )
                            }
                            BodyEmissionAuthority::FunctionizedUnits => {
                                let returned = self.call_declared_unit(
                                    builder,
                                    body,
                                    &call_inputs,
                                    #[cfg(test)]
                                    None,
                                )?;
                                self.lower_computational_match_value_composed(
                                    builder,
                                    RoutedAnswer::direct(returned),
                                    eliminators,
                                )
                            }
                        }
                    }
                    LoweringOperand::Specialized(
                        mut callee @ Lowered::ComputationalRecursorClosure { .. },
                    ) => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(&mut callee)?;
                        let (base, boundary) =
                            decompose_computational_recursor(LoweringOperand::Specialized(callee));
                        let (activation, invocation) =
                            boundary.expect("recursor closure carries an invocation segment");
                        let recursive_unit_body = invocation.recursive_unit_body;
                        // `D5a` checkpoint 4 step 1 — read the retained source
                        // coordinates BEFORE the segment is installed, beside the
                        // existing pre-move field read. Both are facts of the
                        // invocation, so both are taken while it is still in hand
                        // rather than reconstructed afterwards.
                        let carried_coordinates =
                            CarriedInvocationCoordinates::of(&invocation)?;
                        let current = active_recursor_frame(eliminators).ok_or_else(|| {
                            unsupported(
                                "ComputationalRecursor",
                                "recursive producer invocation has no active continuation",
                            )
                        })?;
                        let _resume = find_continuation_cursor(current, invocation.resume_cursor)
                            .ok_or_else(|| {
                            unsupported(
                                "ComputationalRecursor",
                                "recursive producer invocation cursor is not active",
                            )
                        })?;
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
                        let installed = compose_oriented_subcontinuation(
                            self.oriented_subcontinuation_plan.as_ref(),
                            checked_ih_invocation
                                .or_else(|| self.active_recursive_invocations.last().copied()),
                            activation,
                            invocation,
                            dynamic_splice_edges,
                        )?;
                        let mut composed = installed_oriented_eliminator_frames(&installed);
                        composed.push(EliminatorFrame::InvocationReturn);
                        // ⭐⭐ `AC-C4` — the carried residual resumes the SAME
                        // computational eliminator over the carried word, under
                        // the same semantic-region bracket the specialized
                        // `BoundedNat` arm below uses. ⛔ Not `specialized_at`,
                        // ⛔ not a reconstructed `Lowered`, ⛔ not the producer.
                        if let LoweringOperand::Carried(word) = base {
                            if let Some(body) = recursive_unit_body.filter(|_| {
                                matches!(
                                    self.body_emission_authority,
                                    BodyEmissionAuthority::FunctionizedUnits
                                )
                            }) {
                                let inputs = args
                                    .iter()
                                    .enumerate()
                                    .map(|(position, arg)| {
                                        let arg = self.child_occurrence(
                                            static_origin,
                                            1 + position,
                                            arg,
                                        )?;
                                        self.lower_expr(builder, arg, producer_env)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                self.enter_oriented_semantic_region(installed.checked);
                                let coordinates = carried_coordinates;
                                let returned = self
                                    .call_declared_recursive_position_unit(
                                        builder,
                                        body,
                                        &inputs,
                                        Some(coordinates),
                                    )
                                    .and_then(|value| {
                                        self.lower_computational_match_value_composed(
                                            builder, RoutedAnswer::direct(value), &composed,
                                        )
                                    });
                                self.leave_oriented_semantic_region(installed.checked);
                                let returned = returned?;
                                return self.lower_computational_match_value_composed(
                                    builder,
                                    RoutedAnswer::direct(returned),
                                    eliminators,
                                );
                            }
                            Self::reject_carried_residual_arguments(args.len())?;
                            self.enter_oriented_semantic_region(installed.checked);
                            let returned = self.lower_computational_match_value_composed(
                                builder,
                                RoutedAnswer::direct(LoweringOperand::Carried(word)),
                                &composed,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            let returned = returned?;
                            return self.lower_computational_match_value_composed(
                                builder,
                                RoutedAnswer::direct(returned),
                                eliminators,
                            );
                        }
                        let base = base.specialized_at("a recursor residual in a producer call")?;
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            self.enter_oriented_semantic_region(installed.checked);
                            let returned = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &composed,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            let returned = returned?;
                            return self.lower_computational_match_value_composed(
                                builder,
                                RoutedAnswer::direct(returned),
                                eliminators,
                            );
                        }
                        let Lowered::Closure {
                            captures,
                            params,
                            body,
                        } = base
                        else {
                            return Err(unsupported(
                                "ComputationalMatch",
                                "recursive constructor field is not a closure",
                            ));
                        };
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "recursive field expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        let mut call_inputs = args
                            .iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                let lowered = self.lower_expr(builder, arg, producer_env)?;
                                match self.body_emission_authority {
                                    BodyEmissionAuthority::RecursiveDescent => Ok(lowered),
                                    BodyEmissionAuthority::FunctionizedUnits => Ok(match lowered {
                                        LoweringOperand::Carried(word) => {
                                            LoweringOperand::Carried(word)
                                        }
                                        LoweringOperand::Specialized(value) => {
                                            LoweringOperand::Carried(self.transfer_into_carrier(
                                                builder,
                                                arg.static_origin,
                                                &value,
                                            )?)
                                        }
                                    }),
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        // Two roles, as above: ordered unit-call inputs, or the
                        // prefix of a lexical environment. Only the second
                        // crosses the binding authority.
                        call_inputs.extend(captures);
                        self.enter_oriented_semantic_region(installed.checked);
                        let returned = match self.body_emission_authority {
                            BodyEmissionAuthority::RecursiveDescent => {
                                let call_env = env_with_operands(call_inputs, producer_env);
                                let body = self.retained_body_occurrence(body)?;
                                self.lower_computational_producer_expr(
                                    builder, body, &call_env, &composed,
                                )
                            }
                            BodyEmissionAuthority::FunctionizedUnits => {
                                let returned = self.call_declared_unit(
                                    builder,
                                    body,
                                    &call_inputs,
                                    #[cfg(test)]
                                    None,
                                )?;
                                self.lower_computational_match_value_composed(
                                    builder, RoutedAnswer::direct(returned), &composed,
                                )
                            }
                        };
                        self.leave_oriented_semantic_region(installed.checked);
                        let returned = returned?;
                        self.lower_computational_match_value_composed(
                            builder,
                            RoutedAnswer::direct(returned),
                            eliminators,
                        )
                    }
                    _ => Err(unsupported(
                        "ComputationalMatch",
                        "tree producer callee is not a closure",
                    )),
                }
            }
            RuntimeExpr::Construct { constructor, args } => {
                let eliminator = eliminators[0];
                let terminal_exit = constructor == &self.process_symbols.exit_success
                    || constructor == &self.process_symbols.exit_failure;
                let itree_frame = match eliminator {
                    EliminatorFrame::Computational(frame) => frame
                        .cases
                        .iter()
                        .any(|case| case.constructor.contains("::ITree::")),
                    EliminatorFrame::Ordinary(frame) => frame
                        .cases
                        .iter()
                        .any(|case| case.constructor.contains("::ITree::")),
                    EliminatorFrame::PendingLet(_) => {
                        unreachable!("pending Let continuations are consumed before dispatch")
                    }
                    EliminatorFrame::InvocationReturn => {
                        unreachable!("invocation returns are consumed before dispatch")
                    }
                    EliminatorFrame::Active(_) => {
                        unreachable!("active continuation cursors do not consume constructors")
                    }
                };
                if terminal_exit && itree_frame {
                    let lowered_args = args
                        .iter()
                        .enumerate()
                        .map(|(position, arg)| {
                            let arg = self.child_occurrence(static_origin, position, arg)?;
                            self.lower_expr(builder, arg, producer_env)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(LoweringOperand::Specialized(Lowered::Constructor {
                        constructor: constructor.clone(),
                        synthesized_identity: Some(
                            self.static_transition_plan
                                .constructor_symbol_identity(static_origin)?,
                        ),
                        // `D7` -- the allocation lane is the second fact resolved
                        // at the producer and carried with the template.
                        occurrence: Some(self.static_transition_plan.source_aggregate_occurrence(
                            static_origin,
                            PlannedAggregateShape::Constructor,
                        )?),
                        args: specialized_operands_at(&lowered_args, "a constructor argument")?,
                    }));
                }
                // `D3` retains the selected computational case facts: the
                // claim at the producer occurrence below needs the case index
                // and that case's ruled recursive positions, and re-selecting
                // them later would be a second selection authority.
                let mut selected_computational: Option<(StaticOriginId, usize, Vec<usize>)> = None;
                let (case_body, argument_binder_offset) = match eliminator {
                    EliminatorFrame::Computational(eliminator) => {
                        let (case_index, case) = match eliminator
                            .cases
                            .iter()
                            .enumerate()
                            .find(|(_, case)| case.constructor == *constructor)
                        {
                            Some(selected) => selected,
                            None => {
                                self.disposition_statically_unselected_match_cases(
                                    eliminator.static_origin,
                                    None,
                                )?;
                                return Ok(LoweringOperand::Specialized(Lowered::Trap(
                                    eliminator.default.clone(),
                                )));
                            }
                        };
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            Some(case_index),
                        )?;
                        if case.argument_binders != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "case {} expects {} constructor arguments but value has {}",
                                    case.constructor,
                                    case.argument_binders,
                                    args.len()
                                ),
                            ));
                        }
                        let mut seen = BTreeSet::new();
                        for position in case.recursive_positions.iter().copied() {
                            if !seen.insert(position) || position >= args.len() {
                                return Err(unsupported(
                                    "ComputationalMatch",
                                    format!(
                                        "case {} has malformed recursive position {position}",
                                        case.constructor
                                    ),
                                ));
                            }
                        }
                        selected_computational = Some((
                            eliminator.static_origin,
                            case_index,
                            case.recursive_positions.clone(),
                        ));
                        (
                            self.case_body_occurrence(
                                eliminator.static_origin,
                                case_index,
                                &case.body,
                            )?,
                            case.recursive_positions.len(),
                        )
                    }
                    EliminatorFrame::Ordinary(eliminator) => {
                        let (case_index, case) = match select_ordinary_case(eliminator, constructor)
                        {
                            Ok(selected) => selected,
                            Err(trap) => {
                                self.disposition_statically_unselected_match_cases(
                                    eliminator.static_origin,
                                    None,
                                )?;
                                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                            }
                        };
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            Some(case_index),
                        )?;
                        if case.binders != args.len() {
                            return Err(unsupported(
                                "Match",
                                format!(
                                    "case {} expects {} binders but constructor has {} args",
                                    case.constructor,
                                    case.binders,
                                    args.len()
                                ),
                            ));
                        }
                        (
                            self.case_body_occurrence(
                                eliminator.static_origin,
                                case_index,
                                &case.body,
                            )?,
                            0,
                        )
                    }
                    EliminatorFrame::PendingLet(_) => {
                        unreachable!("pending Let continuations are consumed before dispatch")
                    }
                    EliminatorFrame::InvocationReturn => {
                        unreachable!("invocation returns are consumed before dispatch")
                    }
                    EliminatorFrame::Active(_) => {
                        unreachable!("active continuation cursors do not consume constructors")
                    }
                };

                // The bridge eliminator's cases live in the selected case body
                // itself (`immediate_binder_eliminator` matches only a body that
                // IS a match), so that body's origin is their parent.
                let bridge =
                    immediate_binder_eliminator(case_body.expr, argument_binder_offset, args.len());
                let bridge =
                    bridge.filter(|(field, _)| requires_heterogeneous_deforestation(&args[*field]));

                if let Some((field, consumer)) = bridge {
                    let lowered_prefix = args[..field]
                        .iter()
                        .enumerate()
                        .map(|(position, arg)| {
                            let arg = self.child_occurrence(static_origin, position, arg)?;
                            self.lower_expr(builder, arg, producer_env)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if let Some(LoweringOperand::Specialized(Lowered::Trap(trap))) =
                        lowered_prefix.iter().find(|value| {
                            matches!(value, LoweringOperand::Specialized(Lowered::Trap(_)))
                        })
                    {
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap.clone())));
                    }

                    let splice_caller = active_recursor_frame(&eliminators[1..]);
                    let mut selected_ancestry = splice_caller
                        .map(|active| active.selected_ancestry.to_vec())
                        .unwrap_or_default();
                    if let EliminatorFrame::Computational(frame) = eliminator {
                        selected_ancestry.push(frame.provenance);
                    }
                    let mut pending: Vec<_> = eliminators[1..]
                        .iter()
                        .copied()
                        .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                        .collect();
                    if let Some(active) = splice_caller {
                        pending.extend_from_slice(active.pending);
                    }
                    let selected_active = ActiveContinuationFrame {
                        activation: self.mint_continuation_activation(),
                        cursor: self.mint_continuation_cursor(),
                        parent: splice_caller.and_then(|active| active.parent),
                        pending: &pending,
                        selected_ancestry: &selected_ancestry,
                        source_lineage: splice_caller
                            .map(|active| active.source_lineage)
                            .unwrap_or(&[]),
                        source_selected_cursor: splice_caller
                            .and_then(|active| active.source_selected_cursor),
                        selected_scope: splice_caller.and_then(|active| active.selected_scope),
                    };
                    // ⭐ The prefix rebuilds the enclosing constructor's own
                    // **template** below (`outer_scrutinee`), so it is a
                    // specialized-only surface, not a spine edge.
                    let lowered_prefix =
                        specialized_operands_at(&lowered_prefix, "a deferred constructor prefix")?;
                    let deferred = DeferredConstructorCaseEnvironment {
                        constructor,
                        lowered_prefix: &lowered_prefix,
                        selected_field: field,
                        trailing_fields: &args[field + 1..],
                        construct_origin: static_origin,
                        producer_env,
                        outer_eliminator: eliminator,
                        splice_caller,
                        selected_active,
                    };
                    let mut composed = Vec::with_capacity(2);
                    composed.push(match consumer {
                        ImmediateBinderEliminator::Computational { cases, default } => {
                            // `D8m` — which arm this composed site took, at the
                            // one place that knows.
                            #[cfg(test)]
                            crate::cranelift_backend::lowering::record_d8m_bridge_arm(
                                self.defining_function_id,
                                crate::cranelift_backend::lowering::D8mBridgeArm::Computational,
                            );
                            EliminatorFrame::Computational(ComputationalEliminatorFrame {
                                cases,
                                default,
                                env: &[],
                                static_origin: case_body.static_origin,
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                                provenance: self.mint_recursor_frame_provenance(),
                                // `D8m` — an UNWRAPPED bridge stays all-None.
                                // The source declared no frame here, so there is
                                // none to preserve, and inventing one is exactly
                                // what this checkpoint forbids.
                                checked_frame_id: None,
                                checked_invocation_id: None,
                                checked_invocation_source: None,
                                checked_invocation_depth: 0,
                                answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                            })
                        }
                        // ⭐⭐ `D8m` — THE SOURCE FRAME, PRESERVED THROUGH THE
                        // BRIDGE.
                        //
                        // ⛔ Entered and consumed through the EXISTING pair, and
                        // only here, after the existing gates have already
                        // selected the bridge. That is deliberate: the
                        // consumption path is where the fingerprint agreement
                        // and the consumed-once law live, so routing through it
                        // means this adds no second validator and cannot drift
                        // from the one the direct path uses.
                        ImmediateBinderEliminator::CheckedComputational {
                            frame_id,
                            cases,
                            default,
                        } => {
                            #[cfg(test)]
                            crate::cranelift_backend::lowering::record_d8m_bridge_arm(
                                self.defining_function_id,
                                crate::cranelift_backend::lowering::D8mBridgeArm::CheckedComputational,
                            );
                            self.enter_checked_subcontinuation_frame(frame_id)?;
                            // ⛔ `D8m` — CONSUME WITH A SHAPE THE SOURCE MATCH
                            // DOES NOT CARRY, under test only. Same marker, same
                            // cases, one field of the default changed: the
                            // consumption law must refuse, which is what makes
                            // "the bridge is held to the match the marker
                            // wrapped" a measured fact rather than a reading of
                            // the call site.
                            let consumed_default = default;
                            #[cfg(test)]
                            let foreign_default = RuntimeTrap {
                                code: default.code.clone(),
                                message: format!("{} foreign", default.message),
                            };
                            #[cfg(test)]
                            let consumed_default = if d8m_foreign_consumed_shape() {
                                &foreign_default
                            } else {
                                consumed_default
                            };
                            // ⭐ `D8m` — the SAME derivation the direct path
                            // uses, whole. All four facts, not the id alone.
                            let checked =
                                self.checked_computational_frame(cases, consumed_default)?;
                            // ⛔ `D8m` — SUPPRESS THE TRANSPORTED TUPLE, under
                            // test only. The marker is still entered and
                            // consumed above, so the plan side is untouched and
                            // only the transport is withheld: the bridge then
                            // carries what it carried before this checkpoint,
                            // and the detached-frame refusal must come back.
                            #[cfg(test)]
                            let checked = if d8m_suppress_transported_tuple() {
                                CheckedComputationalFrame {
                                    id: None,
                                    invocation_id: None,
                                    invocation_source: None,
                                    invocation_depth: 0,
                                }
                            } else {
                                checked
                            };
                            EliminatorFrame::Computational(ComputationalEliminatorFrame {
                                cases,
                                default,
                                env: &[],
                                // ⛔ CHILD 0 of the marker occurrence, never the
                                // wrapper's own origin. The marker names the
                                // frame; the match IS the frame, and every
                                // origin-keyed lookup downstream -- case bodies,
                                // the planner's continuation origin -- must land
                                // on the match.
                                static_origin: {
                                    let wrapped = self
                                        .child_occurrence(case_body.static_origin, 0, case_body.expr)?
                                        .static_origin;
                                    // `D8m` — SUBSTITUTE THE WRAPPER'S OWN
                                    // OCCURRENCE, under test only. One node off,
                                    // and the only difference is which of two
                                    // nested occurrences downstream lookups key
                                    // on.
                                    #[cfg(test)]
                                    let wrapped = if d8m_wrapper_origin_substitution() {
                                        case_body.static_origin
                                    } else {
                                        wrapped
                                    };
                                    wrapped
                                },
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                                provenance: self.mint_recursor_frame_provenance(),
                                checked_frame_id: checked.id,
                                checked_invocation_id: checked.invocation_id,
                                checked_invocation_source: checked.invocation_source,
                                checked_invocation_depth: checked.invocation_depth,
                                // ⛔ UNCHANGED. Checked-frame presence is an
                                // identity fact, not a routing one; making it
                                // move the answer route would make the two forms
                                // of one source match lower differently.
                                answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                            })
                        }
                        ImmediateBinderEliminator::Ordinary { cases, default } => {
                            #[cfg(test)]
                            crate::cranelift_backend::lowering::record_d8m_bridge_arm(
                                self.defining_function_id,
                                crate::cranelift_backend::lowering::D8mBridgeArm::Ordinary,
                            );
                            EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                cases,
                                default,
                                env: &[],
                                static_origin: case_body.static_origin,
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                            })
                        }
                    });
                    composed.push(EliminatorFrame::Active(selected_active));
                    let selected = self.child_occurrence(static_origin, field, &args[field])?;
                    return self.lower_computational_producer_expr(
                        builder,
                        selected,
                        producer_env,
                        &composed,
                    );
                }

                let lowered_args = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg = self.child_occurrence(static_origin, position, arg)?;
                        self.lower_expr(builder, arg, producer_env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                // `RT-CONTSPEC-ACTIVATE` `D3` — THE PRODUCER OCCURRENCE.
                //
                // Fields are lowered; nothing has been transferred into a
                // carrier and the identity-erasing join has not run. This is
                // the seat where the four-field selector's operands all exist
                // and where the exact token is claimed at most once.
                let mut continuation_result: Option<RoutedAnswer> = None;
                if let Some((frame_origin, case_index, recursive_positions)) =
                    selected_computational.as_ref()
                {
                    for position in recursive_positions.iter().copied() {
                        // `D9` — the WHOLE lowered field run is handed down; the
                        // ordinary run is assembled from the planner's envelope
                        // at the one seat that resolves the unit.
                        //
                        // ⛔ This site used to build the run itself, filtering
                        // out the recursive field, under a comment claiming
                        // *"worker captures follow in capture-ordinal order"*.
                        // They did not: nothing here or downstream appended
                        // them, so a continuation whose selected worker had
                        // captures was called with its declared parameter tail
                        // unfilled.
                        if let Some(returned) = self.claim_and_call_continuation(
                            builder,
                            static_origin,
                            *frame_origin,
                            *case_index,
                            position,
                            &lowered_args,
                            producer_env,
                        )? {
                            // The call's own value is the result after the
                            // consumed computational frame, and it is never
                            // discarded: it is handed to the frame consumption
                            // in place of the constructor that would have been
                            // produced. Returning it directly here would skip
                            // the frame's planned-join disposition, which the
                            // emitter accounts for separately.
                            continuation_result = Some(returned);
                        }
                    }
                }
                let produced = if lowered_args
                    .iter()
                    .any(|argument| matches!(argument, LoweringOperand::Carried(_)))
                {
                    LoweringOperand::Carried(self.transfer_constructor_operands(
                        builder,
                        static_origin,
                        constructor,
                        &lowered_args,
                    )?)
                } else {
                    LoweringOperand::Specialized(Lowered::Constructor {
                        constructor: constructor.clone(),
                        // Carry the plan's already-resolved source identity with
                        // the template.  A later unit boundary may receive this
                        // result after nested producer traversal, where the
                        // caller occurrence is not the constructor occurrence
                        // and therefore cannot lawfully re-query its atom.
                        synthesized_identity: Some(
                            self.static_transition_plan
                                .constructor_symbol_identity(static_origin)?,
                        ),
                        // `D7` -- the allocation lane is the second fact resolved
                        // at the producer and carried with the template.
                        occurrence: Some(self.static_transition_plan.source_aggregate_occurrence(
                            static_origin,
                            PlannedAggregateShape::Constructor,
                        )?),
                        args: specialized_operands_at(&lowered_args, "a constructor argument")?,
                    })
                };
                // `D6a` upstream half -- ordinary evaluation STARTS direct and
                // the exact producer RAISES it. ⛔ Not a default written at the
                // consumer: a site that hard-codes `DirectScrutinee` on a path an
                // exact call result reaches would erase the fact being transported.
                let produced = continuation_result.unwrap_or_else(|| RoutedAnswer::direct(produced));
                self.lower_computational_match_value_composed(builder, produced, eliminators)
            }
            RuntimeExpr::Match {
                scrutinee,
                cases: producer_cases,
                default: producer_default,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                if let LoweringOperand::Specialized(Lowered::Bool { value, known }) = selected {
                    let true_case = producer_cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = producer_cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "ComputationalMatch",
                            "Bool tree producer requires True and False cases",
                        ));
                    };
                    if let Some(known) = known {
                        let (index, case) = if known { true_case } else { false_case };
                        self.disposition_statically_unselected_match_cases(
                            static_origin,
                            Some(index),
                        )?;
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        return self.lower_computational_producer_expr(
                            builder,
                            body,
                            producer_env,
                            eliminators,
                        );
                    }
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = join_plan
                        .has_continuing_predecessor
                        .then(|| builder.create_block());
                    if let Some(merge) = merge {
                        self.append_planned_join_params(builder, merge, &join_plan);
                    }
                    builder.ins().brif(value, true_block, &[], false_block, &[]);
                    let mut merge_kind = None;
                    for (block, (index, producer_case)) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let body =
                            self.case_body_occurrence(static_origin, index, &producer_case.body)?;
                        let lowered = self.lower_computational_producer_expr(
                            builder,
                            body,
                            producer_env,
                            eliminators,
                        )?;
                        if self.seal_source_trap_branch(builder, &lowered)? {
                            continue;
                        }
                        let merge = merge.ok_or_else(|| {
                            backend_module(
                                "join plan omitted a producer Bool Match merge despite a \
                                 continuing predecessor"
                                    .to_string(),
                            )
                        })?;
                        self.jump_planned_join_arm(
                            builder,
                            merge,
                            &join_plan,
                            body.static_origin,
                            lowered,
                            &mut merge_kind,
                            "ComputationalMatch",
                        )?;
                    }
                    let Some(merge) = merge else {
                        let unreachable = builder.create_block();
                        builder.switch_to_block(unreachable);
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(
                            producer_default.clone(),
                        )));
                    };
                    return self.finish_planned_join(
                        builder,
                        merge,
                        &join_plan,
                        merge_kind,
                        "ComputationalMatch",
                    );
                }
                if let LoweringOperand::Specialized(Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                }) = selected
                {
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
                    let ok_block = builder.create_block();
                    let err_block = builder.create_block();
                    let merge = join_plan
                        .has_continuing_predecessor
                        .then(|| builder.create_block());
                    if let Some(merge) = merge {
                        self.append_planned_join_params(builder, merge, &join_plan);
                    }
                    builder.ins().brif(success, ok_block, &[], err_block, &[]);
                    let mut merge_kind = None;
                    for (block, constructor, payload) in [
                        (ok_block, ok_constructor.as_str(), *ok),
                        (err_block, err_constructor.as_str(), *error),
                    ] {
                        builder.switch_to_block(block);
                        let lowered = if let Some((index, producer_case)) =
                            dynamic_host_result_producer_case(producer_cases, constructor)?
                        {
                            let case_env = env_with([payload], producer_env);
                            let body = self.case_body_occurrence(
                                static_origin,
                                index,
                                &producer_case.body,
                            )?;
                            self.lower_computational_producer_expr(
                                builder,
                                body,
                                &case_env,
                                eliminators,
                            )?
                        } else {
                            LoweringOperand::Specialized(Lowered::Trap(producer_default.clone()))
                        };
                        if self.seal_source_trap_branch(builder, &lowered)? {
                            continue;
                        }
                        let merge = merge.ok_or_else(|| {
                            backend_module(
                                "join plan omitted a producer HostResult merge despite a \
                                 continuing predecessor"
                                    .to_string(),
                            )
                        })?;
                        self.jump_planned_join_arm(
                            builder,
                            merge,
                            &join_plan,
                            static_origin,
                            lowered,
                            &mut merge_kind,
                            "ComputationalMatch",
                        )?;
                    }
                    let Some(merge) = merge else {
                        let unreachable = builder.create_block();
                        builder.switch_to_block(unreachable);
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(
                            producer_default.clone(),
                        )));
                    };
                    return self.finish_planned_join(
                        builder,
                        merge,
                        &join_plan,
                        merge_kind,
                        "ComputationalMatch",
                    );
                }
                if let LoweringOperand::Specialized(Lowered::DynamicConstructor(dynamic)) = selected
                {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Producer {
                            cases: producer_cases,
                            default: producer_default,
                            env: producer_env,
                            static_origin,
                            eliminators,
                        },
                    );
                }
                if let LoweringOperand::Specialized(Lowered::BoundedNat(nat)) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        static_origin,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                    };
                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                    composed.push(EliminatorFrame::Ordinary(frame));
                    composed.extend_from_slice(eliminators);
                    return self.lower_bounded_nat_computational(builder, nat, false, &composed);
                }
                if let LoweringOperand::Specialized(Lowered::StructuralNat(nat)) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        static_origin,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                    };
                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                    composed.push(EliminatorFrame::Ordinary(frame));
                    composed.extend_from_slice(eliminators);
                    return self.lower_bounded_nat_computational(
                        builder,
                        BoundedNatV1::derived_from_validated(nat.value),
                        true,
                        &composed,
                    );
                }
                let LoweringOperand::Specialized(Lowered::Constructor {
                    constructor, args, ..
                }) = selected
                else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match scrutinee is not Bool or a constructor",
                    ));
                };
                let Some((case_index, producer_case)) = producer_cases
                    .iter()
                    .enumerate()
                    .find(|(_, case)| case.constructor == constructor)
                else {
                    self.disposition_statically_unselected_match_cases(static_origin, None)?;
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(
                        producer_default.clone(),
                    )));
                };
                self.disposition_statically_unselected_match_cases(
                    static_origin,
                    Some(case_index),
                )?;
                if producer_case.binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match constructor arity changed",
                    ));
                }
                let case_env = env_with(args, producer_env);
                let body =
                    self.case_body_occurrence(static_origin, case_index, &producer_case.body)?;
                self.lower_computational_producer_expr(builder, body, &case_env, eliminators)
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee: inner_scrutinee,
                cases: inner_cases,
                default: inner_default,
            } => {
                // Fuse the inner eliminator ahead of the outer stack. Its
                // selected case body remains a producer for every outer frame;
                // no intermediate aggregate is materialized or exit-lowered.
                let mut composed = Vec::with_capacity(eliminators.len() + 1);
                let provenance = self.mint_recursor_frame_provenance();
                let checked = self.checked_computational_frame(inner_cases, inner_default)?;
                composed.push(EliminatorFrame::Computational(
                    ComputationalEliminatorFrame {
                        cases: inner_cases,
                        default: inner_default,
                        env: producer_env,
                        static_origin,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                        provenance,
                        checked_frame_id: checked.id,
                        checked_invocation_id: checked.invocation_id,
                        checked_invocation_source: checked.invocation_source,
                        checked_invocation_depth: checked.invocation_depth,
                        answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                    },
                ));
                composed.extend_from_slice(eliminators);
                let inner_scrutinee = self.child_occurrence(static_origin, 0, inner_scrutinee)?;
                self.lower_computational_producer_expr(
                    builder,
                    inner_scrutinee,
                    producer_env,
                    &composed,
                )
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                let then_expr = self.child_occurrence(static_origin, 1, then_expr)?;
                let else_expr = self.child_occurrence(static_origin, 2, else_expr)?;
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                let LoweringOperand::Specialized(Lowered::Bool { value, known }) = selected else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing If scrutinee is not Bool",
                    ));
                };
                if let Some(known) = known {
                    let unselected = if known { else_expr } else { then_expr };
                    self.disposition_statically_unselected_source_subtree(
                        unselected.static_origin,
                    )?;
                    return self.lower_computational_producer_expr(
                        builder,
                        if known { then_expr } else { else_expr },
                        producer_env,
                        eliminators,
                    );
                }
                let join_plan = self.consumed_join_plan_token(static_origin)?;
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge = join_plan
                    .has_continuing_predecessor
                    .then(|| builder.create_block());
                if let Some(merge) = merge {
                    self.append_planned_join_params(builder, merge, &join_plan);
                }
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                let mut merge_kind = None;
                let mut terminal_trap = None;
                for (block, branch) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_computational_producer_expr(
                        builder,
                        branch,
                        producer_env,
                        eliminators,
                    )?;
                    if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                        terminal_trap.get_or_insert_with(|| trap.clone());
                    }
                    if self.seal_source_trap_branch(builder, &lowered)? {
                        continue;
                    }
                    let merge = merge.ok_or_else(|| {
                        backend_module(
                            "join plan omitted a producer If merge despite a continuing \
                             predecessor"
                                .to_string(),
                        )
                    })?;
                    self.jump_planned_join_arm(
                        builder,
                        merge,
                        &join_plan,
                        branch.static_origin,
                        lowered,
                        &mut merge_kind,
                        "ComputationalMatch",
                    )?;
                }
                let Some(merge) = merge else {
                    let unreachable = builder.create_block();
                    builder.switch_to_block(unreachable);
                    let trap = terminal_trap.ok_or_else(|| {
                        backend_module(
                            "producer If join omitted both a continuing predecessor and a \
                             source trap"
                                .to_string(),
                        )
                    })?;
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                };
                self.finish_planned_join(
                    builder,
                    merge,
                    &join_plan,
                    merge_kind,
                    "ComputationalMatch",
                )
            }
            _ => {
                // Everything this producer dispatcher does not special-case is
                // handed to `lower_expr` **as the same occurrence**, origin
                // included — the producer-side twin of the source machine's
                // fallback arm.
                let value = self.lower_expr(builder, occurrence, producer_env)?;
                self.lower_computational_match_value_composed(builder, RoutedAnswer::direct(value), eliminators)
            }
        }
    }

    fn lower_computational_match_value_composed(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: RoutedAnswer,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let incoming_route = scrutinee.route;
        let scrutinee = scrutinee.value;
        let Some(eliminator) = eliminators.first().copied() else {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        };
        match eliminator {
            EliminatorFrame::Computational(frame) => {
                self.enter_source_occurrence_plan(frame.static_origin)?;
            }
            EliminatorFrame::Ordinary(frame) => {
                self.enter_source_occurrence_plan(frame.static_origin)?;
            }
            EliminatorFrame::PendingLet(_)
            | EliminatorFrame::InvocationReturn
            | EliminatorFrame::Active(_) => {}
        }
        // ⭐ The forwarding arm comes FIRST and stays phase-preserving: an
        // invocation return hands the operand straight back, so a `Carried`
        // survives it untouched. Only the composition path below reads a
        // compile-time template, so only that path takes the boundary.
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(scrutinee);
        }
        // ⭐⭐ `D3`'s CARRIED arm for the composed route, ahead of the boundary
        // below — otherwise a carried scrutinee reaching a real eliminator
        // would fail closed at `specialized_at` even though `§2g` gives it a
        // route. ⛔ The phase is classified with no wildcard.
        if let LoweringOperand::Carried(word) = scrutinee {
            return match eliminator {
                EliminatorFrame::Computational(mut frame) => {
                    // `D6a` -- the predecessor's route RAISES the frame's, and
                    // never lowers it. The frame's own field stays the
                    // recursor-layer producer's authority; this is the call-result
                    // producer's, and the two join.
                    let frame_field = frame.answer_route;
                    frame.answer_route =
                        RoutedAnswer { value: LoweringOperand::Carried(word), route: incoming_route }
                            .raise(frame.answer_route);
                    #[cfg(test)]
                    if d6a_route_mutation() == D6aRouteMutation::OverwriteIncomingWithFrameField {
                        record_d6a_route_application();
                        frame.answer_route = frame_field;
                    }
                    #[cfg(test)]
                    record_d6a_route_event(D6aRouteEvent::ConsumerRoute {
                        seat: D6aConsumerSeat::Composed,
                        static_origin: frame.static_origin,
                        incoming: incoming_route,
                        frame_field,
                        joined: frame.answer_route,
                    });
                    self.lower_carried_computational_match(builder, word, frame, &eliminators[1..])
                }
                // ── RT-PRODUCER-MATCH-PORT `D2` — THE CELL IS NOW LIVE ──────
                //
                // This arm used to refuse, on the reasoning that "a deforestable
                // producer is by construction one whose shape was read at compile
                // time, so a carried scrutinee cannot arrive here from today's
                // corpus". **That premise is false now, and `RT-SEED-CALL-PORT`
                // `D3` is what falsified it.** `requires_heterogeneous_
                // deforestation` classifies on the SOURCE shape -- a `Call`
                // producing a `Construct` is deforestable -- while the callee is
                // now lowered as a separately owned unit whose result crosses as
                // a carried word. Both are true at once, which the old comment
                // took to be impossible.
                //
                // The port is a delegation, not a new transport: the elimination
                // is `Self::lower_carried_match`, the same one the direct
                // `RuntimeExpr::Match` route uses at its own carried arm. The
                // producer-call scrutinee becomes a separately owned unit and its
                // typed result crosses that boundary into these cases.
                //
                // Three pieces of frame state this delegation CANNOT express are
                // refused rather than dropped. `lower_carried_match` takes cases,
                // default, origin and env and nothing else, so a frame carrying
                // more is not something this port has ported -- and silently
                // discarding it would be an unsound accept, not a narrower one.
                EliminatorFrame::Ordinary(frame) => {
                    if let Some(index) = frame.retained_scrutinee_index {
                        return Err(unsupported(
                            "BoundaryCarrier",
                            format!(
                                "a carried producer-call scrutinee reached an ordinary \
                                 eliminator that retains its scrutinee at binder {index}; the \
                                 carried elimination has no slot for a retained scrutinee, so \
                                 this shape is not ported"
                            ),
                        ));
                    }
                    if frame.deferred_constructor_case.is_some() {
                        return Err(unsupported(
                            "BoundaryCarrier",
                            "a carried producer-call scrutinee reached an ordinary eliminator \
                             carrying a deferred constructor case, whose fields are selected \
                             from a compile-time shape the carrier does not have",
                        ));
                    }
                    if !eliminators[1..].is_empty() {
                        return Err(unsupported(
                            "BoundaryCarrier",
                            "a carried producer-call scrutinee reached an ordinary eliminator \
                             with further composed eliminators behind it; the carried \
                             elimination consumes exactly one frame, so the remainder would be \
                             silently dropped",
                        ));
                    }
                    #[cfg(test)]
                    PRODUCER_MATCH_UNIT_PORTS.with(|calls| calls.set(calls.get() + 1));
                    self.lower_carried_match(
                        builder,
                        word,
                        frame.cases,
                        frame.default,
                        frame.static_origin,
                        frame.env,
                    )
                }
                EliminatorFrame::PendingLet(_) | EliminatorFrame::Active(_) => Err(unsupported(
                    "BoundaryCarrier",
                    "a carried scrutinee reached a continuation frame that resumes a \
                     compile-time value rather than eliminating one",
                )),
                // Answered above, before this match; spelled so the frame set
                // stays wildcard-free.
                EliminatorFrame::InvocationReturn => Ok(LoweringOperand::Carried(word)),
            };
        }
        let scrutinee = scrutinee.specialized_at("a composed computational-match scrutinee")?;
        if let Lowered::BoundedNat(nat) = scrutinee {
            return self.lower_bounded_nat_computational(builder, nat, false, eliminators);
        }
        if let Lowered::StructuralNat(nat) = scrutinee {
            return self.lower_bounded_nat_computational(
                builder,
                BoundedNatV1::derived_from_validated(nat.value),
                true,
                eliminators,
            );
        }
        let Lowered::Constructor {
            constructor,
            synthesized_identity,
            occurrence,
            args,
        } = scrutinee
        else {
            return Err(unsupported(
                "ComputationalMatch",
                "scrutinee is not a constructor value after ordinary expression lowering",
            ));
        };
        let retained_scrutinee = Lowered::Constructor {
            constructor: constructor.clone(),
            synthesized_identity,
            // Retaining a scrutinee re-presents the SAME producer, so it keeps
            // the same occurrence. Dropping it here would make the retained
            // copy refuse where the original emitted.
            occurrence,
            args: args.clone(),
        };
        let remaining_eliminators = &eliminators[1..];
        let (body, case_env) = match eliminator {
            EliminatorFrame::Computational(eliminator) => {
                let (case_index, case, _) = match select_computational_case(
                    std::slice::from_ref(&eliminator),
                    &constructor,
                ) {
                    Ok(selected) => selected,
                    Err(trap) => {
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            None,
                        )?;
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                    }
                };
                self.disposition_statically_unselected_match_cases(
                    eliminator.static_origin,
                    Some(case_index),
                )?;
                if case.argument_binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} expects {} constructor arguments but value has {}",
                            case.constructor,
                            case.argument_binders,
                            args.len()
                        ),
                    ));
                }
                let mut seen = BTreeSet::new();
                for position in case.recursive_positions.iter().copied() {
                    if !seen.insert(position) || position >= args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "case {} has malformed recursive position {position}",
                                case.constructor
                            ),
                        ));
                    }
                }

                let splice_caller = active_recursor_frame(remaining_eliminators);
                let mut selected_ancestry = splice_caller
                    .map(|active| active.selected_ancestry.to_vec())
                    .unwrap_or_default();
                selected_ancestry.push(eliminator.provenance);
                let mut pending: Vec<_> = remaining_eliminators
                    .iter()
                    .copied()
                    .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                    .collect();
                if let Some(active) = splice_caller {
                    pending.extend_from_slice(active.pending);
                }
                let activation = self.mint_continuation_activation();
                let cursor = self.mint_continuation_cursor();
                let producer_origin = self.mint_recursor_producer_origin();
                let selected_scope = OwnedSelectedScope {
                    scope_origin: producer_origin,
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                    frame: ComputationalRecursorFramePayload {
                        cases: eliminator.cases.to_vec(),
                        default: eliminator.default.clone(),
                        outer_env: eliminator.env.to_vec(),
                        static_origin: eliminator.static_origin,
                        provenance: eliminator.provenance,
                        checked_frame_id: eliminator.checked_frame_id,
                        checked_invocation_id: eliminator.checked_invocation_id,
                        checked_invocation_source: eliminator.checked_invocation_source,
                        checked_invocation_depth: eliminator.checked_invocation_depth,
                    },
                };
                let selected_scope = Some(selected_scope);
                let active_state = ActiveContinuationFrame {
                    activation,
                    cursor,
                    parent: splice_caller.and_then(|active| active.parent),
                    pending: &pending,
                    selected_ancestry: &selected_ancestry,
                    source_lineage: splice_caller
                        .map(|active| active.source_lineage)
                        .unwrap_or(&[]),
                    source_selected_cursor: splice_caller
                        .and_then(|active| active.source_selected_cursor),
                    selected_scope: selected_scope.as_ref(),
                };

                #[cfg(test)]
                px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::Composed,
                    origin: producer_origin,
                    cursor,
                    siblings: case.recursive_positions.len(),
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                });
                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                let ih_slots =
                    self.computational_ih_slots_for_case(case, eliminator.checked_frame_id)?;
                for position in case.recursive_positions.iter().rev().copied() {
                    // `RT-CONTSPEC-ACTIVATE` `D3` — THE PRODUCER OCCURRENCE
                    // IS HERE, and the claim cannot be made yet.
                    //
                    // `claim_and_call_continuation` below is ready and takes
                    // the ruled four-field selector. Three of its operands are
                    // in scope: the active computational-frame origin, the
                    // case index, and this recursive position. The fourth --
                    // the actual producer `Construct` origin -- is NOT: this
                    // function receives an already-lowered scrutinee operand,
                    // and `producer_origin` here is a minted
                    // `RecursorProducerOriginId`, a different axis entirely.
                    //
                    // ⛔ Substituting the frame origin or the minted recursor
                    // id would compile and would silently never match, so the
                    // ledger would report leftover claims and point at the
                    // wrong thing. Threading the real origin is a lowering
                    // signature change and is routed rather than invented.
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    let induction_hypothesis = self.make_computational_recursor(
                        // ⭐ `AC-C4` clause 1 — the SPECIALIZED caller wraps
                        // explicitly, so the phase is stated at the call site
                        // rather than inferred by the callee.
                        LoweringOperand::Specialized(args[position].clone()),
                        eliminator.cases.to_vec(),
                        eliminator.default.clone(),
                        eliminator.env.to_vec(),
                        eliminator.static_origin,
                        eliminator.provenance,
                        eliminator.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        activation,
                        cursor,
                        splice_caller,
                        None,
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(Px8jProducerPath::Composed, &induction_hypothesis);
                    induction_hypotheses.push(LoweringEnvironmentBinding::Value(induction_hypothesis));
                }
                let mut case_env = induction_hypotheses;
                extend_specialized(&mut case_env, args);
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Computational(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
                };
                case_env.extend(frame_env);
                let case_body =
                    self.case_body_occurrence(eliminator.static_origin, case_index, &case.body)?;
                if !case.recursive_positions.is_empty() {
                    return self.lower_source_machine(builder, case_body, &case_env, &active_state);
                }
                if remaining_eliminators.is_empty() {
                    return self.lower_expr(builder, case_body, &case_env);
                }
                return self.lower_computational_producer_expr(
                    builder,
                    case_body,
                    &case_env,
                    remaining_eliminators,
                );
            }
            EliminatorFrame::Ordinary(eliminator) => {
                let (case_index, case) = match select_ordinary_case(eliminator, &constructor) {
                    Ok(selected) => selected,
                    Err(trap) => {
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            None,
                        )?;
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                    }
                };
                self.disposition_statically_unselected_match_cases(
                    eliminator.static_origin,
                    Some(case_index),
                )?;
                if case.binders != args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            args.len()
                        ),
                    ));
                }
                let mut case_env = env_with(args, &[]);
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Ordinary(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
                };
                case_env.extend(frame_env);
                (
                    self.case_body_occurrence(eliminator.static_origin, case_index, &case.body)?,
                    case_env,
                )
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations are consumed before value composition")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns are consumed before value composition")
            }
            EliminatorFrame::Active(active) => {
                return self.resume_active_continuation(
                    builder,
                    LoweringOperand::Specialized(retained_scrutinee),
                    active,
                );
            }
        };
        if remaining_eliminators.is_empty() {
            self.lower_expr(builder, body, &case_env)
        } else {
            self.lower_computational_producer_expr(builder, body, &case_env, remaining_eliminators)
        }
    }

    fn lower_bounded_nat_computational(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let eliminator = eliminators[0];
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(if structural {
                LoweringOperand::Specialized(Lowered::StructuralNat(StructuralNatV1 {
                    value: nat.value,
                }))
            } else {
                LoweringOperand::Specialized(Lowered::BoundedNat(nat))
            });
        }
        if let EliminatorFrame::Active(active) = eliminator {
            let value = if structural {
                Lowered::StructuralNat(StructuralNatV1 { value: nat.value })
            } else {
                Lowered::BoundedNat(nat)
            };
            return self.resume_active_continuation(
                builder,
                LoweringOperand::Specialized(value),
                active,
            );
        }
        let remaining = &eliminators[1..];
        let (zero_body, suc_body, computational) = match eliminator {
            EliminatorFrame::Computational(frame) => {
                let zero = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_zero
                        && case.argument_binders == 0
                        && case.recursive_positions.is_empty()
                });
                let suc = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_suc
                        && case.argument_binders == 1
                        && case.recursive_positions.as_slice() == [0]
                });
                let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "computational Nat requires Zero and one recursive Suc predecessor",
                    ));
                };
                (
                    self.case_body_occurrence(frame.static_origin, zero_index, &zero.body)?,
                    self.case_body_occurrence(frame.static_origin, suc_index, &suc.body)?,
                    true,
                )
            }
            EliminatorFrame::Ordinary(frame) => {
                let zero = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_zero && case.binders == 0
                });
                let suc = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_suc && case.binders == 1
                });
                let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "ordinary Nat frame requires exact Zero and Suc predecessor arms",
                    ));
                };
                (
                    self.case_body_occurrence(frame.static_origin, zero_index, &zero.body)?,
                    self.case_body_occurrence(frame.static_origin, suc_index, &suc.body)?,
                    false,
                )
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations are consumed before Nat composition")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns are consumed before Nat composition")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors do not consume Nat values")
            }
        };
        let join_origin = match eliminator {
            EliminatorFrame::Computational(frame) => frame.static_origin,
            EliminatorFrame::Ordinary(frame) => frame.static_origin,
            EliminatorFrame::PendingLet(_)
            | EliminatorFrame::InvocationReturn
            | EliminatorFrame::Active(_) => {
                unreachable!("non-join eliminators returned before bounded-Nat emission")
            }
        };
        let join_plan = self.consumed_join_plan_token(join_origin)?;

        let zero_value = builder.ins().iconst(types::I64, 0);
        let zero_nat = if structural {
            Lowered::StructuralNat(StructuralNatV1 { value: zero_value })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(zero_value))
        };
        let zero_frame_env =
            match self.materialize_eliminator_frame_env(builder, eliminator, &zero_nat)? {
                Ok(env) => env,
                Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
            };
        let zero_lowered = if remaining.is_empty() {
            self.lower_expr(builder, zero_body, &zero_frame_env)?
        } else {
            self.lower_computational_producer_expr(builder, zero_body, &zero_frame_env, remaining)?
        };
        let (initial, result_kind) =
            self.merge_scalar_branch(builder, &join_plan, zero_lowered, "BoundedNat")?;

        let loop_block = builder.create_block();
        let step_block = builder.create_block();
        let done_block = builder.create_block();
        #[cfg(test)]
        let break_decrement =
            self.bounded_nat_mutation == BoundedNatLoweringMutation::BrokenDecrement;
        #[cfg(not(test))]
        let break_decrement = false;
        #[cfg(test)]
        let expose_raw_predecessor =
            self.bounded_nat_mutation == BoundedNatLoweringMutation::RawScalarPredecessor;
        #[cfg(not(test))]
        let expose_raw_predecessor = false;
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        if break_decrement {
            builder.append_block_param(loop_block, types::I64);
        }
        builder.append_block_param(done_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        if break_decrement {
            builder.ins().jump(
                loop_block,
                &[
                    zero_value.into(),
                    initial.tag.into(),
                    initial.payload.into(),
                    zero_value.into(),
                ],
            );
        } else {
            builder.ins().jump(
                loop_block,
                &[
                    zero_value.into(),
                    initial.tag.into(),
                    initial.payload.into(),
                ],
            );
        }

        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction = NativeScalarPairV1 {
            tag: builder.block_params(loop_block)[1],
            payload: builder.block_params(loop_block)[2],
        };
        if break_decrement {
            let fuel = builder.block_params(loop_block)[3];
            let compare_block = builder.create_block();
            let exhausted = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
                fuel,
                nat.value,
            );
            let nontermination = builder.ins().iconst(types::I64, -2);
            builder.ins().brif(
                exhausted,
                done_block,
                &[zero_value.into(), nontermination.into()],
                compare_block,
                &[],
            );
            builder.switch_to_block(compare_block);
        }
        let complete = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            predecessor_value,
            nat.value,
        );
        builder.ins().brif(
            complete,
            done_block,
            &[induction.tag.into(), induction.payload.into()],
            step_block,
            &[],
        );

        builder.switch_to_block(step_block);
        let successor_value = if break_decrement {
            predecessor_value
        } else {
            builder.ins().iadd_imm(predecessor_value, 1)
        };
        let observed_predecessor = if expose_raw_predecessor {
            nat.value
        } else {
            predecessor_value
        };
        let predecessor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: observed_predecessor,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(observed_predecessor))
        };
        let retained = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: successor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(successor_value))
        };
        let frame_env =
            match self.materialize_eliminator_frame_env(builder, eliminator, &retained)? {
                Ok(env) => env,
                Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
            };
        let induction = self.lowered_from_scalar_pair(result_kind, induction);
        let mut suc_env = Vec::new();
        if computational {
            suc_env.push(LoweringEnvironmentBinding::Value(
                LoweringOperand::Specialized(induction),
            ));
        }
        suc_env.push(LoweringEnvironmentBinding::Value(
            LoweringOperand::Specialized(predecessor),
        ));
        suc_env.extend(frame_env);
        let suc_lowered = if remaining.is_empty() {
            self.lower_expr(builder, suc_body, &suc_env)?
        } else {
            self.lower_computational_producer_expr(builder, suc_body, &suc_env, remaining)?
        };
        let (next, next_kind) =
            self.merge_scalar_branch(builder, &join_plan, suc_lowered, "BoundedNat")?;
        if next_kind != result_kind {
            return Err(unsupported(
                "BoundedNat",
                "recursive Suc result disagrees with the Zero result kind",
            ));
        }
        if break_decrement {
            let fuel = builder.block_params(loop_block)[3];
            let next_fuel = builder.ins().iadd_imm(fuel, 1);
            builder.ins().jump(
                loop_block,
                &[
                    successor_value.into(),
                    next.tag.into(),
                    next.payload.into(),
                    next_fuel.into(),
                ],
            );
        } else {
            builder.ins().jump(
                loop_block,
                &[successor_value.into(), next.tag.into(), next.payload.into()],
            );
        }

        builder.switch_to_block(done_block);
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        )))
    }

    fn materialize_eliminator_frame_env(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        eliminator: EliminatorFrame<'_>,
        retained_scrutinee: &Lowered,
    ) -> Result<Result<Vec<LoweringEnvironmentBinding>, RuntimeTrap>, CraneliftBackendError> {
        let (env, retained_index, deferred, construct) = match eliminator {
            EliminatorFrame::Computational(frame) => (
                frame.env,
                frame.retained_scrutinee_index,
                frame.deferred_constructor_case,
                "ComputationalMatch",
            ),
            EliminatorFrame::Ordinary(frame) => (
                frame.env,
                frame.retained_scrutinee_index,
                frame.deferred_constructor_case,
                "Match",
            ),
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations do not materialize environments")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns do not materialize environments")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors do not materialize environments")
            }
        };
        let Some(deferred) = deferred else {
            let mut env = env.to_vec();
            if let Some(index) = retained_index {
                if index > env.len() {
                    return Err(unsupported(
                        construct,
                        "retained scrutinee index exceeds the frame environment",
                    ));
                }
                env.insert(
                    index,
                    LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(
                        retained_scrutinee.clone(),
                    )),
                );
            }
            return Ok(Ok(env));
        };
        if deferred.lowered_prefix.len() != deferred.selected_field {
            return Err(unsupported(
                "Construct",
                "selected constructor field prefix does not match its binder index",
            ));
        }

        let mut constructor_args = deferred.lowered_prefix.to_vec();
        constructor_args.push(retained_scrutinee.clone());
        // The trailing fields are the constructor's own children, continuing
        // past the selected one: `child(construct_origin, selected_field + 1 + j)`.
        for (offset, field) in deferred.trailing_fields.iter().enumerate() {
            let field = self.child_occurrence(
                deferred.construct_origin,
                deferred.selected_field + 1 + offset,
                field,
            )?;
            let lowered = self.lower_expr(builder, field, deferred.producer_env)?;
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = lowered {
                return Ok(Err(trap));
            }
            // ⭐ These become `outer_scrutinee`'s constructor **template** below,
            // so this is a specialized-only surface, not a spine edge.
            constructor_args.push(lowered.specialized_at("a deferred constructor field")?);
        }
        let outer_scrutinee = Lowered::Constructor {
            constructor: deferred.constructor.to_string(),
            synthesized_identity: Some(
                self.static_transition_plan
                    .constructor_symbol_identity(deferred.construct_origin)?,
            ),
            // `D7` -- the allocation lane is the second fact resolved
            // at the producer and carried with the template.
            occurrence: Some(self.static_transition_plan.source_aggregate_occurrence(
                deferred.construct_origin,
                PlannedAggregateShape::Constructor,
            )?),
            args: constructor_args.clone(),
        };
        let outer_tail = match self.materialize_eliminator_frame_env(
            builder,
            deferred.outer_eliminator,
            &outer_scrutinee,
        )? {
            Ok(env) => env,
            Err(trap) => return Ok(Err(trap)),
        };

        match deferred.outer_eliminator {
            EliminatorFrame::Computational(frame) => {
                let Some((alternative, case)) = frame
                    .cases
                    .iter()
                    .enumerate()
                    .find(|(_, case)| case.constructor == deferred.constructor)
                else {
                    return Ok(Err(frame.default.clone()));
                };
                if case.argument_binders != constructor_args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} expects {} constructor arguments but value has {}",
                            case.constructor,
                            case.argument_binders,
                            constructor_args.len()
                        ),
                    ));
                }
                let mut seen = BTreeSet::new();
                for position in case.recursive_positions.iter().copied() {
                    if !seen.insert(position) || position >= constructor_args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "case {} has malformed recursive position {position}",
                                case.constructor
                            ),
                        ));
                    }
                }
                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                let ih_slots =
                    self.computational_ih_slots_for_case(case, frame.checked_frame_id)?;
                let producer_origin = self.mint_recursor_producer_origin();
                #[cfg(test)]
                px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::DeferredConstructor,
                    origin: producer_origin,
                    cursor: deferred.selected_active.cursor,
                    siblings: case.recursive_positions.len(),
                    parent_scope: deferred
                        .selected_active
                        .selected_scope
                        .map(|scope| scope.scope_origin),
                });
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    let induction_hypothesis = self.make_computational_recursor(
                        LoweringOperand::Specialized(constructor_args[position].clone()),
                        frame.cases.to_vec(),
                        frame.default.clone(),
                        outer_tail.clone(),
                        frame.static_origin,
                        frame.provenance,
                        frame.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        deferred.selected_active.activation,
                        deferred.selected_active.cursor,
                        deferred.splice_caller,
                        None,
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(
                        Px8jProducerPath::DeferredConstructor,
                        &induction_hypothesis,
                    );
                    induction_hypotheses.push(LoweringEnvironmentBinding::Value(induction_hypothesis));
                }
                // ⭐ `D8d` — THE ONE ENVIRONMENT AUTHORITY at the selected
                // recursive source-order position.
                //
                // Every nonrecursive position stays a `Value`, the IH prefix
                // above and the outer frame below are untouched, and a recursive
                // position the planner has no target for keeps its existing
                // `Value` binding -- "no target" is the ordinary
                // non-specialized path, exactly as a missing continuation call
                // binding is, and never a licence to invent one.
                //
                // ⛔ A `StaticWorker`, deliberately NOT a specialized
                // `Value(Closure)`. The capsule has no value representation, so
                // reading it in value position fails closed at `value_at` --
                // which is the property `D8e`'s consumer will be the sole lawful
                // way around. Until then this binding is intentionally
                // unreadable, and nothing here manufactures a consumer for it.
                for (position, lowered) in constructor_args.into_iter().enumerate() {
                    let binding = match self.composed_recursive_argument_binding(
                        case,
                        deferred.construct_origin,
                        frame.static_origin,
                        alternative,
                        position,
                        &lowered,
                    )? {
                        Some(worker) => LoweringEnvironmentBinding::StaticWorker(worker),
                        None => LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(
                            lowered,
                        )),
                    };
                    induction_hypotheses.push(binding);
                }
                induction_hypotheses.extend(outer_tail);
                Ok(Ok(induction_hypotheses))
            }
            EliminatorFrame::Ordinary(frame) => {
                let (_case_index, case) = match select_ordinary_case(frame, deferred.constructor) {
                    Ok(selected) => selected,
                    Err(trap) => return Ok(Err(trap)),
                };
                if case.binders != constructor_args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            constructor_args.len()
                        ),
                    ));
                }
                constructor_args.extend(specialized_bindings_at(
                    &outer_tail,
                    "a deferred constructor's trailing field",
                )?);
                Ok(Ok(bound_values(
                    constructor_args.into_iter().map(LoweringOperand::Specialized),
        )))
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations cannot be deferred constructor frames")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns cannot be deferred constructor frames")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors cannot be deferred constructor frames")
            }
        }
    }

    fn lower_source_machine(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        env: &[LoweringEnvironmentBinding],
        active: &ActiveContinuationFrame<'_>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let mut root_authority = self.root_terminal_authority.take();
        if let Some(authority) = &mut root_authority {
            match authority.outer_cursor {
                None => authority.outer_cursor = Some(active.cursor),
                Some(cursor) if cursor == active.cursor => {}
                Some(_) => {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked root answer authority was transplanted to another outer cursor",
                    ));
                }
            }
        }
        let control = SourceControl {
            continuation: SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected: active.cursor,
                active,
                root_authority,
            }),
            selected: SourceSelectedContinuation {
                activation: active.activation,
                cursor: active.cursor,
                parent: active.parent,
                pending: active.pending.to_vec(),
                selected_ancestry: active.selected_ancestry.to_vec(),
                selected_scope: active.selected_scope.cloned(),
            },
            selected_lineage: Vec::new(),
            terminal_outer: active.cursor,
        };
        self.lower_source_machine_with_continuation(
            builder,
            OwnedSourceOccurrence::cloned(occurrence),
            env.to_vec(),
            control,
        )
    }

    fn lower_source_machine_with_continuation<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
        control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let previous_source_root = self.source_control_root.replace(control.terminal_outer);
        self.live_source_continuations = self
            .live_source_continuations
            .checked_add(1)
            .expect("compiler-private live source-continuation depth exhausted");
        let result = self.lower_source_machine_with_continuation_inner(builder, expr, env, control);
        self.live_source_continuations = self
            .live_source_continuations
            .checked_sub(1)
            .expect("source-continuation depth must balance");
        self.source_control_root = previous_source_root;
        result
    }

    fn lower_source_machine_with_continuation_inner<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
        control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let mut state = SourceMachineState::Eval { expr, env, control };
        loop {
            state = match state {
                SourceMachineState::Eval {
                    expr:
                        OwnedSourceOccurrence {
                            expr,
                            static_origin,
                        },
                    env,
                    mut control,
                } => match {
                    // The owned source machine is the third traversal route for
                    // these joins. Record the source occurrence here; later
                    // continuation helpers may only reborrow its token.
                    self.enter_source_occurrence_plan(static_origin)?;
                    expr
                } {
                    RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                        self.enter_checked_subcontinuation_frame(frame_id)?;
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *body)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedRecursiveInvocation {
                        call_template_id,
                        body,
                        ..
                    } => {
                        let instance =
                            self.enter_checked_recursive_invocation(call_template_id, &body)?;
                        control.continuation =
                            SourceContinuation::CheckedRecursiveInvocationReturn {
                                instance,
                                next: Box::new(control.continuation),
                            };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *body)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *body)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHInvocation {
                        call_template_id,
                        body,
                        ..
                    } => {
                        // `D8f` — the machine's own child derivation, taken
                        // BEFORE the marker is entered so the same occurrence is
                        // recorded and evaluated.
                        let body = self.owned_child_occurrence(static_origin, 0, *body)?;
                        self.enter_checked_computational_ih_invocation(
                            call_template_id,
                            &body.expr,
                            body.static_origin,
                        )?;
                        control.continuation =
                            SourceContinuation::CheckedComputationalIHInvocationReturn {
                                call_template_id,
                                next: Box::new(control.continuation),
                            };
                        SourceMachineState::Eval {
                            expr: body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Value(value) => SourceMachineState::Value {
                        value: RoutedAnswer::direct(LoweringOperand::Specialized(self.lower_value(builder, &value)?)),
                        control,
                    },
                    // Same value-producing rule as the direct descent's `Var`:
                    // only `Value` yields a machine value, and a static worker
                    // binding fails closed here rather than entering one.
                    RuntimeExpr::Var(index) => SourceMachineState::Value {
                        value: RoutedAnswer::direct(env
                            .get(index as usize)
                            .ok_or_else(|| {
                                unsupported(
                                    "Var",
                                    format!("no runtime binding for index {index}"),
                                )
                            })?
                            .value_at("a source-machine Var in value position")?
                            .clone()),
                        control,
                    },
                    RuntimeExpr::Let { value, body } => {
                        control.continuation = SourceContinuation::LetBody {
                            body: self.owned_child_occurrence(static_origin, 1, *body)?,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *value)?,
                            env: env.clone(),
                            control,
                        }
                    }
                    RuntimeExpr::Construct {
                        constructor,
                        mut args,
                    } => {
                        if args.is_empty() {
                            SourceMachineState::Value {
                                value: RoutedAnswer::direct(LoweringOperand::Specialized(
                                    self.finish_source_constructor(
                                        builder,
                                        constructor,
                                        static_origin,
                                        vec![],
                                    )?,
                                )),
                                control,
                            }
                        } else {
                            // Argument *i* is child *i*; the suffix keeps each
                            // pending term paired with its own origin, so the
                            // machine's positions cannot drift as it consumes them.
                            let first = args.remove(0);
                            let remaining = args
                                .into_iter()
                                .enumerate()
                                .map(|(offset, arg)| {
                                    self.owned_child_occurrence(static_origin, 1 + offset, arg)
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            control.continuation = SourceContinuation::ConstructArgument {
                                constructor,
                                static_origin,
                                remaining,
                                lowered: Vec::new(),
                                env: env.clone(),
                                next: Box::new(control.continuation),
                            };
                            SourceMachineState::Eval {
                                expr: self.owned_child_occurrence(static_origin, 0, first)?,
                                env,
                                control,
                            }
                        }
                    }
                    RuntimeExpr::Match {
                        scrutinee,
                        cases,
                        default,
                    } => {
                        control.continuation = SourceContinuation::MatchScrutinee {
                            cases,
                            default,
                            env: env.clone(),
                            static_origin,
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *scrutinee)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Call { callee, args } => {
                        let args = args
                            .into_iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                self.owned_child_occurrence(static_origin, 1 + position, arg)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        // **`RT-CONTSRC-PRODUCER-LOCAL` `D8e` — THE SOLE
                        // SOURCE-MACHINE CONSUMER of a `D8d` binding.**
                        //
                        // ⛔ It sits **ahead of the callee's own evaluation**,
                        // and that placement is the mechanism, not a
                        // convenience. Evaluating a `Var` callee first routes it
                        // through the machine's value arm, which calls
                        // `value_at` and fails closed on a static worker by
                        // design. So the binding is either consumed here or
                        // refused everywhere — there is no third outcome, and a
                        // `Var` resolving to `Value` falls through to the
                        // pre-existing route untouched.
                        //
                        // ⭐ Deliberately the same shape as the direct descent's
                        // sole consumer: an exact `Var`, read out of the
                        // environment by index, no shape inference and no
                        // planner query. The environment already holds the
                        // answer because `D8d` put it there; asking the planner
                        // again here would be the consumer-side target lookup
                        // this checkpoint excludes, and a second authority for
                        // one binding.
                        let static_worker = match callee.as_ref() {
                            RuntimeExpr::Var(index) => match env.get(*index as usize) {
                                Some(LoweringEnvironmentBinding::StaticWorker(worker)) => {
                                    Some((u64::from(*index), worker.clone()))
                                }
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some((binder_index, worker)) = static_worker {
                            #[cfg(test)]
                            d8e_record_consumption();
                            // `D8l2` — which facet this consumption carried,
                            // recorded beside the count so "three consumptions"
                            // can be attributed rather than merely counted.
                            #[cfg(test)]
                            crate::cranelift_backend::lowering::record_d8l2_consumed_facet(
                                matches!(
                                    worker.discharge,
                                    ContinuationDischarge::ComposedSourceContinuation(_)
                                ),
                            );
                            // Arguments are evaluated under the machine's own
                            // control and phase, exactly as for a value callee;
                            // only the completion differs.
                            let mut remaining = args;
                            if remaining.is_empty() {
                                // **`D8p` — THE CHECKED-APPLICATION SEAM, on the
                                // source machine's call edge.**
                                //
                                // The direct descent has consulted it since
                                // `D5a`; this edge did not, so a checked-IH
                                // marker entered in a body whose application the
                                // SOURCE MACHINE lowers could never be consumed
                                // and failed closed at the marker's close. The
                                // seam is the same function, on the same exact
                                // occurrence and binder ordinal -- no second
                                // authority, no target lookup, and no new
                                // identity.
                                //
                                // ⛔ Immediately BEFORE the call is written, so
                                // consumption still precedes the instruction it
                                // discharges, and after the arguments are in
                                // hand so an ordinary selected-argument call
                                // reaches the seat first and leaves the marker
                                // for the occurrence that owns it.
                                let disposition = self
                                    .consume_checked_ih_marker_at_static_worker_call(
                                        binder_index,
                                        0,
                                        static_origin,
                                    )?;
                                let before = self.live_source_continuations;
                                let (called, emission) = self.call_static_worker_with_inputs(
                                    builder,
                                    &worker,
                                    Vec::new(),
                                    static_origin,
                                )?;
                                // `D8p` — the TARGET side, under the same key,
                                // written only now that the call instruction
                                // exists and carrying the run it actually took.
                                #[cfg(test)]
                                if disposition == CheckedApplicationDisposition::ConsumedHere {
                                    crate::cranelift_backend::lowering::record_d8p_emitted_target(
                                        crate::cranelift_backend::lowering::D8pEmittedTarget {
                                            function: self.defining_function_id,
                                            application_origin: static_origin,
                                            target_body_origin: worker.body_origin,
                                            declared_arity: worker.declared_arity,
                                            captures: worker.captures.len(),
                                            supplied_operands: emission.supplied_operands,
                                        },
                                    );
                                }
                                // `D8f` — the disposition, recorded AFTER the
                                // call instruction exists. A record here is
                                // therefore "this exact call was emitted, with
                                // this disposition", which is what an omission
                                // control needs: emitted, and not consumed.
                                #[cfg(test)]
                                crate::cranelift_backend::lowering::record_d8f_disposition(
                                    self.defining_function_id,
                                    static_origin,
                                    disposition,
                                );
                                // `D8j` — the call is emitted and its result is
                                // in hand under the SAME `control` this arm was
                                // entered with. Only now may a composed
                                // obligation be claimed.
                                // **`D8f` — THE CLAIM DISPOSITION, three cases,
                                // matched exhaustively.**
                                match disposition {
                                    // The `D8j` population: an ordinary composed
                                    // call, untouched by this seam, claims its
                                    // causal identity exactly as before.
                                    CheckedApplicationDisposition::NoPendingApplication
                                    // The checked application itself, claiming
                                    // the planner-issued identity once.
                                    | CheckedApplicationDisposition::ConsumedHere => {
                                        self.claim_composed_discharge(
                                            &worker, emission, &called, before,
                                        )?;
                                    }
                                    // ⛔ The declined call. It is emitted
                                    // unchanged and it claims NOTHING: the
                                    // identity belongs to the checked
                                    // application the planner issued it for, and
                                    // an ordinary selected-argument call
                                    // answering for it is a second discharge of
                                    // one obligation. The binding is not
                                    // reclassified and no identity is minted --
                                    // this call simply does not answer.
                                    CheckedApplicationDisposition::PendingAtAnotherOccurrence => {
                                        #[cfg(test)]
                                        if d8f_declined_call_claims() {
                                            self.claim_composed_discharge(
                                                &worker, emission, &called, before,
                                            )?;
                                        }
                                    }
                                }
                                SourceMachineState::Value {
                                    value: RoutedAnswer::direct(called),
                                    control,
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    callee: SourceCallee::StaticWorker {
                                        worker,
                                        static_origin,
                                        binder_index,
                                    },
                                    remaining,
                                    lowered: Vec::new(),
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        } else {
                            control.continuation = SourceContinuation::CallCallee {
                                args,
                                env: env.clone(),
                                next: Box::new(control.continuation),
                            };
                            SourceMachineState::Eval {
                                expr: self.owned_child_occurrence(static_origin, 0, *callee)?,
                                env,
                                control,
                            }
                        }
                    }
                    RuntimeExpr::ComputationalMatch {
                        scrutinee,
                        cases,
                        default,
                    } => {
                        let checked_frame_id =
                            self.consume_checked_subcontinuation_frame(&cases, &default)?;
                        control.continuation = SourceContinuation::ComputationalMatchScrutinee {
                            cases,
                            default,
                            env: env.clone(),
                            static_origin,
                            provenance: self.mint_recursor_frame_provenance(),
                            checked_frame_id,
                            answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *scrutinee)?,
                            env,
                            control,
                        }
                    }
                    // ⭐ The delegation point. Every form this dispatcher does not
                    // handle — closures included — goes to `lower_expr` here, and
                    // it now goes **as the same occurrence**: same term, same
                    // origin. This arm is why a "machine-only" subset could never
                    // have been threaded soundly.
                    other => SourceMachineState::Value {
                        value: RoutedAnswer::direct(self.lower_expr(
                            builder,
                            SourceOccurrence {
                                expr: &other,
                                static_origin,
                            },
                            &env,
                        )?),
                        control,
                    },
                },
                SourceMachineState::Value { value, mut control } => {
                    // ⭐⭐ `D6a` upstream half -- SPLIT THE PAIR ONCE, HERE.
                    //
                    // ⛔ `incoming_route` is the route THIS predecessor arrived
                    // by. Every transition below that forwards the same operand
                    // carries it forward; only a transition producing a NEW
                    // value starts at `DirectScrutinee`. Resetting on a forward
                    // is the erasure this checkpoint exists to prevent, and it
                    // is SILENT -- the compile stays green and the checked
                    // answer quietly takes the closed default.
                    let RoutedAnswer { value, route: incoming_route } = value;
                    if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                        control.continuation = Self::discard_source_prefix(control.continuation);
                    }
                    match control.continuation {
                        SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                            return Ok(value);
                        }
                        SourceContinuation::Terminal(
                            SourceContinuationTerminal::ReturnToProducerHole {
                                stack,
                                resume_cursor,
                                expected,
                                active,
                                root_authority,
                            },
                        ) => {
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::ReturnHole {
                                cursor: resume_cursor,
                            });
                            if active.cursor != expected {
                                return Err(unsupported(
                                    "ComputationalRecursor",
                                    "producer-hole terminal cursor mismatch",
                                ));
                            }
                            if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                return Ok(value);
                            }
                            source_active_cursor(
                                &control.selected,
                                &control.selected_lineage,
                                resume_cursor,
                            )
                            .ok_or_else(|| {
                                unsupported(
                                    "ComputationalRecursor",
                                    "producer-hole resume cursor is no longer active",
                                )
                            })?;
                            control.continuation = SourceContinuation::UnwindRecursorSegment {
                                stack,
                                resume_cursor,
                                next: Box::new(SourceContinuation::Terminal(
                                    SourceContinuationTerminal::ResumeOuter {
                                        expected,
                                        active,
                                        root_authority,
                                    },
                                )),
                            };
                            SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                            expected,
                            active,
                            root_authority,
                        }) => {
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::ResumeOuter {
                                cursor: expected,
                            });
                            if active.cursor != expected {
                                return Err(unsupported(
                                    "ComputationalRecursor",
                                    "source continuation terminal cursor mismatch",
                                ));
                            }
                            self.restore_root_terminal_authority(root_authority, expected)?;
                            if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                return Ok(value);
                            }
                            return self.resume_active_continuation(builder, value, *active);
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(
                            edge,
                        )) => {
                            if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                let failure = builder.ins().iconst(types::I64, -4);
                                builder.ins().return_(&[failure]);
                                return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                            }
                            let value = if edge.target.terminal_active_prefix.is_empty() {
                                value
                            } else {
                                let mut prefix = edge.target.terminal_active_prefix;
                                prefix.push(EliminatorFrame::InvocationReturn);
                                self.lower_computational_match_value_composed(
                                    builder, RoutedAnswer { value, route: incoming_route }, &prefix,
                                )?
                            };
                            match edge.target.join_plan.representation {
                                JoinResultRepresentation::NativeScalarPair => {
                                    let (value, actual_kind) =
                                        self.merge_planned_scalar_branch(
                                            builder,
                                            edge.target.join_plan.as_ref(),
                                            value,
                                            edge.target.required_kind,
                                            "NativeJoinPlanV1",
                                        )?;
                                    if actual_kind != ScalarMergeKind::RecursiveBackedge
                                        && actual_kind != edge.target.required_kind
                                    {
                                        return Err(unsupported(
                                            "NativeJoinPlanV1",
                                            format!(
                                                "predecessor {} for join {} produced \
                                                 {actual_kind:?}, planned {:?}",
                                                edge.predecessor_identity,
                                                edge.target.join_id,
                                                edge.target.required_kind
                                            ),
                                        ));
                                    }
                                    builder.ins().jump(
                                        edge.target.block,
                                        &[value.tag.into(), value.payload.into()],
                                    );
                                }
                                JoinResultRepresentation::CarrierWord => {
                                    let word = self.carried_join_arm(
                                        builder,
                                        edge.target.result_origin,
                                        value,
                                        Some(edge.target.required_kind),
                                        "NativeJoinPlanV1",
                                    )?;
                                    builder
                                        .ins()
                                        .jump(edge.target.block, &[word.word.into()]);
                                }
                            }
                            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                        }
                        SourceContinuation::LetBody { body, env, next } => {
                            control.continuation = *next;
                            if matches!(value, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                                SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                            } else if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                            } else {
                                let body_env = env_with_operands([value], &env);
                                SourceMachineState::Eval {
                                    expr: body,
                                    env: body_env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                            self.leave_checked_recursive_invocation(instance)?;
                            control.continuation = *next;
                            SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                        }
                        SourceContinuation::CheckedComputationalIHInvocationReturn {
                            call_template_id,
                            next,
                        } => {
                            if self
                                .pending_computational_ih_call
                                .is_some_and(|pending| pending.call_template_id != call_template_id)
                            {
                                return Err(unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "computational IH invocation return crossed another marker",
                                ));
                            }
                            let value = self.finish_checked_computational_ih_marker(value)?;
                            control.continuation = *next;
                            SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                        }
                        SourceContinuation::ReturnFromSelectedCase { delimiter, next } => {
                            let scope =
                                control.selected.selected_scope.as_ref().ok_or_else(|| {
                                    unsupported(
                                        "OrientedSubcontinuationPlanV1",
                                        "selected-case return has no open control obligation",
                                    )
                                })?;
                            if control.selected.activation != delimiter.activation
                                || control.selected.cursor != delimiter.cursor
                                || scope.scope_origin != delimiter.scope_origin
                                || scope.frame.checked_frame_id != delimiter.frame_id
                                || scope.frame.checked_invocation_id != delimiter.invocation_id
                            {
                                return Err(unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "selected-case return delimiter does not match its open occurrence",
                                ));
                            }
                            let previous = control.selected_lineage.pop().ok_or_else(|| {
                                unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "selected-case return has no exact parent control state",
                                )
                            })?;
                            control.selected = previous;
                            control.continuation = *next;
                            SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                        }
                        SourceContinuation::ApplyRecursorSelection { layer, next } => {
                            #[cfg(test)]
                            match layer.role {
                                RecursorLayerRole::SelectsOccurrence { origin } => {
                                    px8j_record_source_event(Px8jSourceTraceEvent::Selection {
                                        origin,
                                    });
                                }
                                RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                } => px8j_record_source_event(Px8jSourceTraceEvent::Exit {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                }),
                            }
                            let answer_route =
                                SourceComputationalAnswerRoute::for_recursor_layer(&layer);
                            control.continuation =
                                SourceContinuation::ComputationalMatchScrutinee {
                                    cases: layer.cases,
                                    default: layer.default,
                                    env: layer.outer_env,
                                    static_origin: layer.static_origin,
                                    provenance: layer.provenance,
                                    checked_frame_id: layer.checked_frame_id,
                                    answer_route,
                                    next,
                                };
                            SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                        }
                        SourceContinuation::UnwindRecursorSegment {
                            mut stack,
                            resume_cursor,
                            next,
                        } => {
                            source_active_cursor(
                                &control.selected,
                                &control.selected_lineage,
                                resume_cursor,
                            )
                            .ok_or_else(|| {
                                unsupported(
                                    "ComputationalRecursor",
                                    "source recursor resume cursor is no longer active",
                                )
                            })?;
                            if let Some(layer) = stack.later_wrappers_in_construction_order.pop() {
                                #[cfg(test)]
                                if let RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                } = layer.role
                                {
                                    px8j_record_source_event(Px8jSourceTraceEvent::Exit {
                                        origin,
                                        scope_origin,
                                        parent_scope,
                                    });
                                }
                                let answer_route =
                                    SourceComputationalAnswerRoute::for_recursor_layer(&layer);
                                control.continuation =
                                    SourceContinuation::ComputationalMatchScrutinee {
                                        cases: layer.cases,
                                        default: layer.default,
                                        env: layer.outer_env,
                                        static_origin: layer.static_origin,
                                        provenance: layer.provenance,
                                        checked_frame_id: layer.checked_frame_id,
                                        answer_route,
                                        next: Box::new(SourceContinuation::UnwindRecursorSegment {
                                            stack,
                                            resume_cursor,
                                            next,
                                        }),
                                    };
                                SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                            } else {
                                control.continuation = *next;
                                SourceMachineState::Value { value: RoutedAnswer { value, route: incoming_route }, control }
                            }
                        }
                        SourceContinuation::ConstructArgument {
                            constructor,
                            static_origin,
                            mut remaining,
                            mut lowered,
                            env,
                            next,
                        } => {
                            // ⭐ A source `Construct` builds a constructor
                            // **template**, so its arguments take the ruled
                            // fail-closed boundary here.
                            lowered.push(
                                value.specialized_at("a source constructor argument")?,
                            );
                            control.continuation = *next;
                            if remaining.is_empty() {
                                SourceMachineState::Value {
                                    value: RoutedAnswer::direct(LoweringOperand::Specialized(self.finish_source_constructor(
                                        builder,
                                        constructor,
                                        static_origin,
                                        lowered,
                                    )?)),
                                    control,
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::ConstructArgument {
                                    constructor,
                                    static_origin,
                                    remaining,
                                    lowered,
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::MatchScrutinee {
                            cases,
                            default,
                            env,
                            static_origin,
                            next,
                        } => {
                            self.enter_source_occurrence_plan(static_origin)?;
                            control.continuation = *next;
                            match value {
                                LoweringOperand::Specialized(Lowered::BoundedNat(nat)) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder,
                                        nat,
                                        false,
                                        &cases,
                                        &default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::StructuralNat(nat)) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder,
                                        BoundedNatV1::derived_from_validated(nat.value),
                                        true,
                                        &cases,
                                        &default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::Bool { value, known }) => {
                                    let true_case = cases.iter().enumerate().find(|(_, case)| {
                                        case.binders == 0
                                            && case.constructor.ends_with("::Bool::True")
                                    });
                                    let false_case = cases.iter().enumerate().find(|(_, case)| {
                                        case.binders == 0
                                            && case.constructor.ends_with("::Bool::False")
                                    });
                                    let (Some(true_case), Some(false_case)) =
                                        (true_case, false_case)
                                    else {
                                        return Err(unsupported(
                                            "Match",
                                            "Bool match requires zero-binder True and False cases",
                                        ));
                                    };
                                    if let Some(selected) = known {
                                        let (index, case) =
                                            if selected { true_case } else { false_case };
                                        self.disposition_statically_unselected_match_cases(
                                            static_origin,
                                            Some(index),
                                        )?;
                                        SourceMachineState::Eval {
                                            expr: self.owned_case_body_occurrence(
                                                static_origin,
                                                index,
                                                case.body.clone(),
                                            )?,
                                            env,
                                            control,
                                        }
                                    } else {
                                        let (true_index, true_case) = true_case;
                                        let (false_index, false_case) = false_case;
                                        let true_body = self.case_body_occurrence(
                                            static_origin,
                                            true_index,
                                            &true_case.body,
                                        )?;
                                        let false_body = self.case_body_occurrence(
                                            static_origin,
                                            false_index,
                                            &false_case.body,
                                        )?;
                                        return self.lower_source_dynamic_bool_match(
                                            builder,
                                            value,
                                            true_body,
                                            false_body,
                                            static_origin,
                                            &env,
                                            control,
                                        );
                                    }
                                }
                                LoweringOperand::Specialized(Lowered::HostResult {
                                    success,
                                    error,
                                    ok,
                                    err_constructor,
                                    ok_constructor,
                                }) => {
                                    return self.lower_source_dynamic_host_result_match(
                                        builder,
                                        success,
                                        *error,
                                        *ok,
                                        &err_constructor,
                                        &ok_constructor,
                                        &cases,
                                        default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::DynamicConstructor(dynamic)) => {
                                    return self.lower_source_dynamic_constructor_match(
                                        builder,
                                        dynamic,
                                        &cases,
                                        &default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::Constructor {
                                    constructor,
                                    args,
                                    ..
                                }) => {
                                    let Some((case_index, case)) = cases
                                        .iter()
                                        .enumerate()
                                        .find(|(_, case)| case.constructor == constructor)
                                    else {
                                        self.disposition_statically_unselected_match_cases(
                                            static_origin,
                                            None,
                                        )?;
                                        return Ok(LoweringOperand::Specialized(Lowered::Trap(default)));
                                    };
                                    self.disposition_statically_unselected_match_cases(
                                        static_origin,
                                        Some(case_index),
                                    )?;
                                    if case.binders != args.len() {
                                        return Err(unsupported(
                                            "Match",
                                            format!(
                                    "case {} expects {} binders but constructor has {} args",
                                    case.constructor,
                                    case.binders,
                                    args.len()
                                ),
                                        ));
                                    }
                                    let mut case_env = env_with(args, &[]);
                                    case_env.extend(env);
                                    SourceMachineState::Eval {
                                        expr: self.owned_case_body_occurrence(
                                            static_origin,
                                            case_index,
                                            case.body.clone(),
                                        )?,
                                        env: case_env,
                                        control,
                                    }
                                }
                                // A runtime boundary word has no compile-time
                                // template, so it is classified by PHASE before
                                // any arm asks for a `Lowered` shape. Without
                                // this arm it fell past every shape test onto
                                // the refusal below -- a true sentence about the
                                // wrong thing, naming a cause that is not the
                                // cause: the value is fine, the question is.
                                //
                                // The generic `lower_expr` `Match` emitter and
                                // the source machine's
                                // `ComputationalMatchScrutinee` both already
                                // carry this arm. This seat was the only one of
                                // the three missing it.
                                //
                                // It does NOT delegate to the generic seat's
                                // `lower_carried_match`: that helper returns a
                                // value to its caller, while this seat owns a
                                // continuation, so each case body is lowered
                                // under the match's original `next` through its
                                // own source predecessor edge.
                                //
                                // Taken as an explicit arm rather than a
                                // fallback, so this match is exhaustive over
                                // `LoweringOperand` and a third variant is a
                                // compile error here rather than a silent
                                // refusal.
                                LoweringOperand::Carried(word) => {
                                    // Family 5 control seam. The operand is
                                    // already classified `Carried` here, so a
                                    // refusal below is evidence about the real
                                    // dispatch decision and nothing else.
                                    #[cfg(test)]
                                    if let Some(refusal) = source_carried_control_refusal(
                                        SourceCarriedControlMutation::RefuseClassifiedCarried,
                                        "Match",
                                        "scrutinee is not a constructor value",
                                    ) {
                                        return Err(refusal);
                                    }
                                    return self.lower_source_carried_match(
                                        builder,
                                        word,
                                        &cases,
                                        &default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(_) => {
                                    return Err(unsupported(
                                        "Match",
                                        "scrutinee is not a constructor value",
                                    ));
                                }
                            }
                        }
                        SourceContinuation::ComputationalMatchScrutinee {
                            cases,
                            default,
                            env,
                            static_origin,
                            provenance,
                            checked_frame_id,
                            answer_route,
                            next,
                        } => 'computational_scrutinee: {
                            self.enter_source_occurrence_plan(static_origin)?;
                            // ⭐⭐ `AC-C4` — THE RESUMPTION POINT. An induction
                            // hypothesis over a carried child hands its word back
                            // as the machine's value, and it lands **here**: this
                            // continuation is what "resumes the same computational
                            // eliminator over that carried word" means on the
                            // source-machine route.
                            //
                            // ⛔ Taken before the specialized selection below,
                            // which reads `Lowered::Constructor` — a compile-time
                            // template the carried value does not have and must
                            // not be asked for. Without this arm the resumed word
                            // reaches `"source scrutinee is not a constructor
                            // value"`, which is a **true sentence about the wrong
                            // thing**: the value is fine, the question is.
                            if let LoweringOperand::Carried(word) = &value {
                                let word = *word;
                                let frame = ComputationalEliminatorFrame {
                                    cases: &cases,
                                    default: &default,
                                    env: &env,
                                    static_origin,
                                    retained_scrutinee_index: None,
                                    deferred_constructor_case: None,
                                    provenance,
                                    checked_frame_id,
                                    checked_invocation_id: checked_frame_id.map(|_| {
                                        self.active_recursive_invocations
                                            .last()
                                            .map_or(0, |instance| instance.invocation_instance_id)
                                    }),
                                    checked_invocation_source: self
                                        .active_recursive_invocations
                                        .last()
                                        .map(|instance| instance.source),
                                    checked_invocation_depth: self
                                        .active_recursive_invocations
                                        .last()
                                        .map_or(0, |instance| instance.semantic_depth),
                                    // ⭐⭐ `D6a` -- THE PREDECESSOR'S ROUTE RAISES THE
                                    // CONTINUATION'S, and never lowers it.
                                    //
                                    // The continuation's own field stays the
                                    // recursor-layer producer's authority. It is not
                                    // the sole authority: an exact claimed and emitted
                                    // continuation-specialization call result arrives
                                    // here already checked, and letting the
                                    // continuation's `DirectScrutinee` overwrite that
                                    // is precisely the drop measured at `ae45e804`.
                                    answer_route: RoutedAnswer {
                                        value: LoweringOperand::Carried(word),
                                        route: incoming_route,
                                    }
                                    .raise(answer_route),
                                };
                                #[cfg(test)]
                                let mut frame = frame;
                                #[cfg(test)]
                                if d6a_route_mutation()
                                    == D6aRouteMutation::OverwriteIncomingWithFrameField
                                {
                                    record_d6a_route_application();
                                    frame.answer_route = answer_route;
                                }
                                #[cfg(test)]
                                record_d6a_route_event(D6aRouteEvent::ConsumerRoute {
                                    seat: D6aConsumerSeat::SourceMachine,
                                    static_origin,
                                    incoming: incoming_route,
                                    frame_field: answer_route,
                                    joined: frame.answer_route,
                                });
                                let eliminated = self
                                    .lower_carried_computational_match(builder, word, frame, &[])?;
                                control.continuation = *next;
                                break 'computational_scrutinee SourceMachineState::Value {
                                    value: RoutedAnswer::direct(eliminated),
                                    control,
                                };
                            }
                            let retained = value.clone();
                            #[cfg(test)]
                            let actual_constructor = match &value {
                                LoweringOperand::Specialized(Lowered::Constructor {
                                    constructor,
                                    ..
                                }) => Some(constructor.clone()),
                                LoweringOperand::Specialized(_) | LoweringOperand::Carried(_) => {
                                    None
                                }
                            };
                            let selected = match &value {
                                LoweringOperand::Specialized(Lowered::Constructor { constructor, .. }) => cases
                                    .iter()
                                    .enumerate()
                                    .find(|(_, case)| case.constructor == *constructor),
                                _ => None,
                            };
                            let (case_index, case) = if let Some(selected) = selected {
                                self.record_source_machine_computational_match_selection(
                                    static_origin,
                                    Some(selected.0),
                                )?;
                                selected
                            } else if answer_route
                                == SourceComputationalAnswerRoute::CheckedSelectedRecursor
                                && matches!(&value, LoweringOperand::Specialized(Lowered::Constructor { .. }))
                                && px8tr_deforested_answer_route_enabled()
                            {
                                let mut returns = cases.iter().enumerate().filter(|(_, case)| {
                                    case.argument_binders == 1
                                        && case.constructor.ends_with("::ITree::Ret")
                                });
                                let return_case = returns.next();
                                let exact_return = returns.next().is_none();
                                let mut visible = cases
                                    .iter()
                                    .filter(|case| case.constructor.ends_with("::ITree::Vis"));
                                let exact_visible = visible.next().is_some()
                                    && visible.next().is_none()
                                    && cases.len() == 2;
                                let Some((return_index, return_case)) =
                                    return_case.filter(|(_, return_case)| {
                                        exact_return
                                            && exact_visible
                                            && source_case_has_no_checked_control_markers(
                                                &return_case.body,
                                            )
                                    })
                                else {
                                    self.record_source_machine_computational_match_selection(
                                        static_origin,
                                        None,
                                    )?;
                                    #[cfg(test)]
                                    px8tr_record_trap_provenance(
                                        Px8trTrapProvenanceEvent::CheckedRecursorDefault {
                                            checked_frame_id: checked_frame_id.expect(
                                                "checked answer routes carry exact frame ids",
                                            ),
                                            actual_constructor,
                                            trap: default.clone(),
                                        },
                                    );
                                    return Ok(LoweringOperand::Specialized(Lowered::Trap(default)));
                                };
                                #[cfg(test)]
                                px8tr_record_trap_provenance(
                                    Px8trTrapProvenanceEvent::DeforestedAnswerResumed {
                                        checked_frame_id: checked_frame_id
                                            .expect("checked answer routes carry exact frame ids"),
                                        actual_constructor,
                                        return_constructor: return_case.constructor.clone(),
                                    },
                                );
                                self.record_source_machine_computational_match_selection(
                                    static_origin,
                                    Some(return_index),
                                )?;
                                let case_env = env_with_operands([retained], &env);
                                control.continuation = *next;
                                let body = self.owned_case_body_occurrence(
                                    static_origin,
                                    return_index,
                                    return_case.body.clone(),
                                )?;
                                return self.lower_source_machine_with_continuation(
                                    builder,
                                    body,
                                    case_env,
                                    control,
                                );
                            } else {
                                if !matches!(&value, LoweringOperand::Specialized(Lowered::Constructor { .. })) {
                                    return Err(unsupported(
                                        "ComputationalMatch",
                                        "source scrutinee is not a constructor value",
                                    ));
                                }
                                self.record_source_machine_computational_match_selection(
                                    static_origin,
                                    None,
                                )?;
                                #[cfg(test)]
                                if answer_route
                                    == SourceComputationalAnswerRoute::CheckedSelectedRecursor
                                {
                                    px8tr_record_trap_provenance(
                                        Px8trTrapProvenanceEvent::CheckedRecursorDefault {
                                            checked_frame_id: checked_frame_id.expect(
                                                "checked answer routes carry exact frame ids",
                                            ),
                                            actual_constructor,
                                            trap: default.clone(),
                                        },
                                    );
                                }
                                return Ok(LoweringOperand::Specialized(Lowered::Trap(default)));
                            };
                            let LoweringOperand::Specialized(Lowered::Constructor { args, .. }) = value else {
                                unreachable!("a selected source case has a constructor value")
                            };
                            if case.argument_binders != args.len() {
                                return Err(unsupported(
                                    "ComputationalMatch",
                                    format!(
                                        "case {} expects {} constructor arguments but value has {}",
                                        case.constructor,
                                        case.argument_binders,
                                        args.len()
                                    ),
                                ));
                            }
                            let mut seen = BTreeSet::new();
                            for position in case.recursive_positions.iter().copied() {
                                if !seen.insert(position) || position >= args.len() {
                                    return Err(unsupported(
                                        "ComputationalMatch",
                                        format!(
                                            "case {} has malformed recursive position {position}",
                                            case.constructor
                                        ),
                                    ));
                                }
                            }
                            let frame = ComputationalEliminatorFrame {
                                cases: &cases,
                                default: &default,
                                env: &env,
                                static_origin,
                                retained_scrutinee_index: None,
                                deferred_constructor_case: None,
                                provenance,
                                checked_frame_id,
                                checked_invocation_id: checked_frame_id.map(|_| {
                                    self.active_recursive_invocations
                                        .last()
                                        .map_or(0, |instance| instance.invocation_instance_id)
                                }),
                                checked_invocation_source: self
                                    .active_recursive_invocations
                                    .last()
                                    .map(|instance| instance.source),
                                checked_invocation_depth: self
                                    .active_recursive_invocations
                                    .last()
                                    .map_or(0, |instance| instance.semantic_depth),
                                // `D6a` -- the same source continuation's fact. The
                                // specialized arm above already consumed it for its own
                                // selection; a nested CARRIED resumption of this frame must
                                // see the same route rather than a weaker one.
                                answer_route,
                            };
                            let activation = self.mint_continuation_activation();
                            let cursor = self.mint_continuation_cursor();
                            let mut ancestry = control.selected.selected_ancestry.clone();
                            ancestry.push(provenance);
                            let mut induction_hypotheses =
                                Vec::with_capacity(case.recursive_positions.len());
                            let ih_slots =
                                self.computational_ih_slots_for_case(case, frame.checked_frame_id)?;
                            let producer_origin = self.mint_recursor_producer_origin();
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                                path: Px8jProducerPath::SourceMachine,
                                origin: producer_origin,
                                cursor,
                                siblings: case.recursive_positions.len(),
                                parent_scope: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_origin),
                            });
                            let parent = control.selected.parent;
                            {
                                let qold = control.selected.as_active(&control.selected_lineage);
                                for position in case.recursive_positions.iter().rev().copied() {
                                    let slot_template_id = case
                                        .recursive_positions
                                        .iter()
                                        .position(|candidate| *candidate == position)
                                        .and_then(|index| ih_slots[index]);
                                    let induction_hypothesis = self.make_computational_recursor(
                                        LoweringOperand::Specialized(
                                            args[position].clone(),
                                        ),
                                        cases.clone(),
                                        default.clone(),
                                        env.clone(),
                                        static_origin,
                                        provenance,
                                        frame.checked_frame_id,
                                        slot_template_id,
                                        producer_origin,
                                        position,
                                        RecursorLayerRole::SelectsOccurrence {
                                            origin: producer_origin,
                                        },
                                        activation,
                                        cursor,
                                        Some(&qold),
                                        Some((
                                            &control.selected,
                                            control.selected_lineage.as_slice(),
                                        )),
                                        None,
                                    )?;
                                    #[cfg(test)]
                                    px8j_record_recursor_carrier(
                                        Px8jProducerPath::SourceMachine,
                                        &induction_hypothesis,
                                    );
                                    induction_hypotheses.push(LoweringEnvironmentBinding::Value(induction_hypothesis));
                                }
                            }
                            let frame_env = match self.materialize_eliminator_frame_env(
                                builder,
                                EliminatorFrame::Computational(frame),
                                retained.specialized_ref_at("an eliminator frame's scrutinee")?,
                            )? {
                                Ok(frame_env) => frame_env,
                                Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
                            };
                            let mut case_env = induction_hypotheses;
                            extend_specialized(&mut case_env, args);
                            case_env.extend(frame_env);
                            let previous_selected = control.selected.clone();
                            let pending = std::mem::take(&mut control.selected.pending);
                            let selected_scope = OwnedSelectedScope {
                                scope_origin: producer_origin,
                                parent_scope: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_origin),
                                frame: ComputationalRecursorFramePayload {
                                    cases: cases.clone(),
                                    default: default.clone(),
                                    outer_env: env.clone(),
                                    static_origin,
                                    provenance,
                                    checked_frame_id: frame.checked_frame_id,
                                    checked_invocation_id: frame.checked_invocation_id,
                                    checked_invocation_source: frame.checked_invocation_source,
                                    checked_invocation_depth: frame.checked_invocation_depth,
                                },
                            };
                            #[cfg(test)]
                            let selected_scope =
                                (!PX8J_DELETE_OWNED_SELECTED_SCOPE.get()).then_some(selected_scope);
                            #[cfg(not(test))]
                            let selected_scope = Some(selected_scope);
                            control.continuation = if frame.checked_frame_id.is_some() {
                                let selected_scope_ref =
                                    selected_scope.as_ref().ok_or_else(|| {
                                        unsupported(
                                            "OrientedSubcontinuationPlanV1",
                                            "checked selection has no owned open-control obligation",
                                        )
                                    })?;
                                SourceContinuation::ReturnFromSelectedCase {
                                    delimiter: SelectedCaseReturnDelimiter {
                                        activation,
                                        cursor,
                                        scope_origin: selected_scope_ref.scope_origin,
                                        frame_id: selected_scope_ref.frame.checked_frame_id,
                                        invocation_id: selected_scope_ref
                                            .frame
                                            .checked_invocation_id,
                                    },
                                    next,
                                }
                            } else {
                                *next
                            };
                            control.selected = SourceSelectedContinuation {
                                activation,
                                cursor,
                                parent,
                                pending,
                                selected_ancestry: ancestry,
                                selected_scope,
                            };
                            control.selected_lineage.push(previous_selected);
                            let body = self.owned_case_body_occurrence(
                                static_origin,
                                case_index,
                                case.body.clone(),
                            )?;
                            SourceMachineState::Eval {
                                expr: body,
                                env: case_env,
                                control,
                            }
                        }
                        SourceContinuation::CallCallee {
                            mut args,
                            env,
                            next,
                        } => {
                            control.continuation = *next;
                            if args.is_empty() {
                                match self.source_call_state(
                                    builder,
                                    value,
                                    Vec::new(),
                                    env,
                                    control,
                                )? {
                                    SourceCallOutcome::Continue(state) => state,
                                    SourceCallOutcome::Complete(value) => return Ok(value),
                                }
                            } else {
                                let first = args.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    callee: SourceCallee::Value(value),
                                    remaining: args,
                                    lowered: Vec::new(),
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::CallArgument {
                            callee,
                            mut remaining,
                            mut lowered,
                            env,
                            next,
                        } => {
                            lowered.push(value);
                            control.continuation = *next;
                            if remaining.is_empty() {
                                match callee {
                                    // **`D8e` — the sole consumer's completion.**
                                    //
                                    // ⛔ Straight into the SHARED route-selected
                                    // emitter with the arguments the machine
                                    // just evaluated. Nothing is re-evaluated,
                                    // no target is looked up again, and no
                                    // operand run is assembled here: captures,
                                    // the route's suffix and the route's table
                                    // all live in the one emitter the direct
                                    // descent uses.
                                    SourceCallee::StaticWorker {
                                        worker,
                                        static_origin,
                                        binder_index,
                                    } => {
                                        // `D8p` — the same seam, at the
                                        // non-empty argument run's emission
                                        // seat. The arguments are already
                                        // lowered, so any ordinary call among
                                        // them has already reached this seat and
                                        // already declined the marker.
                                        let disposition = self
                                            .consume_checked_ih_marker_at_static_worker_call(
                                                binder_index,
                                                lowered.len(),
                                                static_origin,
                                            )?;
                                        let before = self.live_source_continuations;
                                        let (called, emission) = self
                                            .call_static_worker_with_inputs(
                                                builder,
                                                &worker,
                                                lowered,
                                                static_origin,
                                            )?;
                                        // `D8p` — the target side, post-instruction.
                                        #[cfg(test)]
                                        if disposition
                                            == CheckedApplicationDisposition::ConsumedHere
                                        {
                                            crate::cranelift_backend::lowering::record_d8p_emitted_target(
                                                crate::cranelift_backend::lowering::D8pEmittedTarget {
                                                    function: self.defining_function_id,
                                                    application_origin: static_origin,
                                                    target_body_origin: worker.body_origin,
                                                    declared_arity: worker.declared_arity,
                                                    captures: worker.captures.len(),
                                                    supplied_operands: emission.supplied_operands,
                                                },
                                            );
                                        }
                                        // `D8f` — post-instruction, as above.
                                        #[cfg(test)]
                                        crate::cranelift_backend::lowering::record_d8f_disposition(
                                            self.defining_function_id,
                                            static_origin,
                                            disposition,
                                        );
                                        // `D8j` — the non-empty argument run
                                        // reached here through `CallArgument`
                                        // under the machine's own control, and
                                        // the raw call is now written. This is
                                        // the seat, and it is after all three.
                                        //
                                        // `D8f` — and the same three dispositions
                                        // decide whether it answers.
                                        match disposition {
                                            CheckedApplicationDisposition::NoPendingApplication
                                            | CheckedApplicationDisposition::ConsumedHere => {
                                                self.claim_composed_discharge(
                                                    &worker, emission, &called, before,
                                                )?;
                                            }
                                            CheckedApplicationDisposition::PendingAtAnotherOccurrence => {
                                                #[cfg(test)]
                                                if d8f_declined_call_claims() {
                                                    self.claim_composed_discharge(
                                                        &worker, emission, &called, before,
                                                    )?;
                                                }
                                            }
                                        }
                                        SourceMachineState::Value {
                                            value: RoutedAnswer::direct(called),
                                            control,
                                        }
                                    }
                                    SourceCallee::Value(callee) => match self
                                        .source_call_state(builder, callee, lowered, env, control)?
                                    {
                                        SourceCallOutcome::Continue(state) => state,
                                        SourceCallOutcome::Complete(value) => return Ok(value),
                                    },
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    callee,
                                    remaining,
                                    lowered,
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::IfScrutinee { .. }
                        | SourceContinuation::ProjectRecord { .. } => {
                            return Err(unsupported(
                                "ComputationalRecursor",
                                "source continuation frame is not implemented",
                            ));
                        }
                    }
                }
            };
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_bounded_nat_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let zero = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_zero && case.binders == 0
        });
        let suc = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_suc && case.binders == 1
        });
        let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) else {
            return Err(unsupported(
                "BoundedNat",
                "structural Nat source match requires exact Zero and Suc predecessor arms",
            ));
        };
        let zero_body = self.case_body_occurrence(static_origin, zero_index, &zero.body)?;
        let suc_body = self.case_body_occurrence(static_origin, suc_index, &suc.body)?;

        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let (source_prefix_template, target) = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => {
                let fanout = SourceBranchFanout {
                    source_prefix_template,
                    inherited_edge,
                };
                (fanout.source_prefix_template, fanout.inherited_edge.target)
            }
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                (
                    source_prefix_template,
                    SourceJoinTarget {
                        join_id,
                        block: merge,
                        expected_outer: suffix_control.terminal_outer,
                        required_kind,
                        join_plan,
                        result_origin: static_origin,
                        terminal_active_prefix: prefix,
                    },
                )
            }
        };

        let zero_block = builder.create_block();
        let suc_block = builder.create_block();
        let predecessor = nat.predecessor(builder);
        let is_zero =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, nat.value, 0);
        builder.ins().brif(is_zero, zero_block, &[], suc_block, &[]);

        let mut frame_scope =
            CheckedFrameBranchScope::capture(&self.consumed_subcontinuation_frames);
        for (arm_name, block, case_body, predecessor) in [
            ("Zero", zero_block, zero_body, None),
            ("Suc", suc_block, suc_body, Some(predecessor)),
        ] {
            builder.switch_to_block(block);
            let arm_env = predecessor
                .map(|predecessor| {
                    vec![if structural {
                        Lowered::StructuralNat(StructuralNatV1 {
                            value: predecessor.value,
                        })
                    } else {
                        Lowered::BoundedNat(predecessor)
                    }]
                })
                .unwrap_or_default();
            let mut arm_env = env_with(arm_env, &[]);
            arm_env.extend_from_slice(env);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &mut frame_scope,
                OwnedSourceOccurrence::cloned(case_body),
                arm_env,
                branch_control,
            )?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                let detail = match &lowered {
                    LoweringOperand::Specialized(Lowered::Trap(trap)) => {
                        format!("Trap({}: {:?})", trap.message, trap.code)
                    }
                    LoweringOperand::Specialized(other) => lowered_value_kind(other).to_string(),
                    // ⛔ No wildcard: a carried operand reaching a join
                    // diagnostic must name itself, not fall into `other`.
                    LoweringOperand::Carried(_) => "BoundaryCarrier".to_string(),
                };
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "bounded-Nat {arm_name} arm produced {detail} instead of sealing its distinct affine predecessor edge"
                    ),
                ));
            }
        }
        self.consumed_subcontinuation_frames = frame_scope.finish();

        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        };
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    /// Lower one mutually-exclusive match arm with the checked-subcontinuation-
    /// frame consumption set rewound by `frame_scope`, then fold the arm's
    /// resulting consumptions into that scope's union.
    ///
    /// A dynamic match lowers its shared post-match continuation once per arm —
    /// each arm inlines its own copy of the source-prefix template. The arms are
    /// mutually exclusive at run time (selected by one `brif`), so a checked
    /// subcontinuation frame occurring in that shared continuation is a *distinct
    /// lawful activation per arm*, not a repeated consumption of one activation.
    /// `consumed_subcontinuation_frames` is a single per-lowering set, so without
    /// this fork the second arm's lawful consume of the same
    /// `(invocation_id, frame_id)` is misreported as "consumed more than once"
    /// (RT-ESCAPE: e.g. an escaped resource used by a host op whose `Result`
    /// match fans out). Rewinding to the pre-match baseline before each arm
    /// preserves the affine check *within* a single control-flow path — a real
    /// double-consume on one path still collides — and is neither a set-clear nor
    /// a key-salt: it is per-branch scoping. Unioning the arms afterward keeps
    /// every frame consumed on any arm marked consumed for the post-join
    /// continuation, so a genuine revisit *across* the join still rejects.
    fn lower_forked_branch<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        frame_scope: &mut CheckedFrameBranchScope,
        expr: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
        control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        self.consumed_subcontinuation_frames = frame_scope.start_successor();
        let lowered = self.lower_source_machine_with_continuation(builder, expr, env, control)?;
        frame_scope.merge_successor(&self.consumed_subcontinuation_frames);
        Ok(lowered)
    }

    fn lower_source_dynamic_bool_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        condition: cranelift_codegen::ir::Value,
        true_body: SourceOccurrence<'_>,
        false_body: SourceOccurrence<'_>,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => inherited_edge.target,
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    join_plan,
                    result_origin: static_origin,
                    terminal_active_prefix: prefix,
                }
            }
        };
        let true_block = builder.create_block();
        let false_block = builder.create_block();
        builder
            .ins()
            .brif(condition, true_block, &[], false_block, &[]);
        let mut frame_scope =
            CheckedFrameBranchScope::capture(&self.consumed_subcontinuation_frames);
        for (predecessor_id, block, body) in
            [(0, true_block, true_body), (1, false_block, false_body)]
        {
            builder.switch_to_block(block);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &mut frame_scope,
                OwnedSourceOccurrence::cloned(body),
                env.to_vec(),
                branch_control,
            )?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "Bool predecessor {predecessor_id} did not seal its distinct affine join edge"
                    ),
                ));
            }
        }
        self.consumed_subcontinuation_frames = frame_scope.finish();
        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        };
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_dynamic_host_result_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        cases: &[crate::RuntimeMatchCase],
        default: RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => inherited_edge.target,
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    join_plan,
                    result_origin: static_origin,
                    terminal_active_prefix: prefix,
                }
            }
        };
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        builder.ins().brif(success, ok_block, &[], err_block, &[]);

        let mut frame_scope =
            CheckedFrameBranchScope::capture(&self.consumed_subcontinuation_frames);
        for (predecessor_id, block, constructor, payload) in [
            (0, ok_block, ok_constructor, ok),
            (1, err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = if let Some((index, case)) = cases
                .iter()
                .enumerate()
                .find(|(_, case)| case.constructor == constructor && case.binders == 1)
            {
                let arm_env = env_with([payload], env);
                let body =
                    self.owned_case_body_occurrence(static_origin, index, case.body.clone())?;
                self.lower_forked_branch(
                    builder,
                    &mut frame_scope,
                    body,
                    arm_env,
                    branch_control,
                )?
            } else {
                // ⚠ THE ONE SYNTHESIZED TERM in the whole lowering: no source
                // occurrence exists for "this alternative has no case", so the
                // machine is handed a fresh `Trap` built from the match's own
                // `default`. `default` is an ATOM of the match occurrence, not a
                // child of it, so the honest origin for this term is the match
                // occurrence's own — and `Trap` is a leaf, so no child is ever
                // derived from it. ⛔ Do not mint an origin here.
                self.lower_forked_branch(
                    builder,
                    &mut frame_scope,
                    OwnedSourceOccurrence {
                        expr: RuntimeExpr::Trap(default.clone()),
                        static_origin,
                    },
                    env.to_vec(),
                    branch_control,
                )?
            };
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "HostResult predecessor {predecessor_id} did not seal its distinct affine join edge"
                    ),
                ));
            }
        }
        self.consumed_subcontinuation_frames = frame_scope.finish();

        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        };
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    /// One emitted-case descriptor, resolved BEFORE any block exists.
    ///
    /// Every planner and schema question the carried source-machine `Match`
    /// route asks is answered here. Once the first block is created the
    /// selector construction consults only these descriptors and the emitted
    /// carrier helpers, so a malformed table cannot be discovered halfway
    /// through an emitted graph.
    fn source_carried_descriptors(
        &self,
        cases: &[crate::RuntimeMatchCase],
        static_origin: StaticOriginId,
    ) -> Result<Vec<SourceCarriedCase>, CraneliftBackendError> {
        let mut descriptors = Vec::with_capacity(cases.len());
        for (index, case) in cases.iter().enumerate() {
            let emitted = self.source_carried_case_is_emitted(static_origin, index)?;
            let identity = self
                .static_transition_plan
                .case_constructor_identity(static_origin, index)?
                .tag_abi_word()?;
            let binders = i64::try_from(case.binders).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "a case binds more constructor arguments than the carrier ABI can count",
                )
            })?;
            descriptors.push(SourceCarriedCase {
                index,
                emitted,
                identity,
                binders,
                borrowed: borrowed_constructor_identity(&self.process_symbols, &case.constructor),
            });
        }
        Ok(descriptors)
    }

    /// The planner's emission verdict for one ordinary `Match` case, asked per
    /// case and never defaulted. An origin with no record is a refusal to
    /// answer, so it fails closed rather than being read as `Reachable`.
    fn source_carried_case_is_emitted(
        &self,
        static_origin: StaticOriginId,
        index: usize,
    ) -> Result<bool, CraneliftBackendError> {
        let status = self
            .static_transition_plan
            .case_emission_status(static_origin, index)?
            .ok_or_else(|| {
                unsupported(
                    "Match",
                    "a carried source match has no planned case-emission verdict",
                )
            })?;
        // Exhaustive over the verdict, with no wildcard: a third status would
        // be a compile error here rather than silently reading as "do not
        // emit".
        Ok(match status {
            CaseEmissionStatus::Reachable => true,
            CaseEmissionStatus::Eliminated => false,
        })
    }

    /// Lower one preallocated semantic leaf of a carried source match.
    ///
    /// The leaf already holds its bindings as block parameters, so the selector
    /// graph that jumped here is closed. The CALLER mints this leaf's one
    /// source predecessor and instantiates the prefix on it, then hands the
    /// resulting control in; this lowers the case body under it. The only
    /// accepted terminal results are a sealed trap or a `RecursiveBackedge`.
    #[allow(clippy::too_many_arguments)]
    fn lower_source_carried_leaf<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        frame_scope: &mut CheckedFrameBranchScope,
        static_origin: StaticOriginId,
        index: usize,
        body: RuntimeExpr,
        bindings: Vec<LoweringOperand>,
        env: &[LoweringEnvironmentBinding],
        branch_control: SourceControl<'b>,
    ) -> Result<(), CraneliftBackendError> {
        let case_body = self.owned_case_body_occurrence(static_origin, index, body)?;
        let lowered = self.lower_forked_branch(
            builder,
            frame_scope,
            case_body,
            env_with_operands(bindings, env),
            branch_control,
        )?;
        if self.seal_source_trap_branch(builder, &lowered)? {
            // A trap terminates this mutually exclusive predecessor.
            return Ok(());
        }
        if !matches!(
            lowered,
            LoweringOperand::Specialized(Lowered::RecursiveBackedge)
        ) {
            return Err(unsupported(
                "NativeJoinPlanV1",
                format!("carried-match leaf {index} did not seal its distinct affine join edge"),
            ));
        }
        Ok(())
    }

    /// Eliminate a CARRIED scrutinee at the source-machine ordinary `Match`
    /// seat, under the source machine's own continuation discipline.
    ///
    /// A runtime boundary word has no compile-time template, so the case cannot
    /// be selected at compile time. The dispatch is the carrier ABI's and
    /// nothing else: the canonical case identity comes from the planner, the tag
    /// and field count are read through the carrier, and each projected child
    /// stays `Carried` into the case environment. There is no decode and no
    /// `Carried -> Lowered` reconstruction anywhere on this path.
    ///
    /// This does NOT delegate to `Lowering::lower_carried_match`. That helper
    /// belongs to the generic `lower_expr` seat, which returns a value to its
    /// caller; this seat owns a continuation, so each case body must be lowered
    /// under the match's original `next` through a distinct source predecessor
    /// edge, and the suffix resumed exactly once at the join.
    ///
    /// Case emission is the planner's verdict, not this emitter's: a case the
    /// scrutinee's closed producer set eliminates is not emitted and its body is
    /// not lowered. The runtime closed default covers it, so an eliminated
    /// constructor arriving anyway still reaches a defined outcome rather than
    /// falling into a neighbouring case.
    fn lower_source_carried_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ── PHASE 0 — PREVALIDATE, BEFORE A SINGLE BLOCK EXISTS ──
        //
        // A malformed table must fail as an `Err`, not as half an emitted
        // graph. Every planner and schema question is answered here, including
        // the carrier arena's pointer type, so nothing below the first
        // `create_block` asks one.
        let descriptors = self.source_carried_descriptors(cases, static_origin)?;
        let pointer_type = builder.func.dfg.value_type(self.carrier_arena()?);
        let emitted: Vec<usize> = descriptors
            .iter()
            .filter(|descriptor| descriptor.emitted)
            .map(|descriptor| descriptor.index)
            .collect();

        // The two Result alternatives are resolved INDEPENDENTLY. A decoded
        // `HostResult` is a fact about the word, not about the case list, so
        // neither the absence of one alternative nor the absence of both may
        // route it to representation mismatch -- it takes the closed default
        // instead. The binder shape is therefore validated only for an
        // alternative that is actually present.
        let ok_case = cases
            .iter()
            .position(|case| case.constructor == self.process_symbols.result_ok);
        let err_case = cases
            .iter()
            .position(|case| case.constructor == self.process_symbols.result_err);
        for alternative in [ok_case, err_case].into_iter().flatten() {
            if cases[alternative].binders != 1 {
                return Err(unsupported(
                    "HostResult",
                    "a present carried Result case must bind exactly one selected payload",
                ));
            }
        }

        // The whole case set must be process-input family: a runtime class test
        // alone would wrongly demand borrowed identities of unrelated source
        // constructors, which is why this is decided from the cases.
        let admits_borrowed = self.process_object
            && descriptors
                .iter()
                .all(|descriptor| descriptor.borrowed.is_some());
        if admits_borrowed {
            for descriptor in &descriptors {
                let (_, arity) = descriptor.borrowed.expect("admitted family");
                if cases[descriptor.index].binders != arity {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "{} borrowed arity mismatch",
                            cases[descriptor.index].constructor
                        ),
                    ));
                }
            }
        }

        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => {
                // Family 2a control seam. The prefix split has already
                // classified this terminal as an inherited join; the hook sits
                // before the target is used and never rewrites it.
                #[cfg(test)]
                if let Some(refusal) = source_carried_control_refusal(
                    SourceCarriedControlMutation::RefuseSplitInheritedJoin,
                    "NativeJoinPlanV1",
                    "MUTATION inherited join acquisition perturbed",
                ) {
                    return Err(refusal);
                }
                inherited_edge.target
            }
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    join_plan,
                    result_origin: static_origin,
                    terminal_active_prefix: prefix,
                }
            }
        };
        let arm_selected = suffix_control.selected.clone();
        let arm_lineage = suffix_control.selected_lineage.clone();
        let arm_outer = suffix_control.terminal_outer;

        // ── PHASE 1 — ALLOCATE EVERY BLOCK ──
        //
        // Leaves take their bindings as BLOCK PARAMETERS. That is what makes
        // the selector graph closable before any body is lowered: a selector
        // never has to wait for a leaf, it just jumps with the values it read.
        let mismatch_block = builder.create_block();
        let default_block = builder.create_block();

        // One carried leaf per emitted case. Constructor and `HostResult` SHARE
        // a case's carried leaf, which is sound exactly because their payload
        // shapes agree there: a Result case binds one child, and the
        // represented route projects one field for it.
        let mut carried_leaves = Vec::new();
        for &index in &emitted {
            let leaf = builder.create_block();
            for _ in 0..cases[index].binders {
                builder.append_block_param(leaf, types::I64);
            }
            carried_leaves.push((index, leaf));
        }
        // Borrowed leaves are DISTINCT, never shared with the carried ones:
        // their environment is `BorrowedNativeValue` pointers into host memory,
        // and relabelling those as represented carrier children is precisely
        // the mislabelling this route must not do.
        let mut borrowed_leaves = Vec::new();
        if admits_borrowed {
            for &index in &emitted {
                let leaf = builder.create_block();
                for _ in 0..descriptors[index].borrowed.expect("admitted family").1 {
                    builder.append_block_param(leaf, pointer_type);
                }
                borrowed_leaves.push((index, leaf));
            }
        }
        let leaf_of = |leaves: &[(usize, cranelift_codegen::ir::Block)], index: usize| {
            leaves
                .iter()
                .find(|(candidate, _)| *candidate == index)
                .map(|(_, block)| *block)
        };

        // ── PHASE 2 — EMIT AND TERMINATE THE ENTIRE SELECTOR GRAPH ──
        //
        // Nothing below lowers a source body. Every path created here ends in a
        // `brif`, a `jump` or a `return_`, so no block is left live when leaf
        // lowering begins.
        let class = self.emit_carrier_class(builder, scrutinee)?;
        let mut class_test = builder
            .current_block()
            .expect("carried source match block");

        {
            // Emitted unconditionally: whether the word IS a `HostResult` is a
            // runtime fact, so the selector exists even when the source cases
            // mention neither alternative. Each alternative resolves to its own
            // leaf, or to the shared default when it is absent or eliminated.
            let host_result = builder.create_block();
            let next_class = builder.create_block();
            if builder.current_block() != Some(class_test) {
                builder.switch_to_block(class_test);
            }
            let is_host_result = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                class,
                BoundaryClass::HostResult as i64,
            );
            builder
                .ins()
                .brif(is_host_result, host_result, &[], next_class, &[]);

            builder.switch_to_block(host_result);
            let success = self.emit_carrier_host_success(builder, scrutinee)?;
            let payload = self.emit_carrier_host_payload(builder, scrutinee)?;
            let alternative_leaf = |alternative: Option<usize>| {
                alternative
                    .and_then(|index| leaf_of(&carried_leaves, index))
                    .unwrap_or(default_block)
            };
            let ok_target = alternative_leaf(ok_case);
            let err_target = alternative_leaf(err_case);
            let ok_args: Vec<_> = if ok_target == default_block {
                Vec::new()
            } else {
                vec![payload.word.into()]
            };
            let err_args: Vec<_> = if err_target == default_block {
                Vec::new()
            } else {
                vec![payload.word.into()]
            };
            builder
                .ins()
                .brif(success, ok_target, &ok_args, err_target, &err_args);
            class_test = next_class;
        }

        if admits_borrowed {
            let borrowed = builder.create_block();
            let next_class = builder.create_block();
            if builder.current_block() != Some(class_test) {
                builder.switch_to_block(class_test);
            }
            let is_borrowed = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                class,
                BoundaryClass::BorrowedOpaque as i64,
            );
            builder
                .ins()
                .brif(is_borrowed, borrowed, &[], next_class, &[]);

            builder.switch_to_block(borrowed);
            let pointer = self.emit_carrier_scalar(builder, scrutinee)?;
            let kind = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 0);
            Self::require_i64(builder, kind, 2);
            let tag = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 8);
            let arity = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 24);
            let fields = builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), pointer, 16);
            let mut borrowed_test = builder
                .current_block()
                .expect("borrowed carried source match block");
            for &index in &emitted {
                let (expected_tag, expected_arity) =
                    descriptors[index].borrowed.expect("admitted family");
                let leaf = leaf_of(&borrowed_leaves, index)
                    .expect("every emitted case has a borrowed leaf when admitted");
                let selected = builder.create_block();
                let next = builder.create_block();
                if builder.current_block() != Some(borrowed_test) {
                    builder.switch_to_block(borrowed_test);
                }
                let matched = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    tag,
                    expected_tag,
                );
                builder.ins().brif(matched, selected, &[], next, &[]);

                builder.switch_to_block(selected);
                Self::require_i64(builder, arity, expected_arity as i64);
                if expected_arity != 0 {
                    Self::require_nonzero(builder, fields);
                }
                let projected: Vec<_> = (0..expected_arity)
                    .map(|position| {
                        builder
                            .ins()
                            .iadd_imm(fields, (position * 32) as i64)
                            .into()
                    })
                    .collect();
                builder.ins().jump(leaf, &projected);
                borrowed_test = next;
            }
            builder.switch_to_block(borrowed_test);
            builder.ins().jump(default_block, &[]);
            class_test = next_class;
        }

        // The represented-constructor chain, and the residual class test. A
        // class this case set never admitted reaches `mismatch_block`.
        let constructor_block = builder.create_block();
        if builder.current_block() != Some(class_test) {
            builder.switch_to_block(class_test);
        }
        let is_constructor = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            class,
            BoundaryClass::Constructor as i64,
        );
        builder
            .ins()
            .brif(is_constructor, constructor_block, &[], mismatch_block, &[]);

        builder.switch_to_block(constructor_block);
        let tag = self.emit_carrier_tag(builder, scrutinee)?;
        let field_count = self.emit_carrier_field_count(builder, scrutinee)?;
        let mut tag_test = builder
            .current_block()
            .expect("represented carried source match block");
        for &index in &emitted {
            let leaf =
                leaf_of(&carried_leaves, index).expect("every emitted case has a carried leaf");
            let selected = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(tag_test) {
                builder.switch_to_block(tag_test);
            }
            let identity = descriptors[index].identity;
            let matched = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                identity as i64,
            );
            builder.ins().brif(matched, selected, &[], next, &[]);

            builder.switch_to_block(selected);
            let binders = descriptors[index].binders;
            // The declared arity is checked against the word actually
            // delivered, at runtime, which is where a disagreement between a
            // case header and a boundary value belongs.
            Self::require_i64(builder, field_count, binders);
            let mut projected = Vec::with_capacity(cases[index].binders);
            for position in 0..cases[index].binders {
                let child = self.emit_carrier_field(builder, scrutinee, position)?;
                projected.push(child.word.into());
            }
            builder.ins().jump(leaf, &projected);
            tag_test = next;
        }
        builder.switch_to_block(tag_test);
        builder.ins().jump(default_block, &[]);

        // The representation this case set never decoded. It is NOT the match's
        // default: nothing was decoded, so no alternative was ruled out. It
        // mints no predecessor, consumes no branch frame, joins nothing, and
        // does not lower the default.
        builder.switch_to_block(mismatch_block);
        let mismatch = builder
            .ins()
            .iconst(types::I64, CARRIED_REPRESENTATION_MISMATCH_STATUS);
        builder.ins().return_(&[mismatch]);

        // ── PHASE 3 — LOWER ONLY THE PREALLOCATED SEMANTIC LEAVES ──
        let mut frame_scope =
            CheckedFrameBranchScope::capture(&self.consumed_subcontinuation_frames);

        for (index, leaf) in carried_leaves {
            builder.switch_to_block(leaf);
            let bindings: Vec<_> = builder
                .block_params(leaf)
                .to_vec()
                .into_iter()
                .map(|word| LoweringOperand::Carried(CarriedBoundaryWord { word }))
                .collect();
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: arm_selected.clone(),
                selected_lineage: arm_lineage.clone(),
                terminal_outer: arm_outer,
            };
            self.lower_source_carried_leaf(
                builder,
                &mut frame_scope,
                static_origin,
                index,
                cases[index].body.clone(),
                bindings,
                env,
                branch_control,
            )?;
        }

        for (index, leaf) in borrowed_leaves {
            builder.switch_to_block(leaf);
            let bindings: Vec<_> = builder
                .block_params(leaf)
                .to_vec()
                .into_iter()
                .map(|pointer| {
                    LoweringOperand::Specialized(Lowered::BorrowedNativeValue { pointer })
                })
                .collect();
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: arm_selected.clone(),
                selected_lineage: arm_lineage.clone(),
                terminal_outer: arm_outer,
            };
            self.lower_source_carried_leaf(
                builder,
                &mut frame_scope,
                static_origin,
                index,
                cases[index].body.clone(),
                bindings,
                env,
                branch_control,
            )?;
        }

        // The shared semantic default, lowered exactly once. `default` is an
        // ATOM of the match occurrence rather than a child of it, so the honest
        // origin for this synthesized term is the match occurrence's own;
        // `Trap` is a leaf, so no child is derived from it. Do not mint an
        // origin here.
        builder.switch_to_block(default_block);
        let edge = self.mint_source_predecessor(target.clone());
        let continuation =
            Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
        let default_control = SourceControl {
            continuation,
            selected: arm_selected.clone(),
            selected_lineage: arm_lineage.clone(),
            terminal_outer: arm_outer,
        };
        let lowered = self.lower_forked_branch(
            builder,
            &mut frame_scope,
            OwnedSourceOccurrence {
                expr: RuntimeExpr::Trap(default.clone()),
                static_origin,
            },
            env.to_vec(),
            default_control,
        )?;
        if !self.seal_source_trap_branch(builder, &lowered)?
            && !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            )
        {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "carried-match default predecessor did not seal its distinct affine join edge",
            ));
        }

        // ── PHASE 4 — UNION ONCE, FINISH ONCE ──
        self.consumed_subcontinuation_frames = frame_scope.finish();
        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        };
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    fn lower_source_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;
        if Self::source_terminal_join(&suffix_control.continuation).is_some() {
            return self.lower_source_nested_dynamic_constructor_match(
                builder,
                dynamic,
                cases,
                default,
                static_origin,
                env,
                suffix_control,
            );
        }
        self.lower_source_planned_dynamic_constructor_match(
            builder,
            dynamic,
            cases,
            default,
            static_origin,
            env,
            suffix_control,
        )
    }

    fn lower_source_nested_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let SourcePrefixTerminal::Join(inherited_edge) = terminal else {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "nested dynamic constructor has no affine terminal edge",
            ));
        };
        let fanout = SourceBranchFanout {
            source_prefix_template,
            inherited_edge,
        };
        let target = fanout.inherited_edge.target;
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor source match block");
        let mut frame_scope =
            CheckedFrameBranchScope::capture(&self.consumed_subcontinuation_frames);
        for alternative in dynamic.alternatives {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let (case_index, case) =
                match select_dynamic_constructor_case(cases, &alternative, default)? {
                    Ok(selected) => selected,
                    Err(_) => {
                        let failure = builder.ins().iconst(types::I64, -4);
                        builder.ins().return_(&[failure]);
                        test_block = next;
                        continue;
                    }
                };
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&fanout.source_prefix_template, edge)?;
            let control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &mut frame_scope,
                self.owned_case_body_occurrence(static_origin, case_index, case.body.clone())?,
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "nested dynamic constructor predecessor did not seal its edge",
                ));
            }
            test_block = next;
        }
        self.consumed_subcontinuation_frames = frame_scope.finish();
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge))
    }

    fn lower_source_planned_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let active = suffix_control
            .selected
            .as_active(&suffix_control.selected_lineage);
        let (prefix, suffix_pending, required_kind, site_id) =
            self.planned_active_scalar_cut(active)?;
        let suffix_pending = suffix_pending.to_vec();
        let join_id = self.next_source_join;
        self.next_source_join = self
            .next_source_join
            .checked_add(1)
            .expect("compiler-private source join identity exhausted");
        let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
        let merge = builder.create_block();
        self.append_planned_join_params(builder, merge, join_plan.as_ref());
        let target = SourceJoinTarget {
            join_id,
            block: merge,
            expected_outer: suffix_control.terminal_outer,
            required_kind,
            join_plan,
            result_origin: static_origin,
            terminal_active_prefix: prefix,
        };
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let root_authority = match terminal {
            SourcePrefixTerminal::ResumeOuter { root_authority } => root_authority,
            SourcePrefixTerminal::Join(_) => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "planned dynamic-constructor cut unexpectedly inherited an executable edge",
                ));
            }
        };
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor source match block");
        let mut frame_scope =
            CheckedFrameBranchScope::capture(&self.consumed_subcontinuation_frames);
        for (predecessor_id, alternative) in dynamic.alternatives.into_iter().enumerate() {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let (case_index, case) =
                match select_dynamic_constructor_case(cases, &alternative, default)? {
                    Ok(selected) => selected,
                    Err(_) => {
                        let failure = builder.ins().iconst(types::I64, -4);
                        builder.ins().return_(&[failure]);
                        test_block = next;
                        continue;
                    }
                };
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &mut frame_scope,
                self.owned_case_body_occurrence(static_origin, case_index, case.body.clone())?,
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "dynamic-constructor predecessor {predecessor_id} for checked site {site_id} did not seal its affine join edge"
                    ),
                ));
            }
            test_block = next;
        }
        self.consumed_subcontinuation_frames = frame_scope.finish();
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    fn source_call_state<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        callee: LoweringOperand,
        args: Vec<LoweringOperand>,
        env: Vec<LoweringEnvironmentBinding>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        // ⭐ A call needs a **callable template** — `params`, `captures`, a body
        // occurrence. A carried boundary word carries none of those and cannot
        // acquire them (`§2g`: the carrier holds the SSA word and nothing else),
        // so this is a specialized-only surface. ⛔ Fails closed.
        let callee = callee.specialized_at("a source-machine call's callee")?;
        match callee {
            Lowered::Closure {
                captures,
                params,
                body,
            } => {
                if params.len() != args.len() {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                // Continued source evaluation: unwrap only, no crossing.
                let mut call_env = bound_values(args);
                extend_captures(&mut call_env, captures);
                call_env.extend(env);
                Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                    expr: self.machine_body_occurrence(body)?,
                    env: call_env,
                    control,
                }))
            }
            Lowered::DeclarationClosure {
                reference,
                symbol,
                captures,
                params,
                body,
            } => {
                // `RT-DECL-CLOSURE-PORT` `D4`, consumer 3 of 3 -- the
                // source-machine route. Its arguments are already operands, so
                // it hands them to the shared ordering directly; the arity
                // check lives there for every consumer rather than once here.
                if self.body_emission_authority == BodyEmissionAuthority::FunctionizedUnits {
                    // ⭐ Crossing into a declared generated unit here, rather
                    // than inside `call_declared_unit_target` later. ⛔ All
                    // inputs cross at ONE common transfer coordinate — there is
                    // no per-argument pairing on this route — and the
                    // coordinate is inert: an aggregate carries its own
                    // producer authority and is preflighted against it, and a
                    // non-aggregate queries no aggregate ownership at all.
                    let args = self.carry_source_call_inputs(builder, body, args)?;
                    let called = self.call_declaration_closure_unit(
                        builder, reference, &symbol, &params, captures, args,
                    )?;
                    return Ok(SourceCallOutcome::Complete(called));
                }
                if params.len() != args.len() {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                let body = self.machine_body_occurrence(body)?;
                self.lower_source_declaration_call(
                    builder,
                    symbol,
                    captures,
                    body,
                    args,
                    env,
                    control,
                )
            }
            mut recursor @ Lowered::ComputationalRecursorClosure { .. } => {
                let checked_ih_invocation =
                    self.mint_checked_computational_ih_instance(&mut recursor)?;
                if let Some(CheckedRecursiveInvocationInstance {
                    source: InvocationTemplateRef::ComputationalIHCall(call_template_id),
                    ..
                }) = checked_ih_invocation
                {
                    let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation has no oriented plan",
                        )
                    })?;
                    let call = plan
                        .computational_ih_call(call_template_id)
                        .ok_or_else(|| {
                            unsupported(
                                "OrientedSubcontinuationPlanV1",
                                "checked IH invocation has no exact call template",
                            )
                        })?;
                    let open = control.selected.selected_scope.as_ref().ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation has no selected/open parent occurrence",
                        )
                    })?;
                    self.validate_source_dynamic_splice_parent(
                        checked_ih_invocation.expect("matched checked IH invocation"),
                        open,
                    )?;
                    if call.parent_frame_template_id != open.frame.checked_frame_id
                        || call.parent_segment_site_id
                            != open.frame.checked_frame_id.and_then(|frame_id| {
                                plan.frame(frame_id).map(|frame| frame.segment_site_id)
                            })
                    {
                        return Err(unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation parent edge does not match the active open occurrence",
                        ));
                    }
                }
                let (base, boundary) =
                    decompose_computational_recursor(LoweringOperand::Specialized(recursor));
                let (activation, invocation) =
                    boundary.expect("recursor closure carries an invocation segment");
                let recursive_unit_body = invocation.recursive_unit_body;
                        // `D5a` checkpoint 4 step 1 — read the retained source
                        // coordinates BEFORE the segment is installed, beside the
                        // existing pre-move field read. Both are facts of the
                        // invocation, so both are taken while it is still in hand
                        // rather than reconstructed afterwards.
                        let carried_coordinates =
                            CarriedInvocationCoordinates::of(&invocation)?;
                if source_active_cursor(
                    &control.selected,
                    &control.selected_lineage,
                    invocation.resume_cursor,
                )
                .is_none()
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "recursive invocation cursor is not live in source control",
                    ));
                }
                let armed = ArmedInvocation {
                    suspended: control,
                    expected_selected: invocation.resume_cursor,
                };
                if source_active_cursor(
                    &armed.suspended.selected,
                    &armed.suspended.selected_lineage,
                    armed.expected_selected,
                )
                .is_none()
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "armed invocation endpoint changed selected cursor",
                    ));
                }
                // ⭐⭐ `AC-C4` — the carried residual on the source-machine
                // route. ⚠ This is the site where "installs the ALREADY-CHECKED
                // invocation segment" is literal: the refusal below runs
                // **before** `install_recursor_invocation`, which is exactly the
                // ordering control 5 measures.
                if let LoweringOperand::Carried(word) = base {
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    if let Some(body) = recursive_unit_body.filter(|_| {
                        matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::FunctionizedUnits
                        )
                    }) {
                        let coordinates = carried_coordinates;
                        let args = self.carry_source_call_inputs(builder, body, args)?;
                        let value = self.call_declared_recursive_position_unit(
                            builder,
                            body,
                            &args,
                            Some(coordinates),
                        )?;
                        return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                            // ⛔ `D6a`: a declared recursive-position UNIT call is
                            // not a lawful producer, and its result crosses a
                            // function boundary besides -- which carries only the
                            // word. It starts direct; a caller with an exact
                            // claimed call identity re-attests its own.
                            value: RoutedAnswer::direct(value),
                            control: suspended,
                        }));
                    }
                    Self::reject_carried_residual_arguments(args.len())?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value: RoutedAnswer::direct(LoweringOperand::Carried(word)),
                        control: suspended,
                    }));
                }
                let base = base.specialized_at("a recursor residual in a source call")?;
                if let Lowered::BoundedNat(predecessor) = base {
                    if !args.is_empty() {
                        return Err(unsupported(
                            "BoundedNat",
                            "structural Nat recursive hypothesis takes no arguments",
                        ));
                    }
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value: RoutedAnswer::direct(LoweringOperand::Specialized(Lowered::BoundedNat(predecessor))),
                        control: suspended,
                    }));
                } else {
                    let Lowered::Closure {
                        captures,
                        params,
                        body,
                    } = base
                    else {
                        return Err(unsupported(
                            "ComputationalMatch",
                            "recursive constructor field is not a closure",
                        ));
                    };
                    if params.len() != args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "recursive field expects {} args but call provides {}",
                                params.len(),
                                args.len()
                            ),
                        ));
                    }
                    // Two roles, as elsewhere on this path: ordered unit-call
                    // inputs, or an environment prefix. Only the second binds.
                    // ⚠ The ARGUMENTS cross here; the CAPTURES do not, and that
                    // is a stated boundary rather than an oversight. A capture
                    // arrives inside an already-lowered `Lowered::Closure`, so
                    // a specialized one still reaches
                    // `call_declared_unit_target`'s fallback — where, since it
                    // carries its own producer certificate, it authorizes
                    // itself.
                    let mut call_inputs = if matches!(
                        self.body_emission_authority,
                        BodyEmissionAuthority::FunctionizedUnits
                    ) {
                        self.carry_source_call_inputs(builder, body, args)?
                    } else {
                        args
                    };
                    call_inputs.extend(captures);
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    if matches!(
                        self.body_emission_authority,
                        BodyEmissionAuthority::FunctionizedUnits
                    ) {
                        let coordinates = carried_coordinates;
                        let value = self.call_declared_recursive_position_unit(
                            builder,
                            body,
                            &call_inputs,
                            Some(coordinates),
                        )?;
                        return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                            // ⛔ `D6a`: a declared recursive-position UNIT call is
                            // not a lawful producer, and its result crosses a
                            // function boundary besides -- which carries only the
                            // word. It starts direct; a caller with an exact
                            // claimed call identity re-attests its own.
                            value: RoutedAnswer::direct(value),
                            control: suspended,
                        }));
                    }
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                        expr: self.machine_body_occurrence(body)?,
                        env: env_with_operands(call_inputs, &env),
                        control: suspended,
                    }));
                }
            }
            _ => Err(unsupported("Call", "callee is not a closure")),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_declaration_call<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: RuntimeSymbol,
        captures: Vec<LoweringOperand>,
        body: OwnedSourceOccurrence,
        args: Vec<LoweringOperand>,
        env: Vec<LoweringEnvironmentBinding>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(&symbol)?;
        if !self.declaration_is_recursive(&symbol) {
            let mut call_env = bound_values(args);
            extend_captures(&mut call_env, captures);
            call_env.extend(env);
            return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                expr: body,
                env: call_env,
                control,
            }));
        }

        // ⭐ Past this point the call is genuinely recursive, and its arguments
        // become the **loop header's representation** — compared across
        // iterations by `same_recursive_argument_shapes` and lowered into block
        // params. A carried boundary word has no such shape, so this is a
        // specialized-only surface with the ruled fail-closed arm.
        //
        // ⚠ The boundary sits HERE and not at the parameter, because the
        // non-recursive direct call above forwards `args` into `call_env`
        // untouched — that path stays phase-preserving and must not be made to
        // fail closed for a property only the loop needs.
        let args = specialized_operands_at(&args, "a recursive source-declaration argument")?;
        if let Some(active) = self
            .active_recursive_declarations
            .iter()
            .rev()
            .find(|active| active.symbol == symbol)
            .cloned()
        {
            if !same_recursive_argument_shapes(&active.argument_templates, &args) {
                return Err(unsupported(
                    "DeclarationRef",
                    format!(
                        "recursive declaration {symbol} changes its native argument representation: {:?} -> {:?}",
                        active
                            .argument_templates
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>(),
                        args.iter().map(lowered_value_kind).collect::<Vec<_>>()
                    ),
                ));
            }
            if let Some(induction) = active.induction {
                return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                    value: RoutedAnswer::direct(LoweringOperand::Specialized(induction)),
                    control,
                }));
            }
            let mut values = Vec::new();
            append_recursive_argument_values(
                builder,
                &args,
                &mut values,
                &self.function_local.native_int_tags,
            )?;
            builder.ins().jump(
                active
                    .header
                    .expect("tail-recursive source declarations own a loop header"),
                &values.into_iter().map(Into::into).collect::<Vec<_>>(),
            );
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(SourceCallOutcome::Complete(LoweringOperand::Specialized(
                Lowered::RecursiveBackedge,
            )));
        }

        let header = builder.create_block();
        let mut initial_values = Vec::new();
        append_recursive_argument_values(
            builder,
            &args,
            &mut initial_values,
            &self.function_local.native_int_tags,
        )?;
        for value in &initial_values {
            builder.append_block_param(header, builder.func.dfg.value_type(*value));
        }
        builder.ins().jump(
            header,
            &initial_values
                .iter()
                .copied()
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        builder.switch_to_block(header);

        let mut parameters = builder.block_params(header).iter().copied();
        let mut loop_args = Vec::with_capacity(args.len());
        for template in &args {
            loop_args.push(rebuild_recursive_argument(
                template,
                &mut parameters,
                &mut self.function_local.native_int_tags,
            )?);
        }
        if parameters.next().is_some() {
            return Err(unsupported(
                "DeclarationRef",
                "recursive source declaration loop parameter shape is not closed",
            ));
        }
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: Some(header),
                argument_templates: args,
                induction: None,
            });
        let mut call_inputs = loop_args
            .into_iter()
            .rev()
            .map(LoweringOperand::Specialized)
            .collect::<Vec<_>>();
        call_inputs.extend(captures);
        let call_env = env_with_operands(call_inputs, &env);
        let lowered = self.lower_source_machine_with_continuation(builder, body, call_env, control);
        self.active_recursive_declarations.pop();
        Ok(SourceCallOutcome::Complete(lowered?))
    }

    /// Resolves a retained closure body's static origin back to its source term.
    ///
    /// ⭐ **This is the only `origin -> expression` lookup in the backend, and the
    /// only place `StaticTransitionPlan::source_occurrence` is called.**
    /// `RT-FNSPLIT-B2A-C` shipped a pin (its N3) asserting that no such lookup
    /// existed, because at that point the origin was provenance and a lookup would
    /// have been an unaudited second authority. B2A-S **retires that pin on
    /// purpose** and replaces it with the opposite one: one resolution route,
    /// observed behaviorally by
    /// `every_origin_to_expression_resolution_goes_through_the_single_route`.
    ///
    /// ⛔ Do not call the plan's resolver anywhere else, and do not widen this to
    /// take anything but an origin. The moment a caller can pass a term, a
    /// pointer, or a hash alongside the tag, the tag has stopped being the
    /// authority and this is decoration again.
    ///
    /// ⚠ Stated precisely, **with its window**, because the two counts differ and
    /// conflating them would overclaim: there is **one** lookup (this function),
    /// and `grep -c 'self.retained_body_occurrence('` in this file returns
    /// **eight** — **seven** retained-closure consumption sites (application,
    /// resume, declaration-call) plus **one** internal composition by
    /// `machine_body_occurrence`, which is this function's own caller rather than
    /// a further consumer.
    ///
    /// Those seven do not share a single lowering *entry point*, because a retained
    /// body is lowered by whichever specialized path the call shape selects. What
    /// makes selection closed is therefore not one caller but that
    /// `Lowered::Closure`/`DeclarationClosure` carry only a tag — so this is the
    /// *only* way any of them can reach a term at all.
    /// **`RT-CONTSPEC-ACTIVATE` `D3` — claim one exact causal token at its
    /// producer occurrence and emit the declared direct continuation call.**
    ///
    /// The selector is the ruled four-field one, and the only facts lowering
    /// supplies are ones it actually has here: the actual `Construct` origin,
    /// the active computational-frame origin, the case index, and this member
    /// of the case's ruled recursive positions. The call-site sequence is
    /// never supplied or derived -- it stays opaque inside the identity the
    /// planner returns.
    ///
    /// No binding is the non-specialized path: the producer keeps its existing
    /// route unchanged. A binding claims exactly once, with the unit currently
    /// being defined supplied as an INDEPENDENT owner check, and calls this
    /// Function's own declared `FuncRef` -- never one borrowed from another
    /// function.
    /// **`RT-CONTSPEC-ACTIVATE` `D3` — claim one exact causal token at its
    /// producer occurrence and emit the declared direct continuation call.**
    ///
    /// The selector is the ruled four-field one, built only from facts this
    /// seat actually holds: the planner-issued producer `Construct`
    /// occurrence, the active computational-frame origin, the selected case
    /// index, and one member of that case's ruled recursive positions. The
    /// call-site sequence is never supplied or derived -- it stays opaque
    /// inside the identity the planner returns.
    ///
    /// No binding is the non-specialized path: the producer keeps its existing
    /// route untouched, and the final ledger equality is what catches a
    /// genuinely lost planned call.
    ///
    /// The call goes through the **existing** unit-call protocol with the full
    /// declared target, so no second ABI is invented here.
    #[allow(clippy::too_many_arguments)]
    fn claim_and_call_continuation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        producer_construct_origin: StaticOriginId,
        continuation_origin: StaticOriginId,
        producer_alternative: usize,
        recursive_position: usize,
        fields: &[LoweringOperand],
        producer_env: &[LoweringEnvironmentBinding],
    ) -> Result<Option<RoutedAnswer>, CraneliftBackendError> {
        let alternative = u32::try_from(producer_alternative).map_err(|_| {
            unsupported("ComputationalMatch", "case index exceeds addressable range")
        })?;
        let position = u32::try_from(recursive_position).map_err(|_| {
            unsupported(
                "ComputationalMatch",
                "recursive position exceeds addressable range",
            )
        })?;
        #[cfg(test)]
        d5a_trace(format!(
            "CLAIM entry defining={:?} construct={producer_construct_origin:?} \
             continuation={continuation_origin:?} alt={alternative} pos={position}",
            self.defining_unit
        ));
        // The claim regime exists only while the unit-definition pass is open.
        // Outside it -- a direct lowering harness, or an authority that never
        // opens the ledger -- there is no ledger to claim against and no
        // close() to satisfy, so the producer keeps its existing route. This
        // is the ledger's own presence, not a guess about the caller.
        if self.continuation_claims.is_none() {
            #[cfg(test)]
            d5a_trace("  CLAIM outcome=NoLedger".to_string());
            return Ok(None);
        }
        let Some(identity) = self.static_transition_plan.continuation_call_binding_for(
            producer_construct_origin,
            continuation_origin,
            alternative,
            position,
        )?
        else {
            #[cfg(test)]
            d5a_trace("  CLAIM outcome=NoBinding".to_string());
            return Ok(None);
        };
        #[cfg(test)]
        d5a_trace(format!("  CLAIM bound identity={identity:?}"));
        self.claim_and_call_resolved_continuation(
            builder,
            &identity,
            fields,
            recursive_position,
            producer_env,
        )
            .map(Some)
    }

    /// **`RT-DECL-CLOSURE-PORT` `D5a` — the claim/call machinery, factored to
    /// run after identity resolution and shared by both consumption seats.**
    ///
    /// ⭐ The split is the whole point of `D5a` contract 4. The retained-frame
    /// seat resolves its identity from the four-field selector because the
    /// active computational frame still holds alternative and position; the
    /// detached-result seat resolves the *same kind* of identity from the
    /// planner's result-edge projection because that frame is gone. ⛔ Neither
    /// seat is privileged and neither fakes the other's operands: what is
    /// common — claim once under the defining owner, resolve this Function's
    /// own declared target, append the projected captures, emit one direct
    /// call, record the `Inst` — lives here exactly once.
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D3b` (re-cut) — resolve the DIRECT
    /// EMISSION claim**, the one consumer that reads `producer_env`.
    ///
    /// ⛔ **It takes only `views.direct_emission`.** There is no arm that falls
    /// back to the capture view, and none that searches for "whichever claim
    /// fits": the two consumers hold different environments, which is exactly
    /// what `D3c` measured, so borrowing the other's index is the defect.
    ///
    /// ⛔ **Root provenance is not consulted.** Both root arms are lawful here;
    /// what must hold is that the claim names *this* seat or *this* frame.
    fn resolve_direct_emission_claim(
        &self,
        requested: &ContinuationSourceSlotAuthority,
        views: ContinuationAvailabilityViews,
        defining_owner: ContinuationEmissionOwner,
        seat: ContinuationDirectEmissionSeat,
    ) -> Result<u32, CraneliftBackendError> {
        // ⛔ The complete requested slot is threaded in, not reassembled here.
        // The alias rule tests exact equality of the whole record, so a seam that
        // rebuilt it would be a second definition of "the same value".
        let coordinate = requested.coordinate;
        let Some(claim) = views.direct_emission else {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "a continuation input carries no direct-emission availability claim, so \
                     nothing says where this emission seat holds {coordinate:?}; \
                     RT-CONTSRC-PRODUCER-LOCAL D3b refuses rather than reading the \
                     context-capture claim, which is an index into a different environment"
                ),
            ));
        };
        match claim {
            ContinuationEnvironmentClaim::CurrentLexical {
                emission_owner,
                producer_result_origin,
                emission_origin,
                lexical_environment_origin,
                nearest_alias_index,
            } => {
                // ⛔ A current-lexical claim is authority over a PREDECLARED
                // retained environment. A generated context lowers a raw body
                // and does not stand in the producer's semantic environment, so
                // the claim is refused BEFORE any operand run is indexed.
                let ContinuationEmissionOwner::Predeclared(owner) = defining_owner else {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        "a current-lexical availability claim was presented to a generated \
                         emission context, which holds no retained lexical environment for the \
                         producer; its nearest-alias index has nothing here to index and \
                         RT-CONTSRC-PRODUCER-LOCAL D3b refuses rather than reading it as a \
                         frame slot",
                    ));
                };
                if emission_owner != owner {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "a current-lexical claim names emission owner {emission_owner:?}, \
                             which is not the unit currently being defined ({owner:?}), so its \
                             index counts binders in an environment this seat never stands in"
                        ),
                    ));
                }
                if seat.emission_origin != emission_origin
                    || seat.producer_result_origin != producer_result_origin
                {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        "a current-lexical claim is keyed to a different emission occurrence \
                         than the one now emitting, so the binder depth it counted is not the \
                         depth in force here",
                    ));
                }
                // `D3b` — the two CONSUMPTION-boundary mutations.
                //
                // ⛔ Applied here, between the claim and its revalidation, and
                // nowhere else. They perturb what this consumer INDEXES WITH,
                // which is the decision the revalidation below exists to check;
                // perturbing the claim earlier would measure the planner instead.
                //
                // ⚠ **Re-seated by the re-cut.** These previously fired inside
                // `resolve_continuation_immediate`, which the re-cut deleted, and
                // for one commit they fired nowhere at all -- the row kept
                // asserting while measuring the unmutated route. That is why the
                // row asserts `applications > 0` before it asserts anything about
                // the refusal.
                #[cfg(test)]
                let nearest_alias_index = {
                    use crate::cranelift_backend::lowering::{
                        d3b_consumer_mutation, record_d3b_consumer_application,
                        D3bConsumerMutation,
                    };
                    match d3b_consumer_mutation() {
                        D3bConsumerMutation::Exact => nearest_alias_index,
                        D3bConsumerMutation::ConsumeLocatorIndex => {
                            match coordinate {
                                ContinuationSourceCoordinate::ProducerLocal {
                                    locator, ..
                                } => {
                                    record_d3b_consumer_application();
                                    locator.environment_index
                                }
                                // ⛔ Declines rather than perturbing an entry
                                // root: an entry coordinate carries no locator,
                                // so there is no introduction index to consume
                                // and a substitute would measure an invented
                                // number.
                                ContinuationSourceCoordinate::EntryAbi { .. } => {
                                    nearest_alias_index
                                }
                            }
                        }
                        D3bConsumerMutation::ShiftProducerLocalSlot => match coordinate {
                            ContinuationSourceCoordinate::ProducerLocal { .. } => {
                                record_d3b_consumer_application();
                                nearest_alias_index.wrapping_add(1)
                            }
                            ContinuationSourceCoordinate::EntryAbi { .. } => nearest_alias_index,
                        },
                    }
                };
                // ⭐ The claim's index is CHECKED against the planner's own walk
                // of this seat, never re-derived here. See
                // `verify_current_lexical_availability`.
                verify_current_lexical_availability(
                    &self.static_transition_plan,
                    emission_owner,
                    producer_result_origin,
                    emission_origin,
                    lexical_environment_origin,
                    requested,
                    nearest_alias_index,
                )?;
                Ok(nearest_alias_index)
            }
            ContinuationEnvironmentClaim::EntryFrame {
                frame,
                declared_slot,
            } => {
                // The direct-emission consumer reads an entry frame only when
                // the emitting frame IS that frame -- a generated context
                // emitting from its own operand run.
                self.verify_entry_frame(coordinate, frame, declared_slot, defining_owner)?;
                Ok(declared_slot)
            }
        }
    }

    /// **`D3b` (re-cut) — resolve the CONTEXT-CAPTURE claim**, the consumer that
    /// reads `function_local.defining_abi_operands`.
    ///
    /// ⛔ A current-lexical claim is **refused outright** here: this consumer
    /// holds an entry-frame operand run and no semantic environment at all, so a
    /// nearest-alias index has nothing to index. That refusal is by **consumer
    /// environment identity**, not by root domain.
    fn resolve_context_capture_claim(
        &self,
        coordinate: ContinuationSourceCoordinate,
        views: ContinuationAvailabilityViews,
        defining_owner: ContinuationEmissionOwner,
    ) -> Result<u32, CraneliftBackendError> {
        let Some(claim) = views.context_capture else {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "a generated context capture carries no context-capture availability claim, \
                     so nothing says where this frame holds {coordinate:?}; \
                     RT-CONTSRC-PRODUCER-LOCAL D3b refuses rather than reading the \
                     direct-emission claim, whose index counts binders in a lexical environment \
                     this consumer does not hold"
                ),
            ));
        };
        match claim {
            ContinuationEnvironmentClaim::CurrentLexical { .. } => Err(unsupported(
                "ContinuationSpecialization",
                "a current-lexical availability claim was presented to the entry-frame capture \
                 consumer, which holds an ABI operand run and no semantic environment; a \
                 nearest-alias lexical index is not a frame slot and \
                 RT-CONTSRC-PRODUCER-LOCAL D3b refuses rather than indexing with it",
            )),
            ContinuationEnvironmentClaim::EntryFrame {
                frame,
                declared_slot,
            } => {
                self.verify_entry_frame(coordinate, frame, declared_slot, defining_owner)?;
                Ok(declared_slot)
            }
        }
    }

    /// **`D3b` (re-cut) — an entry-frame claim is lawful only where the frame it
    /// names is the frame this consumer actually holds, AND that frame really
    /// declares a member for the full coordinate at the declared slot.**
    ///
    /// ⛔ Membership is the check, not a numeric agreement. The retired law
    /// compared `immediate_slot` against `source_abi_position`, which is a
    /// relation between a frame position and a ROOT position — the coupling
    /// `D3c` falsified.
    fn verify_entry_frame(
        &self,
        coordinate: ContinuationSourceCoordinate,
        frame: ContinuationFrameIdentity,
        declared_slot: u32,
        defining_owner: ContinuationEmissionOwner,
    ) -> Result<(), CraneliftBackendError> {
        match (frame, defining_owner) {
            (
                ContinuationFrameIdentity::Predeclared(named),
                ContinuationEmissionOwner::Predeclared(held),
            ) => {
                if named != held {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "an entry-frame claim names predeclared frame {named:?}, which is \
                             not the frame being defined ({held:?}); its declared slot indexes \
                             an operand run this consumer does not hold"
                        ),
                    ));
                }
                // ⛔ **Membership, at the exact predeclared descriptor.** A
                // ProducerLocal member cannot be invented here: the entry source
                // enumeration produces exactly the entry ABI run, so a mid-body
                // value is simply absent and this refuses.
                verify_predeclared_entry_frame_membership(
                    &self.static_transition_plan,
                    named,
                    coordinate,
                    declared_slot,
                )
            }
            (
                ContinuationFrameIdentity::GeneratedContext {
                    context: claimed_context,
                    specialization,
                    worker_body_origin,
                },
                ContinuationEmissionOwner::Specialization(held),
            ) => {
                if specialization != held {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "an entry-frame claim names the generated context of \
                             specialization {specialization:?}, which is not the specialization \
                             whose frame is being defined ({held:?})"
                        ),
                    ));
                }
                // ⭐⭐ **The three-sided revalidation the ruling asks for.** The
                // claim is finalized, so it carries both the resolved
                // `ContinuationContextId` and the key it was resolved from. This
                // re-resolves that key against the plan in hand and checks the
                // answer AGREES with the recorded id.
                //
                // ⛔ Not redundant with finalization. Finalization proves the key
                // resolved uniquely *in the plan it ran over*; this proves the
                // consumer is holding that same plan. The two could only disagree
                // if a claim were carried across plans -- which is precisely the
                // failure that must not be able to hide behind a plausible id.
                let context = self
                    .static_transition_plan
                    .continuation_context_for(specialization, worker_body_origin)?
                    .ok_or_else(|| {
                        unsupported(
                            "ContinuationSpecialization",
                            "an entry-frame claim names a generated context frame that the \
                             planner never interned, so no declared capture run exists to \
                             discharge it",
                        )
                    })?;
                // `D4b` — the BEHAVIOURAL non-vacuity counter and the identity
                // mutation, both on the live consumer path rather than in a
                // construction control.
                #[cfg(test)]
                let claimed_context = {
                    crate::cranelift_backend::lowering::record_d4b_generated_frame_consumption();
                    match crate::cranelift_backend::lowering::d4b_frame_mutation() {
                        crate::cranelift_backend::lowering::D4bFrameMutation::Exact => {
                            claimed_context
                        }
                        // ⛔ Perturbs the RECORDED id, which is exactly the input
                        // the agreement check reads. The key it resolves from is
                        // left intact, so what reds is the disagreement and not a
                        // failure to resolve.
                        crate::cranelift_backend::lowering::D4bFrameMutation::WrongClaimedContext => {
                            claimed_context.d4b_displaced()
                        }
                    }
                };
                if context.id() != claimed_context {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "an entry-frame claim carries generated context {claimed_context:?}, \
                             but its own (specialization, worker body) key resolves to {:?} in \
                             the plan being lowered; the recorded identity and the key it was \
                             resolved from disagree",
                            context.id()
                        ),
                    ));
                }
                let captures = context.captures()?;
                let mut found = None;
                for (position, capture) in captures.iter().enumerate() {
                    if capture.coordinate != coordinate {
                        continue;
                    }
                    if found.is_some() {
                        return Err(unsupported(
                            "ContinuationSpecialization",
                            "a generated context declares two members for one continuation \
                             coordinate, so the declared slot is ambiguous; \
                             RT-CONTSRC-PRODUCER-LOCAL D3b refuses rather than taking the first",
                        ));
                    }
                    found = Some(position);
                }
                let position = found.ok_or_else(|| {
                    unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "a generated context declares no member for {coordinate:?}, so its \
                             frame cannot make that value available; this fails closed rather \
                             than falling back to a root position"
                        ),
                    )
                })?;
                let expected = u32::try_from(position)
                    .ok()
                    .and_then(|position| context.parameters().checked_add(position))
                    .ok_or_else(|| {
                        unsupported(
                            "ContinuationSpecialization",
                            "generated context capture slot exhausted",
                        )
                    })?;
                if expected != declared_slot {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "an entry-frame claim declares slot {declared_slot} for \
                             {coordinate:?}, but the generated context declares that member at \
                             slot {expected}; the two disagree, so at least one of them names a \
                             different value"
                        ),
                    ));
                }
                Ok(())
            }
            (ContinuationFrameIdentity::Predeclared(named), ContinuationEmissionOwner::Specialization(held)) => {
                Err(unsupported(
                    "ContinuationSpecialization",
                    format!(
                        "an entry-frame claim names predeclared frame {named:?} while the frame \
                         being defined is the generated context of {held:?}; a predeclared \
                         entry run is not reachable from a generated context and \
                         RT-CONTSRC-PRODUCER-LOCAL D3b refuses rather than indexing its own"
                    ),
                ))
            }
            (
                ContinuationFrameIdentity::GeneratedContext { specialization, .. },
                ContinuationEmissionOwner::Predeclared(held),
            ) => Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "an entry-frame claim names the generated context of {specialization:?} while the \
                     frame being defined is predeclared {held:?}; a generated context's capture \
                     run is not this function's entry run"
                ),
            )),
        }
    }
    fn claim_and_call_resolved_continuation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        identity: &ContinuationCallIdentity,
        // `D9` — the producer constructor's WHOLE lowered field run and the
        // ruled recursive position, not a pre-assembled ordinary run. ⛔ The
        // assembly moved here because `unit` is resolved here: both callers
        // previously built their own run from the nonrecursive fields alone,
        // and each carried a comment claiming the captures were appended by the
        // other side. One authority, one assembly, both callers.
        fields: &[LoweringOperand],
        recursive_position: usize,
        producer_env: &[LoweringEnvironmentBinding],
    ) -> Result<RoutedAnswer, CraneliftBackendError> {
        let identity = identity.clone();
        let defining = self.defining_unit.ok_or_else(|| {
            unsupported(
                "ContinuationSpecialization",
                "a continuation claim was reached with no unit currently being defined",
            )
        })?;
        // `D5a` — the EMISSION owner of the context currently executing, which
        // is a different question from which predeclared unit's body this is.
        // ⛔ Not derived from `defining`: a generated specialization context
        // lowers a raw body and would otherwise be mistaken for that raw body's
        // predeclared owner, which is the whole conflation being removed.
        let defining_owner = self.defining_emission_owner.ok_or_else(|| {
            unsupported(
                "ContinuationSpecialization",
                "a continuation claim was reached with no emission owner bound for the context \
                 currently being defined",
            )
        })?;
        // The EXACT target, resolved first.
        let exact_target = self
            .function_local
            .continuation_calls
            .get(&identity)
            .cloned()
            .ok_or_else(|| {
                unsupported(
                    "ContinuationSpecialization",
                    "the claimed continuation target was not declared into this function",
                )
            })?;
        // `D4` control, on the real causal call seam: substitute ONLY the
        // emitted `FuncRef` with another callable already declared into this
        // same function, on the same unit-call ABI.
        //
        // ⭐ Header, slots, offsets, inputs, identity and owner are all
        // retained, so the call is still emitted and the ONLY thing that moves
        // is the callee identity -- which is what makes the finished-CLIF
        // oracle's rejection attributable to one cause.
        //
        // ⛔ No fall back to exact, and no widening: if this function declares
        // no other unit-call target the control rejects loudly rather than
        // silently becoming the identity.
        #[cfg(test)]
        let target = match CONTINUATION_EMISSION_MUTATION.with(std::cell::Cell::get) {
            ContinuationEmissionMutation::SubstituteEmittedFuncRef => {
                let substitute = self
                    .function_local
                    .worker_calls
                    .values()
                    .chain(self.function_local.unit_calls.values())
                    .chain(self.function_local.declaration_calls.values())
                    .map(|call| call.function)
                    .find(|function| *function != exact_target.function)
                    .ok_or_else(|| {
                        unsupported(
                            "ContinuationSpecialization",
                            "the D4 emitted-ref substitution found no other unit-call target \
                             declared into this function; that is a fact about this function's \
                             declarations, not a licence to import a FuncRef from another \
                             function",
                        )
                    })?;
                units::DeclaredUnitCall {
                    function: substitute,
                    ..exact_target
                }
            }
            // `RT-CONTSPEC-WITNESS` `D7`/`AC-9`: redirect to a DISTINCT
            // SAME-SHAPED target, so no structural check can separate the two
            // and only the executed result can.
            //
            // ⛔ The predicate is `RT-WORKER-BIND`'s: equal declared arity and
            // equal capture count, read as the counts of this unit's own
            // declared `Parameter` and `Capture` slots. It is deliberately NOT
            // a comparison of ABI layout -- no width, alignment, offset,
            // carrier, ownership or header is consulted -- and NOT origin
            // inequality, which would make any other target qualify.
            ContinuationEmissionMutation::RedirectToDistinctSameShapedTarget => {
                // ⛔ Both sides are read from the SAME source -- each call's own
                // declared record -- so the comparison cannot be an artefact of
                // two different derivations disagreeing. The exact target is a
                // continuation specialization and is not an `emittable_units`
                // member, so a plan-side lookup answers `None` for it and the
                // control would refuse for its own reason rather than measure
                // anything. Measured, not reasoned: it did exactly that.
                let declared_shape = |call: &units::DeclaredUnitCall| {
                    (
                        call.slots
                            .iter()
                            .filter(|slot| slot.kind == AbiSlotKind::Parameter)
                            .count(),
                        call.slots
                            .iter()
                            .filter(|slot| slot.kind == AbiSlotKind::Capture)
                            .count(),
                    )
                };
                let exact_shape = declared_shape(&exact_target);
                let substitute = self
                    .function_local
                    .worker_calls
                    .values()
                    .chain(self.function_local.unit_calls.values())
                    .chain(self.function_local.declaration_calls.values())
                    .find(|call| {
                        call.function != exact_target.function
                            && declared_shape(call) == exact_shape
                    })
                    .map(|call| call.function)
                    .ok_or_else(|| {
                        unsupported(
                            "ContinuationSpecialization",
                            "the D7 same-shaped redirect found no DISTINCT target with the same \
                             declared arity and capture count declared into this function; per \
                             the frame that is a missing fixture precondition, not a discharge",
                        )
                    })?;
                units::DeclaredUnitCall {
                    function: substitute,
                    ..exact_target
                }
            }
            _ => exact_target,
        };
        #[cfg(not(test))]
        let target = exact_target;
        // Claim exactly once, with the defining unit supplied independently of
        // the token so the owner check is a real comparison.
        // `D4` controls, on the real claim: present the same exact token a
        // second time, or present an owner that is not the unit actually being
        // defined. Both perturb this call seam, not a pre-body ledger.
        #[cfg(test)]
        let mutation = CONTINUATION_EMISSION_MUTATION.with(std::cell::Cell::get);
        #[cfg(test)]
        let claimed_owner = if mutation == ContinuationEmissionMutation::ClaimUnderWrongOwner {
            // An actually wrong current owner: any planned owner that is not
            // the unit being defined.
            // ⛔ Drawn from the PLANNED UNIT population, not from the causal
            // calls' own owners, and with NO fall back to `defining`.
            //
            // ⭐ It used to read `continuation_calls().find(owner != defining)
            // .unwrap_or(defining)`. In this seam's real population there is
            // exactly one causal token and its owner IS the unit being defined,
            // so `find` returned `None` and the mutation silently became the
            // IDENTITY -- a committed control that could not fire, green since
            // `457b9fc6` for the same structural reason the same-shaped redirect
            // could not reach the call seam. Measured, not reasoned: the control
            // passed until this fallback was removed.
            self.static_transition_plan
                .emittable_units()?
                .iter()
                .map(|unit| ContinuationEmissionOwner::Predeclared(unit.function()))
                .find(|owner| *owner != defining_owner)
                .ok_or_else(|| {
                    unsupported(
                        "ContinuationSpecialization",
                        "the D4 wrong-owner control found no planned unit other than the one being \
                         defined; a control with no wrong owner to present must fail loudly rather \
                         than quietly claim under the right one",
                    )
                })?
        } else {
            defining_owner
        };
        #[cfg(not(test))]
        let claimed_owner = defining_owner;
        let ledger = self.continuation_claims.as_mut().ok_or_else(|| {
            unsupported(
                "ContinuationSpecialization",
                "a continuation claim was reached with no open claim ledger",
            )
        })?;
        ledger.claim_exact(&identity, claimed_owner)?;
        #[cfg(test)]
        d5a_trace(format!(
            "  CLAIM outcome=Claimed target={:?} owner={claimed_owner:?}",
            identity.target()
        ));
        #[cfg(test)]
        if mutation == ContinuationEmissionMutation::ClaimTokenTwice {
            ledger.claim_exact(&identity, claimed_owner)?;
        }

        // Capture slots come from the EXACT producer environment, addressed by
        // the projected `source_owner` + `source_abi_position`. ⛔
        // `ordinary_abi_position` is not a source position and is never used
        // as one -- it indexes the producer's own ABI, which this call is not
        // reading.
        let unit = self
            .static_transition_plan
            .continuation_units()?
            .into_iter()
            .find(|unit| unit.id() == identity.target())
            .ok_or_else(|| {
                unsupported(
                    "ContinuationSpecialization",
                    "the claimed target has no projected continuation unit",
                )
            })?;
        // ⭐⭐ **`RT-CONTSRC-PRODUCER-LOCAL` `D9` — THE ORDINARY RUN IS THE
        // PLANNER'S ENVELOPE, not the nonrecursive fields alone.**
        //
        // ⛔ **What this replaced, and why it was a defect rather than a
        // shortfall.** The previous assembly filtered the ruled recursive field
        // out of `args` and stopped, under a comment claiming *"captures are
        // appended by the shared machinery from the planner's own ordered
        // projection"*. **No such append exists**: the shared machinery adds the
        // continuation inputs and nothing else. So a continuation whose selected
        // worker has captures was called with the nonrecursive prefix only, and
        // the callee's declared `Parameter` run went unfilled. Measured by the
        // Architect on the `AC-1` row (`evt_1y7h08xd7ermp`): a callee declaring
        // 6 `Parameter` + 2 `Capture` received 1 ordinary + 2 continuation
        // inputs.
        //
        // ⭐ The values were never missing. The selected recursive field IS the
        // closure, and its ordered capture vector is already in hand here at the
        // exact source-bearing seat -- this node's landed phase-bearing
        // closure-capture representation put it there. The envelope says which
        // of them go where.
        //
        // ⛔ The envelope is the planner's ORDERED sequence and is consumed in
        // its own order. Each role is resolved from the authority that owns it:
        // a nonrecursive field from its **exact lowered source position**, a
        // worker capture from the **selected closure's own ordered capture
        // vector at that exact ordinal**. ⛔ A capture ordinal is NOT a generic
        // environment index and is never used as one -- that is the cross-plane
        // aliasing this node removed, and reintroducing it here would alias the
        // closure's capture run onto the producer's environment.
        let envelope = unit.ordinary_envelope()?;
        // `D9b` — the envelope perturbations, under test only. ⛔ Each moves ONE
        // fact of the planner's own sequence and leaves the assembler, the field
        // run and the selected closure untouched, so a refusal is attributable
        // to the moved role rather than to a rewritten assembly. Applied to the
        // sequence the assembler is about to read, which is the producer input
        // the role relation is about.
        #[cfg(test)]
        let envelope = crate::cranelift_backend::lowering::d9_perturb_envelope(envelope);
        // `D9b` — the assembled run, recorded per role position AFTER assembly.
        // ⛔ Recorded as a keyed sequence, never a multiset: a bag of five
        // capture values proves only that the right values exist somewhere, and
        // is satisfied by any permutation of them.
        #[cfg(test)]
        let recorded_envelope = envelope.clone();
        // The selected recursive field, by the ruled position. Its identity is
        // checked against the planner's own worker facts before a single
        // capture is read from it, so a capture taken below is known to have
        // come from the closure the plan selected.
        let selected = fields.get(recursive_position).ok_or_else(|| {
            unsupported(
                "ContinuationSpecialization",
                "the ruled recursive position names no field of the planned producer constructor",
            )
        })?;
        let (selected_captures, selected_body, selected_arity) = match selected {
            LoweringOperand::Specialized(Lowered::Closure {
                captures,
                params,
                body,
            }) => (captures, *body, params.len()),
            _ => {
                return Err(unsupported(
                    "ContinuationSpecialization",
                    format!(
                        "the ruled recursive field at position {recursive_position} is not a retained \
                         closure, so this continuation's selected worker has no capture run to \
                         assemble from"
                    ),
                ));
            }
        };
        if selected_body != unit.worker_body_origin() {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "the ruled recursive field is a closure over body {selected_body:?}, but this \
                     continuation's selected worker names body {:?}",
                    unit.worker_body_origin()
                ),
            ));
        }
        if selected_arity != unit.worker_declared_arity() as usize {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "the ruled recursive field is a closure of arity {selected_arity}, but this \
                     continuation's selected worker declares {}",
                    unit.worker_declared_arity()
                ),
            ));
        }
        if selected_captures.len() != unit.worker_capture_count() {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "the ruled recursive field closes over {} values, but this continuation's \
                     selected worker declares {} captures",
                    selected_captures.len(),
                    unit.worker_capture_count()
                ),
            ));
        }
        let mut ordinary = Vec::with_capacity(envelope.len());
        // ⛔ Capture roles are consumed in ascending contiguous ordinal order
        // from zero. The envelope is a SEQUENCE, so two roles carrying swapped
        // ordinals is a different run, and taking each ordinal as it comes
        // without this check would assemble that different run silently.
        let mut next_capture_ordinal = 0u32;
        let mut seen_capture = false;
        for (role_position, role) in envelope.iter().enumerate() {
            match role {
                ContinuationOrdinaryEnvelopeRole::NonrecursiveConstructorField {
                    source_position,
                } => {
                    if seen_capture {
                        return Err(unsupported(
                            "ContinuationSpecialization",
                            format!(
                                "the ordinary envelope holds a nonrecursive field at role \
                                 position {role_position} after a worker capture; the ruled \
                                 envelope is the nonrecursive prefix followed by the capture run, \
                                 and a call assembled from a permuted envelope fills the callee's \
                                 declared parameters with the wrong values"
                            ),
                        ));
                    }
                    let source = *source_position as usize;
                    if source == recursive_position {
                        return Err(unsupported(
                            "ContinuationSpecialization",
                            format!(
                                "the ordinary envelope names source position {source} as a \
                                 nonrecursive field, but that is the ruled recursive position; \
                                 the selected recursive field is a compiler-only member and never \
                                 an ordinary operand"
                            ),
                        ));
                    }
                    let field = fields.get(source).ok_or_else(|| {
                        unsupported(
                            "ContinuationSpecialization",
                            format!(
                                "the ordinary envelope names source position {source}, outside \
                                 the planned producer constructor's {}-field run",
                                fields.len()
                            ),
                        )
                    })?;
                    // ⛔ Cloned WHOLE, at its own phase. A carried field stays
                    // carried; re-wrapping it as specialized here would be the
                    // cross-plane aliasing this node removed.
                    ordinary.push(field.clone());
                }
                ContinuationOrdinaryEnvelopeRole::WorkerCapture {
                    ordinal,
                    closure_origin,
                    ..
                } => {
                    seen_capture = true;
                    if *closure_origin != unit.worker_closure_origin() {
                        return Err(unsupported(
                            "ContinuationSpecialization",
                            format!(
                                "the ordinary envelope's capture at role position {role_position} \
                                 names closure occurrence {closure_origin:?}, but this \
                                 continuation's selected worker is closure {:?}; a capture read \
                                 from another closure's run is another closure's value",
                                unit.worker_closure_origin()
                            ),
                        ));
                    }
                    if *ordinal != next_capture_ordinal {
                        return Err(unsupported(
                            "ContinuationSpecialization",
                            format!(
                                "the ordinary envelope's capture at role position {role_position} \
                                 names ordinal {ordinal}, where the ruled run's next ordinal is \
                                 {next_capture_ordinal}; the capture run is contiguous from zero \
                                 in envelope order, and two roles with exchanged ordinals \
                                 assemble a different call"
                            ),
                        ));
                    }
                    let capture = selected_captures.get(*ordinal as usize).ok_or_else(|| {
                        unsupported(
                            "ContinuationSpecialization",
                            format!(
                                "the ordinary envelope names capture ordinal {ordinal}, outside \
                                 the selected closure's {}-value capture run",
                                selected_captures.len()
                            ),
                        )
                    })?;
                    // ⛔ The operand is taken WHOLE, at its own phase. A carried
                    // capture stays carried: this is the phase-bearing edge, and
                    // re-reading it as a specialized template here is what the
                    // representation exists to prevent.
                    ordinary.push(capture.clone());
                    next_capture_ordinal = next_capture_ordinal.wrapping_add(1);
                }
            }
        }
        if next_capture_ordinal as usize != selected_captures.len() {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "the ordinary envelope consumed {next_capture_ordinal} of the selected \
                     closure's {} captures; the whole capture run is the callee's declared \
                     parameter tail and a short run leaves declared parameters unfilled",
                    selected_captures.len()
                ),
            ));
        }
        // The final cardinality, against the continuation's own declared
        // ordinary-parameter count. ⛔ Checked against the DECLARATION rather
        // than against the envelope's length: the two are separate planner
        // facts, and comparing the assembled run with the sequence it was
        // assembled from would be an identity.
        if ordinary.len() != unit.ordinary_parameters() as usize {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "the assembled ordinary run holds {} operands but this continuation declares \
                     {} ordinary parameters",
                    ordinary.len(),
                    unit.ordinary_parameters()
                ),
            ));
        }
        // `D9b` — the assembled run, recorded per role position AFTER assembly
        // and AFTER the declared-header cardinality validation above.
        //
        // ⛔ **The order is the point, and it was wrong.** This recorder used to
        // stand immediately BEFORE that check, so a run the assembler was about
        // to REJECT still entered the positive relation. The row that reads this
        // log asserts a property of runs the compiler accepted; admitting a
        // rejected one would let a refusal contribute the very evidence that it
        // did not happen.
        //
        // ⛔ Recorded as a keyed sequence, never a multiset: a bag of capture
        // values proves only that the right values exist somewhere, and is
        // satisfied by any permutation of them. The assembler's INPUTS are
        // recorded beside it so the expectation can be derived without reading
        // the assembled run.
        #[cfg(test)]
        crate::cranelift_backend::lowering::record_d9_assembly(
            crate::cranelift_backend::lowering::D9Assembly {
                unit: unit.id(),
                roles: recorded_envelope
                    .iter()
                    .map(crate::cranelift_backend::lowering::d9_role_key)
                    .collect(),
                operands: ordinary
                    .iter()
                    .map(crate::cranelift_backend::lowering::d9_operand_identity)
                    .collect(),
                fields: fields
                    .iter()
                    .map(crate::cranelift_backend::lowering::d9_operand_identity)
                    .collect(),
                captures: selected_captures
                    .iter()
                    .map(crate::cranelift_backend::lowering::d9_operand_identity)
                    .collect(),
            },
        );
        let mut inputs = ordinary;
        #[cfg(test)]
        d5a_trace(format!(
            "  CAPTURES defining={defining:?} consumer={:?} producer={:?} env_len={} inputs={:?}",
            unit.consumer_owner(),
            unit.producer_owner(),
            producer_env.len(),
            unit.continuation_inputs()?
                .iter()
                .map(|input| input.coordinate)
                .collect::<Vec<_>>()
        ));
        // `RT-CONTSRC-PRODUCER-LOCAL` `D4a` — the observatory's seam half.
        //
        // ⛔ **It observes and returns nothing.** It stands ahead of the
        // coordinate refusal below precisely because that refusal is still in
        // force: `D4a` measures the operands a `D3b` consumer WOULD index, and
        // measuring them is not consuming them. Test-only; the refusal below is
        // untouched and still fires on every one of these inputs.
        #[cfg(test)]
        for input in unit
            .continuation_inputs()?
            .into_iter()
            .filter(|_| crate::cranelift_backend::lowering::d4a_armed())
        {
            use crate::cranelift_backend::lowering::{
                d4a_describe_binding, d4a_record_seam, d4a_slot_selection,
                ContinuationEnvironmentClaim, ContinuationSourceCoordinate, D4aSeamObservation,
                D4aSlotSelection,
            };
            let ContinuationSourceCoordinate::ProducerLocal { binding, locator } = input.coordinate
            else {
                continue;
            };
            // `D3b` re-cut — the observatory reads the DIRECT-EMISSION view,
            // which is the one this seat consumes. ⛔ Not "whichever view has a
            // current-lexical claim": taking the context-capture view here would
            // measure an operand run this seat does not hold.
            let Some(ContinuationEnvironmentClaim::CurrentLexical {
                nearest_alias_index, ..
            }) = input.availability.direct_emission
            else {
                continue;
            };
            let locator_index = locator.environment_index;
            // `D4a` mutation point. ⭐ The mutation perturbs WHICH INDEX the
            // instrument reads, which is exactly the choice a `D3b` consumer
            // will have to make. Production is not routed through it.
            let selected = match d4a_slot_selection() {
                D4aSlotSelection::Exact | D4aSlotSelection::SwapSlots => nearest_alias_index,
                D4aSlotSelection::UseLocatorIndex => locator_index,
            };
            let at = |index: u32| d4a_describe_binding(producer_env.get(index as usize));
            let (nearest_alias_operand, locator_operand) =
                if d4a_slot_selection() == D4aSlotSelection::SwapSlots {
                    (at(locator_index), at(selected))
                } else {
                    (at(selected), at(locator_index))
                };
            d4a_record_seam(D4aSeamObservation {
                binding_origin: binding.binding_origin,
                nearest_alias_index,
                locator_index,
                nearest_alias_operand,
                locator_operand,
            });
        }
        // `RT-CONTSRC-PRODUCER-LOCAL` `D3c` — the entry-ABI observatory's seat
        // half.
        //
        // ⛔ **It observes and returns nothing**, exactly like `D4a`'s half
        // above, and it is `#[cfg(test)]` throughout.
        //
        // ⚠ This comment previously added that "the resolution below is
        // untouched and still copies `source_abi_position` into the immediate
        // slot". That was true while `D3c` was a measurement-only checkpoint and
        // is now false: the resolution below consumes the planner-issued
        // nearest-exact-alias availability, and copying a root ABI position is
        // precisely the defect `D3c` measured and `D3b` removed. The observatory
        // itself is unchanged — what changed is what it observes.
        //
        // ⭐ It records the emitting environment and the entry ABI operand run
        // **as production holds them**, and does no reasoning: the control
        // re-derives where the entry value actually sits. The instrument must
        // not be the oracle it is used to test.
        #[cfg(test)]
        if crate::cranelift_backend::lowering::d3c_armed()
            && matches!(defining_owner, ContinuationEmissionOwner::Predeclared(_))
        {
            use crate::cranelift_backend::lowering::{
                d3c_position_selection, d3c_record_seat, d4a_describe_binding,
                ContinuationSourceCoordinate as D3cCoordinate, D3cPositionSelection,
                D3cSeatObservation,
            };
            let vector = unit.continuation_inputs()?;
            let entry_abi_inputs = vector
                .iter()
                .filter(|i| matches!(i.coordinate, D3cCoordinate::EntryAbi { .. }))
                .count();
            let producer_local_inputs = vector
                .iter()
                .filter(|i| matches!(i.coordinate, D3cCoordinate::ProducerLocal { .. }))
                .count();
            let emission_environment = producer_env
                .iter()
                .map(|binding| d4a_describe_binding(Some(binding)))
                .collect::<Vec<_>>();
            let abi_operands = self.function_local.defining_abi_operands.len();
            // The independent source-descriptor authority. Both facts come
            // from the slot walk's recorded KINDS, so the derived position
            // below is computed from the descriptor and never found by looking
            // for the operand in the environment.
            let source_parameter_run = self
                .function_local
                .defining_abi_slot_kinds
                .iter()
                .filter(|kind| **kind == AbiSlotKind::Parameter)
                .count();
            for input in &vector {
                let D3cCoordinate::EntryAbi {
                    source_abi_position, ..
                } = input.coordinate
                else {
                    continue;
                };
                // The entry oracle: production's own record of the operand that
                // arrived at this ABI position, taken at unit entry from the
                // slot walk, with no environment index in play.
                let entry_operand = d4a_describe_binding(
                    self.function_local
                        .defining_abi_operands
                        .get(source_abi_position as usize)
                        .map(|operand| LoweringEnvironmentBinding::Value(operand.clone()))
                        .as_ref(),
                );
                // `D3c` mutation point. The selection perturbs WHICH POSITION
                // of the real emission environment the instrument reads: the
                // measured one, or the root ABI position that the RETIRED
                // `RootIsImmediate` substitution used. Production today reads
                // neither -- it resolves `CurrentLexical`'s
                // `nearest_alias_index`. The root position itself is still
                // lawful production provenance on the `EntryAbi` coordinate;
                // only indexing an environment with it is the retired shape.
                let observed_position = match d3c_position_selection() {
                    D3cPositionSelection::SourceAbiPosition => Some(source_abi_position),
                    D3cPositionSelection::MeasuredImmediate => emission_environment
                        .iter()
                        .position(|operand| *operand == entry_operand)
                        .and_then(|position| u32::try_from(position).ok()),
                };
                let observed_operand = match observed_position {
                    Some(position) => d4a_describe_binding(producer_env.get(position as usize)),
                    None => "none".to_string(),
                };
                d3c_record_seat(D3cSeatObservation {
                    entry_abi_inputs,
                    producer_local_inputs,
                    source_abi_position,
                    entry_operand,
                    abi_operands,
                    source_slot_kind: self
                        .function_local
                        .defining_abi_slot_kinds
                        .get(source_abi_position as usize)
                        .copied(),
                    source_parameter_run,
                    emission_environment: emission_environment.clone(),
                    observed_position,
                    observed_operand,
                });
            }
        }
        let mut resolved_slots: Vec<(ContinuationSourceCoordinate, u32)> = Vec::new();
        for input in unit.continuation_inputs()? {
            // `RT-CONTSRC-PRODUCER-LOCAL` `D1` — present a producer-local
            // coordinate to this seam, so its refusal is measured rather than
            // merely written. ⛔ Applied BEFORE the domain match, because the
            // question is what the match does with a domain it cannot locate.
            #[cfg(test)]
            let input = {
                let mut input = input;
                if d5a_route_mutation() == D5aRouteMutation::PresentProducerLocalCoordinate {
                    record_d5a_route_application();
                    input.coordinate = ContinuationSourceCoordinate::producer_local_probe();
                }
                input
            };
            // `RT-CONTSRC-PRODUCER-LOCAL` `D1` `D3` consumer 3 of 3 — the
            // emission resolver. ⛔ Exhaustive over the coordinate domains with
            // no default and no fallthrough: this seam indexes an environment,
            // and a domain it has not been taught to locate must refuse rather
            // than index with whatever integer it can reach.
            // Retained for the `D5a` route mutation below, which rewrites an
            // availability using the ROOT position and must keep doing so.
            #[cfg(test)]
            let source_abi_position = match input.coordinate {
                ContinuationSourceCoordinate::EntryAbi {
                    source_abi_position, ..
                } => source_abi_position,
                ContinuationSourceCoordinate::ProducerLocal { .. } => u32::MAX,
            };
            // `D5a` checkpoint 4 step 3 -- the capture projection's three
            // reaching mutations. ⛔ Each perturbs one COORDINATE the projection
            // supplies; the two guards below are untouched.
            #[cfg(test)]
            let input = {
                let mut input = input;
                match d5a_route_mutation() {
                    // ⭐ **The re-cut sharpens this row rather than porting it.**
                    // Under the retired law "read the root position as the
                    // immediate slot" was a type-level substitution; `D3c`
                    // measured it as the live defect, so it is now expressed as
                    // what it actually is — the claim's index replaced by
                    // `source_abi_position` with the claim's ENVIRONMENT left
                    // intact. That is the substitution `D3c` flipped, and it is
                    // now caught by the planner walk rather than by an equality.
                    D5aRouteMutation::ReadRootPositionAsImmediateSlot => {
                        record_d5a_route_application();
                        input.availability.direct_emission = input
                            .availability
                            .direct_emission
                            .map(|claim| d3b_replace_claim_index(claim, source_abi_position));
                    }
                    // ⛔ Scoped to the SPECIALIZATION arm, whose direct-emission
                    // claim is an entry-frame one. Applied to a predeclared
                    // emitter the current-lexical revalidation refuses first, and
                    // the row would name the bounds guard while measuring the
                    // membership one.
                    D5aRouteMutation::PerturbImmediateSlotOutOfRange => {
                        if matches!(defining_owner, ContinuationEmissionOwner::Specialization(_)) {
                            record_d5a_route_application();
                            let out_of_range =
                                u32::try_from(producer_env.len()).unwrap_or(u32::MAX);
                            input.availability.direct_emission = input
                                .availability
                                .direct_emission
                                .map(|claim| d3b_replace_claim_index(claim, out_of_range));
                        }
                    }
                    // ⛔ Scoped to the PREDECLARED arm, whose direct-emission
                    // claim is current-lexical. `+1` moves the nearest-alias index
                    // off the binder depth the planner walked, which the
                    // revalidation must catch.
                    D5aRouteMutation::PerturbPredeclaredImmediateSlot => {
                        if matches!(defining_owner, ContinuationEmissionOwner::Predeclared(_)) {
                            if let Some(claim) = input.availability.direct_emission {
                                record_d5a_route_application();
                                let moved = d3b_claim_index(claim).wrapping_add(1);
                                input.availability.direct_emission =
                                    Some(d3b_replace_claim_index(claim, moved));
                            }
                        }
                    }
                    _ => {}
                }
                input
            };
            // `RT-CONTSRC-PRODUCER-LOCAL` `D2b` — the emission seam matches the
            // AVAILABILITY domain, exactly as it already matches the coordinate
            // domain above. ⛔ No wildcard: `D2b` projects the two producer-local
            // availabilities and `D3` teaches this seam to consume them. Until
            // then a lexical index must not be handed to `producer_env`, which
            // is an ABI operand run — the two are different environments, and
            // indexing one with the other's index names a different value.
            //
            // ⭐ Unreachable today by construction, since the coordinate match
            // above has already refused every producer-local coordinate and the
            // projection only builds these arms for that domain. It is written
            // anyway because "unreachable" is a claim about the current
            // projection, and this seam must not be the place that discovers it
            // was wrong.
            // `RT-CONTSRC-PRODUCER-LOCAL` `D3b` — the seam resolves the
            // (coordinate, availability) PAIR, not each half in turn, and it
            // does so AFTER the `D5a` route mutations so that what they perturb
            // is what this reads.
            //
            // ⛔ **Pairing is the unit because the domains are not independent.**
            // The projection builds `EntryAbi` availability only for an entry
            // coordinate and the two producer-local availabilities only for a
            // producer-local one, so a crossed pair is not a case to handle but
            // a statement that the projection and this seam disagree about what
            // the value IS. Matching the halves separately lets a crossed pair
            // through whenever both halves are individually well-formed, which
            // is exactly the shape that reads as safe.
            let immediate_slot = self.resolve_direct_emission_claim(
                &input.requested_source_slot(),
                input.availability,
                defining_owner,
                ContinuationDirectEmissionSeat {
                    producer_result_origin: unit.producer_result_origin(),
                    emission_origin: unit.producer_construct_origin(),
                },
            )?;
            // `D3b` (re-cut) — INJECTIVITY, over the whole emission.
            //
            // ⭐ **The re-cut widens this law, and that is a consequence of the
            // correction rather than an extra.** Under the retired representation
            // the law had to be scoped to the producer-local domain, because an
            // entry-ABI `immediate_slot` was a position in the ABI frame while a
            // producer-local index was a position in the lexical frame, and
            // comparing the two integers was itself the cross-frame conflation.
            //
            // ⛔ Now every claim at one direct-emission seat names a position in
            // **the same environment** — this seat's own. So two inputs
            // resolving to one slot is unambiguously two values claiming one
            // place, and at least one of them would be emitted carrying the
            // other's operand. It is the consumption-side dual of the planner's
            // refusal of a coordinate present at two positions of the seat.
            if let Some((seen, _)) = resolved_slots
                .iter()
                .find(|(_, slot)| *slot == immediate_slot)
            {
                if *seen != input.coordinate {
                    return Err(unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "two distinct continuation inputs of one emission resolve to the \
                             same immediate slot {immediate_slot}: {seen:?} and {:?}. One \
                             position cannot hold both values, so at least one would be emitted \
                             carrying the other's operand",
                            input.coordinate
                        ),
                    ));
                }
            }
            resolved_slots.push((input.coordinate, immediate_slot));
            let binding = producer_env
                .get(immediate_slot as usize)
                .ok_or_else(|| {
                    unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "a continuation input names immediate slot {} outside the emitting \
                             context's environment of {} bindings; note this is the IMMEDIATE \
                             slot, whose meaning is fixed by the availability domain {:?} and \
                             not by any root position beside it",
                            immediate_slot,
                            producer_env.len(),
                            input.availability,
                        ),
                    )
                })?;
            inputs.push(
                binding
                    .value_at("a continuation capture input")?
                    .clone(),
            );
        }

        let (returned, call) = self.call_declared_unit_target(
            builder,
            target,
            &inputs,
            #[cfg(test)]
            None,
        )?;
        // `4b` -- anchor the emitted instruction to the exact causal token.
        //
        // ⭐ The `Inst`, not the target: the callee is decoded back out of the
        // finished CLIF by `verify_emitted_continuation_calls`. Recording
        // `target` here would compare the emitter's own input with itself and
        // would agree with the `D4` redirect, which is precisely the vacuous
        // shape this gate exists to avoid.
        //
        // ⛔ A second record for one token is a rejection: emission is once per
        // causal identity, and the claim ledger's affinity does not entail it --
        // that ledger would be satisfied by a claim with no call at all.
        // `4b` closure control: emit the call and skip the record, so the
        // finished-CLIF sweep has an emission the records cannot account for.
        #[cfg(test)]
        let record = CONTINUATION_EMISSION_MUTATION.with(std::cell::Cell::get)
            != ContinuationEmissionMutation::SuppressEmissionRecord;
        #[cfg(not(test))]
        let record = true;
        if record
            && self
                .function_local
                .continuation_emissions
                .insert(identity.clone(), call)
                .is_some()
        {
            return Err(unsupported(
                "ContinuationSpecialization",
                "a causal token emitted more than one direct continuation call",
            ));
        }
        #[cfg(test)]
        d5a_trace("  CLAIM outcome=CallEmitted".to_string());
        // ⭐⭐ `D6a` upstream half — PRODUCER 2, and this is the only place it
        // fires. The authority is the opaque `ContinuationCallIdentity` this
        // function consumed: the owner/affine claim has succeeded above and the
        // emitted callee has been checked against `identity.target()`, so the
        // value being returned is the result of that exact call and nothing
        // else. ⛔ Nothing about the origin, the frame, the owner, the tag or
        // the ABI is consulted, and a static-worker or raw unit call cannot
        // reach this line at all.
        //
        // The trace records `identity.target()` — the authority's OWN exact
        // identity, read back out of the value this function consumed. ⛔ Not
        // `target`, which is the emitter's input and would agree with itself.
        #[cfg(test)]
        record_d6a_route_event(D6aRouteEvent::CallResultRaised {
            target: identity.target(),
        });
        #[cfg(test)]
        if d6a_route_mutation() == D6aRouteMutation::DropCallResultRoute {
            record_d6a_route_application();
            return Ok(RoutedAnswer::direct(returned));
        }
        Ok(RoutedAnswer::checked(returned))
    }

    /// **`RT-DECL-CLOSURE-PORT` `D5a` — the detached-result consumption seat.**
    ///
    /// Where fixed-point discovery detaches a producer as an ordinary unit
    /// result, the active computational frame is already gone by the time the
    /// result exists, so the four-field selector's operands do not exist here.
    /// ⛔ They are not reconstructed and not faked: authority comes from the
    /// planner's own result-edge projection for the unit being defined, taken
    /// **before** this function was defined.
    ///
    /// This seat sits after the exact retained result is lowered and **before**
    /// [`Lowering::transfer_unit_result_into_carrier`], allocation, publication
    /// or join — which is where the landed object fixture was measured to
    /// refuse (`UNIT-RESULT transfer origin=36` immediately precedes
    /// `BOUNDARY-REFUSAL`).
    ///
    /// ## What each outcome means, because "zero" is the one that reads wrong
    ///
    /// - **No residual edge** — this owner has no planner-issued call left to
    ///   discharge here, either because it never had one or because the
    ///   retained-frame seat already emitted it. The ordinary path is correct
    ///   and untouched. ⚠ This is *not* the contract's "zero member is a hard
    ///   stop": an owner that genuinely owes a call and emits none is caught by
    ///   the whole-pass `planned = resolved = declared = emitted` equality,
    ///   which is the global affine closure of contract 5. A local stop here
    ///   would red every unit in the program that lawfully owns no call.
    /// - **More than one residual edge** — an unresolved multi-member
    ///   composition. ⛔ Rejected. One result value cannot discharge two causal
    ///   calls, and picking one would make lowering the authority for a fact the
    ///   planner owns.
    /// - **Exactly one** — the ruled case. The lowered result must *be* the
    ///   planned constructor at that edge's construct origin, checked against
    ///   the planner's own identity rather than against its name or shape.
    ///
    /// ## The five guards, and how each one is reached
    ///
    /// ⭐ These were carried as **explicitly unexercised** through checkpoints 1
    /// to 3: the only fixture reaching this seat refused further along, so a
    /// control written then would have compared a red against a red. Checkpoint
    /// 4 made the route positive, and each is now red by its own reaching
    /// mutation on the witness that compiles —
    /// `d5a_the_detached_result_seats_five_guards_are_each_reached_by_a_real_mutation`,
    /// with the [`D5aRouteMutation`] variant named beside it:
    ///
    /// | guard | reaching mutation |
    /// |---|---|
    /// | multi-member projection | `DuplicateResidualEdge` |
    /// | result is not a constructor | `CarryNonConstructorResult` |
    /// | identity disagreement | `StripLoweredConstructorIdentity` |
    /// | position outside the field run | `PerturbRecursivePosition` |
    /// | field run versus declared run | `PerturbOrdinaryParameterCount` |
    ///
    /// ⛔ Every one perturbs what this seat is **handed**, never the guard that
    /// inspects it.
    pub(super) fn eliminate_detached_producer_continuation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        result_edges: &[ContinuationResultEdge],
        lowered: LoweringOperand,
        unit_env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let mut residual = result_edges
            .iter()
            .filter(|edge| {
                // ⭐⭐ `RT-CONTSRC-PRODUCER-LOCAL` `D8k` — the residual is what
                // NEITHER verified form has discharged.
                //
                // A causal call answered by a composed source continuation is
                // not detached: the producer's constructor was eliminated in
                // place and the obligation was met by the raw-worker call the
                // source machine made. Before this, only a direct emission
                // could clear an edge, which is why the declaration-owned
                // composed witness refused here — the seat asked the unit to
                // RETURN a producer constructor the composed path exists to
                // consume.
                //
                // ⛔ **The claim, not only the verified relation, and that
                // ordering is forced.** This seat runs while the function is
                // still being built; `composed_discharges` is populated from
                // `pending_composed_discharges` only once the CLIF is finished,
                // which is strictly later. Reading the verified relation alone
                // here would always see it empty and clear nothing.
                //
                // ⇒ What makes this sound is that an unverified claim cannot
                // survive to an artifact: `verify_recorded_composed_discharges`
                // runs before this function is published and refuses the whole
                // compile if any claim fails. So no object exists in which a
                // claim that was never verified suppressed a residual. That is
                // a property of the ORDER of the two passes, and it is why
                // neither may be moved without re-deriving it.
                let claimed = self
                    .function_local
                    .pending_composed_discharges
                    .iter()
                    .any(|pending| pending.identity == edge.identity);
                !self
                    .function_local
                    .continuation_emissions
                    .contains_key(&edge.identity)
                    && !self
                        .function_local
                        .composed_discharges
                        .contains_key(&edge.identity)
                    && !claimed
            })
            .collect::<Vec<_>>();
        // `D5a` checkpoint 4 step 3 — the multi-member reaching mutation.
        //
        // ⛔ The SAME edge presented twice, not a fabricated second one: what the
        // guard rejects is two undischarged causal calls landing on one result
        // value, and a synthesized edge would additionally differ in identity,
        // leaving the refusal attributable to either fact.
        #[cfg(test)]
        if d5a_route_mutation() == D5aRouteMutation::DuplicateResidualEdge {
            if let Some(first) = residual.first().copied() {
                residual.push(first);
            }
        }
        // `D5a` checkpoint 4 step 3 — the non-constructor reaching mutation.
        //
        // A carried boundary word is a real operand of the only other class this
        // seat can be handed, so the guard rejects the case it names rather than
        // a shape invented for the test.
        #[cfg(test)]
        let lowered = if d5a_route_mutation() == D5aRouteMutation::CarryNonConstructorResult
            && !residual.is_empty()
        {
            LoweringOperand::Carried(CarriedBoundaryWord {
                word: builder.ins().iconst(types::I64, 0),
            })
        } else {
            lowered
        };
        // `D5a` checkpoint 4 step 3 — the identity-disagreement reaching
        // mutation. The result stays a specialized constructor with the same
        // fields; only its synthesized identity is withdrawn, so what the guard
        // reads is exactly the planner-identity comparison and nothing else.
        #[cfg(test)]
        let lowered = if d5a_route_mutation() == D5aRouteMutation::StripLoweredConstructorIdentity
            && !residual.is_empty()
        {
            match lowered {
                LoweringOperand::Specialized(Lowered::Constructor {
                    constructor,
                    synthesized_identity: _,
                    occurrence,
                    args,
                }) => LoweringOperand::Specialized(Lowered::Constructor {
                    constructor,
                    synthesized_identity: None,
                    // `D7` — carried through UNCHANGED, because this mutation's
                    // whole claim is that it withdraws the identity and nothing
                    // else. Withdrawing the occurrence as well would make the
                    // guard's refusal attributable to either field, and the row
                    // would stop measuring the identity comparison it names.
                    occurrence,
                    args,
                }),
                other => other,
            }
        } else {
            lowered
        };
        let edge = match residual.as_slice() {
            [] => return Ok(lowered),
            [edge] => *edge,
            _ => {
                return Err(unsupported(
                    "ContinuationSpecialization",
                    format!(
                        "the detached-result seat projected {} undischarged causal calls onto one \
                         unit result; a multi-member projection is a hard stop, never a preference \
                         rule, because one result value cannot discharge two causal calls",
                        residual.len()
                    ),
                ));
            }
        };
        #[cfg(test)]
        d5a_trace(format!(
            "  DETACHED-SEAT edge result={:?} construct={:?} pos={} target={:?}",
            edge.producer_result_origin,
            edge.producer_construct_origin,
            edge.recursive_position,
            edge.identity.target()
        ));
        // Contract 4, first clause: validate the specialized result's PLANNED
        // constructor identity and field run.
        //
        // ⛔ The check runs against the planner's identity for the projected
        // construct origin -- not against the constructor's name, its arity
        // alone, or the presence of a closure among its fields. Using the
        // emitted closure shape as the selector is exactly what contract 3
        // forbids; here the shape is not consulted at all, and a result that is
        // not the planned constructor rejects instead of being carried.
        let LoweringOperand::Specialized(Lowered::Constructor {
            synthesized_identity,
            args,
            ..
        }) = &lowered
        else {
            return Err(unsupported(
                "ContinuationSpecialization",
                "a projected causal call reached the detached-result seat with a unit result that \
                 is not a specialized constructor, so the planned producer constructor it must \
                 replace is not present",
            ));
        };
        let planned_identity = self
            .static_transition_plan
            .constructor_symbol_identity(edge.producer_construct_origin)?;
        if *synthesized_identity != Some(planned_identity) {
            return Err(unsupported(
                "ContinuationSpecialization",
                "the unit result at a projected causal edge is not the planner's own constructor \
                 for that edge's producer Construct origin",
            ));
        }
        // `D5a` checkpoint 4 step 3 — the out-of-range reaching mutation. Taken
        // as the planned constructor's own arity, so the perturbation is "one
        // past the field run" by construction rather than a literal that a
        // wider constructor would quietly bring back into range.
        #[cfg(test)]
        let position = if d5a_route_mutation() == D5aRouteMutation::PerturbRecursivePosition {
            args.len()
        } else {
            edge.recursive_position as usize
        };
        #[cfg(not(test))]
        let position = edge.recursive_position as usize;
        if position >= args.len() {
            return Err(unsupported(
                "ContinuationSpecialization",
                "a projected causal edge names a ruled recursive position outside the planned \
                 constructor's field run",
            ));
        }
        let unit = self
            .static_transition_plan
            .continuation_units()?
            .into_iter()
            .find(|unit| unit.id() == edge.identity.target())
            .ok_or_else(|| {
                unsupported(
                    "ContinuationSpecialization",
                    "the projected target has no continuation unit",
                )
            })?;
        // `D5a` checkpoint 4 step 3 — the field-run reaching mutation.
        //
        // ⛔ Applied AFTER the position guard above, so the two are separable:
        // the position it reads is still the ruled one and still in range, and
        // the only thing that moves is the declared ordinary run this count is
        // compared against.
        #[cfg(test)]
        let ordinary_declared = if d5a_route_mutation()
            == D5aRouteMutation::PerturbOrdinaryParameterCount
        {
            unit.ordinary_parameters() as usize + 1
        } else {
            unit.ordinary_parameters() as usize
        };
        #[cfg(not(test))]
        let ordinary_declared = unit.ordinary_parameters() as usize;
        // The field run, against the PLANNER'S OWN stated relation:
        // `nonrecursive_field_count = ordinary_parameters - worker_capture_count`
        // (`ContinuationUnit::ordinary_envelope`). Every field but the ruled
        // recursive one becomes an ordinary operand, and the selected worker's
        // capture run is the rest of the declared parameter tail.
        //
        // ⛔ **This used to read `args.len() != ordinary_declared + 1`, and that
        // is the pre-`D9` premise surviving as a precondition.** It is the same
        // defect `D9` removed one level down — *"the ordinary run is the
        // nonrecursive fields alone"* — left behind at the guard ABOVE the
        // assembly when the assembly itself was corrected. It is exact whenever
        // the selected worker has zero captures, which is every witness this
        // crate had, so nothing measured it. On a worker with captures it
        // refuses a LAWFUL producer: a 2-field constructor whose continuation
        // declares 3 ordinary parameters (1 nonrecursive + 2 captures) is
        // exactly the ruled shape, and the old form rejected it as a
        // disagreement.
        let nonrecursive_declared = ordinary_declared
            .checked_sub(unit.worker_capture_count())
            .ok_or_else(|| {
                unsupported(
                    "ContinuationSpecialization",
                    format!(
                        "this continuation declares {ordinary_declared} ordinary parameters but its \
                         selected worker declares {} captures, so the ruled envelope has no \
                         nonrecursive prefix",
                        unit.worker_capture_count()
                    ),
                )
            })?;
        if args.len() != nonrecursive_declared + 1 {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "the planned producer constructor has {} fields but its continuation declares \
                     {ordinary_declared} ordinary parameters over a selected worker with {} \
                     captures, leaving a {nonrecursive_declared}-field nonrecursive prefix; with \
                     exactly the ruled recursive field omitted the field run must exceed that \
                     prefix by one",
                    args.len(),
                    unit.worker_capture_count()
                ),
            ));
        }
        let identity = edge.identity.clone();
        // ⚠ `D6a`: this result becomes the defining unit's own result and
        // therefore crosses a FUNCTION BOUNDARY, which carries only the word.
        // The route is dropped here on purpose -- the caller re-attests from
        // its own exact claimed call identity, and a callee that wrote a hidden
        // route bit is exactly what the transport contract forbids.
        // `D9` — the whole planned field run, at its own phase, plus the ruled
        // position. The assembly is the shared seat's.
        let field_run = args
            .iter()
            .map(|arg| LoweringOperand::Specialized(arg.clone()))
            .collect::<Vec<_>>();
        Ok(self
            .claim_and_call_resolved_continuation(
                builder,
                &identity,
                &field_run,
                position,
                unit_env,
            )?
            .value)
    }

    /// **`D3` -- the callee-only consumer.**
    ///
    /// Emits the call to a statically-bound worker: validate the argument
    /// count against the declared arity, lower the explicit arguments in
    /// source order, append the stored captures **without phase conversion**,
    /// resolve this function's own `DeclaredUnitCall` by exact body origin,
    /// and call it through `call_declared_unit_target`.
    ///
    /// There is no `call_indirect`, no runtime selection, no tag or layout
    /// dispatch, no environment decode, and no body re-lowering: the target is
    /// a declared unit and the call is an ordinary direct call to it.
    ///
    /// The target comes from `worker_calls`, which was minted **into this
    /// function** (`D4`). Reading it here rather than from the binding is what
    /// keeps a `FuncRef` from ever crossing a function boundary.
    fn call_static_worker(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        worker: &StaticWorkerBinding,
        args: &[RuntimeExpr],
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // Explicit arguments in source order: argument `i` is child `1 + i` of
        // the `Call` occurrence, the callee being child `0`.
        //
        // ⛔ **This is the direct descent's argument phase and only its own.**
        // `D8e`'s source-machine consumer evaluates its arguments under the
        // machine's control and phase instead, and enters at
        // [`Self::call_static_worker_with_inputs`] below. The two share every
        // line after this point, which is the whole reason the split is here and
        // not lower: captures, the route's suffix, the route's table and the
        // emitted call are one assembly with one owner.
        let inputs = args
            .iter()
            .enumerate()
            .map(|(position, argument)| {
                let argument = self.child_occurrence(static_origin, 1 + position, argument)?;
                self.lower_expr(builder, argument, env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.call_static_worker_with_inputs(builder, worker, inputs, static_origin)
            // `D8j` — direct descent DISCARDS the emission handle. It is not a
            // composed consumption and has no causal obligation to answer for;
            // dropping the handle here is that statement, made where the
            // decision belongs.
            .map(|(operand, _)| operand)
    }

    /// **The route-selected static-worker emitter, from evaluated arguments
    /// onward.** Shared verbatim by the direct descent and by `D8e`'s
    /// source-machine consumer; neither reassembles any part of it.
    fn call_static_worker_with_inputs(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        worker: &StaticWorkerBinding,
        mut inputs: Vec<LoweringOperand>,
        static_origin: StaticOriginId,
    ) -> Result<(LoweringOperand, StaticWorkerEmission), CraneliftBackendError> {
        // ⛔ Arity is checked HERE, not in either caller. The explicit-argument
        // run is exactly `inputs` at entry -- captures are appended below -- so
        // this is the one place both consumers can be held to the declared
        // arity without either restating it.
        let supplied = u32::try_from(inputs.len()).map_err(|_| {
            unsupported("Call", "call argument count exceeds addressable range")
        })?;
        if supplied != worker.declared_arity {
            return Err(unsupported(
                "Call",
                format!(
                    "static worker expects {} arguments but call provides {supplied}",
                    worker.declared_arity
                ),
            ));
        }

        // Stored captures follow, unchanged. A carried capture stays carried:
        // there is deliberately no conversion on this edge, which is the whole
        // reason the binding holds operands rather than templates.
        inputs.extend(worker.captures.iter().cloned());

        // `RT-DECL-CLOSURE-PORT` `D5a` -- the generated-context capture suffix.
        //
        // ⭐ The prefix above is untouched, which is the point: the raw worker's
        // operand run is exactly what it always was, and a context call is that
        // run plus the enclosing specialization's continuation inputs. That is
        // why "keep raw fn2's ABI unchanged" and "carry the inputs across the
        // worker execution" are not in tension -- one is a prefix of the other.
        //
        // ⛔ Guarded on the exact body origin. A suffix appended to some other
        // worker call would be an arity error against a frame that might be
        // large enough to absorb it silently.
        // `D5a` checkpoint 4 step 3 -- the raw run, captured before the suffix
        // so the two are separable in the event log below. ⭐ "No suffix" and
        // "a suffix of length zero" are different facts and one total cannot
        // tell them apart.
        #[cfg(test)]
        let raw_operands = inputs.len();
        // ⭐ **`RT-CONTSRC-PRODUCER-LOCAL` `D6b` — THE ROUTE IS CONSUMED HERE,
        // and it decides BOTH halves of the call: the operand run and the
        // callee.** The two must be decided by one fact, because they are one
        // contract -- a generated context's ABI has a capture run the raw body
        // does not, so appending the suffix to a raw target, or omitting it from
        // a context target, is an arity disagreement against a frame that may be
        // large enough to absorb it silently.
        //
        // ⛔ The pre-`D6b` reading -- "append iff
        // `generated_context_captures.worker_body_origin == worker.body_origin`"
        // -- is *blind by construction* now that `D6a` binds two workers over
        // one body origin. It answers the same for the induction hypothesis and
        // for the selected recursive argument, which are exactly the two the
        // suffix must separate. The body-origin equality is retained below, but
        // demoted to a consistency check on the route's own choice.
        match worker.route {
            StaticWorkerCallRoute::GeneratedContext => {
                // The enclosing specialization's continuation inputs, carried
                // across this worker execution. Absent is a hard stop: a
                // context-routed call with no suffix to append means the
                // binding named a context this frame never stashed operands
                // for, and calling it would underflow the context's ABI.
                let extra = self
                    .function_local
                    .generated_context_captures
                    .as_ref()
                    .ok_or_else(|| {
                        unsupported(
                            "Call",
                            format!(
                                "a static worker binding for body origin {:?} is routed to a \
                                 generated context, but this frame stashed no continuation-input \
                                 suffix for any body",
                                worker.body_origin
                            ),
                        )
                    })?;
                if extra.worker_body_origin != worker.body_origin {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "a context-routed static worker binding names body origin {:?}, but \
                             the stashed continuation-input suffix belongs to body {:?}",
                            worker.body_origin, extra.worker_body_origin
                        ),
                    ));
                }
                // `D8g` — WITHHOLD THE SUFFIX, under test only. The sole
                // producer of the generated-context capture run is withheld and
                // the raw run and the call itself stay exact, so the difference
                // this isolates is the suffix and nothing else.
                #[cfg(test)]
                let withhold = crate::cranelift_backend::lowering::d8g_mutation()
                    == crate::cranelift_backend::lowering::D8gMutation::WithholdContextSuffix;
                #[cfg(not(test))]
                let withhold = false;
                if withhold {
                    #[cfg(test)]
                    crate::cranelift_backend::lowering::record_d8g_mutation_application();
                } else {
                    inputs.extend(extra.operands.iter().cloned());
                }
            }
            StaticWorkerCallRoute::RawWorker => {
                // ⛔ Appends NOTHING, unconditionally -- not even when this
                // frame does hold a suffix for this very body origin, which is
                // precisely the `D6a` case where the induction hypothesis
                // beside this binding is context-routed over the same body.
                // "No suffix" and "a suffix of length zero" stay different
                // facts, and this arm is the first.
            }
        }

        // Resolve the exact target FIRST. The mutation below perturbs only what
        // the consumer is handed, never what the binding named.
        //
        // `D6b` -- from the route's OWN table. In a retargeted specialization
        // `worker_calls[body]` is the generated context and the raw callee
        // survives only in `raw_worker_calls`; taking whichever entry exists is
        // the inference `D6a` made impossible.
        // `D8g` — WRONG TABLE, under test only. Each route reads the other's
        // table. The call key, the binding, the route field and the operand run
        // are all untouched; only which table answers moves, which is the one
        // producer input the table-choice relation is about.
        #[cfg(test)]
        let swap_tables = if crate::cranelift_backend::lowering::d8g_mutation()
            == crate::cranelift_backend::lowering::D8gMutation::WrongTable
        {
            crate::cranelift_backend::lowering::record_d8g_mutation_application();
            true
        } else {
            false
        };
        #[cfg(not(test))]
        let swap_tables = false;
        let (table, table_name) = match (worker.route, swap_tables) {
            (StaticWorkerCallRoute::GeneratedContext, false)
            | (StaticWorkerCallRoute::RawWorker, true) => {
                (&self.function_local.worker_calls, "worker_calls")
            }
            (StaticWorkerCallRoute::RawWorker, false)
            | (StaticWorkerCallRoute::GeneratedContext, true) => {
                (&self.function_local.raw_worker_calls, "raw_worker_calls")
            }
        };
        let exact = table
            .get(&worker.body_origin)
            .cloned()
            .ok_or_else(|| {
                unsupported(
                    "Call",
                    format!(
                        "no {table_name} target for body origin {:?} was declared into this \
                         function, so the {:?} route has no callee. ⛔ This never falls back to \
                         the other route's table: a raw call answered by a generated context \
                         would underflow that context's capture run, and a context call answered \
                         by the raw body would drop the continuation inputs the context exists \
                         to carry",
                        worker.body_origin, worker.route
                    ),
                )
            })?;
        // `D8g` — the emitted callee identity, captured where the table
        // answered. The FuncRef, not the origin: the two routes share a worker
        // body origin by design, so only this separates them.
        #[cfg(test)]
        let emitted_callee = {
            use cranelift_codegen::entity::EntityRef;
            exact.function.index() as u32
        };

        // `AC-5` clause (b): the redirect selects a **distinct** target by
        // `AC-6`'s definition of same-shape -- **same declared arity and same
        // capture count** -- and by nothing else.
        //
        // Selecting on `origin != body_origin` would establish only
        // difference. The target map is populated from every projected
        // emittable unit, so over a heterogeneous fixture that picks an
        // arbitrary unrelated unit and the red proves nothing about the origin
        // check. Requiring full header/slot/offset equality is the opposite
        // error: it over-constrains past the shape `AC-6` actually names.
        //
        // No candidate is a loud failure, never a fall back to exact: a silent
        // fallback would make this control vacuously green.
        // `D6b` -- the redirect searches the SAME table the exact resolution
        // used. Searching `worker_calls` while the exact answer came from
        // `raw_worker_calls` would make the mutation a route swap rather than
        // the same-shape redirect `AC-5` names, and the resulting red would be
        // attributable to the wrong mechanism.
        #[cfg(test)]
        let target = if STATIC_WORKER_MUTATION.with(std::cell::Cell::get)
            == StaticWorkerMutation::RedirectResolvedWorkerTarget
        {
            table
                .iter()
                .find(|(origin, call)| {
                    **origin != worker.body_origin
                        && call.header.parameters == exact.header.parameters
                        && call.header.captures == exact.header.captures
                })
                .map(|(_, call)| call.clone())
                .ok_or_else(|| {
                    unsupported(
                        "StaticWorkerMutation",
                        "the AC-5 redirect found no DISTINCT target of the same declared arity \
                         and capture count; it never falls back to exact, because a fallback \
                         would make this control vacuously green. Clause (c): run this switch \
                         on the two-same-shape-worker program only",
                    )
                })?
        } else {
            exact
        };
        #[cfg(not(test))]
        let target = exact;

        if target.origin != worker.body_origin {
            return Err(unsupported(
                "Call",
                format!(
                    "worker call target carries origin {:?} but the binding names body origin \
                     {:?}",
                    target.origin, worker.body_origin
                ),
            ));
        }

        let emitted = self.call_declared_unit_target(
            builder,
            target,
            &inputs,
            #[cfg(test)]
            None,
        )?;
        // `D5a` -- the emission half of the marker's ordered log. ⛔ Recorded
        // AFTER the instruction exists, so "consumed before emitted" is a fact
        // about the log's order rather than about where the line was written.
        #[cfg(test)]
        record_d5a_marker_event(D5aMarkerEvent::WorkerCallEmitted {
            body_origin: worker.body_origin,
            raw_operands,
            supplied_operands: inputs.len(),
            route: worker.route,
        });
        // `D8g` — the same instant, recorded with the facts a keyed relation
        // needs: which body is emitting, which call occurrence, the decoded raw
        // callee, its declared contract, the route that chose the table, the run
        // the instruction carried, and whether the binding answers for a
        // composed causal obligation.
        //
        // ⛔ This is the ONE emitter the functionized and composed populations
        // share. Recording here is what lets `D8g` relate two different programs
        // at one seat rather than compare two logs that merely look alike.
        #[cfg(test)]
        crate::cranelift_backend::lowering::record_d8g_emission(
            crate::cranelift_backend::lowering::D8gEmission {
                function: self.defining_function_id,
                call_origin: static_origin,
                target_body_origin: worker.body_origin,
                declared_arity: worker.declared_arity,
                captures: worker.captures.len(),
                route: worker.route,
                raw_operands,
                supplied_operands: inputs.len(),
                composed_discharge: worker.composed_continuation_authority().is_ok(),
                emitted_callee,
            },
        );
        // `D8j` — the instruction is HANDED BACK, not recorded. Which consumer
        // may answer for a causal obligation with it is the caller's question,
        // and both this emitter's callers reach here.
        // ⭐ The operand run is reported from the vector that was just written,
        // read adjacent to the call that consumed it. ⛔ Not recomputed from the
        // binding's declared arity and captures: that is the quantity
        // verification 4b compares this against, and deriving both from one
        // source would make the comparison an identity.
        let emission = StaticWorkerEmission {
            inst: emitted.1,
            supplied_operands: inputs.len(),
        };
        // `D8j` verification 4b's discriminator, applied AFTER the real vector
        // was assembled and emitted. ⛔ It moves the EVIDENCE about the run, not
        // the run: the call that was written is unchanged, and every other input
        // to the verifier stays exact.
        #[cfg(test)]
        let emission = if d8j_mutation() == D8jMutation::SupplyOperandCountDisagreesWithTarget {
            StaticWorkerEmission {
                supplied_operands: emission.supplied_operands.saturating_add(1),
                ..emission
            }
        } else {
            emission
        };
        Ok((emitted.0, emission))
    }

    /// **`D2` -- the binder-lowering helper.** Lowers a `Let`'s bound value
    /// into the one binding authority.
    ///
    /// The default is exactly what it was before this node:
    /// `Value(lower_expr(..))`. The single new outcome is a lexical closure
    /// **any** of whose captures is carried, which installs a
    /// [`LoweringEnvironmentBinding::StaticWorker`] instead -- that case
    /// previously failed closed in `specialized_operands_at`, which is the
    /// narrowing this node removes.
    ///
    /// All captures are classified **exhaustively before** the outcome is
    /// selected. The selection is never made from syntax spelling, and never
    /// from a partially emitted `specialized_at` failure: the captures are
    /// lowered in full first, then counted.
    ///
    /// `RuntimeExpr::Closure` keeps its own arm rather than sharing this one.
    /// Its captures are seed-provenance symbols resolved to JIT-time ground
    /// values, not lexical children, so it has no carried capture to find and
    /// projecting it as one would conflate the two provenances the ABI keeps
    /// apart.
    fn lower_binder(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringEnvironmentBinding, CraneliftBackendError> {
        let SourceOccurrence {
            expr,
            static_origin,
        } = occurrence;
        let RuntimeExpr::LexicalClosure {
            captures,
            params,
            body,
        } = expr
        else {
            // Every other binder, `RuntimeExpr::Closure` included, keeps the
            // pre-existing route unchanged.
            return self
                .lower_expr(builder, occurrence, env)
                .map(LoweringEnvironmentBinding::Value);
        };
        if !matches!(
            self.body_emission_authority,
            BodyEmissionAuthority::FunctionizedUnits
        ) {
            return self
                .lower_expr(builder, occurrence, env)
                .map(LoweringEnvironmentBinding::Value);
        }

        // Positional projection from this occurrence: body is exact child 0,
        // lexical capture `i` is exact child `1 + i`. The declaration order
        // (`captures, params, body`) is NOT the child order.
        let body = self.child_occurrence(static_origin, 0, body)?;
        let lowered_captures = captures
            .iter()
            .enumerate()
            .map(|(position, capture)| {
                let capture = self.child_occurrence(static_origin, 1 + position, capture)?;
                self.lower_expr(builder, capture, env)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Exhaustive classification, completed before anything is selected.
        let mut carried = 0usize;
        let mut specialized = 0usize;
        for capture in &lowered_captures {
            match capture {
                LoweringOperand::Carried(_) => carried += 1,
                LoweringOperand::Specialized(_) => specialized += 1,
            }
        }
        debug_assert_eq!(carried + specialized, lowered_captures.len());

        // `AC-5` mutation 1 lives on this exact branch: with the narrowing
        // restored, a carried capture takes the specialized-only fold again
        // and the ordinary witness must go red.
        #[cfg(test)]
        let narrowing_restored = STATIC_WORKER_MUTATION.with(std::cell::Cell::get)
            == StaticWorkerMutation::RestoreCarriedCaptureNarrowing;
        #[cfg(not(test))]
        let narrowing_restored = false;
        if carried == 0 || narrowing_restored {
            // All-specialized: preserve the existing compile-time closure.
            //
            // ⭐ **The narrowing STAYS on this branch, and deleting it would
            // make `AC-5` mutation 1 vacuous.** `captures` is now phase-bearing,
            // so storing the operands unchanged would compile — and under
            // `RestoreCarriedCaptureNarrowing` a carried capture would then sail
            // through the very fold the mutation exists to force it into,
            // turning the control into the identity. On the exact path
            // `carried == 0`, so this is a total unwrap-and-reassert; under the
            // mutation it is the pre-node refusal, unchanged.
            return Ok(LoweringEnvironmentBinding::Value(
                LoweringOperand::Specialized(Lowered::Closure {
                    captures: specialized_operands_at(&lowered_captures, "a closure capture")?
                        .into_iter()
                        .map(LoweringOperand::Specialized)
                        .collect(),
                    params: params.clone(),
                    body: body.static_origin,
                }),
            ));
        }

        let declared_arity = u32::try_from(params.len()).map_err(|_| {
            unsupported(
                "StaticWorkerBinding",
                "source closure parameter count exceeds addressable range",
            )
        })?;
        self.construct_static_worker_binding(
            static_origin,
            body.static_origin,
            declared_arity,
            captures.len(),
            lowered_captures,
            // `D6a` — an ordinary lexical closure binder. There is no enclosing
            // specialization here, so the planner issued no generated context
            // for this occurrence and the raw body is the only callee it can
            // ever have named.
            StaticWorkerCallRoute::RawWorker,
            // `D8i` — a lexical-closure capsule answers for no composed causal
            // obligation. Stated, not defaulted.
            ContinuationDischarge::DirectSpecializationCall,
        )
        .map(LoweringEnvironmentBinding::StaticWorker)
    }

    /// **`D7` -- THE PRE-EMISSION CAPTURE-CONTRACT GATE for a retained callable
    /// whose environment is mixed-phase.**
    ///
    /// ⭐⭐ **Membership in `worker_templates` is necessary and INSUFFICIENT, and
    /// that is the whole reason this exists.** The measured stop that opened this
    /// checkpoint had a template for its body origin and still could not be
    /// represented; "the planner knows this closure" answers *whether* a contract
    /// was issued, never *what it says*. A capsule that closes over a carried
    /// word commits the callee to reading that word out of an activation-frame
    /// slot at an exact ordinal, so every one of those facts is checked **before
    /// any function definition or object emission**, not discovered when the
    /// callee loads the wrong offset.
    ///
    /// ⛔ **No policy is invented here.** The expected capture slot is projected
    /// from [`expected_capture_slot`], the same authority that laid the
    /// descriptor, so carrier, ownership, storage owner, width, alignment and
    /// ordinal are compared against the planner's own answer rather than against
    /// a second copy of the rule that could drift from it.
    ///
    /// The one fact this layer adds is **phase admissibility**, which the ABI
    /// plane cannot state because it has no operands: a `Carried` capture is an
    /// invocation-time SSA word, so it is lawful only in a slot the **activation
    /// frame** owns. `ArtifactStatic` material is minted before execution begins
    /// -- a seed capture's lane -- and no word this activation computes can be
    /// that, so a carried capture in a seed slot is refused rather than stored.
    fn validate_retained_callable_capture_contract(
        &self,
        closure_origin: StaticOriginId,
        body_origin: StaticOriginId,
        provenance: AbiCaptureProvenance,
        declared_arity: usize,
        captures: &[LoweringOperand],
    ) -> Result<(), CraneliftBackendError> {
        // 1. Exactly one planner-issued template for this exact body.
        //
        // ⭐ Uniqueness is upstream and structural: `worker_templates` is keyed
        // by body origin, and the population walk that fills it already refuses
        // when "two emittable units claim the same body origin". So the only
        // failure this lookup can still see is OMISSION -- which is the half a
        // keyed map cannot make unrepresentable.
        let target = self
            .function_local
            .worker_templates
            .get(&body_origin)
            .ok_or_else(|| {
                unsupported(
                    "RetainedCallableCaptureContract",
                    format!(
                        "a mixed-phase retained callable at {closure_origin:?} has no planner-issued \
                         worker template for body origin {body_origin:?} in this function"
                    ),
                )
            })?;

        // 2. The record is keyed by the callable SOURCE BODY, so a disagreement
        //    here is a wrong-body contract rather than a lookup miss.
        if target.call_site_origin != body_origin {
            return Err(unsupported(
                "RetainedCallableCaptureContract",
                format!(
                    "the worker template reached for body origin {body_origin:?} is keyed by \
                     source body {:?}",
                    target.call_site_origin
                ),
            ));
        }

        // 3. Declared arity and capture count, against the descriptor header.
        if target.header.parameters as usize != declared_arity {
            return Err(unsupported(
                "RetainedCallableCaptureContract",
                format!(
                    "worker descriptor declares {} parameters but the retained callable declares \
                     {declared_arity}",
                    target.header.parameters
                ),
            ));
        }
        if target.header.captures as usize != captures.len() {
            return Err(unsupported(
                "RetainedCallableCaptureContract",
                format!(
                    "worker descriptor declares {} captures but {} were projected from the \
                     retained definition",
                    target.header.captures,
                    captures.len()
                ),
            ));
        }

        // 4. The ORDERED capture run, taken in slot order. Its length must agree
        //    with the header independently -- the header and the slot run are two
        //    recorded facts, and a gate that trusted one to speak for the other
        //    would be blind to exactly the descriptor it exists to reject.
        let capture_slots = target
            .slots
            .iter()
            .filter(|slot| slot.kind == AbiSlotKind::Capture)
            .collect::<Vec<_>>();
        if capture_slots.len() != captures.len() {
            return Err(unsupported(
                "RetainedCallableCaptureContract",
                format!(
                    "worker descriptor's slot run declares {} capture slots against a header of \
                     {} and {} projected captures",
                    capture_slots.len(),
                    target.header.captures,
                    captures.len()
                ),
            ));
        }

        for (position, (slot, capture)) in capture_slots.iter().zip(captures).enumerate() {
            let ordinal = u32::try_from(position).map_err(|_| {
                unsupported(
                    "RetainedCallableCaptureContract",
                    "retained callable capture count exceeds addressable range",
                )
            })?;
            // 5. Phase admissibility, then ordinal/provenance/owner/lifetime in
            //    ONE comparison
            //    against the planner's own projection. ⛔ Comparing field by
            //    field here would let a field added to `AbiSlot` later go
            //    unchecked; whole-slot equality cannot.
            //
            //    Ordinal density falls out of this rather than needing its own
            //    pass: slot *i* of the capture run must carry ordinal *i*, so a
            //    duplicated, permuted or gapped ordinal fails here.
            //    ⚠ **The phase check runs FIRST, and the order is load-bearing
            //    rather than stylistic.** Behind the whole-slot equality below
            //    it would be DEAD for the only provenance production calls this
            //    with: equality already forces a lexical capture's slot to be
            //    `ValueWord`, whose storage owner is the activation frame, so no
            //    descriptor could ever reach the phase arm. Asking whether the
            //    refusal could be provoked at all is what surfaced that -- a
            //    guard nothing can trip is not a guard.
            match capture {
                // A compile-time template is lawful in any capture slot: it is
                // read where the descriptor says, whatever owns that storage.
                LoweringOperand::Specialized(_) => {}
                // ⛔ An invocation-time word cannot inhabit storage minted
                // before execution began, nor the persistent store.
                LoweringOperand::Carried(_) => {
                    if slot.storage_owner != AbiStorageOwner::ActivationFrame {
                        return Err(unsupported(
                            "RetainedCallableCaptureContract",
                            format!(
                                "capture {position} of the retained callable at \
                                 {closure_origin:?} arrived carried, but its slot's storage is \
                                 owned by {:?} -- an invocation-time word cannot inhabit storage \
                                 that outlives the activation that computes it",
                                slot.storage_owner
                            ),
                        ));
                    }
                }
            }
            let expected = expected_capture_slot(provenance, ordinal);
            if **slot != expected {
                return Err(unsupported(
                    "RetainedCallableCaptureContract",
                    format!(
                        "capture {position} of the retained callable at {closure_origin:?} \
                         declares slot {slot:?} but its {provenance:?} provenance projects \
                         {expected:?}"
                    ),
                ));
            }
        }
        Ok(())
    }

    /// **`D2` -- THE SOLE CONSTRUCTION ROUTE for a static worker binding.**
    ///
    /// The projection is positional and exact, taken from the closure's own
    /// retained occurrence: the closure origin is the occurrence origin, the
    /// body origin is exact child `0`, and lexical capture `i` is exact child
    /// `1 + i`. Declared arity is the source parameter count.
    ///
    /// There is no source re-walk and no global closure search here -- the
    /// caller has already resolved the occurrence and lowered the captures
    /// through `child_occurrence`, and this routine only validates and
    /// installs.
    ///
    /// Every one of the checks below runs **before** the binding is returned,
    /// so a missing, duplicate, wrong-body, wrong-arity or wrong-capture fact
    /// rejects before any worker call could be emitted. `captures` arrive as
    /// `LoweringOperand` and are stored unchanged: a carried capture stays
    /// carried, and nothing here converts a phase.
    ///
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D6a`** — `route` is supplied by the
    /// caller, which is the only party that knows the **role** the binding is
    /// being built for. Nothing here derives it: every check below is against
    /// the raw template contract and is identical on both routes, which is
    /// exactly why the route cannot be recovered from anything this function
    /// validates. See [`StaticWorkerCallRoute`].
    pub(super) fn construct_static_worker_binding(
        &self,
        closure_origin: StaticOriginId,
        body_origin: StaticOriginId,
        declared_arity: u32,
        source_capture_count: usize,
        captures: Vec<LoweringOperand>,
        route: StaticWorkerCallRoute,
        // `D8i` — REQUIRED, so omission is a compile error rather than a silent
        // `DirectSpecializationCall`. ⛔ Not an `Option`, not defaulted, and not
        // inferred from `route`: the two facets are independent, and a caller
        // that has not decided which causal obligation its binding may answer
        // for has not finished building it.
        discharge: ContinuationDischarge,
    ) -> Result<StaticWorkerBinding, CraneliftBackendError> {
        // 1. The capture vector agrees with the retained definition. The caller
        //    builds it from the retained children, so a disagreement means the
        //    two walks diverged rather than that the source is odd.
        if captures.len() != source_capture_count {
            return Err(unsupported(
                "StaticWorkerBinding",
                format!(
                    "retained definition declares {source_capture_count} lexical captures but \
                     {} were projected",
                    captures.len()
                ),
            ));
        }

        // 2. This function has the RAW TEMPLATE contract for `body_origin`.
        //
        // `D5a` checkpoint 1: this reads `worker_templates`, not `unit_calls`.
        // The template is the raw worker's own descriptor and carries no
        // `FuncRef`, so every check below is about the raw contract even when
        // the call this binding eventually drives has been retargeted to a
        // generated context. ⛔ Reading the call target here instead would
        // validate the context against the raw closure's arity and either
        // reject a lawful retarget or, worse, silently accept a wrong one.
        let target = self
            .function_local
            .worker_templates
            .get(&body_origin)
            .ok_or_else(|| {
                unsupported(
                    "StaticWorkerBinding",
                    format!(
                        "no raw worker template for body origin {body_origin:?} in this function"
                    ),
                )
            })?;

        // 3. The declared unit call is the one for this exact body origin. The
        //    map is keyed by origin, so a disagreement here means the entry was
        //    built for another body -- a wrong-body fact, not a lookup miss.
        // `RT-CONTSPEC-ACTIVATE` `D1b` — the PAIRED invariant, replacing the
        // invalid `target.origin == body_origin` step.
        //
        // The declared record is keyed by the callable SOURCE BODY and retains
        // the scheduling entry as its target origin. Comparing the target
        // origin to the body origin was checking the wrong end of the pair;
        // both ends are now checked against what each actually names.
        if target.call_site_origin != body_origin {
            return Err(unsupported(
                "StaticWorkerBinding",
                format!(
                    "declared unit call is keyed by source body {:?} but the worker body origin \
                     is {body_origin:?}",
                    target.call_site_origin
                ),
            ));
        }
        // The target origin is the scheduling entry by construction of the
        // declared record. It is NOT required to differ from the source body:
        // for the root adapter and for single-occurrence units the two ends
        // legitimately coincide, and demanding distinctness here was my own
        // over-strictness rather than the ruled invariant.

        // 4. The descriptor's parameter count is the declared arity.
        if target.header.parameters != declared_arity {
            return Err(unsupported(
                "StaticWorkerBinding",
                format!(
                    "worker descriptor declares {} parameters but the source closure declares \
                     {declared_arity}",
                    target.header.parameters
                ),
            ));
        }

        // 5. The descriptor's capture count is the capture vector's length.
        let descriptor_captures = usize::try_from(target.header.captures).map_err(|_| {
            unsupported(
                "StaticWorkerBinding",
                "worker descriptor capture count exceeds addressable range",
            )
        })?;
        if descriptor_captures != captures.len() {
            return Err(unsupported(
                "StaticWorkerBinding",
                format!(
                    "worker descriptor declares {descriptor_captures} captures but {} were \
                     projected from the retained definition",
                    captures.len()
                ),
            ));
        }

        // 6. Slots, offsets and frame bytes are taken from that descriptor
        //    unchanged, so they must agree with it before installation. The
        //    slot run is the authority for the counts, and `frame_bytes` is
        //    derived from it -- a slot run that disagrees with the header is a
        //    descriptor this binding must not carry.
        if target.slots.len() != target.offsets.len() {
            return Err(unsupported(
                "StaticWorkerBinding",
                format!(
                    "worker descriptor has {} slots but {} offsets",
                    target.slots.len(),
                    target.offsets.len()
                ),
            ));
        }
        let slot_parameters = target
            .slots
            .iter()
            .filter(|slot| matches!(slot.kind, AbiSlotKind::Parameter))
            .count();
        let slot_captures = target
            .slots
            .iter()
            .filter(|slot| matches!(slot.kind, AbiSlotKind::Capture))
            .count();
        if slot_parameters != declared_arity as usize || slot_captures != descriptor_captures {
            return Err(unsupported(
                "StaticWorkerBinding",
                format!(
                    "worker descriptor slot run declares {slot_parameters} parameters and \
                     {slot_captures} captures, disagreeing with its header's {declared_arity} \
                     and {descriptor_captures}"
                ),
            ));
        }
        if target.header.frame_bytes == 0 && !target.slots.is_empty() {
            return Err(unsupported(
                "StaticWorkerBinding",
                "worker descriptor declares a non-empty slot run in a zero-byte frame",
            ));
        }

        // ⭐ `D7` — the ORDERED capture contract, on the route that already
        // owned the count. Checks 1-6 above validate the descriptor's shape;
        // this validates the capture RUN against it, so the binder route and the
        // value route are gated by one function rather than two spellings that
        // can drift.
        self.validate_retained_callable_capture_contract(
            closure_origin,
            body_origin,
            AbiCaptureProvenance::Lexical,
            declared_arity as usize,
            &captures,
        )?;
        // `D8i` — the defect switch, applied to the SUPPLIED facet and only to
        // an ordinary one. ⛔ It substitutes a real, planner-issued authority
        // taken from the target population, searched for an emission owner that
        // is not this unit's. Nothing here fabricates an identity, because
        // nothing outside planning can.
        #[cfg(test)]
        let discharge = if crate::cranelift_backend::lowering::d8i_foreign_authority()
            && matches!(discharge, ContinuationDischarge::DirectSpecializationCall)
        {
            let mut foreign = None;
            for target in self.static_transition_plan.composed_call_targets()? {
                if Some(target.call_identity().emission_owner()) != self.defining_emission_owner {
                    foreign = Some(target.call_identity().clone());
                    break;
                }
            }
            match foreign {
                Some(identity) => ContinuationDischarge::ComposedSourceContinuation(identity),
                None => discharge,
            }
        } else {
            discharge
        };

        // ⭐⭐ `D8i` — THE AUTHORITY GUARD, and it is about the OWNER.
        //
        // A composed authority names the emission owner its own causal call
        // belongs to; `D8h` held the target and the token to that agreement at
        // minting. This binding is being built inside one defining emission
        // owner's pass. If the two differ, the binding would transport an
        // obligation belonging to a function that is not the one emitting it,
        // and a later discharge would answer for a call this frame cannot make.
        //
        // ⛔ Refused, not corrected: there is no lawful repair, because the
        // right authority for this owner may not exist at all. And checked HERE
        // rather than at consumption -- a binding that never should have
        // carried the authority must not exist to be consumed, and `D8j` must
        // not have to re-litigate provenance it was handed.
        if let ContinuationDischarge::ComposedSourceContinuation(identity) = &discharge {
            if Some(identity.emission_owner()) != self.defining_emission_owner {
                return Err(unsupported(
                    "StaticWorkerBinding",
                    format!(
                        "a composed causal authority names emission owner {:?}, but this binding \
                         is being constructed under {:?}; a binding cannot transport an \
                         obligation that belongs to a different emitter",
                        identity.emission_owner(),
                        self.defining_emission_owner
                    ),
                ));
            }
        }

        // The observation, written AFTER every validation, from the facet the
        // call site supplied.
        #[cfg(test)]
        crate::cranelift_backend::lowering::record_d8i_discharge(
            crate::cranelift_backend::lowering::D8iDischargeRecord {
                body_origin,
                composed: match &discharge {
                    ContinuationDischarge::DirectSpecializationCall => None,
                    ContinuationDischarge::ComposedSourceContinuation(identity) => {
                        Some((identity.emission_owner(), identity.target()))
                    }
                },
            },
        );
        Ok(StaticWorkerBinding {
            closure_origin,
            body_origin,
            declared_arity,
            captures,
            route,
            discharge,
        })
    }

    pub(super) fn retained_body_occurrence(
        &self,
        static_origin: StaticOriginId,
    ) -> Result<SourceOccurrence<'a>, CraneliftBackendError> {
        // ⭐ `AC-4`'s behavioural half. This route and
        // `StaticTransitionPlan::source_occurrence` are counted separately, and
        // the claim is that they move **together**: a resolution performed
        // without passing through here is the second route `AC-4` forbids, and
        // it shows up as `resolutions > invocations`.
        #[cfg(test)]
        crate::cranelift_backend::planning::ac4_note_route_invocation();
        Ok(SourceOccurrence {
            expr: self
                .static_transition_plan
                .source_occurrence(static_origin)?,
            static_origin,
        })
    }

    /// The source machine's **owned working copy** of a retained body.
    ///
    /// ⚠ The machine's pending frames own their terms (`OwnedSourceOccurrence`)
    /// and must keep doing so — this is the population boundary of B2A-S, and it
    /// is forced rather than chosen. `lower_source_forked_match` hands the machine
    /// a **synthesized** `RuntimeExpr::Trap` that exists nowhere in the source
    /// tree and therefore has no planned occurrence to be resolved from; a frame
    /// that could only hold a borrowed view of a planned term could not represent
    /// it. So the frames stay owned, and this is where a tag becomes one.
    ///
    /// ⛔ That is **not** a surviving retained-body carrier. The distinction is
    /// which value is authoritative: a `Lowered::Closure` names its body by origin
    /// and holds no term, and this copy is made *at the point of use* from that
    /// name. Re-lowering the resolved term per call site is symptom-inventory
    /// entry 2, which `RT-FNSPLIT-B2F` owns and this WP does not claim.
    fn machine_body_occurrence(
        &self,
        static_origin: StaticOriginId,
    ) -> Result<OwnedSourceOccurrence, CraneliftBackendError> {
        Ok(OwnedSourceOccurrence::cloned(
            self.retained_body_occurrence(static_origin)?,
        ))
    }

    /// Derives one **positional** child occurrence of `parent`.
    ///
    /// This is the lowering's sole route to a child's static name. `position` is
    /// the child's source-field ordinal in the planner's own child order (see the
    /// table on `lower_expr`), and the value comes out of B1R's checked
    /// positional child-origin range. There is deliberately no other route: not
    /// pointer identity, not the term's content or hash, not clone order, not
    /// visit order, and no arithmetic that mints an origin
    /// for it.
    fn child_occurrence<'x>(
        &self,
        parent: StaticOriginId,
        position: usize,
        child: &'x RuntimeExpr,
    ) -> Result<SourceOccurrence<'x>, CraneliftBackendError> {
        Ok(SourceOccurrence {
            expr: child,
            static_origin: self
                .static_transition_plan
                .child_static_origin(parent, position)?,
        })
    }

    /// The owned form of `child_occurrence`, for the source machine's pending
    /// frames: it takes the child term **by value** and pairs it with its origin
    /// in one constructor, so no step of the machine can hold a term whose origin
    /// was dropped was dropped.
    fn owned_child_occurrence(
        &self,
        parent: StaticOriginId,
        position: usize,
        child: RuntimeExpr,
    ) -> Result<OwnedSourceOccurrence, CraneliftBackendError> {
        Ok(OwnedSourceOccurrence {
            expr: child,
            static_origin: self
                .static_transition_plan
                .child_static_origin(parent, position)?,
        })
    }

    /// The owned form of `case_body_occurrence`, for the source machine.
    fn owned_case_body_occurrence(
        &self,
        parent: StaticOriginId,
        index: usize,
        body: RuntimeExpr,
    ) -> Result<OwnedSourceOccurrence, CraneliftBackendError> {
        self.owned_child_occurrence(parent, 1 + index, body)
    }

    /// Derives the occurrence of case *index*'s body under a match occurrence.
    ///
    /// Both match variants lay their children out as `[scrutinee, case 0 body,
    /// case 1 body, …]`, so a case body is child `1 + index`. Cases are the one
    /// place the lowering reaches a body by *searching* (by constructor name),
    /// and a search recovers no position — so every such site enumerates to
    /// recover the index rather than deriving identity from the match it found.
    pub(super) fn case_body_occurrence<'x>(
        &self,
        parent: StaticOriginId,
        index: usize,
        body: &'x RuntimeExpr,
    ) -> Result<SourceOccurrence<'x>, CraneliftBackendError> {
        self.child_occurrence(parent, 1 + index, body)
    }

    /// ⭐ **The dual of [`LoweringOperand::specialized_join_arm`]** — a join
    /// whose single lane is the **carrier word**, not a native scalar pair.
    ///
    /// ⚠⚠ **Read the two together: they refuse in OPPOSITE directions, and the
    /// asymmetry is the point.** `specialized_join_arm` guards a join that has
    /// no carried lane, so its `Carried` arm fails closed. This guards a join
    /// that has *only* a carried lane, so a **specialized** arm must cross into
    /// it. ⛔ Neither is a `Carried -> Lowered` conversion; this one moves
    /// `Lowered -> CarriedBoundaryWord`, which is precisely the direction `§2g`
    /// rules as the producer's one-way seam.
    ///
    /// ⭐ **Why a carried match's join has one lane and it is this one.** An arm
    /// of a carried `Match` may return a **projected child**, which `§2g`
    /// requires to stay `Carried` and which has no compile-time template to
    /// re-specialize. So the merge cannot be a `Lowered` join, and every arm
    /// must arrive as a carrier word.
    ///
    /// ⚠ **The producer's coverage is partial and this inherits that
    /// deliberately.** An arm whose value is a form `transfer_into_carrier`
    /// defers — a spillable `Int`, a `String`, borrowed ingress — fails closed
    /// with **the producer's own message**, ⛔ never a second refusal invented
    /// here. ⇒ The carried match's arm coverage widens exactly when the
    /// producer's does, with no list to keep in sync and no second authority to
    /// let drift.
    fn carried_join_arm(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        lowered: LoweringOperand,
        required_kind: Option<ScalarMergeKind>,
        join: &'static str,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        match lowered {
            LoweringOperand::Carried(word) => {
                #[cfg(test)]
                D8_CARRIED_JOIN_UNCHANGED.with(|count| count.set(count.get() + 1));
                Ok(word)
            }
            // ── ⛔ DEFERRED, said plainly ──────────────────────────────────
            //
            // ⚠ A deferral is honest; a deferral that reads as delivery is not.
            // A compile-time trap arm must **not** reach the merge at all — it
            // returns instead (`seal_source_trap_branch`) — so the merge block
            // would have fewer predecessors than the case chain has arms. That
            // is a control-flow shape this route does not build yet, and
            // refusing is strictly better than emitting a half-formed merge.
            LoweringOperand::Specialized(Lowered::Trap(trap)) => Err(unsupported(
                "BoundaryCarrier",
                format!(
                    "{join} resolves at compile time to a trap ({}), and the carried join \
                     does not yet build a merge with a trapping predecessor",
                    trap.message
                ),
            )),
            LoweringOperand::Specialized(lowered) => {
                #[cfg(test)]
                D8_SPECIALIZED_JOIN_PRODUCTIONS.with(|count| count.set(count.get() + 1));
                let terminal_exit = self.process_object
                    && (required_kind == Some(ScalarMergeKind::ExitCode)
                        || self
                            .function_local
                            .terminal_result_origins
                            .contains(&origin))
                    && matches!(
                        &lowered,
                        Lowered::Constructor { constructor, .. }
                            if constructor == &self.process_symbols.exit_success
                                || constructor == &self.process_symbols.exit_failure
                    );
                if terminal_exit {
                    let status = self.emit_process_exit_status(builder, lowered);
                    self.emit_carrier_immediate(builder, BoundaryTag::ImmediateExitStatus, status)
                } else {
                    self.transfer_into_carrier(builder, origin, &lowered)
                }
            }
        }
    }

    /// Give one already-planned join exactly the lanes named by its D8 token.
    fn append_planned_join_params(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
    ) {
        self.function_local
            .materialized_join_blocks
            .entry(join_plan.origin)
            .or_default()
            .insert(merge);
        builder.append_block_param(merge, types::I64);
        if join_plan.representation == JoinResultRepresentation::NativeScalarPair {
            builder.append_block_param(merge, types::I64);
        }
    }

    /// Send one continuing predecessor through the representation selected
    /// before CFG emission. Source traps are sealed by the caller and never
    /// reach this value-only operation.
    fn jump_planned_join_arm(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
        origin: StaticOriginId,
        lowered: LoweringOperand,
        merge_kind: &mut Option<ScalarMergeKind>,
        join: &'static str,
    ) -> Result<(), CraneliftBackendError> {
        match join_plan.representation {
            JoinResultRepresentation::NativeScalarPair => {
                if matches!(lowered, LoweringOperand::Carried(_)) {
                    let planned = self.retained_body_occurrence(join_plan.origin)?;
                    return Err(backend_module(format!(
                        "{join} source join {:?} ({:?}) planned native scalar lanes but \
                         lowering produced a carried boundary word",
                        join_plan.origin, planned.expr
                    )));
                }
                let (value, kind) = self.merge_scalar_branch(builder, join_plan, lowered, join)?;
                Self::record_scalar_merge_kind(join, merge_kind, kind)?;
                builder
                    .ins()
                    .jump(merge, &[value.tag.into(), value.payload.into()]);
            }
            JoinResultRepresentation::CarrierWord => {
                let word = self.carried_join_arm(builder, origin, lowered, None, join)?;
                builder.ins().jump(merge, &[word.word.into()]);
            }
        }
        Ok(())
    }

    /// Recover the typed result of a planned join after all continuing
    /// predecessors have been emitted.
    fn finish_planned_join(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
        merge_kind: Option<ScalarMergeKind>,
        join: &'static str,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        builder.switch_to_block(merge);
        match join_plan.representation {
            JoinResultRepresentation::NativeScalarPair => {
                let pair = NativeScalarPairV1 {
                    tag: builder.block_params(merge)[0],
                    payload: builder.block_params(merge)[1],
                };
                let kind = merge_kind.ok_or_else(|| {
                    backend_module(format!(
                        "{join} has a continuing native predecessor without a result kind"
                    ))
                })?;
                Ok(LoweringOperand::Specialized(
                    self.lowered_from_scalar_pair(kind, pair),
                ))
            }
            JoinResultRepresentation::CarrierWord => {
                Ok(LoweringOperand::Carried(CarriedBoundaryWord {
                    word: builder.block_params(merge)[0],
                }))
            }
        }
    }

    /// Build one source constructor directly in the boundary carrier when at
    /// least one child has already crossed a generated-unit edge.
    ///
    /// The constructor identity and child origins still come exclusively from
    /// the static plan.  A carried child is stored unchanged; a specialized
    /// sibling crosses through the sole producer before both are joined in the
    /// same runtime node.
    fn transfer_constructor_operands(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        constructor: &str,
        args: &[LoweringOperand],
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        if constructor == self.process_symbols.exit_failure {
            if let [LoweringOperand::Carried(code)] = args {
                return self.transfer_carried_failure_exit_status(builder, *code);
            }
        }
        let identity = self
            .static_transition_plan
            .constructor_symbol_identity(origin)?
            .tag_abi_word()?;
        // ⛔ **This was an unconditional `PersistentGround`, and it is the
        // defect `D7`'s aggregate subclosure exists to remove.** Every carried
        // aggregate was allocated persistent regardless of its children, so a
        // constructor over an invocation-owned child became a persistent parent
        // naming storage that dies first — the dangling relation `store_field`
        // refuses, surfacing at runtime as `BOUNDARY_ERR_ESCAPE` rather than at
        // the producer that created it.
        let (occurrence, class) = self.aggregate_carrier_authority(
            origin,
            &Lowered::Constructor {
                constructor: RuntimeSymbol::from(constructor),
                synthesized_identity: None,
                occurrence: None,
                args: Vec::new(),
            },
            PlannedAggregateShape::Constructor,
        )?;
        let word = self.emit_checked_aggregate_alloc(
            builder,
            GovernedAllocationSite::CarriedConstructor,
            occurrence,
            PlannedAggregateShape::Constructor,
            class,
            args.len(),
        )?;
        self.emit_carrier_store_tag_id(builder, word, identity)?;
        for (position, argument) in args.iter().enumerate() {
            let child_origin = self
                .static_transition_plan
                .child_static_origin(origin, position)?;
            let child = match argument {
                LoweringOperand::Carried(child) => *child,
                LoweringOperand::Specialized(value) => {
                    self.transfer_into_carrier(builder, child_origin, value)?
                }
            };
            self.emit_carrier_store_field(builder, word, position, child)?;
        }
        Ok(word)
    }

    /// Preserve the established process-exit mapping when the failure code
    /// crosses a unit edge before its enclosing constructor is lowered.
    ///
    /// Every valid native exit code is inside the immediate-Int domain. A
    /// non-immediate Int is therefore invalid without decoding its magnitude;
    /// the immediate arm reads the scalar through the carrier ABI and applies
    /// the same `0 -> 1`, `1..=255 -> self`, otherwise `-3` mapping used by
    /// `emit_process_exit_status`.
    fn transfer_carried_failure_exit_status(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        code: CarriedBoundaryWord,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let tag = builder
            .ins()
            .band_imm(code.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
        let immediate = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            tag,
            BoundaryTag::ImmediateInt as i64,
        );
        let immediate_block = builder.create_block();
        let invalid_block = builder.create_block();
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder
            .ins()
            .brif(immediate, immediate_block, &[], invalid_block, &[]);

        builder.switch_to_block(immediate_block);
        let value = self.emit_carrier_scalar(builder, code)?;
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        let malformed = builder.ins().iconst(types::I64, -3);
        let positive = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            value,
            zero,
        );
        let within_max = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            value,
            255,
        );
        let valid = builder.ins().band(positive, within_max);
        let nonzero = builder.ins().select(valid, value, malformed);
        let is_zero =
            builder
                .ins()
                .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, value, zero);
        let status = builder.ins().select(is_zero, one, nonzero);
        builder.ins().jump(merge, &[status.into()]);

        builder.switch_to_block(invalid_block);
        let malformed = builder.ins().iconst(types::I64, -3);
        builder.ins().jump(merge, &[malformed.into()]);

        builder.switch_to_block(merge);
        self.emit_carrier_immediate(
            builder,
            BoundaryTag::ImmediateExitStatus,
            builder.block_params(merge)[0],
        )
    }

    /// ⭐⭐ **`D3` — `Match` eliminating a value that has NO compile-time
    /// template.** This is the second, *executable* route the whole node exists
    /// to build.
    ///
    /// The specialized route answers *"which constructor?"* by reading a
    /// `Lowered::Constructor`'s own `constructor` field while compiling. Here
    /// there is no such value and no such field — only a boundary word — so
    /// **every** question becomes a call into the emitted carrier ABI:
    ///
    /// | question | specialized route | this route |
    /// |---|---|---|
    /// | which Result carrier representation? | compile-time `Lowered` variant | `class(word)` |
    /// | which constructor? | `case.constructor == constructor` | `tag(word)` vs `case_constructor_identity` |
    /// | how many children? | `args.len()` | `field_count(word)` |
    /// | child *i*? | `args[i]` | `field(word, i)` — ⭐ **stays `Carried`** |
    /// | nothing matched? | a compile-time `Lowered::Trap` | a **runtime** closed default |
    ///
    /// ⭐ **Both columns read ONE identity authority** (`D2`). The producer
    /// wrote either `constructor_symbol_identity(..)` for a source occurrence
    /// or `synthesized_constructor_identity(..)` for a closed compiler role;
    /// this compares against `case_constructor_identity(..).tag_abi_word()`.
    /// Equal spellings intern to one canonical span, so the two agree **because
    /// they are the same number**, not because two derivations happen to
    /// coincide.
    /// ⛔ There is no decode step and no reverse table: the comparison is word
    /// against word, ⛔ never word against a reconstructed name.
    ///
    /// ⚠ **This changes no production behaviour today.** Nothing in production
    /// emits a `Carried` scrutinee (`AC-C10` — zero `B2F` activation), so this
    /// route is reached only by a test that seeds one. Stated here so the
    /// reachability is not overclaimed by a later reader.
    fn lower_carried_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        if cases.is_empty() {
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        }
        // Only the closed process-input constructor family can arrive through
        // the borrowed-opaque lane.  Do not materialize that branch for an
        // ordinary carried match: Cranelift must compile both successors, so a
        // runtime class test alone would incorrectly require borrowed
        // identities for unrelated source constructors.
        let admits_borrowed_input = self.process_object
            && cases.iter().all(|case| {
                borrowed_constructor_identity(&self.process_symbols, &case.constructor).is_some()
            });
        if !admits_borrowed_input {
            return self.lower_nonborrowed_carried_match(
                builder,
                scrutinee,
                cases,
                default,
                static_origin,
                env,
                &join_plan,
            );
        }
        let class = self.emit_carrier_class(builder, scrutinee)?;
        let is_borrowed = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            class,
            BoundaryClass::BorrowedOpaque as i64,
        );
        let borrowed = builder.create_block();
        let represented = builder.create_block();
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            self.append_planned_join_params(builder, merge, &join_plan);
        }
        let mut merge_kind = None;
        builder
            .ins()
            .brif(is_borrowed, borrowed, &[], represented, &[]);

        builder.switch_to_block(borrowed);
        let pointer = self.emit_carrier_scalar(builder, scrutinee)?;
        let borrowed_result = self.lower_borrowed_match(
            builder,
            pointer,
            cases,
            default,
            static_origin,
            env,
            &join_plan,
        )?;
        if self.seal_source_trap_branch(builder, &borrowed_result)? {
            // This runtime representation has no continuing predecessor.
        } else {
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                static_origin,
                borrowed_result,
                &mut merge_kind,
                "a carried borrowed-input match",
            )?;
        }

        builder.switch_to_block(represented);
        let represented_result = self.lower_nonborrowed_carried_match(
            builder,
            scrutinee,
            cases,
            default,
            static_origin,
            env,
            &join_plan,
        )?;
        if self.seal_source_trap_branch(builder, &represented_result)? {
            // This runtime representation has no continuing predecessor.
        } else {
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                static_origin,
                represented_result,
                &mut merge_kind,
                "a carried represented-value match",
            )?;
        }

        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        };
        self.finish_planned_join(
            builder,
            merge,
            &join_plan,
            merge_kind,
            "a carried representation split",
        )
    }

    fn lower_nonborrowed_carried_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        join_plan: &JoinPlanToken,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⭐ Handled before any block is created, and that ordering matters: a
        // case-free match reaches the default unconditionally, so building a
        // merge block for it would leave one with no predecessor.
        if cases.is_empty() {
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        }
        let ok_case = cases
            .iter()
            .enumerate()
            .find(|(_, case)| case.constructor == self.process_symbols.result_ok);
        let err_case = cases
            .iter()
            .enumerate()
            .find(|(_, case)| case.constructor == self.process_symbols.result_err);
        if ok_case.is_some() || err_case.is_some() {
            let (Some(ok_case), Some(err_case)) = (ok_case, err_case) else {
                return Err(unsupported(
                    "HostResult",
                    "a carried HostResult match requires both closed Result cases",
                ));
            };
            if ok_case.1.binders != 1 || err_case.1.binders != 1 {
                return Err(unsupported(
                    "HostResult",
                    "carried Result cases must each bind exactly one selected payload",
                ));
            }
            // Dispatch both carried representations into one pair of source
            // case blocks. A nested source join is therefore emitted exactly
            // once even though either representation can select its owner.
            let ok_body = builder.create_block();
            builder.append_block_param(ok_body, types::I64);
            let err_body = builder.create_block();
            builder.append_block_param(err_body, types::I64);
            let merge = join_plan
                .has_continuing_predecessor
                .then(|| builder.create_block());
            if let Some(merge) = merge {
                self.append_planned_join_params(builder, merge, join_plan);
            }
            let mut merge_kind = None;

            let class = self.emit_carrier_class(builder, scrutinee)?;
            let is_host_result = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                class,
                BoundaryClass::HostResult as i64,
            );
            let host_result = builder.create_block();
            let constructor = builder.create_block();
            builder
                .ins()
                .brif(is_host_result, host_result, &[], constructor, &[]);

            builder.switch_to_block(host_result);
            let success = self.emit_carrier_host_success(builder, scrutinee)?;
            let payload = self.emit_carrier_host_payload(builder, scrutinee)?;
            builder.ins().brif(
                success,
                ok_body,
                &[payload.word.into()],
                err_body,
                &[payload.word.into()],
            );

            builder.switch_to_block(constructor);
            let tag = self.emit_carrier_tag(builder, scrutinee)?;
            let field_count = self.emit_carrier_field_count(builder, scrutinee)?;
            for (body_block, (index, _case)) in [(ok_body, ok_case), (err_body, err_case)] {
                let identity = self
                    .static_transition_plan
                    .case_constructor_identity(static_origin, index)?
                    .tag_abi_word()?;
                let identity = Self::carrier_identity_immediate(builder, identity);
                let selected = builder.create_block();
                let next = builder.create_block();
                let matched = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    tag,
                    identity,
                );
                builder.ins().brif(matched, selected, &[], next, &[]);
                builder.switch_to_block(selected);
                Self::require_i64(builder, field_count, 1);
                let payload = self.emit_carrier_field(builder, scrutinee, 0)?;
                builder.ins().jump(body_block, &[payload.word.into()]);
                builder.switch_to_block(next);
            }
            let defaulted = LoweringOperand::Specialized(Lowered::Trap(default.clone()));
            if !self.seal_source_trap_branch(builder, &defaulted)? {
                return Err(unsupported(
                    "Match",
                    "the carried Result match's closed default did not seal its branch",
                ));
            }

            for (block, (index, case)) in [(ok_body, ok_case), (err_body, err_case)] {
                builder.switch_to_block(block);
                let payload = CarriedBoundaryWord {
                    word: builder.block_params(block)[0],
                };
                let case_env = env_with_operands([LoweringOperand::Carried(payload)], env);
                let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                let body_origin = body.static_origin;
                let lowered = self.lower_expr(builder, body, &case_env)?;
                if self.seal_source_trap_branch(builder, &lowered)? {
                    continue;
                }
                let merge = merge.ok_or_else(|| {
                    backend_module(
                        "join plan omitted a merge despite a continuing predecessor".to_string(),
                    )
                })?;
                self.jump_planned_join_arm(
                    builder,
                    merge,
                    join_plan,
                    body_origin,
                    lowered,
                    &mut merge_kind,
                    "a carried Result arm",
                )?;
            }

            let Some(merge) = merge else {
                let unreachable_continuation = builder.create_block();
                builder.switch_to_block(unreachable_continuation);
                return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
            };
            return self.finish_planned_join(
                builder,
                merge,
                join_plan,
                merge_kind,
                "a carried Result join",
            );
        }

        self.lower_carried_constructor_match(
            builder,
            scrutinee,
            cases,
            default,
            static_origin,
            env,
            join_plan,
        )
    }

    fn lower_carried_constructor_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        join_plan: &JoinPlanToken,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // Read identity and arity ONCE, ahead of the chain: both are properties
        // of the scrutinee, not of any case, and re-reading per case would be a
        // second answer to a question that has one.
        let tag = self.emit_carrier_tag(builder, scrutinee)?;
        let field_count = self.emit_carrier_field_count(builder, scrutinee)?;

        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            self.append_planned_join_params(builder, merge, join_plan);
        }
        let mut merge_kind = None;

        for (index, case) in cases.iter().enumerate() {
            // ⭐ `D1` — the case's identity, keyed on this `Match` occurrence's
            // origin and the case's ordinal. ⚠ `case.constructor`, the
            // **string**, is deliberately not the key: keying on the spelling
            // would be the second derivation `D2` forbids.
            let identity = self
                .static_transition_plan
                .case_constructor_identity(static_origin, index)?
                .tag_abi_word()?;
            let identity = Self::carrier_identity_immediate(builder, identity);
            let selected = builder.create_block();
            let next = builder.create_block();
            let matched = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                identity,
            );
            builder.ins().brif(matched, selected, &[], next, &[]);

            builder.switch_to_block(selected);
            // ⚠ **The arity check the specialized route performs while
            // compiling has to be EMITTED here**, because neither operand is
            // known until the value exists. It is a real guard, not ceremony:
            // binding *n* binders over a node with fewer children would read
            // past the node. A mismatch means the producer's `field_count` and
            // the elaborator's binder count disagree — corruption, not an input
            // condition — so it takes the same failure status as every other
            // carrier ABI violation.
            let binders = i64::try_from(case.binders).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "a case binds more binders than the carrier ABI can count",
                )
            })?;
            Self::require_i64(builder, field_count, binders);

            // ⭐ `§2g`: *"projected children remain `Carried`."* Each binder is
            // a runtime projection, and it enters `case_env` **in the carried
            // phase** — which is the exact clause `§2h`'s control demands.
            let mut bindings = Vec::with_capacity(case.binders);
            for position in 0..case.binders {
                bindings.push(LoweringOperand::Carried(
                    self.emit_carrier_field(builder, scrutinee, position)?,
                ));
            }
            let case_env = env_with_operands(bindings, env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let body_origin = body.static_origin;
            let lowered = self.lower_expr(builder, body, &case_env)?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                builder.switch_to_block(next);
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                join_plan,
                body_origin,
                lowered,
                &mut merge_kind,
                "a carried `Match` arm",
            )?;

            builder.switch_to_block(next);
        }

        // ── ⛔ THE CLOSED DEFAULT — `AC-C3`'s negative arm ─────────────────
        //
        // ⭐ Routed through the existing [`Self::seal_source_trap_branch`]
        // rather than spelling the trap encoding a second time: if the encoding
        // ever changes, both move together. A constructor outside the
        // artifact-static case set lands here, at runtime, and returns.
        let defaulted = LoweringOperand::Specialized(Lowered::Trap(default.clone()));
        if !self.seal_source_trap_branch(builder, &defaulted)? {
            return Err(unsupported(
                "Match",
                "the carried match's closed default did not seal its branch",
            ));
        }

        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        };
        self.finish_planned_join(
            builder,
            merge,
            join_plan,
            merge_kind,
            "a carried `Match` join",
        )
    }

    /// Emit the declared call that evaluates one computational recursive
    /// position on the functionized path.
    ///
    /// Keeping this as a distinct operation makes the S1 boundary mechanical:
    /// a recursive position cannot accidentally return to source-body
    /// re-lowering without bypassing the one operation that emits its call.
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D8d` — the one target-derived environment
    /// binding, at the selected recursive source-order position.**
    ///
    /// `None` for every nonrecursive position, and for a recursive position the
    /// planner issued no [`ComposedCallTarget`] for. ⛔ That second `None` is
    /// the **ordinary non-specialized path**, not a gap: it is the same shape
    /// `continuation_call_binding_for` already uses, where no binding means the
    /// producer keeps its existing route untouched. The entire pre-`D5a`
    /// population lives there.
    ///
    /// ⛔ The selector is the exact `D8a` five-field one, and every field is
    /// supplied from a fact this seat already holds: the defining unit's
    /// emission owner, the deferred constructor's own occurrence, the active
    /// computational frame's origin, the selected case index, and this position.
    /// Nothing is derived from the lowered value's shape, from arity, or from
    /// which of the two targets exists.
    ///
    /// ## What comes from where, and why it is split that way
    ///
    /// | fact | source |
    /// |---|---|
    /// | closure occurrence, raw body, declared arity, capture **count** | the planner-issued target |
    /// | capture **operands** | the lowered closure at this position |
    ///
    /// ⭐ That split is the established `D6a` idiom, not a compromise. Identity
    /// and shape are planner facts and lowering may not re-derive them; the
    /// operands are runtime values that only this frame holds, and the planner
    /// carries provenance for them rather than the values themselves. The
    /// constructor then re-checks the two against each other, so a divergence
    /// refuses rather than silently binding a short capture run.
    ///
    /// ⛔ The route is [`StaticWorkerCallRoute::RawWorker`], fixed by `D6a`'s
    /// law that a selected recursive argument takes the raw worker
    /// unconditionally. It is **not selected** here — nothing reads route
    /// eligibility to decide it, which would be the route-selected emission this
    /// checkpoint excludes.
    ///
    /// Fails closed when the planner names a target and the lowered value at
    /// that position is not a closure: the planner's provenance says it is one,
    /// so a disagreement is two authorities differing about the same source
    /// position, and binding either answer would pick a winner.
    fn composed_recursive_argument_binding(
        &self,
        case: &crate::RuntimeComputationalMatchCase,
        construct_origin: StaticOriginId,
        frame_origin: StaticOriginId,
        alternative: usize,
        position: usize,
        lowered: &Lowered,
    ) -> Result<Option<StaticWorkerBinding>, CraneliftBackendError> {
        if !case.recursive_positions.contains(&position) {
            return Ok(None);
        }
        // No defining owner means no unit-definition pass is open, so there is
        // no owner to key the selector on. The producer keeps its existing
        // route, as it does when the claim ledger is absent.
        #[cfg(test)]
        d8d_record_site();
        let Some(emission_owner) = self.defining_emission_owner else {
            return Ok(None);
        };
        let alternative = u32::try_from(alternative).map_err(|_| {
            unsupported("ComputationalMatch", "case index exceeds addressable range")
        })?;
        let recursive_position = u32::try_from(position).map_err(|_| {
            unsupported(
                "ComputationalMatch",
                "recursive position exceeds addressable range",
            )
        })?;

        let selector = (
            emission_owner,
            construct_origin,
            frame_origin,
            alternative,
            recursive_position,
        );
        let Some(target) = self
            .static_transition_plan
            .composed_call_targets()?
            .into_iter()
            .find(|target| target.selector() == selector)
        else {
            return Ok(None);
        };

        let Lowered::Closure { captures, .. } = lowered else {
            return Err(unsupported(
                "ComputationalMatch",
                format!(
                    "the planner issued a composed-call target at recursive position \
                     {position}, whose provenance names a closure occurrence, but this frame \
                     lowered {} there; two authorities disagree about one source position and \
                     binding either would be choosing between them",
                    lowered_value_kind(lowered)
                ),
            ));
        };

        #[cfg(test)]
        d8d_record_binding();
        let worker = target.worker();
        self.construct_static_worker_binding(
            worker.closure_origin(),
            worker.body_origin(),
            worker.declared_arity(),
            worker.captures().len(),
            captures.clone(),
            StaticWorkerCallRoute::RawWorker,
            // ⭐ `D8i` — the transported authority, taken from `D8h`'s pairing
            // on this exact target and carried unchanged. ⛔ Not resolved again
            // here: the target already carries the identity its own five-field
            // coordinate selects, and a second lookup would be a second
            // authority for one fact.
            ContinuationDischarge::ComposedSourceContinuation(target.call_identity().clone()),
        )
        .map(Some)
    }

    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D8j` — CLAIM one composed discharge.**
    ///
    /// Reached from `D8e`'s two source-machine static-worker completions and
    /// from nowhere else. By the time it runs, the raw-worker call has been
    /// emitted by the shared route-selected emitter and its result is in hand
    /// under the control the seat was entered with.
    ///
    /// ⛔ **Installing the binding, beginning the argument run, or seeing a
    /// worker-shaped value is not discharge, and none of those reaches here.**
    /// The two callers are the completions, after the emitter.
    ///
    /// ⭐ This performs verifications 1 and 2 — the ones whose evidence exists
    /// now — and records the rest for the finished CLIF. Verifications 3, 4 and
    /// 5 need the instruction stream, which does not exist until the function
    /// is finalized; claiming them here would mean asserting facts about
    /// instructions nobody has read.
    fn claim_composed_discharge(
        &mut self,
        worker: &StaticWorkerBinding,
        emission: StaticWorkerEmission,
        result: &LoweringOperand,
        source_control_before: usize,
    ) -> Result<(), CraneliftBackendError> {
        // Whether a composed obligation is owed at all is the BINDING's own
        // statement, read from the facet `D8i` transports.
        //
        // ⛔ The test switch below forces the attempt on an ordinary binding.
        // That population is unreachable in production -- an ordinary binding
        // owes nothing and this arm is not entered for one -- but it is
        // reachable by a defect, which is what separates this guard from the
        // owner-collision refusal `D8b` deleted: that one's population the
        // planner proved impossible, and no switch could instantiate it.
        // ⛔ Under the switch the binding itself is replaced by an ORDINARY
        // clone of the very binding that was about to discharge -- same
        // closure, body, arity, captures and route, direct facet. That is what
        // makes the refusal below attributable to the facet and to nothing
        // else. ⭐ The composed arm cannot be built here even to undo it: it
        // needs a planner-issued identity.
        #[cfg(test)]
        let ordinary_clone;
        #[cfg(test)]
        let worker = if d8j_mutation() == D8jMutation::DischargeFromOrdinaryBinding {
            ordinary_clone = StaticWorkerBinding {
                discharge: ContinuationDischarge::DirectSpecializationCall,
                ..worker.clone()
            };
            &ordinary_clone
        } else {
            worker
        };
        #[cfg(test)]
        let attempt = matches!(
            worker.discharge,
            ContinuationDischarge::ComposedSourceContinuation(_)
        ) || d8j_mutation() == D8jMutation::DischargeFromOrdinaryBinding;
        #[cfg(not(test))]
        let attempt = matches!(
            worker.discharge,
            ContinuationDischarge::ComposedSourceContinuation(_)
        );
        if !attempt {
            return Ok(());
        }
        // ⭐ The authority is obtained THROUGH the accessor, which is what makes
        // an ordinary binding refuse here rather than silently discharge
        // nothing.
        let identity = worker.composed_continuation_authority()?.clone();
        // `D8o` correction 4 — the EMITTER BODY, recorded at the real composed
        // claim seam from the live ambient fields plus the Function being
        // defined. ⛔ Never inferred from `identity.emission_owner()`: that is
        // the field the owner guard below validates, so reading it here would
        // make the population question answer itself.
        #[cfg(test)]
        crate::cranelift_backend::lowering::record_d8o_composed_claim_body(
            self.defining_function_id,
            self.defining_emission_owner,
        );

        // `D8j` verification 1 -- the identity came from the exact paired
        // planner target.
        //
        // ⛔ Resolved by finding the target whose OWN pairing is this identity,
        // then holding that target's worker provenance against the binding.
        // Searching by body origin or by the binding's shape would be the
        // reconstruction `D8h` forbids, and would also answer for the wrong
        // layer wherever two layers share a body.
        let targets = self.static_transition_plan.composed_call_targets()?;
        #[cfg(test)]
        let identity = match d8j_mutation() {
            // A different EXACT identity, taken from the population -- the
            // shape a same-symbol shortcut would produce. ⛔ Not fabricated:
            // nothing outside planning can build one.
            D8jMutation::SubstituteAnotherExactIdentity => targets
                .iter()
                .map(|target| target.call_identity().clone())
                .find(|candidate| *candidate != identity)
                .unwrap_or(identity),
            _ => identity,
        };
        let mut paired = None;
        for target in &targets {
            if *target.call_identity() == identity {
                if paired.is_some() {
                    return Err(unsupported(
                        "ContinuationDischarge",
                        "two composed-call targets carry one causal identity, so a discharge \
                         cannot say which coordinate it answers for",
                    ));
                }
                paired = Some(target);
            }
        }
        let paired = paired.ok_or_else(|| {
            unsupported(
                "ContinuationDischarge",
                "a composed discharge presents a causal identity no composed-call target is \
                 paired with, so the authority did not come from the planner target this \
                 consumption is standing in for",
            )
        })?;
        let declared_operands = paired
            .worker()
            .captures()
            .len()
            .checked_add(paired.worker().declared_arity() as usize)
            .ok_or_else(|| {
                unsupported("ContinuationDischarge", "declared operand run exceeds range")
            })?;
        if paired.worker().body_origin() != worker.body_origin
            || paired.worker().declared_arity() != worker.declared_arity
            || paired.worker().captures().len() != worker.captures.len()
        {
            return Err(unsupported(
                "ContinuationDischarge",
                format!(
                    "a composed discharge's paired target names worker body {:?} with arity {} \
                     and {} captures, but the binding being consumed names {:?} with arity {} \
                     and {} captures; the authority and the callee come from different targets",
                    paired.worker().body_origin(),
                    paired.worker().declared_arity(),
                    paired.worker().captures().len(),
                    worker.body_origin,
                    worker.declared_arity,
                    worker.captures.len()
                ),
            ));
        }

        // `D8j` verification 2 -- the CLAIMING function is the identity's own
        // emission owner.
        //
        // ⛔ Independent of `D8i`'s construction-time guard, and deliberately
        // re-derived: that one asked whether the binding could be BUILT here,
        // this asks whether this function may ANSWER with it. A binding can
        // legitimately be constructed in one pass and, were it ever to travel,
        // consumed in another.
        #[cfg(test)]
        let claiming = match d8j_mutation() {
            D8jMutation::WrongClaimingOwner => None,
            _ => self.defining_emission_owner,
        };
        #[cfg(not(test))]
        let claiming = self.defining_emission_owner;
        if claiming != Some(identity.emission_owner()) {
            return Err(unsupported(
                "ContinuationDischarge",
                format!(
                    "a composed discharge claims a causal call owned by {:?}, but the function \
                     making the claim is {claiming:?}; only the emitting owner may answer for \
                     its own causal call",
                    identity.emission_owner()
                ),
            ));
        }

        // Recorded for the finished CLIF. ⛔ Suppressible under test so that
        // "the call was emitted and nothing was discharged" is a state the row
        // can distinguish from a correct run.
        #[cfg(test)]
        if d8j_mutation() == D8jMutation::SuppressDischargeAfterRealCall {
            return Ok(());
        }
        let inst = emission.inst;
        self.function_local
            .pending_composed_discharges
            .push(PendingComposedDischarge {
                identity,
                inst,
                worker_body_origin: worker.body_origin,
                declared_operands,
                supplied_operands: emission.supplied_operands,
                result: match result {
                    LoweringOperand::Carried(word) => Some(word.word),
                    LoweringOperand::Specialized(_) => None,
                },
                source_control: (source_control_before, self.live_source_continuations),
            });
        Ok(())
    }

    fn call_declared_recursive_position_unit(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        body_origin: StaticOriginId,
        inputs: &[LoweringOperand],
        coordinates: Option<CarriedInvocationCoordinates>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // `RT-DECL-CLOSURE-PORT` `D5a` checkpoint 4 step 1 — THE CARRIED
        // INVOCATION BINDING.
        //
        // The binding is resolved from the invocation's retained source
        // coordinates through planner authority. ⛔ Lowering supplies only the
        // coordinates it already holds; it does not reconstruct the binding
        // from `body_origin`, from the callee's ABI shape, from the existence
        // of a context, or by taking a first match — the planner rejects an
        // ambiguous resolution rather than choosing.
        // `D5a` checkpoint 4 step 3 -- perturb the RETAINED SOURCE COORDINATES
        // the invocation presents, so the lookup is asked for a coordinate the
        // planner never issued. ⛔ The lookup is untouched: what moves is the
        // key, which is the thing the ruling says the binding must be resolved
        // by.
        #[cfg(test)]
        let coordinates = coordinates.map(|mut coordinates| {
            if d5a_route_mutation() == D5aRouteMutation::PerturbCarriedInvocationCoordinates {
                record_d5a_route_application();
                coordinates.recursive_position = coordinates.recursive_position.wrapping_add(1);
            }
            coordinates
        });
        let context = match coordinates {
            Some(coordinates) => self.static_transition_plan.carried_invocation_context(
                coordinates.continuation_origin,
                coordinates.recursive_position,
                body_origin,
            )?,
            None => None,
        };
        // ⛔ FAIL CLOSED, over BOTH ways of arriving at "no context".
        //
        // A site that cannot supply the coordinates cannot be *asked* whether
        // it should retarget; a site whose coordinates resolve to nothing was
        // asked and got no answer. Either way, emitting the raw call is a miss
        // that looks exactly like a lawful ordinary call. It is only safe when
        // no context exists for this body at all.
        //
        // ⭐⭐ The second half is new. This test used to guard the missing-
        // coordinates arm ALONE, so perturbed-but-present coordinates fell
        // straight through to the raw target. On this witness that still
        // refused — but only incidentally, because the superseded body has no
        // `Function` left to call. Had it remained executable, the retarget
        // would have been dropped in silence. ⇒ The guard belongs to the
        // outcome, not to one of the two routes into it.
        if context.is_none()
            && self
                .static_transition_plan
                .continuation_contexts()?
                .iter()
                .any(|context| context.worker_body_origin() == body_origin)
        {
            return Err(unsupported(
                "ContinuationSpecialization",
                format!(
                    "a carried recursive-position invocation of body {body_origin:?} resolved no \
                     generated execution context, and that body has one; emitting the raw target \
                     here would drop the retarget silently rather than refuse it. Retained source \
                     coordinates presented: {:?}",
                    coordinates.map(|c| (c.continuation_origin, c.recursive_position)),
                ),
            ));
        }
        #[cfg(test)]
        d5a_trace(format!(
            "  CARRIED-INVOCATION body={body_origin:?} coords={:?} -> {}",
            coordinates.map(|c| (c.continuation_origin, c.recursive_position)),
            match context {
                Some(context) => format!("context {context:?}"),
                None => "raw target".to_string(),
            }
        ));
        let result = match context {
            Some(context) => self.call_declared_context(builder, context, body_origin, inputs)?,
            None => self.call_declared_unit(
                builder,
                body_origin,
                inputs,
                #[cfg(test)]
                None,
            )?,
        };
        #[cfg(test)]
        RECURSIVE_POSITION_UNIT_CALLS.with(|calls| calls.set(calls.get() + 1));
        Ok(result)
    }

    /// Emit the one exact retargeted callee for a carried invocation.
    ///
    /// ⭐ Only the **callee** moves. The call is the same already-planned
    /// emitted call, at the same site, with the same operand prefix and the
    /// same causal ancestry; its source edge, its predecessor and its
    /// provenance are untouched. That is what makes this a retarget rather than
    /// a deletion.
    fn call_declared_context(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        context: ContinuationContextId,
        body_origin: StaticOriginId,
        inputs: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let target = self
            .function_local
            .context_calls
            .get(&context)
            .cloned()
            .ok_or_else(|| {
                unsupported(
                    "ContinuationSpecialization",
                    format!(
                        "the generated execution context bound to body {body_origin:?} was not                          declared into this function"
                    ),
                )
            })?;
        // `D5a` checkpoint 4 step 1b — THE EXACT CAPTURE SUFFIX.
        //
        // The context declares a frame of `parameters + captures`; the carried
        // invocation supplies the parameter run, so retargeting its callee
        // without appending the captures is a call that does not match the
        // frame it now names. ⛔ Appended in the context's DECLARED ORDER, and
        // taken only from the immediate slots the planner assigned -- nothing
        // is reconstructed from the raw worker, chosen by shape, or routed
        // through a runtime transport.
        let view = self
            .static_transition_plan
            .continuation_contexts()?
            .into_iter()
            .find(|candidate| candidate.id() == context)
            .ok_or_else(|| {
                unsupported(
                    "ContinuationSpecialization",
                    "the bound generated context has no projected view",
                )
            })?;
        let defining_owner = self.defining_emission_owner.ok_or_else(|| {
            unsupported(
                "ContinuationSpecialization",
                "a carried invocation retarget was reached with no emission owner bound for the                  context currently being defined",
            )
        })?;
        let mut inputs = inputs.to_vec();
        for capture in view.captures()? {
            // `RT-CONTSRC-PRODUCER-LOCAL` `D1` — present a producer-local
            // coordinate to this seam, so its refusal is measured rather than
            // merely written. ⛔ Applied BEFORE the domain match, because the
            // question is what the match does with a domain it cannot locate.
            #[cfg(test)]
            let capture = {
                let mut capture = capture;
                if d5a_route_mutation() == D5aRouteMutation::PresentProducerLocalCoordinate {
                    record_d5a_route_application();
                    capture.coordinate = ContinuationSourceCoordinate::producer_local_probe();
                }
                capture
            };
            // `RT-CONTSRC-PRODUCER-LOCAL` `D1` `D3` consumer 3 of 3, context
            // half. ⛔ Exhaustive over the coordinate domains with no default,
            // for the same reason as the specialization seam: this reads an ABI
            // operand run, which a producer-local value is not in.
            // `RT-CONTSRC-PRODUCER-LOCAL` `D3b` — the same pairing resolver as
            // the emission seam, told what THIS seam is holding.
            //
            // ⛔ `ContinuationImmediateSeat::AbiOperandRun` is not a weaker
            // `Emission`: this seam holds an ABI operand run and no semantic
            // environment at all, so a current-lexical nearest-alias index has
            // nothing here to index and is refused rather than read as an ABI
            // position. The generated-context capture arm IS resolvable here,
            // because a capture slot is a position in exactly this run.
            let immediate_slot = self.resolve_context_capture_claim(
                capture.coordinate,
                capture.availability,
                defining_owner,
            )?;
            // `RT-CONTSRC-PRODUCER-LOCAL` `D2b` — the availability domain, matched
            // exhaustively with no wildcard exactly as the coordinate domain is
            // above. `D3` teaches this seam the two producer-local arms; until
            // then it must not index `defining_abi_operands` — an ABI operand run
            // — with a lexical environment index.
            let operand = self
                .function_local
                .defining_abi_operands
                .get(immediate_slot as usize)
                .ok_or_else(|| {
                    unsupported(
                        "ContinuationSpecialization",
                        format!(
                            "a generated context capture names immediate slot {} outside the                              emitting function's {} ABI operands; note this is the IMMEDIATE                              slot, whose meaning is fixed by the availability domain {:?}",
                            immediate_slot,
                            self.function_local.defining_abi_operands.len(),
                            capture.availability,
                        ),
                    )
                })?
                .clone();
            inputs.push(operand);
        }
        self.call_declared_unit_target(
            builder,
            target,
            &inputs,
            #[cfg(test)]
            None,
        )
        .map(|(operand, _inst)| operand)
    }

    /// Resolve the declared body unit of a callable recursive position in the
    /// source form that owns the carried child.
    ///
    /// Structural-data recursive positions return `None`; they resume the
    /// eliminator directly and take no arguments. A lexical closure with
    /// captures also returns `None` because its carried value does not expose
    /// those capture operands to a generated call frame.
    fn recursive_position_unit_body(
        &self,
        eliminator_origin: StaticOriginId,
        position: usize,
    ) -> Result<Option<StaticOriginId>, CraneliftBackendError> {
        let eliminator = self.retained_body_occurrence(eliminator_origin)?;
        let RuntimeExpr::ComputationalMatch { scrutinee, .. } = eliminator.expr else {
            return Err(backend_module(
                "recursive-position metadata names a non-computational eliminator".to_string(),
            ));
        };
        let scrutinee = self.child_occurrence(eliminator_origin, 0, scrutinee)?;
        let RuntimeExpr::Construct { args, .. } = scrutinee.expr else {
            return Ok(None);
        };
        let Some(argument) = args.get(position) else {
            return Err(backend_module(
                "recursive position is outside its source constructor".to_string(),
            ));
        };
        let argument = self.child_occurrence(scrutinee.static_origin, position, argument)?;
        match argument.expr {
            RuntimeExpr::LexicalClosure { captures, body, .. } if captures.is_empty() => Ok(Some(
                self.child_occurrence(argument.static_origin, 0, body)?
                    .static_origin,
            )),
            RuntimeExpr::Closure { body, .. } => Ok(Some(
                self.child_occurrence(argument.static_origin, 0, body)?
                    .static_origin,
            )),
            RuntimeExpr::Value(_)
            | RuntimeExpr::Var(_)
            | RuntimeExpr::Let { .. }
            | RuntimeExpr::If { .. }
            | RuntimeExpr::PrimitiveCall { .. }
            | RuntimeExpr::Construct { .. }
            | RuntimeExpr::Match { .. }
            | RuntimeExpr::ComputationalMatch { .. }
            | RuntimeExpr::Record { .. }
            | RuntimeExpr::Project { .. }
            | RuntimeExpr::LexicalClosure { .. }
            | RuntimeExpr::DeclarationRef { .. }
            | RuntimeExpr::ImportedDeclarationRef { .. }
            | RuntimeExpr::Call { .. }
            | RuntimeExpr::Effect { .. }
            | RuntimeExpr::Trap(_)
            | RuntimeExpr::CheckedJoinSite { .. }
            | RuntimeExpr::CheckedSubcontinuationFrame { .. }
            | RuntimeExpr::CheckedRecursiveInvocation { .. }
            | RuntimeExpr::CheckedComputationalIHSlots { .. }
            | RuntimeExpr::CheckedComputationalIHInvocation { .. } => Ok(None),
        }
    }

    /// ⭐⭐ **`D3` — `ComputationalMatch` eliminating a carried value.**
    ///
    /// Structurally the same three runtime questions as
    /// [`Self::lower_carried_match`] — identity, arity, positional child — over
    /// the same one authority. The differences are the computational frame's:
    /// the arity compared is `argument_binders`, the frame contributes its own
    /// environment, and a case may declare **recursive positions**.
    ///
    /// ## ⭐⭐ Recursive positions are BUILT here — `AC-C4`, on the Architect's
    /// ## single-field license
    ///
    /// A recursive position builds an *induction hypothesis* over the child at
    /// that position. Over a carried scrutinee that child is a **carried word**,
    /// so the IH's residual must hold one — which
    /// [`Lowered::ComputationalRecursorClosure::residual`] now does, as a
    /// `Box<LoweringOperand>`.
    ///
    /// ⚠ **This function previously refused recursive positions and named the
    /// fork for the Architect. That refusal was SCAFFOLD, and the ruling
    /// rejected the branch it was holding open** — *"the recursive-position
    /// refusal is not an acceptable `C1` residual."* It is recorded here rather
    /// than deleted silently, because the arm and a shipped boundary are
    /// textually the same thing and only the prose says which one this is.
    ///
    /// ⭐ **The metadata is MINTED exactly as the specialized composed path
    /// mints it, and ⛔ none of it is derived from the carried word.** Static
    /// origin, checked-frame id, IH slot templates, activation, cursor and
    /// producer origin all come from `eliminator` and the compiler's own
    /// counters — the carried word contributes the *value* and nothing else.
    /// That separation is the ruling's clause 5, and it is what control 3
    /// perturbs.
    fn lower_carried_computational_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        eliminator: ComputationalEliminatorFrame<'_>,
        remaining_eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⛔⛔ TERMINATION — refused BEFORE any block is created, so a hang can
        // never be half-emitted. See
        // `Lowering::active_carried_computational_eliminations` for why inlining
        // a carried recursion cannot terminate.
        //
        // ⚠ **This bounds INVOKING an induction hypothesis, ⛔ not declaring a
        // recursive position.** A case with `recursive_positions` still mints
        // its IH over the carried child, still puts it in `case_env`, and still
        // eliminates — everything below runs. Only re-entering *this same*
        // eliminator refuses.
        if let Some((_, header)) = self
            .active_carried_computational_eliminations
            .iter()
            .rev()
            .find(|(origin, _)| *origin == eliminator.static_origin)
        {
            builder.ins().jump(*header, &[scrutinee.word.into()]);
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        }

        let header = builder.create_block();
        builder.append_block_param(header, types::I64);
        builder.ins().jump(header, &[scrutinee.word.into()]);
        builder.switch_to_block(header);
        let scrutinee = CarriedBoundaryWord {
            word: builder.block_params(header)[0],
        };
        self.active_carried_computational_eliminations
            .push((eliminator.static_origin, header));
        let lowered = self.lower_carried_computational_match_inner(
            builder,
            scrutinee,
            eliminator,
            remaining_eliminators,
        );
        let popped = self.active_carried_computational_eliminations.pop();
        debug_assert_eq!(
            popped,
            Some((eliminator.static_origin, header)),
            "the carried elimination stack must unwind in the order it was pushed"
        );
        lowered
    }

    fn lower_carried_computational_match_inner(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        eliminator: ComputationalEliminatorFrame<'_>,
        remaining_eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        #[cfg(test)]
        record_d6a_route_event(D6aRouteEvent::CarriedEliminationEntered {
            static_origin: eliminator.static_origin,
            route: eliminator.answer_route,
            cases: eliminator.cases.len(),
        });
        if eliminator.cases.is_empty() {
            #[cfg(test)]
            record_d6a_route_event(D6aRouteEvent::CarriedDefaultSealed {
                static_origin: eliminator.static_origin,
                route: eliminator.answer_route,
            });
            return Ok(LoweringOperand::Specialized(Lowered::Trap(
                eliminator.default.clone(),
            )));
        }
        // ⛔ A deferred constructor case rebuilds a `Lowered::Constructor`
        // *around* the scrutinee, which needs a compile-time template for the
        // parent. Refused rather than approximated.
        if eliminator.deferred_constructor_case.is_some() {
            return Err(unsupported(
                "BoundaryCarrier",
                "a carried scrutinee reached a deferred constructor case, which \
                 reconstructs a compile-time constructor around the eliminated value",
            ));
        }

        let tag = self.emit_carrier_tag(builder, scrutinee)?;
        let field_count = self.emit_carrier_field_count(builder, scrutinee)?;

        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);

        for (index, case) in eliminator.cases.iter().enumerate() {
            // ⛔ Malformed recursive positions are rejected before any code is
            // emitted for this case, exactly as the specialized composed path
            // rejects them. ⚠ The bound is `argument_binders` — the case's own
            // declared arity — and ⛔ NOT anything read off the carried word:
            // the word's `field_count` is checked against that same arity
            // below, at runtime, which is where a disagreement belongs.
            let mut seen = BTreeSet::new();
            for position in case.recursive_positions.iter().copied() {
                if !seen.insert(position) || position >= case.argument_binders {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} has malformed recursive position {position}",
                            case.constructor
                        ),
                    ));
                }
            }
            let identity = self
                .static_transition_plan
                .case_constructor_identity(eliminator.static_origin, index)?
                .tag_abi_word()?;
            let identity = Self::carrier_identity_immediate(builder, identity);
            let selected = builder.create_block();
            let next = builder.create_block();
            let matched = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                identity,
            );
            builder.ins().brif(matched, selected, &[], next, &[]);

            builder.switch_to_block(selected);
            let binders = i64::try_from(case.argument_binders).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "a case binds more constructor arguments than the carrier ABI can count",
                )
            })?;
            Self::require_i64(builder, field_count, binders);

            let mut children = Vec::with_capacity(case.argument_binders);
            for position in 0..case.argument_binders {
                // ⭐ `§2g` — the projected child stays `Carried` into `case_env`.
                let child = self.emit_carrier_field(builder, scrutinee, position)?;
                // ⭐ The residual edge's oracle, written here and keyed on THIS
                // loop's own counter — before any selection among the children
                // happens. ⛔ Not derived from `recursive_positions`.
                #[cfg(test)]
                px8j_record_carrier_field_projection(Px8jProducerPath::Composed, position, child);
                children.push(LoweringOperand::Carried(child));
            }

            // ── ⭐⭐ `AC-C4` — the induction hypotheses over carried children ──
            //
            // ⚠ **Order is load-bearing and matches the specialized composed
            // path exactly:** `[IHs, reversed] ++ [children] ++ [frame env]`.
            // De Bruijn indices in the case body are positional, so a different
            // order here would silently rebind every recursive-position body.
            let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
            let mut active_scope = None;
            if !case.recursive_positions.is_empty() {
                let ih_slots =
                    self.computational_ih_slots_for_case(case, eliminator.checked_frame_id)?;
                let activation = self.mint_continuation_activation();
                let cursor = self.mint_continuation_cursor();
                let producer_origin = self.mint_recursor_producer_origin();
                let splice_caller = active_recursor_frame(remaining_eliminators);
                #[cfg(test)]
                px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::Composed,
                    origin: producer_origin,
                    cursor,
                    siblings: case.recursive_positions.len(),
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                });
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    // ⭐ Clause 1 — the CARRIED arm passes its projected operand
                    // **directly**. ⛔ No wrap, no `specialized_at`, no template.
                    let induction_hypothesis = self.make_computational_recursor(
                        children[position].clone(),
                        eliminator.cases.to_vec(),
                        eliminator.default.clone(),
                        eliminator.env.to_vec(),
                        eliminator.static_origin,
                        eliminator.provenance,
                        eliminator.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        activation,
                        cursor,
                        splice_caller,
                        None,
                        self.recursive_position_unit_body(eliminator.static_origin, position)?,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(Px8jProducerPath::Composed, &induction_hypothesis);
                    induction_hypotheses.push(LoweringEnvironmentBinding::Value(induction_hypothesis));
                }
                active_scope = Some((activation, cursor, producer_origin, splice_caller));
            }

            let mut case_env = induction_hypotheses;
            case_env.extend(bound_values(children));
            // The frame's own environment, with the retained scrutinee inserted
            // where the frame asked for it. ⭐ Retention is phase-preserving:
            // the retained value is the **same carried word**, ⛔ never a
            // materialized template of it.
            let mut frame_env = eliminator.env.to_vec();
            if let Some(retained) = eliminator.retained_scrutinee_index {
                if retained > frame_env.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "retained scrutinee index exceeds the frame environment",
                    ));
                }
                frame_env.insert(
                    retained,
                    LoweringEnvironmentBinding::Value(LoweringOperand::Carried(scrutinee)),
                );
            }
            case_env.extend(frame_env);

            let body = self.case_body_occurrence(eliminator.static_origin, index, &case.body)?;
            let body_origin = body.static_origin;
            let lowered =
                if let Some((activation, cursor, producer_origin, splice_caller)) = active_scope {
                    // ⭐ A case with recursive positions descends through the SOURCE
                    // MACHINE, as the specialized composed path does — the body's IH
                    // call needs a live continuation to resume into, and that is the
                    // machinery that supplies one. ⛔ The only difference between
                    // this block and its specialized twin is the phase of the
                    // children; every identity below is the frame's.
                    let mut selected_ancestry = splice_caller
                        .map(|active| active.selected_ancestry.to_vec())
                        .unwrap_or_default();
                    selected_ancestry.push(eliminator.provenance);
                    let mut pending: Vec<_> = remaining_eliminators
                        .iter()
                        .copied()
                        .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                        .collect();
                    if let Some(active) = splice_caller {
                        pending.extend_from_slice(active.pending);
                    }
                    let selected_scope = OwnedSelectedScope {
                        scope_origin: producer_origin,
                        parent_scope: splice_caller
                            .and_then(|active| active.selected_scope)
                            .map(|scope| scope.scope_origin),
                        frame: ComputationalRecursorFramePayload {
                            cases: eliminator.cases.to_vec(),
                            default: eliminator.default.clone(),
                            outer_env: eliminator.env.to_vec(),
                            static_origin: eliminator.static_origin,
                            provenance: eliminator.provenance,
                            checked_frame_id: eliminator.checked_frame_id,
                            checked_invocation_id: eliminator.checked_invocation_id,
                            checked_invocation_source: eliminator.checked_invocation_source,
                            checked_invocation_depth: eliminator.checked_invocation_depth,
                        },
                    };
                    let active_state = ActiveContinuationFrame {
                        activation,
                        cursor,
                        parent: splice_caller.and_then(|active| active.parent),
                        pending: &pending,
                        selected_ancestry: &selected_ancestry,
                        source_lineage: splice_caller
                            .map(|active| active.source_lineage)
                            .unwrap_or(&[]),
                        source_selected_cursor: splice_caller
                            .and_then(|active| active.source_selected_cursor),
                        selected_scope: Some(&selected_scope),
                    };
                    self.lower_source_machine(builder, body, &case_env, &active_state)?
                } else if remaining_eliminators.is_empty() {
                    self.lower_expr(builder, body, &case_env)?
                } else {
                    self.lower_computational_producer_expr(
                        builder,
                        body,
                        &case_env,
                        remaining_eliminators,
                    )?
                };
            if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                let word = self.carried_join_arm(
                    builder,
                    body_origin,
                    lowered,
                    None,
                    "a carried `ComputationalMatch` arm",
                )?;
                builder.ins().jump(merge, &[word.word.into()]);
            }

            builder.switch_to_block(next);
        }

        // ── ⭐⭐ `RT-DECL-CLOSURE-PORT` `D6a` — THE CHECKED-ANSWER FALLBACK ──
        //
        // Reached only after **every** ordinary case's exact planner-issued tag
        // has been compared and missed. ⛔ That ordering is the mechanism, not a
        // convenience: an ordinary carried `ITree` constructor still takes its
        // own case, and this arm can never shadow one.
        //
        // The checked recursive worker returns the checked *answer*
        // (`Result::Ok` on the governed witness), which the **specialized** arm
        // sends to the unique guarded `ITree::Ret` continuation. The carried arm
        // used to ask instead whether that answer was literally an `ITree`
        // constructor — it is not — and seal the closed default. This restores
        // the same guarded route on the same fact.
        //
        // ⛔ **No phase forgery.** The carried word is fed to the return case's
        // one retained argument **as itself**: nothing is decoded, converted to
        // `Lowered`, recovered as a template, or selected at runtime, and no
        // constructor is matched by name. The guard is a compile-time property
        // of the case *topology*, identical to the specialized arm's, and the
        // word never participates in it.
        let checked_answer_fallback = eliminator.answer_route
            == SourceComputationalAnswerRoute::CheckedSelectedRecursor
            && px8tr_deforested_answer_route_enabled();
        let return_case = if checked_answer_fallback {
            // The strict existing topology, re-derived here exactly as the
            // specialized arm derives it: one `Ret` case with one binder, one
            // `Vis` case, two cases total, and no checked control markers in the
            // return body.
            let mut returns = eliminator.cases.iter().enumerate().filter(|(_, case)| {
                case.argument_binders == 1 && case.constructor.ends_with("::ITree::Ret")
            });
            let return_case = returns.next();
            let exact_return = returns.next().is_none();
            let mut visible = eliminator
                .cases
                .iter()
                .filter(|case| case.constructor.ends_with("::ITree::Vis"));
            let exact_visible =
                visible.next().is_some() && visible.next().is_none() && eliminator.cases.len() == 2;
            return_case.filter(|(_, return_case)| {
                exact_return
                    && exact_visible
                    && source_case_has_no_checked_control_markers(&return_case.body)
            })
        } else {
            None
        };

        if let Some((return_index, return_case)) = return_case {
            // ⭐ The EMISSION discriminator for this branch.
            //
            // ⚠ Deliberately not `DeforestedAnswerResumed`. That event is
            // recorded while lowering the **specialized** branch and is
            // therefore compile-time evidence about a choice the emitted CFG
            // makes elsewhere — it cannot testify that a *carried* runtime word
            // took this route, and reusing it here would make a compile-time
            // fact read as a runtime one. This says only what it knows: the
            // carried route was emitted, for this frame, into this return case.
            // The runtime half is the linked artifact's exit status.
            #[cfg(test)]
            px8tr_record_trap_provenance(Px8trTrapProvenanceEvent::CarriedAnswerRouteEmitted {
                checked_frame_id: eliminator
                    .checked_frame_id
                    .expect("checked answer routes carry exact frame ids"),
                return_constructor: return_case.constructor.clone(),
            });
            #[cfg(test)]
            record_d6a_route_event(D6aRouteEvent::CarriedFallbackEmitted {
                static_origin: eliminator.static_origin,
            });
            // The one retained argument is the SAME carried word. ⛔ Not a
            // projected field of it: the checked answer is the value the return
            // case binds, and projecting would ask the carrier for structure
            // this route never claimed it has.
            let mut case_env =
                vec![LoweringEnvironmentBinding::Value(LoweringOperand::Carried(scrutinee))];
            case_env.extend(eliminator.env.to_vec());
            let body =
                self.case_body_occurrence(eliminator.static_origin, return_index, &return_case.body)?;
            let body_origin = body.static_origin;
            // ⭐ Lowered through the ORDINARY continuation of this eliminator,
            // exactly as a non-recursive case body beside it is. The eliminated
            // value returns to `SourceContinuation::ComputationalMatchScrutinee`,
            // which resumes the original source control — so the source
            // continuation after the return case is observed, and a helper that
            // returned an isolated value could not stand in for it.
            let lowered = if remaining_eliminators.is_empty() {
                self.lower_expr(builder, body, &case_env)?
            } else {
                self.lower_computational_producer_expr(
                    builder,
                    body,
                    &case_env,
                    remaining_eliminators,
                )?
            };
            if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                let word = self.carried_join_arm(
                    builder,
                    body_origin,
                    lowered,
                    None,
                    "a carried checked-answer arm",
                )?;
                builder.ins().jump(merge, &[word.word.into()]);
            }
        } else {
            // ⛔ Every other way of arriving here keeps the existing closed
            // default, unchanged: `DirectScrutinee`, a disabled fallback,
            // malformed or ambiguous return topology, and every unmatched
            // ordinary carried scrutinee.
            #[cfg(test)]
            record_d6a_route_event(D6aRouteEvent::CarriedDefaultSealed {
                static_origin: eliminator.static_origin,
                route: eliminator.answer_route,
            });
            let defaulted = LoweringOperand::Specialized(Lowered::Trap(eliminator.default.clone()));
            if !self.seal_source_trap_branch(builder, &defaulted)? {
                return Err(unsupported(
                    "ComputationalMatch",
                    "the carried computational match's closed default did not seal its branch",
                ));
            }
        }

        builder.switch_to_block(merge);
        Ok(LoweringOperand::Carried(CarriedBoundaryWord {
            word: builder.block_params(merge)[0],
        }))
    }

    /// Lowers one source occurrence.
    ///
    /// ## The per-variant child-position table
    ///
    /// ⭐ **A child's position is its index in the `children: &[StaticNodeId]`
    /// slice the planner hands to `expression_node` / `expression_seed`** — *not*
    /// the `ordinal` parameter of `plan_expr`, which keys the frame's syntax and
    /// path stores instead. Reading positions off the `plan_expr` call sites
    /// gives the wrong table for every multi-child variant.
    ///
    /// | variant | positions (planner `children` order) |
    /// |---|---|
    /// | `CheckedJoinSite` / `CheckedSubcontinuationFrame` / `CheckedRecursiveInvocation` / `CheckedComputationalIHSlots` / `CheckedComputationalIHInvocation` | `0` = body |
    /// | `Value` / `Var` / `DeclarationRef` / `ImportedDeclarationRef` / `Trap` | no expression children |
    /// | `Let` | `0` = value, `1` = body |
    /// | `If` | `0` = scrutinee, `1` = then, `2` = else |
    /// | `PrimitiveCall` / `Construct` | `i` = `args[i]` |
    /// | `Record` | `i` = `fields[i]`'s value |
    /// | `Project` | `0` = record |
    /// | `Match` | `0` = scrutinee, `1 + i` = `cases[i].body` |
    /// | `ComputationalMatch` | `0` = scrutinee, `1 + i` = `cases[i].body` — ⚠ and it is the **sole** variant whose `entry != occurrence.node` (second axis below) |
    /// | `Closure` | `0` = body |
    /// | `LexicalClosure` | ⚠ `0` = **body**, `1 + i` = `captures[i]` |
    /// | `Call` | `0` = callee, `1 + i` = `args[i]` |
    /// | `Effect` | ⚠ capability present: `0` = `capability.value`, `1 + i` = `args[i]`; absent: `i` = `args[i]` |
    ///
    /// The planner's order and this walk's traversal **agree** on every variant.
    /// Two of them disagree with *declaration field order*, which is the trap a
    /// future author would fall into, so they are marked ⚠ above and again at
    /// their arms:
    ///
    /// 1. `LexicalClosure` declares `captures, params, body` but plans **body
    ///    first**, because the body is planned before the capture sequence.
    /// 2. `Effect`'s capability takes position `0` **only when present**, so the
    ///    argument base is a conditional offset rather than a constant.
    ///
    /// ## ⭐ THE SECOND AXIS: `entry` vs `occurrence`
    ///
    /// Positional agreement does **not** imply that the identity a parent
    /// schedules is the identity that owns the child record. `plan_expr` returns
    /// both (`PlannedExpr { entry, occurrence }`), and the positions above are
    /// always indexed by the **occurrence**.
    ///
    /// | | `entry == occurrence.node`? |
    /// |---|---|
    /// | every variant except `ComputationalMatch` | **yes**, by construction — they all return through `expression_node` |
    /// | `ComputationalMatch` | **no**, and deliberately: its record is seeded on its `SourceReturnResume` while a parent still schedules its scrutinee. It is the SOLE split. |
    ///
    /// ⛔ Passing an `entry` where an `occurrence` belongs is a category error, not
    /// an off-by-one. The seed API takes `&[StaticOriginId]`
    /// so the type now prevents it; do not re-conflate the two axes.
    ///
    /// ⛔ Where the two orders could ever disagree the **planner's** position
    /// wins: the plane's records are already laid out against it.
    pub(super) fn lower_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let SourceOccurrence {
            expr,
            static_origin,
        } = occurrence;
        self.enter_source_occurrence_plan(static_origin)?;
        match expr {
            RuntimeExpr::Value(value) => self
                .lower_value(builder, value)
                .map(LoweringOperand::Specialized),
            RuntimeExpr::CheckedJoinSite { site_id, body } => {
                if self.active_join_site.replace(*site_id).is_some() {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "nested checked join occurrence marker",
                    ));
                }
                let body = self.child_occurrence(static_origin, 0, body)?;
                let result = self.lower_expr(builder, body, env);
                if self.active_join_site.take().is_some() {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked join occurrence marker was not consumed",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                self.enter_checked_subcontinuation_frame(*frame_id)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let result = self.lower_expr(builder, body, env);
                if self.active_subcontinuation_frame.take().is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "checked subcontinuation marker was not consumed by its frame",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                body,
                ..
            } => {
                let instance =
                    self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let result = self.lower_expr(builder, body, env);
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.lower_expr(builder, body, env)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                // `D8f` — same derivation, same reason.
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.enter_checked_computational_ih_invocation(
                    *call_template_id,
                    body.expr,
                    body.static_origin,
                )?;
                let value = self.lower_expr(builder, body, env)?;
                self.finish_checked_computational_ih_marker(value)
            }
            // A `Var` here is a value-producing position, so it accepts only
            // `Value`. A static worker binding fails closed in `value_at`
            // rather than being cloned out as if it were a value.
            RuntimeExpr::Var(index) => env
                .get(*index as usize)
                .ok_or_else(|| unsupported("Var", format!("no runtime binding for index {index}")))?
                .value_at("a Var in value position")
                .cloned(),
            RuntimeExpr::PrimitiveCall { primitive, args } => {
                self.lower_primitive_call(builder, primitive, args, static_origin, env)
            }
            RuntimeExpr::Let { value, body } => {
                let value = self.child_occurrence(static_origin, 0, value)?;
                let bound = self.lower_binder(builder, value, env)?;
                // `RT-CONTSRC-PRODUCER-LOCAL` `D4a` — the binder-creation seat.
                // ⭐ The observatory's independent half: the operand and the
                // occurrence that creates it are both in hand HERE, with no
                // environment index involved. Test-only; production is
                // unchanged.
                #[cfg(test)]
                if crate::cranelift_backend::lowering::d4a_armed() {
                    crate::cranelift_backend::lowering::d4a_record_created(
                        value.static_origin,
                        crate::cranelift_backend::lowering::d4a_describe_binding(Some(&bound)),
                    );
                }
                // The two short-circuits below are value-shaped, so they read
                // through the binding rather than around it. A static worker
                // binding is neither a backedge nor a trap, so it falls
                // through to the ordinary installation.
                if let LoweringEnvironmentBinding::Value(lowered_value) = &bound {
                    if matches!(
                        lowered_value,
                        LoweringOperand::Specialized(Lowered::RecursiveBackedge)
                    ) {
                        return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                    }
                    if let LoweringOperand::Specialized(Lowered::Trap(trap)) = lowered_value {
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap.clone())));
                    }
                }
                let mut body_env = vec![bound];
                body_env.extend_from_slice(env);
                let body = self.child_occurrence(static_origin, 1, body)?;
                self.lower_expr(builder, body, &body_env)
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                let then_expr = self.child_occurrence(static_origin, 1, then_expr)?;
                let else_expr = self.child_occurrence(static_origin, 2, else_expr)?;
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                if matches!(lowered_scrutinee, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                    return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                }
                let LoweringOperand::Specialized(Lowered::Bool { value, known }) = lowered_scrutinee else {
                    return Err(unsupported(
                        "If",
                        "branch lowering requires a Bool scrutinee",
                    ));
                };
                if let Some(scrutinee) = known {
                    let unselected = if scrutinee { else_expr } else { then_expr };
                    self.disposition_statically_unselected_source_subtree(
                        unselected.static_origin,
                    )?;
                    return if scrutinee {
                        self.lower_expr(builder, then_expr, env)
                    } else {
                        self.lower_expr(builder, else_expr, env)
                    };
                }
                let join_plan = self.consumed_join_plan_token(static_origin)?;
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge = join_plan
                    .has_continuing_predecessor
                    .then(|| builder.create_block());
                if let Some(merge) = merge {
                    self.append_planned_join_params(builder, merge, &join_plan);
                }
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                let mut merge_kind = None;
                let mut terminal_trap = None;
                for (block, arm) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_expr(builder, arm, env)?;
                    if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                        terminal_trap.get_or_insert_with(|| trap.clone());
                    }
                    if self.seal_source_trap_branch(builder, &lowered)? {
                        continue;
                    }
                    let merge = merge.ok_or_else(|| {
                        backend_module(
                            "join plan omitted an If merge despite a continuing predecessor"
                                .to_string(),
                        )
                    })?;
                    self.jump_planned_join_arm(
                        builder,
                        merge,
                        &join_plan,
                        arm.static_origin,
                        lowered,
                        &mut merge_kind,
                        "If",
                    )?;
                }
                let Some(merge) = merge else {
                    let unreachable = builder.create_block();
                    builder.switch_to_block(unreachable);
                    let trap = terminal_trap.ok_or_else(|| {
                        backend_module(
                            "If join omitted both a continuing predecessor and a source trap"
                                .to_string(),
                        )
                    })?;
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                };
                self.finish_planned_join(builder, merge, &join_plan, merge_kind, "If")
            }
            RuntimeExpr::Construct { constructor, args } => {
                let lowered_args = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg = self.child_occurrence(static_origin, position, arg)?;
                        self.lower_expr(builder, arg, env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if lowered_args
                    .iter()
                    .any(|arg| matches!(arg, LoweringOperand::Specialized(Lowered::RecursiveBackedge)))
                {
                    return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                }
                if lowered_args.is_empty()
                    && (constructor == &self.process_symbols.bool_true
                        || constructor == &self.process_symbols.bool_false)
                {
                    let known = constructor == &self.process_symbols.bool_true;
                    return Ok(LoweringOperand::Specialized(Lowered::Bool {
                        value: builder.ins().iconst(types::I64, i64::from(known)),
                        known: Some(known),
                    }));
                }
                if constructor == &self.process_symbols.nat_zero && lowered_args.is_empty() {
                    return Ok(LoweringOperand::Specialized(Lowered::StructuralNat(StructuralNatV1 {
                        value: builder.ins().iconst(types::I64, 0),
                    })));
                }
                if constructor == &self.process_symbols.nat_suc {
                    if let [LoweringOperand::Specialized(Lowered::StructuralNat(predecessor))] = lowered_args.as_slice() {
                        return Ok(LoweringOperand::Specialized(Lowered::StructuralNat(StructuralNatV1 {
                            value: builder.ins().iadd_imm(predecessor.value, 1),
                        })));
                    }
                }
                if lowered_args
                    .iter()
                    .any(|argument| matches!(argument, LoweringOperand::Carried(_)))
                {
                    return Ok(LoweringOperand::Carried(
                        self.transfer_constructor_operands(
                            builder,
                            static_origin,
                            constructor,
                            &lowered_args,
                        )?,
                    ));
                }
                Ok(LoweringOperand::Specialized(Lowered::Constructor {
                    constructor: constructor.clone(),
                    synthesized_identity: Some(
                        self.static_transition_plan
                            .constructor_symbol_identity(static_origin)?,
                    ),
                    // `D7` -- the allocation lane is the second fact resolved
                    // at the producer and carried with the template.
                    occurrence: Some(self.static_transition_plan.source_aggregate_occurrence(
                        static_origin,
                        PlannedAggregateShape::Constructor,
                    )?),
                    args: specialized_operands_at(&lowered_args, "a constructor argument")?,
                }))
            }
            RuntimeExpr::Match {
                scrutinee,
                cases,
                default,
            } => {
                let scrutinee_occurrence = self.child_occurrence(static_origin, 0, scrutinee)?;
                if requires_heterogeneous_deforestation(scrutinee)
                    || self.declaration_call_produces_deforestable_aggregate(scrutinee)
                {
                    return self.lower_computational_producer_expr(
                        builder,
                        scrutinee_occurrence,
                        env,
                        &[EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                            cases,
                            default,
                            env,
                            static_origin,
                            retained_scrutinee_index: None,
                            deferred_constructor_case: None,
                        })],
                    );
                }
                let lowered_scrutinee = self.lower_expr(builder, scrutinee_occurrence, env)?;
                // ⭐⭐ `D3`'s CARRIED arm, and it MUST come first.
                //
                // ⚠ Every test below asks for a specific `Lowered` shape, and
                // the chain ends in *"scrutinee is not a constructor value"*. A
                // carried scrutinee would fall past all of them and land on
                // that refusal — a **true sentence about the wrong thing**,
                // which is worse than an error, because it names a cause that
                // is not the cause. Classifying the phase first is what makes
                // the rest of the chain a statement about `Lowered` only.
                if let LoweringOperand::Carried(word) = lowered_scrutinee {
                    return self.lower_carried_match(
                        builder,
                        word,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::BorrowedNativeValue { pointer }) = lowered_scrutinee {
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
                    return self.lower_borrowed_match(
                        builder,
                        pointer,
                        cases,
                        default,
                        static_origin,
                        env,
                        &join_plan,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::BorrowedOption {
                    present,
                    value,
                    none,
                    some,
                }) = lowered_scrutinee
                {
                    return self.lower_borrowed_option_match(
                        builder,
                        present,
                        value,
                        &none,
                        &some,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::BoundedNat(nat)) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(
                        builder,
                        nat,
                        false,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::StructuralNat(nat)) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(
                        builder,
                        BoundedNatV1::derived_from_validated(nat.value),
                        true,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                }) = lowered_scrutinee
                {
                    return self.lower_dynamic_host_result_match(
                        builder,
                        success,
                        *error,
                        *ok,
                        &err_constructor,
                        &ok_constructor,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::DynamicConstructor(dynamic)) = lowered_scrutinee {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Ordinary {
                            cases,
                            default,
                            env,
                            static_origin,
                        },
                    );
                }
                if let LoweringOperand::Specialized(Lowered::Bool { value, known }) = lowered_scrutinee {
                    // ⭐ These two cases are found by CONSTRUCTOR NAME, and a
                    // search yields no position — so both lookups enumerate and
                    // keep the index. The index, not the found body, is what the
                    // origin is derived from.
                    let true_case = cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "Match",
                            "Bool match requires zero-binder True and False cases",
                        ));
                    };
                    if let Some(selected) = known {
                        let (index, case) = if selected { true_case } else { false_case };
                        self.disposition_statically_unselected_match_cases(
                            static_origin,
                            Some(index),
                        )?;
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        return self.lower_expr(builder, body, env);
                    }
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = join_plan
                        .has_continuing_predecessor
                        .then(|| builder.create_block());
                    if let Some(merge) = merge {
                        self.append_planned_join_params(builder, merge, &join_plan);
                    }
                    builder
                        .ins()
                        .brif(value, true_block, &[], false_block, &[]);
                    let mut merge_kind = None;
                    let mut terminal_trap = None;
                    for (block, (index, case)) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        let lowered = self.lower_expr(builder, body, env)?;
                        if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                            terminal_trap.get_or_insert_with(|| trap.clone());
                        }
                        if self.seal_source_trap_branch(builder, &lowered)? {
                            continue;
                        }
                        let merge = merge.ok_or_else(|| {
                            backend_module(
                                "join plan omitted a Bool Match merge despite a continuing \
                                 predecessor"
                                    .to_string(),
                            )
                        })?;
                        self.jump_planned_join_arm(
                            builder,
                            merge,
                            &join_plan,
                            body.static_origin,
                            lowered,
                            &mut merge_kind,
                            "Match",
                        )?;
                    }
                    let Some(merge) = merge else {
                        let unreachable = builder.create_block();
                        builder.switch_to_block(unreachable);
                        let trap = terminal_trap.ok_or_else(|| {
                            backend_module(
                                "Bool Match join omitted both a continuing predecessor and a \
                                 source trap"
                                    .to_string(),
                            )
                        })?;
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                    };
                    return self.finish_planned_join(
                        builder,
                        merge,
                        &join_plan,
                        merge_kind,
                        "Match",
                    );
                }
                let LoweringOperand::Specialized(Lowered::Constructor {
                    constructor,
                    args,
                    ..
                }) = lowered_scrutinee else {
                    return Err(unsupported("Match", "scrutinee is not a constructor value"));
                };
                let Some((index, case)) = cases
                    .iter()
                    .enumerate()
                    .find(|(_, case)| case.constructor == constructor)
                else {
                    self.disposition_statically_unselected_match_cases(
                        static_origin,
                        None,
                    )?;
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
                };
                self.disposition_statically_unselected_match_cases(
                    static_origin,
                    Some(index),
                )?;
                if case.binders != args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            args.len()
                        ),
                    ));
                }
                let case_env = env_with(args, env);
                let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                self.lower_expr(builder, body, &case_env)
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee,
                cases,
                default,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                self.lower_computational_match_expr(
                    builder,
                    scrutinee,
                    cases,
                    default,
                    static_origin,
                    env,
                    env,
                )
            }
            RuntimeExpr::Record { fields } => {
                let lowered_fields = fields
                    .iter()
                    .enumerate()
                    .map(|(position, (name, expr))| {
                        let expr = self.child_occurrence(static_origin, position, expr)?;
                        Ok((position, name.clone(), self.lower_expr(builder, expr, env)?))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
                let lowered_fields = lowered_fields
                    .into_iter()
                    .map(|(position, name, value)| {
                        Ok(LoweredRecordField {
                            name,
                            // ⭐ `D7` — the field SCHEMA is resolved at the
                            // producer, beside the ownership record, and for the
                            // same reason: a record forwarded through a `Var` or
                            // handed to a call arrives where the plan cannot
                            // lawfully be re-queried for it. ⛔ The `name` above
                            // is the compile-time spelling and is never the key.
                            identity: Some(
                                self.static_transition_plan
                                    .record_field_identity(static_origin, position)?,
                            ),
                            value: value.specialized_at("a record field's value")?,
                        })
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
                Ok(LoweringOperand::Specialized(Lowered::Record {
                    // ⭐ `D7` — resolved AT THE PRODUCER and carried with the
                    // template, exactly as the `Construct` arms resolve theirs.
                    // This is the one place a source record's ownership record
                    // may be looked up: everywhere downstream is a use.
                    occurrence: Some(self.static_transition_plan.source_aggregate_occurrence(
                        static_origin,
                        PlannedAggregateShape::Record,
                    )?),
                    fields: lowered_fields,
                }))
            }
            RuntimeExpr::Project { record, field } => {
                let record = self.child_occurrence(static_origin, 0, record)?;
                let lowered_record = self.lower_expr(builder, record, env)?;
                // ⭐ `D4`'s two-phase arm. ⛔ No wildcard over the phase: a
                // third `LoweringOperand` inhabitant must break compilation
                // here rather than silently taking whichever arm a `_` had
                // swallowed (`§2g`, `D5`).
                match lowered_record {
                    // ── the CARRIED route — `record_field` at runtime ──────
                    LoweringOperand::Carried(word) => {
                        // ⭐ `D1`/`D2`: the key is the artifact-static field
                        // identity of **this `Project` occurrence**, from the
                        // one authority the producer's `store_name` also used.
                        //
                        // ⚠ The `field` **string** is deliberately NOT the key.
                        // It is the compile-time spelling; keying on it would be
                        // the second derivation `D2` forbids — and it is also
                        // what makes `AC-C5` work, because a record whose fields
                        // are reordered relative to declaration still projects
                        // correctly when the lookup is by interned name rather
                        // than by position.
                        let identity = self
                            .static_transition_plan
                            .project_field_identity(static_origin)?
                            .name_abi_word()?;
                        let selected = self.emit_carrier_record_field(builder, word, identity)?;
                        // ⭐ `§2g`, verbatim: *"projected children remain
                        // `Carried`."* ⛔ Not materialized into a `Lowered`
                        // template — that is the wall itself.
                        Ok(LoweringOperand::Carried(selected))
                    }
                    // ── the pre-existing SPECIALIZED route, unchanged ──────
                    LoweringOperand::Specialized(lowered) => {
                        let Lowered::Record { fields, .. } = lowered else {
                            return Err(unsupported(
                                "Project",
                                "record projection needs a record value",
                            ));
                        };
                        fields
                            .into_iter()
                            .find_map(|held| (held.name == *field).then_some(held.value))
                            .map(LoweringOperand::Specialized)
                            .ok_or_else(|| unsupported("Project", format!("missing field {field}")))
                    }
                }
            }
            // Site 1 of 3. The occurrence's own origin is in scope here, so the
            // body's origin is `child(_, 0)` — determined, not searched for.
            // ⭐ The origin is now the *whole* carrier: the body is not cloned into
            // the closure at all, and the term is recovered from the plan by this
            // name when a call site re-lowers it. The clone this site used to make
            // was the second authority the chain exists to remove.
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let body = self.child_occurrence(static_origin, 0, body)?;
                let lowered_captures = captures
                    .iter()
                    .map(|symbol| {
                        // Seed captures are resolved to JIT-time ground values,
                        // so this arm asserts the phase; there is no carried
                        // seed capture for it to lose.
                        self.lower_seed_capture(builder, symbol)
                            .map(LoweringOperand::Specialized)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(LoweringOperand::Specialized(Lowered::Closure {
                    captures: lowered_captures,
                    params: params.clone(),
                    body: body.static_origin,
                }))
            }
            // D7, site 2 of 3.
            //
            // ⚠ HAZARD 1 (D3): the planner plans the **body first** and the
            // capture sequence after it, so body is position `0` and capture *i*
            // is `1 + i` — the declaration order (`captures, params, body`) is
            // NOT the child order. Evaluation order below is unchanged: the
            // captures are still lowered before the body is retained.
            RuntimeExpr::LexicalClosure {
                captures,
                params,
                body,
            } => {
                let body = self.child_occurrence(static_origin, 0, body)?;
                let captures = captures
                    .iter()
                    .enumerate()
                    .map(|(position, capture)| {
                        let capture = self.child_occurrence(static_origin, 1 + position, capture)?;
                        self.lower_expr(builder, capture, env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                // ⭐⭐ **`D7` — THE closure-capture cell, and the seat the
                // framed reaching row stops at.**
                //
                // A lexical capture reaches this closure at its own phase, and a
                // capture that arrived through a declared ABI slot is `Carried`.
                // The former `specialized_operands_at` fold demanded a
                // compile-time template for every one of them, so a lawfully
                // mixed environment had no representation at all and refused
                // here — even when, as measured, the enclosing aggregate stays a
                // specialized template and nothing ever crosses a boundary.
                //
                // ⛔ Storing the operands unchanged does not make the capsule
                // transferable. `boundary_transfer_admissibility` still refuses
                // the outer closure **before** descending into these captures,
                // so a carried capture cannot become a way to reach the carrier
                // through a callable that is itself refused.
                //
                // ⭐ The gate runs on the MIXED case only, and the scoping is
                // deliberate. An all-specialized lexical closure is the
                // pre-existing shape and does not require a planner-issued
                // worker template to exist at all; demanding one would reject
                // programs this node never touched. A carried capture is what
                // creates the obligation, because it is what commits the callee
                // to loading an exact activation-frame slot.
                if captures
                    .iter()
                    .any(|capture| matches!(capture, LoweringOperand::Carried(_)))
                {
                    self.validate_retained_callable_capture_contract(
                        static_origin,
                        body.static_origin,
                        AbiCaptureProvenance::Lexical,
                        params.len(),
                        &captures,
                    )?;
                }
                Ok(LoweringOperand::Specialized(Lowered::Closure {
                    captures,
                    params: params.clone(),
                    body: body.static_origin,
                }))
            }
            RuntimeExpr::DeclarationRef { symbol } => {
                self.lower_declaration_ref(builder, static_origin, symbol)
            }
            RuntimeExpr::ImportedDeclarationRef {
                symbol,
                dependency,
                dependency_semantic_hash,
            } => Err(unsupported(
                "ImportedDeclarationRef",
                format!(
                    "imported declaration {symbol} from {dependency} @ {dependency_semantic_hash} requires dependency linking"
                ),
            )),
            RuntimeExpr::Call { callee, args } => {
                let join_plan = self.consumed_join_plan_token(static_origin)?;
                let callee = self.child_occurrence(static_origin, 0, callee)?;
                // **`D3` -- THE SOLE CONSUMER of a static worker binding.**
                //
                // Only a `Call` whose callee is an exact `Var` resolving to
                // `StaticWorker` may consume one, and this is the only place
                // that reads the arm without going through `value_at`. It sits
                // ahead of every other callee route precisely so the binding is
                // consumed here or fails closed everywhere else: a `Var`
                // resolving to `Value` falls through to the pre-existing paths
                // untouched.
                if let RuntimeExpr::Var(index) = callee.expr {
                    if let Some(LoweringEnvironmentBinding::StaticWorker(worker)) =
                        env.get(*index as usize)
                    {
                        let worker = worker.clone();
                        // **`D5a` — the checked-IH marker is consumed HERE, on
                        // the application, before a single instruction of it is
                        // emitted.** Every identity is cross-checked against the
                        // checked plan first; a refusal leaves the marker
                        // pending so closeout still fails closed.
                        // `D8f` — the occurrence of THIS call, so a marker is
                        // consumed only by the application the plan issued it
                        // for. An ordinary selected-argument call reaching this
                        // seat with a marker pending leaves it pending.
                        self.consume_checked_ih_marker_at_static_worker_call(
                            u64::from(*index),
                            args.len(),
                            static_origin,
                        )?;
                        return self.call_static_worker(
                            builder,
                            &worker,
                            args,
                            static_origin,
                            env,
                        );
                    }
                }
                if matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::FunctionizedUnits
                ) {
                    // **`RT-SEED-CALL-PORT` `D2` — the callee-position seed
                    // unit** (Architect `evt_7p8dmg1rez02c`).
                    //
                    // This is the `LexicalClosure` arm below applied one form
                    // over, and it differs in exactly one thing: the capture
                    // MATERIAL. A `Closure`'s captures are seed symbols
                    // resolved out of the explicit seed environment, so they go
                    // through `lower_seed_capture` and never through
                    // `child_occurrence`/`lower_expr`. That is the same
                    // `Seed`/`Lexical` split the declaration-body arm already
                    // makes; this reaches it from call position.
                    //
                    // Deliberately NOT routed through `Lowered::
                    // DeclarationClosure` below: a literal callee has no
                    // declaration reference, no symbol identity and no
                    // checked-call template, so there is nothing for the
                    // identity join to validate. The unit transport is reused;
                    // the declaration-specific identity route is not.
                    //
                    // No `Constructor`/`Record` capture environment is
                    // synthesized and no aggregate is allocated, so the
                    // campaign doc's Trap 5 preflight obligation is VACUOUS
                    // here. Recorded rather than discharged, and no token is
                    // minted -- a per-unit token would manufacture state this
                    // mechanism does not have.
                    if let RuntimeExpr::Closure {
                        captures,
                        params,
                        body,
                    } = callee.expr
                    {
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "Call",
                                format!(
                                    "closure expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        let mut inputs = args
                            .iter()
                            .enumerate()
                            .map(|(position, argument)| {
                                let argument = self.child_occurrence(
                                    static_origin,
                                    1 + position,
                                    argument,
                                )?;
                                let lowered = self.lower_expr(builder, argument, env)?;
                                self.carry_call_input(builder, argument.static_origin, lowered)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let closure_origin = callee.static_origin;
                        // Capture SOURCE order, appended after the full
                        // parameter run: the callee unit's ABI inputs are
                        // `Parameter ++ Capture`, and the two runs are not
                        // interchangeable even when their arities coincide.
                        inputs.extend(
                            captures
                                .iter()
                                .map(|capture| {
                                    // Seed captures resolve to JIT-time ground
                                    // values, so this asserts the phase rather
                                    // than preserving one. There is no carried
                                    // seed capture to lose, which is why these
                                    // do not go through `carry_call_input`.
                                    self.lower_seed_capture(builder, capture)
                                        .map(LoweringOperand::Specialized)
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        // The planner plans the body FIRST, so the body is the
                        // closure's child `0`. A `Closure`'s captures are
                        // symbols rather than expressions, so unlike the
                        // lexical form they occupy no child slots at all.
                        let body = self
                            .child_occurrence(closure_origin, 0, body)?
                            .static_origin;
                        // Counted at the HANDOFF point, AFTER arity and every
                        // capture have resolved: an arity or capture refusal
                        // leaves this at zero, so the count means "this arm
                        // resolved its inputs and handed them to the typed call
                        // path" rather than "the arm was entered".
                        //
                        // It does NOT mean a call instruction exists. The
                        // transport below can still refuse, so pair the count
                        // with the run's outcome before claiming emission.
                        #[cfg(test)]
                        SEED_CALLEE_UNIT_PORTS.with(|calls| calls.set(calls.get() + 1));
                        return self.call_declared_unit(
                            builder,
                            body,
                            &inputs,
                            #[cfg(test)]
                            None,
                        );
                    }
                    if let RuntimeExpr::LexicalClosure {
                        captures,
                        params,
                        body,
                    } = callee.expr
                    {
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "Call",
                                format!(
                                    "closure expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        // ⭐ **Each input crosses the boundary at ITS OWN
                        // caller-side occurrence.** The exact origins are
                        // already issued two lines up — `child_occurrence` for
                        // every argument and every capture — and the
                        // predecessor threw them away, leaving
                        // `call_declared_unit_target` to transfer whatever
                        // arrived specialized at `target.origin`, the callee's
                        // scheduling entry. Nothing new is derived here and no
                        // body is searched; the answer the planner already gave
                        // is simply carried through to the transfer.
                        let mut inputs = args
                            .iter()
                            .enumerate()
                            .map(|(position, argument)| {
                                let argument = self.child_occurrence(
                                    static_origin,
                                    1 + position,
                                    argument,
                                )?;
                                let lowered = self.lower_expr(builder, argument, env)?;
                                self.carry_call_input(builder, argument.static_origin, lowered)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let closure_origin = callee.static_origin;
                        inputs.extend(
                            captures
                                .iter()
                                .enumerate()
                                .map(|(position, capture)| {
                                    let capture = self.child_occurrence(
                                        closure_origin,
                                        1 + position,
                                        capture,
                                    )?;
                                    let lowered = self.lower_expr(builder, capture, env)?;
                                    self.carry_call_input(builder, capture.static_origin, lowered)
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        let body = self
                            .child_occurrence(closure_origin, 0, body)?
                            .static_origin;
                        return self.call_declared_unit(
                            builder,
                            body,
                            &inputs,
                            #[cfg(test)]
                            None,
                        );
                    }
                }
                let lowered_callee = self.lower_expr(builder, callee, env)?;
                match lowered_callee {
                    LoweringOperand::Specialized(Lowered::DeclarationClosure {
                        reference,
                        symbol,
                        captures,
                        params,
                        body,
                    }) => {
                        // `RT-DECL-CLOSURE-PORT` `D4`, consumer 1 of 3 -- the
                        // ordinary lowering route.
                        if self.body_emission_authority
                            == BodyEmissionAuthority::FunctionizedUnits
                        {
                            let args = args
                                .iter()
                                .enumerate()
                                .map(|(position, argument)| {
                                    let argument = self.child_occurrence(
                                        static_origin,
                                        1 + position,
                                        argument,
                                    )?;
                                    self.lower_expr(builder, argument, env)
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            return self.call_declaration_closure_unit(
                                builder, reference, &symbol, &params, captures, args,
                            );
                        }
                        self.lower_recursive_declaration_call(
                            builder,
                            &symbol,
                            &captures,
                            &params,
                            self.retained_body_occurrence(body)?,
                            args,
                            static_origin,
                            env,
                            None,
                            join_plan,
                        )
                    }
                    LoweringOperand::Specialized(Lowered::Closure {
                        captures,
                        params,
                        body,
                    }) => {
                        let mut call_inputs = args
                            .iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                let lowered = self.lower_expr(builder, arg, env)?;
                                match self.body_emission_authority {
                                    BodyEmissionAuthority::RecursiveDescent => Ok(lowered),
                                    BodyEmissionAuthority::FunctionizedUnits => {
                                        Ok(match lowered {
                                            LoweringOperand::Carried(word) => {
                                                LoweringOperand::Carried(word)
                                            }
                                            LoweringOperand::Specialized(value) => {
                                                LoweringOperand::Carried(
                                                    self.transfer_into_carrier(
                                                        builder,
                                                        arg.static_origin,
                                                        &value,
                                                    )?,
                                                )
                                            }
                                        })
                                    }
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        if params.len() != call_inputs.len() {
                            return Err(unsupported(
                                "Call",
                                format!(
                                    "closure expects {} args but call provides {}",
                                    params.len(),
                                    call_inputs.len()
                                ),
                            ));
                        }
                        // Two roles, as above. A lexical closure's body sees
                        // exactly its arguments and captures, so the
                        // environment role is the bare installation with no
                        // enclosing spine behind it.
                        call_inputs.extend(captures);
                        match self.body_emission_authority {
                            BodyEmissionAuthority::RecursiveDescent => {
                                let call_env = bound_values(call_inputs);
                                let body = self.retained_body_occurrence(body)?;
                                self.lower_expr(builder, body, &call_env)
                            }
                            BodyEmissionAuthority::FunctionizedUnits => {
                                self.call_declared_unit(
                                    builder,
                                    body,
                                    &call_inputs,
                                    #[cfg(test)]
                                    None,
                                )
                            }
                        }
                    }
                    LoweringOperand::Specialized(
                        mut callee @ Lowered::ComputationalRecursorClosure { .. },
                    ) => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(&mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(
                            LoweringOperand::Specialized(callee),
                        );
                        let (activation, invocation) = boundary.expect(
                            "recursor closure carries an invocation segment",
                        );
                        let recursive_unit_body = invocation.recursive_unit_body;
                        // `D5a` checkpoint 4 step 1 — read the retained source
                        // coordinates BEFORE the segment is installed, beside the
                        // existing pre-move field read. Both are facts of the
                        // invocation, so both are taken while it is still in hand
                        // rather than reconstructed afterwards.
                        let carried_coordinates =
                            CarriedInvocationCoordinates::of(&invocation)?;
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let dynamic_splice_edges =
                            self.take_dynamic_splice_edges(&invocation)?;
                        let installed = compose_oriented_subcontinuation(
                            self.oriented_subcontinuation_plan.as_ref(),
                            checked_ih_invocation
                                .or_else(|| self.active_recursive_invocations.last().copied()),
                            activation,
                            invocation,
                            dynamic_splice_edges,
                        )?;
                        let mut frames = installed_oriented_eliminator_frames(&installed);
                        frames.push(EliminatorFrame::InvocationReturn);
                        // ⭐⭐ `AC-C4` — the carried residual on the direct
                        // `lower_expr` call route.
                        if let LoweringOperand::Carried(word) = base {
                            if let Some(body) = recursive_unit_body.filter(|_| {
                                matches!(
                                    self.body_emission_authority,
                                    BodyEmissionAuthority::FunctionizedUnits
                                )
                            }) {
                                let inputs = args
                                    .iter()
                                    .enumerate()
                                    .map(|(position, arg)| {
                                        let arg = self.child_occurrence(
                                            static_origin,
                                            1 + position,
                                            arg,
                                        )?;
                                        self.lower_expr(builder, arg, env)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                self.enter_oriented_semantic_region(installed.checked);
                                let coordinates = carried_coordinates;
                                let result = self
                                    .call_declared_recursive_position_unit(
                                        builder,
                                        body,
                                        &inputs,
                                        Some(coordinates),
                                    )
                                    .and_then(|value| {
                                        self.lower_computational_match_value_composed(
                                            builder,
                                            RoutedAnswer::direct(value),
                                            &frames,
                                        )
                                    });
                                self.leave_oriented_semantic_region(installed.checked);
                                return result;
                            }
                            Self::reject_carried_residual_arguments(args.len())?;
                            self.enter_oriented_semantic_region(installed.checked);
                            let result = self.lower_computational_match_value_composed(
                                builder,
                                RoutedAnswer::direct(LoweringOperand::Carried(word)),
                                &frames,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            return result;
                        }
                        let base =
                            base.specialized_at("a recursor residual in a direct call")?;
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            self.enter_oriented_semantic_region(installed.checked);
                            let result = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &frames,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            return result;
                        }
                        let Lowered::Closure {
                            captures,
                            params,
                            body,
                        } = base
                        else {
                            return Err(unsupported(
                                "ComputationalMatch",
                                "recursive constructor field is not a closure",
                            ));
                        };
                        let mut call_inputs = args
                            .iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                self.lower_expr(builder, arg, env)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        if params.len() != call_inputs.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "recursive field expects {} args but call provides {}",
                                    params.len(),
                                    call_inputs.len()
                                ),
                            ));
                        }
                        // Two roles, as above: ordered unit-call inputs, or an
                        // environment prefix. Only the second is bound.
                        call_inputs.extend(captures);
                        if matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::FunctionizedUnits
                        ) {
                            self.enter_oriented_semantic_region(installed.checked);
                            let coordinates = carried_coordinates;
                            let result = self
                                .call_declared_recursive_position_unit(
                                    builder,
                                    body,
                                    &call_inputs,
                                    Some(coordinates),
                                )
                                .and_then(|value| {
                                    self.lower_computational_match_value_composed(
                                        builder,
                                        RoutedAnswer::direct(value),
                                        &frames,
                                    )
                                });
                            self.leave_oriented_semantic_region(installed.checked);
                            return result;
                        }
                        let call_env = env_with_operands(call_inputs, env);
                        self.enter_oriented_semantic_region(installed.checked);
                        let result = self.lower_computational_producer_expr(
                            builder,
                            self.retained_body_occurrence(body)?,
                            &call_env,
                            &frames,
                        );
                        self.leave_oriented_semantic_region(installed.checked);
                        result
                    }
                    _ => Err(unsupported("Call", "callee is not a closure")),
                }
            }
            RuntimeExpr::Trap(trap) => Ok(LoweringOperand::Specialized(Lowered::Trap(trap.clone()))),
            // ⚠ HAZARD 2 (D3): the capability occupies child position `0` **only
            // when present**, so the argument base is `1` with a capability and
            // `0` without it. A constant base would mis-key every argument of
            // every capability-carrying effect, and nothing in the types would
            // notice.
            RuntimeExpr::Effect {
                family,
                operation,
                capability,
                args,
            } if self.process_object => self.lower_process_host_effect(
                builder,
                family,
                *operation,
                capability.as_ref(),
                args,
                static_origin,
                env,
            ),
            RuntimeExpr::Effect { family, operation, .. } => Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is not modeled in the supported native subset",
                    *operation as u16
                ),
            )),
        }
    }

    fn lower_buffer_freeze_resource_seat(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        operand: &LoweringOperand,
        seat: &'static str,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        match operand {
            LoweringOperand::Specialized(Lowered::ResourceToken { value }) => Ok(*value),
            LoweringOperand::Specialized(_) => Err(unsupported(
                "Effect",
                format!("BufferFreeze {seat} is not a resource"),
            )),
            LoweringOperand::Carried(word) => {
                let tag = self.emit_carrier_tag(builder, *word)?;
                Self::require_i64(
                    builder,
                    tag,
                    crate::boundary_value::BoundaryTag::InvocationBorrowed as i64,
                );
                let class = self.emit_carrier_class(builder, *word)?;
                Self::require_i64(builder, class, BoundaryClass::BorrowedOpaque as i64);
                self.emit_carrier_scalar(builder, *word)
            }
        }
    }

    /// `static_origin` is the `Effect` occurrence's own origin.
    ///
    /// ⚠ HAZARD 2 (D3): the planner plans `capability.value` **first when it is
    /// present**, so the argument base is `1` with a capability and `0` without
    /// one. `argument_base` below is that conditional offset, computed once from
    /// the same `Option` the planner tested (`static_transition.rs` `Effect`
    /// arm) rather than assumed.
    #[allow(clippy::too_many_arguments)]
    fn lower_process_host_effect(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        family: &RuntimeSymbol,
        operation: ken_host::HostOpV1,
        capability: Option<&crate::RuntimeCapabilityUse>,
        args: &[RuntimeExpr],
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        if !CRANELIFT_HOST_EFFECT_CONSUMERS_V1.contains(&operation) {
            return Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is a represented unavailable lane",
                    operation as u16
                ),
            ));
        }
        let argument_base = usize::from(capability.is_some());
        let lowered = args
            .iter()
            .enumerate()
            .map(|(position, argument)| {
                let argument =
                    self.child_occurrence(static_origin, argument_base + position, argument)?;
                self.lower_expr(builder, argument, env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // ⛔ The capability is lowered HERE rather than inside the FS arm, and
        // the reason is the claim group's window. An operand's own lowering can
        // visit a nested effect, so every operand must be in hand before the
        // group opens — a capability lowered inside the arm would lower while
        // this occurrence's group was open and its nested claims would land in
        // the wrong visit.
        let capability_operand = match capability {
            Some(capability) => {
                // Present ⇒ the capability value is child 0 of this occurrence.
                let value = self.child_occurrence(static_origin, 0, &capability.value)?;
                Some(self.lower_expr(builder, value, env)?)
            }
            None => None,
        };
        // ⭐ `D7` — ONE claim group per compiler-side visit to this occurrence,
        // opened before any seat of it is observed and closed before dispatch.
        // Completeness is asked of this visit alone: a ledger accumulating
        // claims per occurrence would accept two visits that each read half the
        // seats, and two half-reads are the defect.
        let group = self.open_host_effect_seat_group(static_origin, operation)?;
        #[cfg(test)]
        let omitted = match effect_seat_visit_mutation() {
            // Alternate which slot this visit drops, so successive visits'
            // omissions are complementary and their union is complete.
            EffectSeatVisitMutation::OmitComplementary => {
                let slots = self
                    .static_transition_plan
                    .host_effect_seat_slots(static_origin)
                    .into_iter()
                    .collect::<Vec<_>>();
                let index = effect_seat_next_visit_index();
                slots.get(index % slots.len().max(1)).copied()
            }
            _ => None,
        };
        #[cfg(not(test))]
        let omitted: Option<EffectSeatSlot> = None;
        let mut claimed = BTreeMap::new();
        let mut claim = |lowering: &mut Self,
                         claimed: &mut BTreeMap<EffectSeatSlot, PlannedEffectSeat>,
                         slot,
                         operand: &LoweringOperand| {
            if omitted == Some(slot) {
                return Ok(());
            }
            let record = lowering.claim_host_effect_seat(group, static_origin, slot, operand)?;
            claimed.insert(slot, record);
            Ok(())
        };
        if let Some(operand) = &capability_operand {
            claim(self, &mut claimed, EffectSeatSlot::Capability, operand)?;
        }
        for (ordinal, operand) in lowered.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).map_err(|_| {
                unsupported("Effect", "host effect argument ordinal exceeds the seat space")
            })?;
            claim(self, &mut claimed, EffectSeatSlot::Argument(ordinal), operand)?;
        }
        #[cfg(test)]
        if effect_seat_visit_mutation() == EffectSeatVisitMutation::DuplicateWithinVisit {
            if let Some(operand) = lowered.first() {
                claim(self, &mut claimed, EffectSeatSlot::Argument(0), operand)?;
            }
        }
        self.close_host_effect_seat_group(group)?;
        // ⭐ **The bulk pre-operation conversion is gone.** It crossed every
        // operand to a specialized template BEFORE the operation was known, so a
        // seat that could not be read that way failed as "a host-effect operand"
        // -- naming neither the operation nor the seat, and answering for seats
        // the arm never reads. Each arm now reads its own claimed seats.
        let seats = ClaimedEffectSeats {
            claimed: &claimed,
            capability: capability_operand.as_ref(),
            arguments: &lowered,
        };
        // The four ordinals any arm below reads, named once. ⛔ These are
        // SEMANTIC ordinals: the capability offset is already applied, so
        // `SEAT_0` is the first semantic argument whether or not the operation
        // carries a capability.
        const SEAT_0: EffectSeatSlot = EffectSeatSlot::Argument(0);
        const SEAT_1: EffectSeatSlot = EffectSeatSlot::Argument(1);
        const SEAT_2: EffectSeatSlot = EffectSeatSlot::Argument(2);
        const SEAT_3: EffectSeatSlot = EffectSeatSlot::Argument(3);
        let pointer_type = builder.func.dfg.value_type(
            self.function_local
                .host_dispatch_context
                .expect("process effect lowering owns a direct host context"),
        );
        let wire = ken_host::host_effect_wire_layout_v1(operation).map_err(|error| {
            unsupported(
                "Effect",
                format!("generated HostEffectAbiV1 layout rejected: {error:?}"),
            )
        })?;
        let request_offset = |index: usize| {
            i32::try_from(wire.request_offsets[index])
                .expect("C-probed request offset was checked as u32")
        };
        let request = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            wire.request_size,
            wire.request_align_shift,
        ));
        // The two reply surfaces a synthesized pre-dispatch failure can land on,
        // materialized once so no arm re-derives one.
        let resource_error_reply_tag = builder
            .ins()
            .iconst(types::I64, wire.reply_resource_error_tag as i64);
        let error_reply_tag = builder
            .ins()
            .iconst(types::I64, wire.reply_error_tag as i64);
        // Wrap a `ResourceErrorV1` code as `IOError::Other <code>`, in the exact
        // encoding `ken_host::abi_v1::io_error_tag` uses: the payload in the
        // high 32 bits, discriminator `11` in the low byte, which the decoder
        // below recovers with `sshr_imm(detail, 32)`.
        //
        // `ConsoleWrite` and `FsWriteFile` declare `IOError` surfaces, so their
        // `detail` is read as an `IOError` discriminator. Handing them a RAW
        // resource code would silently reinterpret `1` and `7` as
        // `PermissionDenied` and `IsDirectory` — two real, wrong errors. `Other`
        // is the one variant that carries an integer whose meaning is the
        // payload rather than the discriminator, which is why the observer's
        // refusal can be represented on this surface at all without inventing a
        // constructor.
        let io_error_other_detail =
            |builder: &mut FunctionBuilder<'_>, code: cranelift_codegen::ir::Value| {
                let payload = builder.ins().ishl_imm(code, 32);
                builder
                    .ins()
                    .bor_imm(payload, IO_ERROR_OTHER_DISCRIMINATOR)
            };
        // `(invalid, reply tag, detail)`. The TAG is carried rather than assumed
        // because a synthesized pre-dispatch failure must land on the surface
        // the operation actually declares: the resource operations accept
        // `reply_resource_error_tag`, and `ConsoleWrite` / `FsWriteFile` accept
        // only `success` or `reply_error_tag`. Writing the wrong one is not a
        // mis-labelled error — `require_one_of_i64` refuses the reply outright
        // and the whole compiled function fails generically, which is the defect
        // this triple exists to make unspellable.
        let mut narrow_failure: Option<(
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        )> = None;
        let mut positioned_bounds: Option<(
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        )> = None;
        // `RT-CARRIER-BYTESPAN-OBSERVE` `D5` — `detail` is a VALUE rather than a
        // constant, because a byte-span refusal chooses its `ResourceErrorV1`
        // code at run time from the observer's outcome. Every pre-existing
        // caller still passes a constant and is unchanged in meaning.
        let mut record_narrow_failure = |builder: &mut FunctionBuilder<'_>,
                                         invalid,
                                         tag: cranelift_codegen::ir::Value,
                                         detail: cranelift_codegen::ir::Value| {
            narrow_failure = Some(match narrow_failure.take() {
                Some((prior_invalid, prior_tag, prior_detail)) => (
                    builder.ins().bor(prior_invalid, invalid),
                    builder.ins().select(prior_invalid, prior_tag, tag),
                    builder.ins().select(prior_invalid, prior_detail, detail),
                ),
                None => (invalid, tag, detail),
            });
        };
        match operation {
            ken_host::HostOpV1::ConsoleWrite
            | ken_host::HostOpV1::ConsoleFlush
            | ken_host::HostOpV1::ConsoleIsTerminal => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "ambient Console carried a capability",
                    ));
                }
                let stream = console_stream_tag(seats.specialized(SEAT_0)?)
                    .ok_or_else(|| {
                        unsupported("Effect", "Console operation has a malformed Stream operand")
                    })?;
                let stream = builder.ins().iconst(types::I64, stream);
                builder
                    .ins()
                    .stack_store(stream, request, request_offset(0));
                if operation == ken_host::HostOpV1::ConsoleWrite {
                    let span = self.wire_bytes_seat(builder, &seats, SEAT_1)?;
                    if let Some((invalid, resource_code)) = span.refusal {
                        let detail = io_error_other_detail(builder, resource_code);
                        record_narrow_failure(builder, invalid, error_reply_tag, detail);
                    }
                    builder
                        .ins()
                        .stack_store(span.pointer, request, request_offset(1));
                    builder
                        .ins()
                        .stack_store(span.len, request, request_offset(2));
                }
            }
            ken_host::HostOpV1::FsReadFile
            | ken_host::HostOpV1::FsWriteFile
            | ken_host::HostOpV1::FsChangeMode
            | ken_host::HostOpV1::FsOpen => {
                // Lowered and claimed above, with every other operand, so the
                // claim group's window contains no operand lowering.
                let (capability_record, capability_operand) =
                    seats.operand(EffectSeatSlot::Capability)?;
                // ⛔ Exhaustive over both phases with no wildcard. The capability
                // is an either-phase seat: a specialized `CapabilityToken`
                // template is read directly, a carried word through the emitted
                // scalar read. A specialized template that is neither is the
                // third case, and it names the seat rather than falling into a
                // catch-all.
                let token = match capability_operand {
                    LoweringOperand::Specialized(Lowered::CapabilityToken { value }) => *value,
                    LoweringOperand::Carried(word) => self.emit_carrier_scalar(builder, *word)?,
                    LoweringOperand::Specialized(other) => {
                        return Err(unsupported(
                            "Effect",
                            format!(
                                "seat {:?} of {:?} needs {:?}, but the specialized template at \
                                 it is a {} rather than the opaque invocation token",
                                capability_record.slot,
                                capability_record.operation,
                                capability_record.need,
                                lowered_value_kind(other)
                            ),
                        ));
                    }
                };
                builder.ins().stack_store(token, request, request_offset(0));
                let path = self.wire_bytes_seat(builder, &seats, SEAT_0)?;
                if let Some((invalid, resource_code)) = path.refusal {
                    let detail = io_error_other_detail(builder, resource_code);
                    record_narrow_failure(builder, invalid, error_reply_tag, detail);
                }
                builder
                    .ins()
                    .stack_store(path.pointer, request, request_offset(1));
                builder
                    .ins()
                    .stack_store(path.len, request, request_offset(2));
                if operation == ken_host::HostOpV1::FsWriteFile {
                    let policy = create_policy_tag(seats.specialized(SEAT_1)?).ok_or_else(|| {
                        unsupported("Effect", "FS.WriteFile has a malformed CreatePolicy")
                    })?;
                    let contents = self.wire_bytes_seat(builder, &seats, SEAT_2)?;
                    if let Some((invalid, resource_code)) = contents.refusal {
                        let detail = io_error_other_detail(builder, resource_code);
                        record_narrow_failure(builder, invalid, error_reply_tag, detail);
                    }
                    let policy = builder.ins().iconst(types::I64, policy);
                    builder
                        .ins()
                        .stack_store(policy, request, request_offset(3));
                    builder
                        .ins()
                        .stack_store(contents.pointer, request, request_offset(4));
                    builder
                        .ins()
                        .stack_store(contents.len, request, request_offset(5));
                } else if operation == ken_host::HostOpV1::FsChangeMode {
                    let mode = seats.specialized(SEAT_1)?;
                    let (mode, valid_int) = self.narrow_native_int_u64(builder, mode)?;
                    let in_range = builder.ins().icmp_imm(
                        cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
                        mode,
                        0o7777,
                    );
                    let in_range = builder.ins().band(valid_int, in_range);
                    let narrowed = builder.ins().ireduce(types::I16, mode);
                    let invalid = builder.ins().iconst(types::I16, 0xffff);
                    let mode = builder.ins().select(in_range, narrowed, invalid);
                    builder.ins().stack_store(mode, request, request_offset(3));
                } else if operation == ken_host::HostOpV1::FsOpen {
                    let mode = resource_open_mode_tag(seats.specialized(SEAT_1)?)
                        .ok_or_else(|| {
                            unsupported("Effect", "FS.Open has a malformed ResourceOpenMode")
                        })?;
                    let mode = builder.ins().iconst(types::I64, mode);
                    builder.ins().stack_store(mode, request, request_offset(3));
                }
            }
            ken_host::HostOpV1::FsHandleMetadata | ken_host::HostOpV1::ResourceRelease => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "resource operation carried a capability",
                    ));
                }
                let Lowered::ResourceToken { value: token } = seats.specialized(SEAT_0)? else {
                    return Err(unsupported(
                        "Effect",
                        "resource operand is not an opaque resource token",
                    ));
                };
                builder
                    .ins()
                    .stack_store(*token, request, request_offset(0));
            }
            ken_host::HostOpV1::BufferAllocate => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "buffer allocation carried a capability",
                    ));
                }
                // ⭐ **The one seat whose `Avail` admits BOTH phases and now
                // has a route for each.** The claim is consumed here and the
                // observed phase selects the decoder, so the dispatch is bound
                // to the exact operand this arm reads rather than to a
                // conversion performed on its behalf.
                //
                // ⛔ Exhaustive, no wildcard. Both arms return `(u64, valid)`
                // and feed the SAME `InvalidBounds` lane below, so a capacity
                // that does not fit is one outcome regardless of how it
                // arrived — which is the point: the phase is a fact about how
                // the value reached the seat, never about what the program
                // means.
                // ⛔ The mutation deletes the CARRIED arm, leaving the
                // specialized read the whole route -- the exact state that
                // produced the `264 -> 262 / position 1` refusal the frame
                // names.
                #[cfg(test)]
                let carried_arm_removed = effect_seat_dispatch_mutation()
                    == EffectSeatDispatchMutation::RemoveCarriedCapacityArm;
                #[cfg(not(test))]
                let carried_arm_removed = false;
                let (_, capacity_operand) = seats.operand(SEAT_0)?;
                let (capacity, valid) = if carried_arm_removed {
                    let capacity = seats.specialized(SEAT_0)?.clone();
                    Self::record_capacity_phase_dispatch(false);
                    self.narrow_native_int_u64(builder, &capacity)?
                } else {
                    match capacity_operand {
                        LoweringOperand::Specialized(lowered) => {
                            let lowered = lowered.clone();
                            Self::record_capacity_phase_dispatch(false);
                            self.narrow_native_int_u64(builder, &lowered)?
                        }
                        LoweringOperand::Carried(word) => {
                            let word = *word;
                            Self::record_capacity_phase_dispatch(true);
                            self.narrow_carried_int_u64(builder, word)?
                        }
                    }
                };
                let invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    valid,
                    0,
                );
                let detail = builder.ins().iconst(types::I64, RESOURCE_ERROR_INVALID_BOUNDS);
                record_narrow_failure(builder, invalid, resource_error_reply_tag, detail);
                builder
                    .ins()
                    .stack_store(capacity, request, request_offset(0));
            }
            ken_host::HostOpV1::BufferFreeze => {
                if capability.is_some() {
                    return Err(unsupported("Effect", "BufferFreeze carried a capability"));
                }
                let token = self.lower_buffer_freeze_resource_seat(
                    builder,
                    seats.operand(SEAT_0)?.1,
                    "buffer",
                )?;
                let start = seats.specialized(SEAT_1)?;
                let length = seats.specialized(SEAT_2)?;
                let (start, start_valid) = self.narrow_native_int_u64(builder, start)?;
                let (length, length_valid) = self.narrow_native_int_u64(builder, length)?;
                let valid = builder.ins().band(start_valid, length_valid);
                let invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    valid,
                    0,
                );
                let detail = builder.ins().iconst(types::I64, RESOURCE_ERROR_INVALID_BOUNDS);
                record_narrow_failure(builder, invalid, resource_error_reply_tag, detail);
                // PX8-SPAN-PROV: trailing `span_origin` acquisition token.
                let span_origin = self.lower_buffer_freeze_resource_seat(
                    builder,
                    seats.operand(SEAT_3)?.1,
                    "span origin",
                )?;
                for (index, value) in [token, start, length, span_origin].into_iter().enumerate() {
                    builder
                        .ins()
                        .stack_store(value, request, request_offset(index));
                }
            }
            ken_host::HostOpV1::FsReadAt | ken_host::HostOpV1::FsWriteAt => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "positioned resource operation carried a capability",
                    ));
                }
                let resource = |index: usize, name: &str| {
                    let Some(Lowered::ResourceToken { value }) =
                        seats.specialized(EffectSeatSlot::Argument(index as u32)).ok()
                    else {
                        return Err(unsupported(
                            "Effect",
                            format!("positioned {name} operand is not a resource"),
                        ));
                    };
                    Ok(*value)
                };
                let integer = |index: usize, name: &str| {
                    let Some(value @ Lowered::Int { .. }) =
                        seats.specialized(EffectSeatSlot::Argument(index as u32)).ok()
                    else {
                        return Err(unsupported(
                            "Effect",
                            format!("positioned {name} operand is not Int"),
                        ));
                    };
                    Ok(value)
                };
                let file = resource(0, "file")?;
                let (file_offset, file_offset_valid) =
                    self.narrow_native_int_u64(builder, integer(1, "file offset")?)?;
                let buffer = resource(2, "buffer")?;
                let (buffer_start, buffer_start_valid) =
                    self.narrow_native_int_u64(builder, integer(3, "buffer start")?)?;
                let (length, length_valid) =
                    self.narrow_native_int_u64(builder, integer(4, "length")?)?;
                positioned_bounds = Some((buffer_start, length));
                let file_offset_invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    file_offset_valid,
                    0,
                );
                let detail = builder.ins().iconst(types::I64, RESOURCE_ERROR_INVALID_OFFSET);
                record_narrow_failure(builder, file_offset_invalid, resource_error_reply_tag, detail);
                let bounds_valid = builder.ins().band(buffer_start_valid, length_valid);
                let bounds_invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    bounds_valid,
                    0,
                );
                let detail = builder.ins().iconst(types::I64, RESOURCE_ERROR_INVALID_BOUNDS);
                record_narrow_failure(builder, bounds_invalid, resource_error_reply_tag, detail);
                if operation == ken_host::HostOpV1::FsWriteAt {
                    // PX8-SPAN-PROV: `FsWriteAt` carries the trailing
                    // `span_origin` acquisition token; `FsReadAt` mints the span
                    // and has no origin operand.
                    let span_origin = resource(5, "span origin")?;
                    for (index, value) in
                        [file, buffer, file_offset, buffer_start, length, span_origin]
                            .into_iter()
                            .enumerate()
                    {
                        builder
                            .ins()
                            .stack_store(value, request, request_offset(index));
                    }
                } else {
                    for (index, value) in [file, buffer, file_offset, buffer_start, length]
                        .into_iter()
                        .enumerate()
                    {
                        builder
                            .ins()
                            .stack_store(value, request, request_offset(index));
                    }
                }
            }
            _ => unreachable!("availability was checked above"),
        }
        let reply = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            wire.reply_size,
            wire.reply_align_shift,
        ));
        let host_context = self
            .function_local
            .host_dispatch_context
            .expect("process effect lowering owns a direct host context");
        let op = builder.ins().iconst(types::I64, operation as i64);
        let request_pointer = builder.ins().stack_addr(pointer_type, request, 0);
        let request_size = builder
            .ins()
            .iconst(types::I64, i64::from(wire.request_size));
        let reply_pointer = builder.ins().stack_addr(pointer_type, reply, 0);
        if let Some((invalid, failure_tag, detail)) = narrow_failure {
            let dispatch = builder.create_block();
            let synthesize = builder.create_block();
            let decoded = builder.create_block();
            builder.ins().brif(invalid, synthesize, &[], dispatch, &[]);

            builder.switch_to_block(dispatch);
            let call = builder.ins().call(
                self.function_local
                    .host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[
                    host_context,
                    op,
                    request_pointer,
                    request_size,
                    reply_pointer,
                ],
            );
            let status = builder.inst_results(call)[0];
            Self::require_i64(builder, status, 0);
            builder.ins().jump(decoded, &[]);

            builder.switch_to_block(synthesize);
            let zero = builder.ins().iconst(types::I64, 0);
            for offset in [
                wire.reply_resource_error_schema_offset,
                wire.reply_resource_error_kind_offset,
                wire.reply_resource_error_identity_offset,
                wire.reply_resource_error_io_offset,
                wire.reply_resource_error_required_offset,
                wire.reply_resource_error_held_offset,
                wire.reply_resource_error_expected_kind_offset,
                wire.reply_resource_error_actual_kind_offset,
                wire.reply_bytes_data_offset,
                wire.reply_bytes_len_offset,
                wire.reply_effective_request_offset,
            ] {
                builder.ins().stack_store(
                    zero,
                    reply,
                    i32::try_from(offset).expect("reply field offset is u32"),
                );
            }
            // The tag comes from whoever recorded the failure, because only they
            // know which surface this operation declares. Hardcoding the
            // resource-error tag here is what made a byte-span refusal on
            // `ConsoleWrite` / `FsWriteFile` fail `require_one_of_i64` below and
            // collapse into the generic compiled-function failure instead of
            // reaching Ken as a value.
            builder.ins().stack_store(
                failure_tag,
                reply,
                i32::try_from(wire.reply_tag_offset).expect("reply tag offset is u32"),
            );
            builder.ins().stack_store(
                detail,
                reply,
                i32::try_from(wire.reply_detail_offset).expect("reply detail offset is u32"),
            );
            builder.ins().jump(decoded, &[]);
            builder.switch_to_block(decoded);
        } else {
            let call = builder.ins().call(
                self.function_local
                    .host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[
                    host_context,
                    op,
                    request_pointer,
                    request_size,
                    reply_pointer,
                ],
            );
            let status = builder.inst_results(call)[0];
            Self::require_i64(builder, status, 0);
        }
        let tag = builder.ins().stack_load(
            types::I64,
            reply,
            i32::try_from(wire.reply_tag_offset).expect("reply tag offset is u32"),
        );
        let detail = builder.ins().stack_load(
            types::I64,
            reply,
            i32::try_from(wire.reply_detail_offset).expect("reply detail offset is u32"),
        );
        if operation == ken_host::HostOpV1::ConsoleIsTerminal {
            Self::require_i64(builder, tag, wire.reply_bool_tag as i64);
            Ok(LoweringOperand::Specialized(Lowered::Bool {
                value: detail,
                known: None,
            }))
        } else {
            let success_tag = match operation {
                ken_host::HostOpV1::FsReadFile => wire.reply_bytes_tag,
                ken_host::HostOpV1::FsOpen => wire.reply_resource_tag,
                ken_host::HostOpV1::FsHandleMetadata => wire.reply_metadata_tag,
                ken_host::HostOpV1::BufferAllocate => wire.reply_resource_tag,
                ken_host::HostOpV1::BufferFreeze => wire.reply_bytes_tag,
                ken_host::HostOpV1::FsReadAt => wire.reply_read_progress_tag,
                ken_host::HostOpV1::FsWriteAt => wire.reply_write_progress_tag,
                _ => wire.reply_unit_tag,
            } as i64;
            let accepted_tags = match operation {
                ken_host::HostOpV1::FsHandleMetadata => vec![
                    success_tag,
                    wire.reply_error_tag as i64,
                    wire.reply_resource_error_tag as i64,
                ],
                ken_host::HostOpV1::ResourceRelease => {
                    vec![success_tag, wire.reply_resource_error_tag as i64]
                }
                ken_host::HostOpV1::BufferAllocate | ken_host::HostOpV1::BufferFreeze => {
                    vec![success_tag, wire.reply_resource_error_tag as i64]
                }
                ken_host::HostOpV1::FsReadAt | ken_host::HostOpV1::FsWriteAt => vec![
                    success_tag,
                    wire.reply_error_tag as i64,
                    wire.reply_resource_error_tag as i64,
                ],
                _ => vec![success_tag, wire.reply_error_tag as i64],
            };
            Self::require_one_of_i64(builder, tag, &accepted_tags);
            let resource_schema = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_schema_offset)
                    .expect("resource error schema offset is u32"),
            );
            let resource_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_kind_offset)
                    .expect("resource error kind offset is u32"),
            );
            let resource_identity = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_identity_offset)
                    .expect("resource error identity offset is u32"),
            );
            let resource_io = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_io_offset)
                    .expect("resource error io offset is u32"),
            );
            let resource_required = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_required_offset)
                    .expect("resource error required offset is u32"),
            );
            let resource_held = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_held_offset)
                    .expect("resource error held offset is u32"),
            );
            let resource_expected_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_expected_kind_offset)
                    .expect("resource error expected-kind offset is u32"),
            );
            let resource_actual_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_actual_kind_offset)
                    .expect("resource error actual-kind offset is u32"),
            );
            Self::validate_resource_error_reply(
                builder,
                tag,
                wire.reply_resource_error_tag,
                detail,
                resource_schema,
                resource_kind,
                resource_identity,
                resource_io,
                resource_required,
                resource_held,
                resource_expected_kind,
                resource_actual_kind,
                wire.resource_error_reply_schema,
                wire.resource_kind_fs_handle,
                wire.resource_kind_buffer,
            );
            let payload = builder.ins().sshr_imm(detail, 32);
            let payload_int = self.lower_dynamic_small_int(builder, payload);
            // `D7` — the two roots of this operation's synthesized aggregate
            // trees. Every synthesized producer below states its own path from
            // one of them explicitly; nothing is rebound as the walk descends,
            // so a path in this file names exactly one node in the planner's
            // tree and the two can be compared.
            // ⭐ **The claim-backed view itself is what reply synthesis
            // carries** -- positional semantics, projected LAZILY.
            //
            // ⛔ A dense `Vec<Lowered>` used to be realized here by demanding a
            // specialized template for every argument the operation has. That
            // was the removed pre-operation bulk conversion RELOCATED after
            // dispatch: knowing the operation narrowed the diagnostic but did
            // not authorize reading a seat no synthesized node uses. Only two
            // site-bound children exist in the measured trees -- the file-error
            // path's `SiteOperand(0)` and `FsReadAt`'s private-buffer-span
            // `SiteOperand(2)` -- so an operation with no `SiteOperand` child
            // must project no template at all. `BufferAllocate`'s carried
            // capacity is the case that made this load-bearing: its own arm
            // consumes the seat, and the dense vector then refused it again on
            // behalf of a consumer that never wanted it.
            //
            // ⚠ The `BufferFreeze` special case is gone rather than moved. It
            // existed only to keep the dense realization off an operation whose
            // seats are not templates; with the projection driven by declared
            // uses, an operation with no site-bound child asks for nothing
            // without needing to be named.
            //
            // ⛔ The mutation puts the dense realization back, exactly as it
            // stood: every argument demanded as a template, before any declared
            // use has said which ones it wants.
            #[cfg(test)]
            if effect_seat_dispatch_mutation() == EffectSeatDispatchMutation::RestoreBulkConversion
                && operation != ken_host::HostOpV1::BufferFreeze
            {
                for ordinal in 0..lowered.len() as u32 {
                    let _ = seats.specialized(EffectSeatSlot::Argument(ordinal))?;
                }
            }
            let error_root =
                SynthesizedAggregatePath::root(SynthesizedAggregateRoot::HostResultError);
            let ok_root = SynthesizedAggregatePath::root(SynthesizedAggregateRoot::HostResultOk);
            // ⭐ `D7` — the generic `IOError` value is built **per branch, at
            // the path that branch puts it at**, not once up front.
            //
            // ⛔ It used to be constructed eagerly, before the operation match
            // knew which arm it was in. The six resource-surface operations
            // build their own `surface_io_error` and never referenced it, so
            // that template was ABANDONED — an allocation-shaped value with no
            // semantic use. Now it is never built for them at all, which is
            // strictly better than planning it and proving it unreachable: it
            // contributes neither a record nor an event because it does not
            // exist.
            let generic_io_error =
                |this: &Self,
                 builder: &mut FunctionBuilder<'_>,
                 payload: Lowered,
                 node: &SynthesizedAggregatePath| {
                    Ok::<_, CraneliftBackendError>(Lowered::DynamicConstructor(
                        DynamicConstructorV1 {
                            discriminator: builder.ins().band_imm(detail, 0xff),
                            alternatives: this.synthesized_io_error_alternatives(
                                static_origin,
                                node,
                                payload,
                                &seats,
                            )?,
                        },
                    ))
                };
            let error = if matches!(
                operation,
                ken_host::HostOpV1::FsReadFile
                    | ken_host::HostOpV1::FsWriteFile
                    | ken_host::HostOpV1::FsChangeMode
                    | ken_host::HostOpV1::FsOpen
            ) {
                // ⛔ No eager read of the path seat here. It used to be
                // demanded as a template and then dropped -- the same
                // unauthorized eager demand at a smaller scale. The one thing
                // that legitimately needs it is the `OptionSome` child below,
                // which projects it through `site_operand_argument` at its
                // declared `SiteOperand(0)`.
                let (operation_role, operation_symbol) = match operation {
                    ken_host::HostOpV1::FsReadFile | ken_host::HostOpV1::FsOpen => (
                        SynthesizedFixedConstructorRole::FileOperationRead,
                        self.process_symbols.file_operation_read.clone(),
                    ),
                    ken_host::HostOpV1::FsWriteFile => (
                        SynthesizedFixedConstructorRole::FileOperationWrite,
                        self.process_symbols.file_operation_write.clone(),
                    ),
                    ken_host::HostOpV1::FsChangeMode => (
                        SynthesizedFixedConstructorRole::FileOperationChangeMode,
                        self.process_symbols.file_operation_change_mode.clone(),
                    ),
                    _ => unreachable!("validated FS result operation"),
                };
                let operation = self.synthesized_constructor(
                    static_origin,
                    &error_root.field(0),
                    operation_role,
                    operation_symbol,
                    Vec::new(),
                    &seats,
                )?;
                let path = self.synthesized_constructor(
                    static_origin,
                    &error_root.field(1),
                    SynthesizedFixedConstructorRole::OptionSome,
                    self.process_symbols.option_some.clone(),
                    // The seat's operand 0 — projected, not passed.
                    vec![self.site_operand_argument(static_origin, 0, &seats)?],
                    &seats,
                )?;
                let io_error =
                    generic_io_error(self, builder, payload_int, &error_root.field(2))?;
                self.synthesized_constructor(
                    static_origin,
                    &error_root,
                    SynthesizedFixedConstructorRole::FileError,
                    self.process_symbols.file_error.clone(),
                    vec![
                        SynthesizedArgument::Nested(operation),
                        SynthesizedArgument::Nested(path),
                        SynthesizedArgument::Dynamic(io_error),
                    ],
                    &seats,
                )?
            } else if matches!(
                operation,
                ken_host::HostOpV1::FsHandleMetadata
                    | ken_host::HostOpV1::ResourceRelease
                    | ken_host::HostOpV1::BufferAllocate
                    | ken_host::HostOpV1::BufferFreeze
                    | ken_host::HostOpV1::FsReadAt
                    | ken_host::HostOpV1::FsWriteAt
            ) {
                let generic = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    tag,
                    wire.reply_error_tag as i64,
                );
                let zero = builder.ins().iconst(types::I64, 0);
                let resource_surface_tag = builder.ins().iadd_imm(detail, 1);
                let surface_tag = builder.ins().select(generic, zero, resource_surface_tag);
                let surface_io = builder.ins().select(generic, detail, resource_io);
                let surface_io_payload = builder.ins().sshr_imm(surface_io, 32);
                let surface_io_payload_int =
                    self.lower_dynamic_small_int(builder, surface_io_payload);
                let resource_required_int =
                    self.lower_unsigned_u64_int(builder, resource_required)?;
                let resource_held_int = self.lower_unsigned_u64_int(builder, resource_held)?;
                // ⭐ Built ONCE PER SEMANTIC USE. `ResourceHostIo` field 0 and
                // `ResourceReleaseFailed` field 2 are two allocations at two
                // paths; cloning one template into both would carry one
                // occurrence to two allocations, which is exactly the aliasing
                // the path key exists to prevent.
                let surface_io_error = |this: &Self,
                                        builder: &mut FunctionBuilder<'_>,
                                        node: &SynthesizedAggregatePath| {
                    Ok::<_, CraneliftBackendError>(Lowered::DynamicConstructor(
                        DynamicConstructorV1 {
                            discriminator: builder.ins().band_imm(surface_io, 0xff),
                            alternatives: this.synthesized_io_error_alternatives(
                                static_origin,
                                node,
                                surface_io_payload_int.clone(),
                                &seats,
                            )?,
                        },
                    ))
                };
                let identity_low = builder.ins().band_imm(resource_identity, 0xffff_ffff);
                let identity_high = builder.ins().ushr_imm(resource_identity, 32);
                let identity_low_int = self.lower_dynamic_small_int(builder, identity_low);
                let identity_high_int = self.lower_dynamic_small_int(builder, identity_high);
                // ⭐ `ResourceKind` is built THREE times at this one seat, so
                // the closure takes the path of the node it is building rather
                // than closing over one. Passing a path in is what makes the
                // three uses three occurrences; a closure that knew its own
                // path could only ever describe one of them.
                let resource_kind_value = |this: &Self,
                                           discriminator,
                                           node: &SynthesizedAggregatePath| {
                    Ok::<_, CraneliftBackendError>(Lowered::DynamicConstructor(
                        DynamicConstructorV1 {
                            discriminator,
                            alternatives: vec![
                                this.synthesized_dynamic_alternative(
                                    static_origin,
                                    node,
                                    0,
                                    wire.resource_kind_fs_handle as i64,
                                    SynthesizedFixedConstructorRole::ResourceKindFsHandle,
                                    this.process_symbols.resource_kind_fs_handle.clone(),
                                    Vec::new(),
                                    &seats,
                                )?,
                                this.synthesized_dynamic_alternative(
                                    static_origin,
                                    node,
                                    1,
                                    wire.resource_kind_buffer as i64,
                                    SynthesizedFixedConstructorRole::ResourceKindBuffer,
                                    this.process_symbols.resource_kind_buffer.clone(),
                                    Vec::new(),
                                    &seats,
                                )?,
                            ],
                        },
                    ))
                };
                // `ResourceReleaseFailed` field 1, alternative 4 of the
                // resource surface -- measured, not assumed.
                let trace_identity = self.synthesized_constructor(
                    static_origin,
                    &error_root.alternative(4).field(1),
                    SynthesizedFixedConstructorRole::ResourceTraceIdentity,
                    self.process_symbols.resource_trace_identity.clone(),
                    vec![
                        SynthesizedArgument::Scalar(identity_low_int),
                        SynthesizedArgument::Scalar(identity_high_int),
                    ],
                    &seats,
                )?;
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: surface_tag,
                    alternatives: vec![
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            0,
                            0,
                            SynthesizedFixedConstructorRole::ResourceHostIo,
                            self.process_symbols.resource_host_io.clone(),
                            vec![SynthesizedArgument::Dynamic(surface_io_error(
                                self,
                                builder,
                                &error_root.alternative(0).field(0),
                            )?)],
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            1,
                            1,
                            SynthesizedFixedConstructorRole::ResourceClosed,
                            self.process_symbols.resource_closed.clone(),
                            Vec::new(),
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            2,
                            2,
                            SynthesizedFixedConstructorRole::ResourceMalformed,
                            self.process_symbols.resource_malformed.clone(),
                            Vec::new(),
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            3,
                            3,
                            SynthesizedFixedConstructorRole::ResourceRightNotHeld,
                            self.process_symbols.resource_right_not_held.clone(),
                            vec![
                                SynthesizedArgument::Scalar(resource_required_int),
                                SynthesizedArgument::Scalar(resource_held_int),
                            ],
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            4,
                            4,
                            SynthesizedFixedConstructorRole::ResourceReleaseFailed,
                            self.process_symbols.resource_release_failed.clone(),
                            vec![
                                SynthesizedArgument::Dynamic(resource_kind_value(
                                    self,
                                    resource_kind,
                                    &error_root.alternative(4).field(0),
                                )?),
                                SynthesizedArgument::Nested(trace_identity),
                                SynthesizedArgument::Dynamic(surface_io_error(
                                    self,
                                    builder,
                                    &error_root.alternative(4).field(2),
                                )?),
                            ],
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            5,
                            5,
                            SynthesizedFixedConstructorRole::ResourceKindMismatch,
                            self.process_symbols.resource_kind_mismatch.clone(),
                            vec![
                                SynthesizedArgument::Dynamic(resource_kind_value(
                                    self,
                                    resource_expected_kind,
                                    &error_root.alternative(5).field(0),
                                )?),
                                SynthesizedArgument::Dynamic(resource_kind_value(
                                    self,
                                    resource_actual_kind,
                                    &error_root.alternative(5).field(1),
                                )?),
                            ],
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            6,
                            6,
                            SynthesizedFixedConstructorRole::ResourceBufferLimit,
                            self.process_symbols.resource_buffer_limit.clone(),
                            Vec::new(),
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            7,
                            7,
                            SynthesizedFixedConstructorRole::ResourceInvalidOffset,
                            self.process_symbols.resource_invalid_offset.clone(),
                            Vec::new(),
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            8,
                            8,
                            SynthesizedFixedConstructorRole::ResourceInvalidBounds,
                            self.process_symbols.resource_invalid_bounds.clone(),
                            Vec::new(),
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &error_root,
                            9,
                            9,
                            SynthesizedFixedConstructorRole::ResourceNoProgress,
                            self.process_symbols.resource_no_progress.clone(),
                            Vec::new(),
                            &seats,
                        )?,
                    ],
                })
            } else {
                generic_io_error(self, builder, payload_int, &error_root)?
            };
            let success = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                success_tag,
            );
            let ok = if operation == ken_host::HostOpV1::FsReadFile {
                // `D2` — self-validating span; see `masked_reply_response_bytes`.
                masked_reply_response_bytes(
                    builder,
                    pointer_type,
                    reply,
                    wire.reply_bytes_data_offset,
                    wire.reply_bytes_len_offset,
                    success,
                )
            } else if operation == ken_host::HostOpV1::FsOpen {
                Lowered::ResourceToken { value: detail }
            } else if operation == ken_host::HostOpV1::BufferAllocate {
                Lowered::ResourceToken { value: detail }
            } else if operation == ken_host::HostOpV1::BufferFreeze {
                // `D2` — the SECOND site, and it needs the mask for the same
                // reason. ⛔ Not a copy of the arm above by accident: both
                // construct a `ResponseBytes` from the reply span, so a mask at
                // only one of them leaves the other dereferencing an
                // unestablished pointer on its failure path.
                masked_reply_response_bytes(
                    builder,
                    pointer_type,
                    reply,
                    wire.reply_bytes_data_offset,
                    wire.reply_bytes_len_offset,
                    success,
                )
            } else if operation == ken_host::HostOpV1::FsReadAt {
                let reply_data = builder.ins().stack_load(
                    pointer_type,
                    reply,
                    i32::try_from(wire.reply_bytes_data_offset)
                        .expect("reply bytes data offset is u32"),
                );
                let reply_start = builder.ins().stack_load(
                    types::I64,
                    reply,
                    i32::try_from(wire.reply_bytes_len_offset)
                        .expect("reply bytes len offset is u32"),
                );
                let nonzero = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    detail,
                    0,
                );
                let read_some = builder.ins().band(success, nonzero);
                let zero = builder.ins().iconst(types::I64, 0);
                let eof_data = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    reply_data,
                    zero,
                );
                let eof_start = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    reply_start,
                    0,
                );
                let eof_valid = builder.ins().band(eof_data, eof_start);
                let is_zero = builder.ins().bnot(nonzero);
                let read_eof = builder.ins().band(success, is_zero);
                Self::require_when(builder, read_eof, eof_valid);
                Self::require_when(builder, read_some, eof_data);
                let (request_start, request_length) = positioned_bounds
                    .expect("positioned request bounds were narrowed before dispatch");
                let effective_request = builder.ins().stack_load(
                    types::I64,
                    reply,
                    i32::try_from(wire.reply_effective_request_offset)
                        .expect("reply effective request offset is u32"),
                );
                let (count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    read_some,
                    detail,
                    request_start,
                    request_length,
                    effective_request,
                    Some(reply_start),
                );
                let reply_start_int = self.lower_unsigned_u64_int(builder, reply_start)?;
                // PX8-SPAN-PROV: bind the minted span to this `readAt`'s buffer
                // operand acquisition (lowered arg 2, the request seat).
                let Lowered::ResourceToken { value: span_origin } = seats.specialized(SEAT_2)?
                else {
                    return Err(unsupported(
                        "Effect",
                        "FsReadAt buffer operand is not a resource",
                    ));
                };
                let span_origin = *span_origin;
                let span = self.synthesized_constructor(
                    static_origin,
                    &ok_root.alternative(1).field(0),
                    SynthesizedFixedConstructorRole::PrivateBufferSpan,
                    self.process_symbols.private_buffer_span.clone(),
                    vec![
                        // The seat's operand 2 — the buffer this span is bound
                        // to (`PX8-SPAN-PROV`), projected from the operand list
                        // rather than rebuilt from its destructured payload.
                        self.site_operand_argument(static_origin, 2, &seats)?,
                        SynthesizedArgument::Scalar(reply_start_int),
                        SynthesizedArgument::Scalar(Lowered::BoundedNat(count)),
                    ],
                    &seats,
                )?;
                let transferred = self.synthesized_constructor(
                    static_origin,
                    &ok_root.alternative(1).field(1),
                    SynthesizedFixedConstructorRole::PrivateTransferCount,
                    self.process_symbols.private_transfer_count.clone(),
                    vec![
                        SynthesizedArgument::Scalar(Lowered::BoundedNat(predecessor)),
                        SynthesizedArgument::Scalar(Lowered::BoundedNat(remaining)),
                    ],
                    &seats,
                )?;
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: builder.ins().uextend(types::I64, nonzero),
                    alternatives: vec![
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &ok_root,
                            0,
                            0,
                            SynthesizedFixedConstructorRole::ReadEof,
                            self.process_symbols.read_eof.clone(),
                            Vec::new(),
                            &seats,
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            &ok_root,
                            1,
                            1,
                            SynthesizedFixedConstructorRole::ReadSome,
                            self.process_symbols.read_some.clone(),
                            vec![
                                SynthesizedArgument::Nested(span),
                                SynthesizedArgument::Nested(transferred),
                            ],
                            &seats,
                        )?,
                    ],
                })
            } else if operation == ken_host::HostOpV1::FsWriteAt {
                let (request_start, request_length) = positioned_bounds
                    .expect("positioned request bounds were narrowed before dispatch");
                let effective_request = builder.ins().stack_load(
                    types::I64,
                    reply,
                    i32::try_from(wire.reply_effective_request_offset)
                        .expect("reply effective request offset is u32"),
                );
                let (_count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    success,
                    detail,
                    request_start,
                    request_length,
                    effective_request,
                    None,
                );
                let transferred = self.synthesized_constructor(
                    static_origin,
                    &ok_root.field(0),
                    SynthesizedFixedConstructorRole::PrivateTransferCount,
                    self.process_symbols.private_transfer_count.clone(),
                    vec![
                        SynthesizedArgument::Scalar(Lowered::BoundedNat(predecessor)),
                        SynthesizedArgument::Scalar(Lowered::BoundedNat(remaining)),
                    ],
                    &seats,
                )?;
                self.synthesized_constructor(
                    static_origin,
                    &ok_root,
                    SynthesizedFixedConstructorRole::Wrote,
                    self.process_symbols.wrote.clone(),
                    vec![SynthesizedArgument::Nested(transferred)],
                    &seats,
                )?
            } else if operation == ken_host::HostOpV1::FsHandleMetadata {
                self.lower_unsigned_u64_int(builder, detail)?
            } else {
                self.synthesized_constructor(
                    static_origin,
                    &ok_root,
                    SynthesizedFixedConstructorRole::Unit,
                    self.process_symbols.unit.clone(),
                    Vec::new(),
                    &seats,
                )?
            };
            // `D7` — the two ROOTS, which no node declares. Every other
            // synthesized allocation is reached through a parent's ordered
            // child model and reconciled on the way down; these two are
            // returned straight into the host result with nothing above them,
            // so the population equality is asked for here explicitly.
            self.reconcile_host_result_root(static_origin, &error_root, &error)?;
            self.reconcile_host_result_root(static_origin, &ok_root, &ok)?;
            Ok(LoweringOperand::Specialized(Lowered::HostResult {
                success,
                error: Box::new(error),
                ok: Box::new(ok),
                err_constructor: self.process_symbols.result_err.clone(),
                ok_constructor: self.process_symbols.result_ok.clone(),
            }))
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn lower_unary_recursive_nat_fold(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        join_origin: StaticOriginId,
        symbol: &RuntimeSymbol,
        captures: &[LoweringOperand],
        argument: Lowered,
        zero_body: SourceOccurrence<'_>,
        suc_body: SourceOccurrence<'_>,
        producer_env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let _join_plan = self.consumed_join_plan_token(join_origin)?;
        let (target, structural) = match argument {
            Lowered::StructuralNat(nat) => (nat.value, true),
            Lowered::BoundedNat(nat) => (nat.value, false),
            _ => {
                return Err(unsupported(
                    "DeclarationRef",
                    "unary Nat recursion received a non-Nat representation",
                ));
            }
        };
        let zero = builder.ins().iconst(types::I64, 0);
        let zero_nat = if structural {
            Lowered::StructuralNat(StructuralNatV1 { value: zero })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(zero))
        };
        let mut zero_env = env_with([zero_nat], &[]);
        extend_captures(&mut zero_env, captures.iter().cloned());
        zero_env.extend_from_slice(producer_env);
        let zero_lowered = self.lower_expr(builder, zero_body, &zero_env)?;
        let (initial, result_kind) =
            self.merge_scalar_operand(builder, zero_lowered, None, "DeclarationRef")?;
        if result_kind == ScalarMergeKind::RecursiveBackedge {
            return Err(unsupported(
                "DeclarationRef",
                "unary Nat recursion has no finite base result",
            ));
        }

        let loop_block = builder.create_block();
        let step_block = builder.create_block();
        let done_block = builder.create_block();
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        builder.ins().jump(
            loop_block,
            &[zero.into(), initial.tag.into(), initial.payload.into()],
        );
        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction = NativeScalarPairV1 {
            tag: builder.block_params(loop_block)[1],
            payload: builder.block_params(loop_block)[2],
        };
        let complete = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            predecessor_value,
            target,
        );
        builder.ins().brif(
            complete,
            done_block,
            &[induction.tag.into(), induction.payload.into()],
            step_block,
            &[],
        );

        builder.switch_to_block(step_block);
        let successor_value = builder.ins().iadd_imm(predecessor_value, 1);
        let predecessor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: predecessor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(predecessor_value))
        };
        let successor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: successor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(successor_value))
        };
        let induction = self.lowered_from_scalar_pair(result_kind, induction);
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: None,
                argument_templates: vec![predecessor.clone()],
                induction: Some(induction),
            });
        // A Suc case sees its predecessor first, followed by the retained
        // scrutinee and the declaration's outer environment.
        let mut suc_env = env_with([predecessor, successor], &[]);
        extend_captures(&mut suc_env, captures.iter().cloned());
        suc_env.extend_from_slice(producer_env);
        let next = self.lower_expr(builder, suc_body, &suc_env);
        self.active_recursive_declarations.pop();
        let (next, next_kind) =
            self.merge_scalar_operand(builder, next?, Some(result_kind), "DeclarationRef")?;
        if next_kind != result_kind {
            return Err(unsupported(
                "DeclarationRef",
                "unary Nat recursion changes its native result representation",
            ));
        }
        builder.ins().jump(
            loop_block,
            &[successor_value.into(), next.tag.into(), next.payload.into()],
        );
        builder.switch_to_block(done_block);
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        )))
    }

    /// `body` is the declaration closure's body occurrence (reachable by symbol,
    /// D6); `call_origin` is the origin of the **`Call` occurrence** whose
    /// arguments these are, so argument *i* is `child(call_origin, 1 + i)`.
    #[allow(clippy::too_many_arguments)]
    fn lower_recursive_declaration_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
        captures: &[LoweringOperand],
        params: &[String],
        body: SourceOccurrence<'_>,
        args: &[RuntimeExpr],
        call_origin: StaticOriginId,
        producer_env: &[LoweringEnvironmentBinding],
        eliminators: Option<&[EliminatorFrame<'_>]>,
        join_plan: JoinPlanToken,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(symbol)?;
        let lowered_args = args
            .iter()
            .enumerate()
            .map(|(position, arg)| {
                let arg = self.child_occurrence(call_origin, 1 + position, arg)?;
                self.lower_expr(builder, arg, producer_env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // ⭐ A recursive declaration's arguments are its **loop-header
        // representation**: their shapes are compared across iterations
        // (`same_recursive_argument_shapes`) and lowered into block params. A
        // carried boundary word has no such shape, so this is a
        // specialized-only surface with the ruled fail-closed arm.
        let lowered_args = specialized_operands_at(&lowered_args, "a recursive declaration argument")?;
        if params.len() != lowered_args.len() {
            return Err(unsupported(
                "DeclarationRef",
                format!(
                    "recursive declaration {symbol} expects {} args but call provides {}",
                    params.len(),
                    lowered_args.len()
                ),
            ));
        }

        if let Some(active) = self
            .active_recursive_declarations
            .iter()
            .rev()
            .find(|active| active.symbol == *symbol)
            .cloned()
        {
            if !same_recursive_argument_shapes(&active.argument_templates, &lowered_args) {
                return Err(unsupported(
                    "DeclarationRef",
                    format!(
                        "recursive declaration {symbol} changes its native argument representation: {:?} -> {:?}",
                        active
                            .argument_templates
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>(),
                        lowered_args
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>()
                    ),
                ));
            }
            if let Some(induction) = active.induction {
                return Ok(LoweringOperand::Specialized(induction));
            }
            let mut values = Vec::new();
            append_recursive_argument_values(
                builder,
                &lowered_args,
                &mut values,
                &self.function_local.native_int_tags,
            )?;
            builder.ins().jump(
                active
                    .header
                    .expect("tail-recursive declarations own a loop header"),
                &values.into_iter().map(Into::into).collect::<Vec<_>>(),
            );

            // Continue lowering only in a predecessor-free block. This keeps
            // the structured builder usable while the real recursive edge
            // returns directly to the loop header.
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        }

        // Only declarations in an actual recursive SCC need the loop/result
        // closure below. Preserve the established direct-call lowering for
        // ordinary declarations, including constructor-valued HostIO trees.
        if !self.declaration_is_recursive(symbol) {
            let mut call_inputs = lowered_args
                .into_iter()
                .rev()
                .map(LoweringOperand::Specialized)
                .collect::<Vec<_>>();
            call_inputs.extend(captures.iter().cloned());
            let call_env = env_with_operands(call_inputs, producer_env);
            return if let Some(eliminators) = eliminators {
                self.lower_computational_producer_expr(builder, body, &call_env, eliminators)
            } else {
                self.lower_expr(builder, body, &call_env)
            };
        }

        if eliminators.is_none() && params.len() == 1 && lowered_args.len() == 1 {
            if let RuntimeExpr::Match {
                scrutinee, cases, ..
            } = body.expr
            {
                if matches!(scrutinee.as_ref(), RuntimeExpr::Var(0)) {
                    // These two arms are found by constructor name under the
                    // BODY occurrence's match, so their bodies are its children
                    // `1 + index` — the index the search would otherwise discard.
                    let zero = cases.iter().enumerate().find(|(_, case)| {
                        case.constructor == self.process_symbols.nat_zero && case.binders == 0
                    });
                    let suc = cases.iter().enumerate().find(|(_, case)| {
                        case.constructor == self.process_symbols.nat_suc && case.binders == 1
                    });
                    if let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) {
                        // The closed unary-fold fast path emits the declaration
                        // body's source `Match` without re-entering
                        // `lower_expr`; consume the same origin-keyed join plan
                        // here before its merge helper reborrows it.
                        self.enter_source_occurrence_plan(body.static_origin)?;
                        let zero_body =
                            self.case_body_occurrence(body.static_origin, zero_index, &zero.body)?;
                        let suc_body =
                            self.case_body_occurrence(body.static_origin, suc_index, &suc.body)?;
                        return self.lower_unary_recursive_nat_fold(
                            builder,
                            body.static_origin,
                            symbol,
                            captures,
                            lowered_args
                                .into_iter()
                                .next()
                                .expect("unary recursion owns one argument"),
                            zero_body,
                            suc_body,
                            producer_env,
                        );
                    }
                }
            }
        }

        let header = builder.create_block();
        let done = builder.create_block();
        let mut initial_values = Vec::new();
        append_recursive_argument_values(
            builder,
            &lowered_args,
            &mut initial_values,
            &self.function_local.native_int_tags,
        )?;
        for value in &initial_values {
            builder.append_block_param(header, builder.func.dfg.value_type(*value));
        }
        builder.append_block_param(done, types::I64);
        builder.append_block_param(done, types::I64);
        builder.ins().jump(
            header,
            &initial_values
                .iter()
                .copied()
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        builder.switch_to_block(header);

        let mut parameters = builder.block_params(header).iter().copied();
        let mut loop_args = Vec::with_capacity(lowered_args.len());
        for template in &lowered_args {
            loop_args.push(rebuild_recursive_argument(
                template,
                &mut parameters,
                &mut self.function_local.native_int_tags,
            )?);
        }
        if parameters.next().is_some() {
            return Err(unsupported(
                "DeclarationRef",
                "recursive declaration loop parameter shape is not closed",
            ));
        }
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: Some(header),
                argument_templates: lowered_args,
                induction: None,
            });
        // Runtime environments are de Bruijn-nearest first: source arguments
        // are evaluated left-to-right, then installed in reverse binder order,
        // followed by captures and the producer environment.
        let mut call_inputs = loop_args
            .into_iter()
            .rev()
            .map(LoweringOperand::Specialized)
            .collect::<Vec<_>>();
        call_inputs.extend(captures.iter().cloned());
        let call_env = env_with_operands(call_inputs, producer_env);
        let lowered = if let Some(eliminators) = eliminators {
            self.lower_computational_producer_expr(builder, body, &call_env, eliminators)
        } else {
            self.lower_expr(builder, body, &call_env)
        };
        self.active_recursive_declarations.pop();
        let lowered = lowered?;
        let (value, result_kind) =
            self.merge_scalar_branch(builder, &join_plan, lowered, "DeclarationRef")?;
        builder
            .ins()
            .jump(done, &[value.tag.into(), value.payload.into()]);
        builder.switch_to_block(done);
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done)[0],
                payload: builder.block_params(done)[1],
            },
        )))
    }

    fn lower_declaration_ref(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        reference_origin: StaticOriginId,
        symbol: &RuntimeSymbol,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let declaration = self
            .declarations
            .get(symbol.as_str())
            .copied()
            .ok_or_else(|| {
                unsupported(
                    "DeclarationRef",
                    format!("{symbol} is not present in the exact RuntimeProgram"),
                )
            })?;
        let RuntimeDeclarationKind::Transparent { body } = &declaration.kind else {
            return Err(unsupported(
                "DeclarationRef",
                format!("{symbol} is not an executable transparent declaration"),
            ));
        };
        // D6/D7: a `DeclarationRef` is a childless leaf, and the declaration's
        // body is a **separately planned** source occurrence, reachable by name.
        // That is why this construction site needs no threading — and why it must
        // not suggest the two `lower_expr` closure arms are nearly done. Only
        // transparent declarations are planned, which is exactly the set that
        // survives the rejection above, so a missing occurrence is a planner bug.
        let declaration_origin = self
            .static_transition_plan
            .declaration_occurrence_origin(symbol.as_str())
            .ok_or_else(|| {
                // A planner invariant, not a capacity limit: this declaration is
                // transparent, so the planner planned it.
                backend(BackendFailure::PlannerInvariant(format!(
                    "transparent declaration {symbol} has no planned source occurrence"
                )))
            })?;
        let declaration_body = SourceOccurrence {
            expr: body,
            static_origin: declaration_origin,
        };
        // ⭐⭐ **`RT-DECL-CLOSURE-PORT` `D4` — BOTH closure seed forms retain a
        // compiler-only callable binding, and evaluating the naked
        // `DeclarationRef` never calls the unit.**
        //
        // ⚠ Before `D4` only the `Closure` arm produced a binding here. A
        // `LexicalClosure`-bodied declaration fell through to the
        // `FunctionizedUnits` arm below and was called with `&[]` — an empty
        // input slice against a unit that declares this declaration's
        // parameters and captures. That call was unreachable in production
        // (the `TransparentDeclarationClosure` residual still forces
        // `RecursiveDescent`) but it was wrong-arity by construction, and
        // "unreachable today" is not the property this needs.
        //
        // ⚠ The two arms differ in their capture MATERIAL and only there.
        // `Closure` captures are seed symbols resolved out of seed material;
        // `LexicalClosure` captures are expressions of the declaration's own
        // body. That is the `Seed` / `Lexical` split `D3` established at the
        // ABI boundary, reaching the same distinction one layer up.
        let seed_binding = match body {
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let body = self.child_occurrence(declaration_origin, 0, body)?;
                let captures = captures
                    .iter()
                    .map(|capture| {
                        // Seed captures resolve to JIT-time ground values, so
                        // this arm ASSERTS the phase rather than preserving
                        // one: there is no carried seed capture to lose.
                        self.lower_seed_capture(builder, capture)
                            .map(LoweringOperand::Specialized)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Some((captures, params.clone(), body.static_origin))
            }
            RuntimeExpr::LexicalClosure {
                captures,
                params,
                body,
            } => {
                // ⚠ HAZARD (`D3`): the planner plans the body FIRST and the
                // capture sequence after it, so body is child `0` and capture
                // *i* is child `1 + i` — the declaration order
                // (`captures, params, body`) is NOT the child order.
                let body = self.child_occurrence(declaration_origin, 0, body)?;
                let captures = captures
                    .iter()
                    .enumerate()
                    .map(|(position, capture)| {
                        let capture =
                            self.child_occurrence(declaration_origin, 1 + position, capture)?;
                        // ⛔ The EMPTY environment, not the reference site's.
                        // A declaration body is closed — that is why the
                        // recursive-descent arm below lowers it with `&[]` too
                        // — so a capture expression of this closure cannot see,
                        // and must not be able to see, whatever bindings happen
                        // to be live where the declaration was referenced.
                        self.lower_expr(builder, capture, &[])
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                // `D7` — a declaration closure's LEXICAL captures reach it at
                // their own phases, exactly as a lexical closure's do, so they
                // are stored unchanged. The former `specialized_operands_at`
                // fold here is the same narrowing removed at the lexical sites.
                Some((captures, params.clone(), body.static_origin))
            }
            _ => None,
        };
        if let Some((captures, params, body)) = seed_binding {
            return Ok(LoweringOperand::Specialized(Lowered::DeclarationClosure {
                reference: reference_origin,
                symbol: symbol.clone(),
                captures,
                params,
                body,
            }));
        }
        if self.body_emission_authority == BodyEmissionAuthority::FunctionizedUnits {
            // ⛔⛔ **The empty-input call, and the guard that keeps it lawful.**
            //
            // Reaching here means this declaration's body is not a closure
            // seed, so its scheduling entry IS its unit and the call genuinely
            // takes no inputs. The planner decided the same thing
            // independently — from the declaration's *planned occurrence*, not
            // from the `RuntimeExpr` arm matched above — and recorded it. ⇒ The
            // two derivations are cross-checked here, so a callable target can
            // only reach this `&[]` call if the planner and the lowering
            // disagree about what the declaration is, which fails closed rather
            // than emitting a wrong-arity call.
            let class = self
                .static_transition_plan
                .declaration_call_target_class(reference_origin)
                .ok_or_else(|| {
                    backend(BackendFailure::PlannerInvariant(format!(
                        "declaration reference to {symbol} has no planned call target class"
                    )))
                })?;
            if class != DeclarationCallTargetClass::SchedulingEntry {
                return Err(backend(BackendFailure::PlannerInvariant(format!(
                    "declaration {symbol} lowers as a zero-input thunk but its planned call \
                     targets a declaration-owned callable unit"
                ))));
            }
            return self.call_declared_declaration_unit(builder, reference_origin, &[], None);
        }
        if self.declaration_stack.contains(symbol) {
            return Err(unsupported(
                "DeclarationRef",
                format!("recursive non-function declaration {symbol} is unsupported"),
            ));
        }
        self.declaration_stack.push(symbol.clone());
        let result = self.lower_expr(builder, declaration_body, &[]);
        self.declaration_stack.pop();
        result
    }

    /// **`RT-DECL-CLOSURE-PORT` `D4` — the complete call to a declaration-owned
    /// callable unit, and the SOLE place its input order is decided.**
    ///
    /// Every `Call` consumer that can reach a [`Lowered::DeclarationClosure`]
    /// routes through here, because the input slice is the one thing they must
    /// not each remember for themselves: the callee descriptor declares its
    /// `Parameter` slots and then its `Capture` slots, so
    ///
    /// ```text
    /// inputs = actual arguments in PARAMETER order ++ retained captures in D3 order
    /// ```
    ///
    /// is a property of the ABI, not of any one call site. ⛔ A consumer that
    /// assembled its own slice would be a second ordering authority, and a
    /// swapped one still type-checks — every input is a word.
    ///
    /// ⚠ `args` arrive already lowered because the three consumers lower them
    /// differently (ordinary child occurrences, producer-env child occurrences,
    /// source-machine operands). What they must NOT differ on is what happens
    /// afterwards.
    fn call_declaration_closure_unit(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        reference: StaticOriginId,
        symbol: &RuntimeSymbol,
        params: &[String],
        captures: Vec<LoweringOperand>,
        args: Vec<LoweringOperand>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⛔⛔ **The pending checked-recursion marker is CONSUMED here.**
        //
        // ⚠ Every consumer this route diverts from reached
        // `lower_recursive_declaration_call`, which consumes the marker on
        // entry. A branch that returns before it would leave the marker set —
        // to be picked up by whatever call came next, which
        // `consume_checked_recursive_invocation_call` would then report as a
        // marker "transplanted to another callee". That is a silent
        // mis-attribution, not a missing feature. ⇒ Taking it here is not
        // optional bookkeeping; it is what keeps the diversion from corrupting
        // an unrelated call site.
        //
        // `D5` replaces `D4`'s conservative refusal of a present marker with
        // the split-domain validation below. The marker remains **validation
        // authority only**: nothing it carries becomes a runtime operand, an
        // identity word, a selector, a capsule, a capture, an alternate ABI, a
        // copied body, or a fallback.
        let checked = self.consume_checked_recursive_invocation_call(symbol)?;
        // The declared arity, checked against the call before anything is
        // emitted. The descriptor rejects a mismatched slice too, but it can
        // only say "the frame is missing an input"; this says which call.
        if params.len() != args.len() {
            return Err(unsupported(
                "Call",
                format!(
                    "closure expects {} args but call provides {}",
                    params.len(),
                    args.len()
                ),
            ));
        }
        self.validate_declaration_unit_call(reference, symbol, checked, params.len(), captures.len())?;
        // ⭐ The consumed template id is carried from the validation that
        // accepted it to the emission that realizes it — one value, one path.
        // ⛔ Re-deriving it at the emission site would be a second authority on
        // "which template is this call", and the two could disagree silently.
        let checked_template = match checked.map(|instance| instance.source) {
            Some(InvocationTemplateRef::SameSccCall(id)) => Some(id),
            // `validate_declaration_unit_call` already refused every other
            // source, so this arm is unreachable-by-validation rather than a
            // fallback. Spelled, not wildcarded.
            Some(InvocationTemplateRef::ComputationalIHCall(_)) => None,
            None => None,
        };
        let mut inputs = args;
        // `D7` — the captures are already phase-bearing: a carried capture
        // passes as its existing word and a specialized one is unchanged. The
        // former `map(Specialized)` here asserted a phase the edge now owns.
        inputs.extend(captures);
        self.call_declared_declaration_unit(builder, reference, &inputs, checked_template)
    }

    /// **`RT-DECL-CLOSURE-PORT` `D5` — the split-domain validation that stands
    /// between a call occurrence and the declaration-owned unit it reaches.**
    ///
    /// Two authorities are joined on **one exact occurrence and one exact
    /// target**, and neither is derived from the other:
    ///
    /// 1. **the checked-plan domain** validates the facts it actually owns —
    ///    same-SCC identity, admission, application arity, and the canonical
    ///    local telescope;
    /// 2. **the identity join** binds the consumed checked occurrence to the
    ///    planner-issued `DeclarationCall` reference: exact callee symbol,
    ///    exact application arity, exact resolved `CallableDeclaration` target.
    ///    These three equalities are the *only* lawful shared facts between the
    ///    two domains;
    /// 3. **the ABI domain** validates that same target's `D3`-established
    ///    `Parameter ++ Capture` descriptor run and `D4` input order.
    ///
    /// ⛔⛔ **There is no checked-interface → ABI projection, and D5 must not
    /// invent one** (Architect ruling, 2026-08-04). `CheckedAnswerInterfaceV1`
    /// is canonical *semantic* bytes plus a fingerprint of those bytes: it
    /// encodes no [`AbiCarrier`], no [`AbiOwnership`], no [`AbiStorageOwner`],
    /// no slot ordinal and no capture provenance. So owner and phase are read
    /// where they actually live — on the resolved descriptor — and never parsed
    /// out of canonical bytes, surrogated by a fingerprint comparison, or
    /// assigned by telescope position.
    ///
    /// ⛔ **The ABI half compares two independently produced copies**, which is
    /// the whole reason it is not tautological. `resolve_call_edges` copies the
    /// already-validated unit header/slots/offsets into the function-local
    /// [`units::DeclaredUnitCall`] record; the plan's own descriptor is
    /// immutable and was independently checked by `D3` against the caller-side
    /// `StaticBody` signature. Reading `slot.carrier` from the declared record
    /// and then deriving that same record's expected owner from it would be a
    /// check with one operand. ⇒ The chain preserved here is
    /// `caller-side D3 signature ↔ validated target descriptor ↔ exact declared
    /// call record`.
    ///
    /// ⚠ **Axes deliberately NOT re-checked here, because an earlier authority
    /// already fails closed on them.** `OrientedSubcontinuationPlanV1::validate`
    /// runs on the compile path (`planning.rs`, inside
    /// `oriented_subcontinuation_plan_for_program`) and there establishes, for
    /// every recursive-call template: the callee segment site and the exact
    /// callee frame-template set against the plan's own frames; the composition
    /// of the last callee frame's output interface with `result_interface` and
    /// `caller_interface`; and `occurrence_binding_fingerprint` over **every**
    /// field of the template. `planning.rs` additionally reconciles the exact
    /// per-declaration marker locations, which is what binds `declaration` to
    /// the body the marker actually sits in. Restating those here would put a
    /// second copy of each law in a file where the two can disagree, and would
    /// let a control mis-attribute an upstream refusal to `D5`.
    fn validate_declaration_unit_call(
        &self,
        reference: StaticOriginId,
        symbol: &RuntimeSymbol,
        checked: Option<CheckedRecursiveInvocationInstance>,
        params: usize,
        captures: usize,
    ) -> Result<(), CraneliftBackendError> {
        // ── The identity join, half one: the planner's resolved target class.
        //
        // ⛔ Every call through this route is a call to a declaration-OWNED
        // unit, so a planner that resolved this same reference to the
        // declaration's zero-arity scheduling entry disagrees with the lowering
        // about what the declaration is. That fails closed rather than emitting
        // a call whose arity the descriptor would then have to reject.
        let class = self
            .static_transition_plan
            .declaration_call_target_class(reference)
            .ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(format!(
                    "declaration reference to {symbol} has no planned call target class"
                )))
            })?;
        if class != DeclarationCallTargetClass::CallableDeclaration {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "declaration {symbol} lowers as a declaration-owned unit call but its planned \
                 call targets the declaration's scheduling entry"
            ))));
        }
        let declared = self
            .function_local
            .declaration_calls
            .get(&reference)
            .ok_or_else(|| {
                backend_module(
                    "DeclarationRef has no planner-derived declaration call target".to_string(),
                )
            })?;
        // ── The ABI domain, on that exact target.
        let mut found = None;
        for unit in self.static_transition_plan.emittable_units()? {
            if unit.origin() != declared.origin {
                continue;
            }
            if found.is_some() {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "two abi descriptors claim one declaration call target origin".to_string(),
                )));
            }
            found = Some(unit);
        }
        let unit = found.ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "declaration call target has no abi descriptor".to_string(),
            ))
        })?;
        if !matches!(
            unit.definition(),
            AbiUnitDefinition::CallableDeclaration { .. }
        ) {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "declaration call to {symbol} resolves a target that is not a declaration-owned \
                 callable unit"
            ))));
        }
        // The declared call record against the plan's immutable descriptor:
        // header, the whole slot run (carrier, ownership, storage owner,
        // width/alignment, kind and ordinal), and the offsets.
        if unit.header() != declared.header {
            return Err(backend(BackendFailure::PlannerInvariant(
                "declaration call record header disagrees with its validated descriptor"
                    .to_string(),
            )));
        }
        if unit.slots() != declared.slots.as_slice() {
            return Err(backend(BackendFailure::PlannerInvariant(
                "declaration call record slot run disagrees with its validated descriptor"
                    .to_string(),
            )));
        }
        let (offsets, frame_bytes) = unit.slot_offsets()?;
        if offsets != declared.offsets || frame_bytes != declared.header.frame_bytes {
            return Err(backend(BackendFailure::PlannerInvariant(
                "declaration call record offsets disagree with its validated descriptor"
                    .to_string(),
            )));
        }
        // ── The `D4` input order, stated against the descriptor that receives
        // it. `inputs = args in PARAMETER order ++ captures in D3 order` is only
        // meaningful if the descriptor's leading run is exactly that many
        // `Parameter` slots followed by exactly that many `Capture` slots, each
        // densely ordinalled within its own kind-run.
        let mut expected = (0..params)
            .map(|ordinal| (AbiSlotKind::Parameter, ordinal as u32))
            .chain((0..captures).map(|ordinal| (AbiSlotKind::Capture, ordinal as u32)));
        for slot in unit.slots() {
            match slot.kind {
                AbiSlotKind::Parameter | AbiSlotKind::Capture => {
                    if expected.next() != Some((slot.kind, slot.ordinal)) {
                        return Err(backend(BackendFailure::PlannerInvariant(format!(
                            "declaration call to {symbol} does not match its callable unit's \
                             parameter-then-capture input run"
                        ))));
                    }
                }
                AbiSlotKind::Result
                | AbiSlotKind::Control
                | AbiSlotKind::Trap
                | AbiSlotKind::Store => {}
            }
        }
        if expected.next().is_some() {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "declaration call to {symbol} supplies more inputs than its callable unit declares"
            ))));
        }
        // ── The checked-plan domain. An unchecked call has nothing further to
        // reconcile: it carries no same-SCC obligation, and the ABI half above
        // is unconditional.
        let Some(instance) = checked else {
            return Ok(());
        };
        let InvocationTemplateRef::SameSccCall(call_template_id) = instance.source else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "declaration-owned unit call received a computational IH invocation",
            ));
        };
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked declaration-owned unit call has no checked plan",
            )
        })?;
        let call = plan.recursive_call(call_template_id).ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked declaration-owned unit call has no checked template",
            )
        })?;
        // ── The identity join, half two: callee symbol and application arity.
        // These are the lawful shared equalities — the callee the checked plan
        // names is the callee the emitter is about to call, applied to exactly
        // the arity the plan admitted.
        if &call.callee != symbol {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked declaration-owned unit call names another callee",
            ));
        }
        if call.arity != params as u64 {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                format!(
                    "checked recursive invocation of {symbol} is admitted at arity {} but the \
                     declaration-owned unit call applies {params}",
                    call.arity
                ),
            ));
        }
        if call.local_telescope.len() as u64 != call.arity {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked recursive invocation's local telescope does not cover its arity",
            ));
        }
        // ── Same-SCC identity and admission.
        //
        // ⭐ **What makes this call *same-SCC* is that its callee is itself a
        // recursive member of the caller's group** — not that the template says
        // so. The independent side is the rest of the plan: some template in
        // this recursion group must be the callee's OWN call template, and the
        // group must be internally consistent in `scc_index` and `admission`.
        //
        // ⚠ For a self-call the witness is this very template, so `recursion_
        // group` alone is not discriminated by a single-declaration fixture;
        // the mutual same-SCC fixture is what closes that. Stated rather than
        // left to be discovered — the rule is identical either way, and there
        // is deliberately no `caller == callee` shortcut.
        let group = plan
            .recursive_calls
            .iter()
            .filter(|other| other.recursion_group == call.recursion_group)
            .collect::<Vec<_>>();
        if !group.iter().any(|other| other.declaration == call.callee) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked recursive invocation's callee is not a recursive member of its own \
                 recursion group",
            ));
        }
        // ⚠ Split, so each axis gets its OWN first refusal. One composite
        // predicate is discharged by either disagreement holding, so it could
        // not tell the `scc_index` mutation from the `admission` one.
        if group.iter().any(|other| other.scc_index != call.scc_index) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked recursion group disagrees about its scc index",
            ));
        }
        if group.iter().any(|other| other.admission != call.admission) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked recursion group disagrees about its admission",
            ));
        }
        if plan
            .recursive_calls
            .iter()
            .any(|other| other.scc_index == call.scc_index && other.recursion_group != call.recursion_group)
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "two checked recursion groups claim one scc index",
            ));
        }
        Ok(())
    }

    /// `static_origin` is the origin of the **match occurrence** whose cases
    /// these are; case *i*'s body is `child(static_origin, 1 + i)`.
    fn lower_borrowed_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        pointer: cranelift_codegen::ir::Value,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
        join_plan: &JoinPlanToken,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let kind = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 0);
        Self::require_i64(builder, kind, 2);
        let tag = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 8);
        let arity = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 24);
        let pointer_type = builder.func.dfg.value_type(pointer);
        let fields = builder
            .ins()
            .load(pointer_type, MemFlags::trusted(), pointer, 16);
        if let [case] = cases {
            let (expected_tag, expected_arity) =
                borrowed_constructor_identity(&self.process_symbols, &case.constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "Match",
                            format!("{} has no borrowed constructor identity", case.constructor),
                        )
                    })?;
            if case.binders != expected_arity {
                return Err(unsupported(
                    "Match",
                    format!("{} borrowed arity mismatch", case.constructor),
                ));
            }
            let arm = builder.create_block();
            let rejected = builder.create_block();
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                expected_tag,
            );
            builder.ins().brif(selected, arm, &[], rejected, &[]);
            builder.switch_to_block(rejected);
            let failure = builder.ins().iconst(types::I64, -1);
            builder.ins().return_(&[failure]);
            builder.switch_to_block(arm);
            Self::require_i64(builder, arity, expected_arity as i64);
            if expected_arity != 0 {
                Self::require_nonzero(builder, fields);
            }
            let arm_env = env_with_operands(
                (0..expected_arity).map(|index| {
                    let field = builder.ins().iadd_imm(fields, (index * 32) as i64);
                    LoweringOperand::Specialized(Lowered::BorrowedNativeValue { pointer: field })
                }),
                env,
            );
            // The single-case fast path is still case 0 of this match.
            let body = self.case_body_occurrence(static_origin, 0, &case.body)?;
            return self.lower_expr(builder, body, &arm_env);
        }
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            self.append_planned_join_params(builder, merge, join_plan);
        }
        let mut test_block = builder.current_block().expect("borrowed match block");
        let mut merge_kind = None;
        for (index, case) in cases.iter().enumerate() {
            let (expected_tag, expected_arity) =
                borrowed_constructor_identity(&self.process_symbols, &case.constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "Match",
                            format!("{} has no borrowed constructor identity", case.constructor),
                        )
                    })?;
            if case.binders != expected_arity {
                return Err(unsupported(
                    "Match",
                    format!("{} borrowed arity mismatch", case.constructor),
                ));
            }
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                expected_tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            Self::require_i64(builder, arity, expected_arity as i64);
            if expected_arity != 0 {
                Self::require_nonzero(builder, fields);
            }
            let arm_env = env_with_operands(
                (0..expected_arity).map(|index| {
                    let field = builder.ins().iadd_imm(fields, (index * 32) as i64);
                    LoweringOperand::Specialized(Lowered::BorrowedNativeValue { pointer: field })
                }),
                env,
            );
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
            if !self.seal_source_trap_branch(builder, &lowered)? {
                let merge = merge.ok_or_else(|| {
                    backend_module(
                        "join plan omitted a merge despite a continuing predecessor".to_string(),
                    )
                })?;
                self.jump_planned_join_arm(
                    builder,
                    merge,
                    join_plan,
                    body.static_origin,
                    lowered,
                    &mut merge_kind,
                    "a borrowed `Match` arm",
                )?;
            }
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        };
        self.finish_planned_join(
            builder,
            merge,
            join_plan,
            merge_kind,
            "a borrowed `Match` join",
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_borrowed_option_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        present: cranelift_codegen::ir::Value,
        value: cranelift_codegen::ir::Value,
        none: &str,
        some: &str,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            self.append_planned_join_params(builder, merge, &join_plan);
        }
        let some_block = builder.create_block();
        let none_block = builder.create_block();
        let mut merge_kind = None;
        let mut terminal_trap = None;
        builder
            .ins()
            .brif(present, some_block, &[], none_block, &[]);
        for (block, symbol, fields) in [
            (some_block, some, vec![Lowered::Int { value, known: None }]),
            (none_block, none, Vec::new()),
        ] {
            builder.switch_to_block(block);
            let case = cases
                .iter()
                .enumerate()
                .find(|(_, case)| case.constructor == symbol);
            let Some((index, case)) = case else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            if case.binders != fields.len() {
                return Err(unsupported("Match", "borrowed Option arity mismatch"));
            }
            let arm_env = env_with(fields, env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                terminal_trap.get_or_insert_with(|| trap.clone());
            }
            if self.seal_source_trap_branch(builder, &lowered)? {
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "borrowed Option join omitted a merge despite a continuing predecessor"
                        .to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                body.static_origin,
                lowered,
                &mut merge_kind,
                "Match",
            )?;
        }
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            let trap = terminal_trap.ok_or_else(|| {
                backend_module(
                    "borrowed Option join omitted both a continuing predecessor and a source \
                     trap"
                        .to_string(),
                )
            })?;
            return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
        };
        self.finish_planned_join(builder, merge, &join_plan, merge_kind, "Match")
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_dynamic_host_result_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // D8: the source traversal consumed the origin-keyed contract before
        // reaching this helper. Reborrow it before creating a block or lowering
        // either arm. The specialized HostResult scrutinee is not a selector
        // for the result representation.
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            #[cfg(test)]
            D8_JOIN_MERGES_CREATED.with(|count| count.set(count.get() + 1));
            self.append_planned_join_params(builder, merge, &join_plan);
            #[cfg(test)]
            if D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get)
                == JoinConsumptionMutation::DispositionDynamicHostResultMerge
            {
                self.function_local
                    .dispositioned_join_origins
                    .insert(static_origin);
            }
        }
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        let mut merge_kind = None;
        builder.ins().brif(success, ok_block, &[], err_block, &[]);
        for (block, constructor, payload) in [
            (ok_block, ok_constructor, ok),
            (err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let Some((index, case)) = cases
                .iter()
                .enumerate()
                .find(|(_, case)| case.constructor == constructor && case.binders == 1)
            else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            let arm_env = env_with([payload], env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            match join_plan.representation {
                JoinResultRepresentation::NativeScalarPair => {
                    let (value, branch_kind) =
                        self.merge_scalar_branch(builder, &join_plan, lowered, "Match")?;
                    Self::record_scalar_merge_kind("Match", &mut merge_kind, branch_kind)?;
                    builder
                        .ins()
                        .jump(merge, &[value.tag.into(), value.payload.into()]);
                }
                JoinResultRepresentation::CarrierWord => {
                    let word = self.carried_join_arm(
                        builder,
                        body.static_origin,
                        lowered,
                        None,
                        "Match",
                    )?;
                    builder.ins().jump(merge, &[word.word.into()]);
                }
            }
        }
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        };
        builder.switch_to_block(merge);
        match join_plan.representation {
            JoinResultRepresentation::NativeScalarPair => {
                let pair = NativeScalarPairV1 {
                    tag: builder.block_params(merge)[0],
                    payload: builder.block_params(merge)[1],
                };
                Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
                    merge_kind.expect("HostResult emits a continuing closed alternative"),
                    pair,
                )))
            }
            JoinResultRepresentation::CarrierWord => {
                Ok(LoweringOperand::Carried(CarriedBoundaryWord {
                    word: builder.block_params(merge)[0],
                }))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_bounded_nat_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        let zero = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_zero && case.binders == 0
        });
        let suc = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_suc && case.binders == 1
        });
        let (Some(zero), Some(suc)) = (zero, suc) else {
            return Err(unsupported(
                "BoundedNat",
                "structural Nat match requires exact Zero and Suc predecessor arms",
            ));
        };
        let zero_block = builder.create_block();
        let suc_block = builder.create_block();
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            self.append_planned_join_params(builder, merge, &join_plan);
        }
        let predecessor = nat.predecessor(builder);
        let is_zero =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, nat.value, 0);
        builder.ins().brif(is_zero, zero_block, &[], suc_block, &[]);
        let mut merge_kind = None;
        let mut terminal_trap = None;
        for (block, (index, case), predecessor) in [
            (zero_block, zero, None),
            (suc_block, suc, Some(predecessor)),
        ] {
            builder.switch_to_block(block);
            let arm_env = predecessor
                .map(|predecessor| {
                    vec![if structural {
                        Lowered::StructuralNat(StructuralNatV1 {
                            value: predecessor.value,
                        })
                    } else {
                        Lowered::BoundedNat(predecessor)
                    }]
                })
                .unwrap_or_default();
            let mut arm_env = env_with(arm_env, &[]);
            arm_env.extend_from_slice(env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                terminal_trap.get_or_insert_with(|| trap.clone());
            }
            if self.seal_source_trap_branch(builder, &lowered)? {
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a BoundedNat merge despite a continuing predecessor"
                        .to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                body.static_origin,
                lowered,
                &mut merge_kind,
                "BoundedNat",
            )?;
        }
        let Some(merge) = merge else {
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            let trap = terminal_trap.ok_or_else(|| {
                backend_module(
                    "BoundedNat join omitted both a continuing predecessor and a source trap"
                        .to_string(),
                )
            })?;
            return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
        };
        self.finish_planned_join(builder, merge, &join_plan, merge_kind, "BoundedNat")
    }

    fn lower_dynamic_constructor_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        continuation: DynamicConstructorContinuation<'_>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;

        let source_default = match continuation {
            DynamicConstructorContinuation::Ordinary { default, .. }
            | DynamicConstructorContinuation::Producer { default, .. } => default,
        };
        let static_origin = match continuation {
            DynamicConstructorContinuation::Ordinary { static_origin, .. }
            | DynamicConstructorContinuation::Producer { static_origin, .. } => static_origin,
        };
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        let merge = join_plan.has_continuing_predecessor.then(|| {
            let merge = builder.create_block();
            self.append_planned_join_params(builder, merge, &join_plan);
            merge
        });
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor match block");
        let mut merge_kind = None;
        for alternative in dynamic.alternatives {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let (cases, default, env, static_origin) = match continuation {
                DynamicConstructorContinuation::Ordinary {
                    cases,
                    default,
                    env,
                    static_origin,
                }
                | DynamicConstructorContinuation::Producer {
                    cases,
                    default,
                    env,
                    static_origin,
                    ..
                } => (cases, default, env, static_origin),
            };
            let (index, case) = match select_dynamic_constructor_case(cases, &alternative, default)?
            {
                Ok(selected) => selected,
                Err(_owned_default) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let arm_env = materialize_dynamic_constructor_env(&alternative, env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = match continuation {
                DynamicConstructorContinuation::Ordinary { .. } => {
                    self.lower_expr(builder, body, &arm_env)?
                }
                DynamicConstructorContinuation::Producer { eliminators, .. } => {
                    self.lower_computational_producer_expr(builder, body, &arm_env, eliminators)?
                }
            };
            if self.seal_source_trap_branch(builder, &lowered)? {
                test_block = next;
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a DynamicConstructor merge despite a continuing \
                     predecessor"
                        .to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                body.static_origin,
                lowered,
                &mut merge_kind,
                "DynamicConstructor",
            )?;
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(
                source_default.clone(),
            )));
        };
        self.finish_planned_join(builder, merge, &join_plan, merge_kind, "DynamicConstructor")
    }

    /// `static_origin` is the `PrimitiveCall` occurrence's own origin; argument
    /// *i* is child *i* (a primitive symbol is an atom, not a child).
    fn lower_primitive_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        primitive: &RuntimePrimitive,
        args: &[RuntimeExpr],
        static_origin: StaticOriginId,
        env: &[LoweringEnvironmentBinding],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let lowered_args = args
            .iter()
            .enumerate()
            .map(|(position, arg)| {
                let arg = self.child_occurrence(static_origin, position, arg)?;
                self.lower_expr(builder, arg, env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if lowered_args.iter().any(|arg| {
            matches!(
                arg,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            )
        }) {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        }

        match &primitive.partiality {
            RuntimePartiality::Total => {}
            RuntimePartiality::SafeOption { .. } | RuntimePartiality::SafeResult { .. } => {}
            RuntimePartiality::CheckedTrap { obligation } => {
                self.assumptions.insert(format!(
                    "checked partial obligation {obligation} not discharged"
                ));
                let trap =
                    crate::cranelift_backend::planning::planned_partiality_trap(primitive)
                        .expect("CheckedTrap has one planner-derived trap");
                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
            }
            RuntimePartiality::TrustedTrap { assumption } => {
                self.assumptions.insert(format!(
                    "trusted partial assumption {assumption} remains visible"
                ));
                let trap =
                    crate::cranelift_backend::planning::planned_partiality_trap(primitive)
                        .expect("TrustedTrap has one planner-derived trap");
                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
            }
        }

        // A primitive's static symbol determines whether its operands are
        // scalar Ints or Bools. A carried word in one of those positions is
        // projected through the emitted scalar helper; no runtime tag chooses
        // which source type the operand is.
        let scalar_kind = match primitive.symbol.as_str() {
            "add_int" | "sub_int" | "mul_int" | "eq_int" | "leq_int" | "uint8_to_int"
            | "int_to_uint8_raw" => Some("Int"),
            "not_bool" | "and_bool" | "or_bool" => Some("Bool"),
            _ => None,
        };
        let lowered_args = if primitive.symbol == "bytes_length" {
            match lowered_args.as_slice() {
                [LoweringOperand::Specialized(_)] => {
                    specialized_operands_at(&lowered_args, "the bytes_length operand")?
                }
                [LoweringOperand::Carried(word)] => {
                    let class = self.emit_carrier_class(builder, *word)?;
                    Self::require_i64(builder, class, BoundaryClass::BorrowedOpaque as i64);
                    let pointer = self.emit_carrier_scalar(builder, *word)?;
                    vec![Lowered::BorrowedNativeValue { pointer }]
                }
                _ => {
                    return Err(unsupported(
                        "PrimitiveCall",
                        "bytes_length requires exactly one bytes operand",
                    ));
                }
            }
        } else if primitive.symbol == "bytes_at" {
            lowered_args
                .into_iter()
                .enumerate()
                .map(|(position, arg)| match (position, arg) {
                    (_, LoweringOperand::Specialized(value)) => Ok(value),
                    (0, LoweringOperand::Carried(word)) => {
                        let class = self.emit_carrier_class(builder, word)?;
                        Self::require_i64(builder, class, BoundaryClass::BorrowedOpaque as i64);
                        let pointer = self.emit_carrier_scalar(builder, word)?;
                        Ok(Lowered::BorrowedNativeValue { pointer })
                    }
                    (1, LoweringOperand::Carried(word)) => {
                        let tag = builder
                            .ins()
                            .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
                        Self::require_i64(
                            builder,
                            tag,
                            crate::boundary_value::BoundaryTag::ImmediateInt as i64,
                        );
                        let value = self.emit_carrier_scalar(builder, word)?;
                        Ok(self.lower_dynamic_small_int(builder, value))
                    }
                    (_, LoweringOperand::Carried(_)) => Err(unsupported(
                        "PrimitiveCall",
                        "bytes_at received more operands than its closed static signature",
                    )),
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?
        } else if let Some(kind) = scalar_kind {
            lowered_args
                .into_iter()
                .map(|arg| match arg {
                    LoweringOperand::Specialized(value) => Ok(value),
                    LoweringOperand::Carried(word) => {
                        let value = self.emit_carrier_scalar(builder, word)?;
                        let tag = builder
                            .ins()
                            .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
                        Ok(match kind {
                            "Int" => {
                                Self::require_i64(
                                    builder,
                                    tag,
                                    crate::boundary_value::BoundaryTag::ImmediateInt as i64,
                                );
                                self.lower_dynamic_small_int(builder, value)
                            }
                            "Bool" => {
                                Self::require_i64(
                                    builder,
                                    tag,
                                    crate::boundary_value::BoundaryTag::ImmediateBool as i64,
                                );
                                Lowered::Bool { value, known: None }
                            }
                            _ => unreachable!("closed primitive scalar kind"),
                        })
                    }
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?
        } else {
            specialized_operands_at(&lowered_args, "a primitive-call operand")?
        };
        let lowered = match primitive.symbol.as_str() {
            "add_int" => self.lower_int_binop(builder, "add_int", lowered_args, |lhs, rhs| {
                lhs.checked_add(rhs)
            }),
            "sub_int" => self.lower_int_binop(builder, "sub_int", lowered_args, |lhs, rhs| {
                lhs.checked_sub(rhs)
            }),
            "mul_int" => self.lower_int_binop(builder, "mul_int", lowered_args, |lhs, rhs| {
                lhs.checked_mul(rhs)
            }),
            "eq_int" => self.lower_int_cmp(
                builder,
                "eq_int",
                lowered_args,
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                |lhs, rhs| lhs == rhs,
            ),
            "leq_int" => self.lower_int_cmp(
                builder,
                "leq_int",
                lowered_args,
                cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                |lhs, rhs| lhs <= rhs,
            ),
            "uint8_to_int" | "int_to_uint8_raw" => {
                let [value]: [Lowered; 1] = lowered_args.try_into().map_err(|args: Vec<_>| {
                    unsupported(
                        "PrimitiveCall",
                        format!(
                            "{} expects one argument, got {}",
                            primitive.symbol,
                            args.len()
                        ),
                    )
                })?;
                let Lowered::Int { .. } = value else {
                    return Err(unsupported(
                        "PrimitiveCall",
                        format!("{} expects an Int-represented value", primitive.symbol),
                    ));
                };
                Ok(value)
            }
            "not_bool" => self.lower_bool_not(builder, lowered_args),
            "and_bool" => self.lower_bool_binop(
                builder,
                "and_bool",
                lowered_args,
                |builder, lhs, rhs| builder.ins().band(lhs, rhs),
                |lhs, rhs| lhs && rhs,
            ),
            "or_bool" => self.lower_bool_binop(
                builder,
                "or_bool",
                lowered_args,
                |builder, lhs, rhs| builder.ins().bor(lhs, rhs),
                |lhs, rhs| lhs || rhs,
            ),
            "bytes_length" => self.lower_bytes_length(builder, lowered_args),
            "bytes_at" => self.lower_bytes_at(builder, lowered_args, &primitive.partiality),
            "bytes_slice" => self.lower_bytes_slice(lowered_args, &primitive.partiality),
            "bytes_concat" => self.lower_bytes_concat(lowered_args),
            "bytes_encode" => self.lower_bytes_encode(lowered_args),
            "bytes_decode" => self.lower_bytes_decode(lowered_args, &primitive.partiality),
            "list_char_to_string" => {
                let [value]: [Lowered; 1] = lowered_args.try_into().map_err(|args: Vec<_>| {
                    unsupported(
                        "PrimitiveCall",
                        format!(
                            "list_char_to_string expects one argument, got {}",
                            args.len()
                        ),
                    )
                })?;
                let bytes = lowered_char_list(&value).ok_or_else(|| {
                    unsupported(
                        "PrimitiveCall",
                        "list_char_to_string requires a closed List Char",
                    )
                })?;
                let value = String::from_utf8(bytes).map_err(|_| {
                    unsupported(
                        "PrimitiveCall",
                        "list_char_to_string received non-UTF-8 Char values",
                    )
                })?;
                Ok(Lowered::String(value))
            }
            "byte_length" => self.lower_string_byte_length(builder, lowered_args),
            "char_length" => self.lower_string_char_length(builder, lowered_args),
            other => Err(unsupported(
                "PrimitiveCall",
                format!("primitive {other} is not in the supported native set"),
            )),
        };
        // ⭐ Back onto the spine: a primitive's result is a fresh specialized
        // value re-entering the phase sum.
        lowered.map(LoweringOperand::Specialized)
    }
}


/// Which environment a `D3b` resolution's index is an index INTO, and what root
/// provenance it carries. ⛔ Two fields, not one: the index alone cannot say
/// which environment it belongs to, and that is the conflation the closed
/// availability sum exists to prevent.
#[derive(Clone, Copy, Debug)]
struct ContinuationImmediateResolution {
    immediate_slot: u32,
    root: ContinuationImmediateRoot,
}

#[derive(Clone, Copy, Debug)]
enum ContinuationImmediateRoot {
    EntryAbi {
        source_owner: PredeclaredFunctionId,
        source_abi_position: u32,
    },
    /// ⛔ Carries no position, deliberately. A producer-local value has no ABI
    /// position in any environment, and a field here would be a place for one to
    /// be invented.
    ProducerLocal,
}

/// **`D3b` re-cut — the exact occurrence the direct-emission consumer stands at.**
///
/// ⛔ A `CurrentLexical` claim counts binders at ONE seat. Presenting it at a
/// different occurrence would read an index derived at one depth as if it held
/// at another, so the consumer carries its own seat and refuses a claim keyed
/// elsewhere.
#[derive(Clone, Copy, Debug)]
struct ContinuationDirectEmissionSeat {
    producer_result_origin: StaticOriginId,
    emission_origin: StaticOriginId,
}

/// The index a claim carries, whichever environment it names. **Mutation
/// support only.**
///
/// ⛔ Deliberately NOT available to production. Reading "the index" without
/// first answering "which environment" is the conflation the claim sum exists
/// to prevent; a mutation is allowed to ask it precisely because its job is to
/// corrupt the index while leaving the environment intact, so that what the
/// consumer catches is the index and not a shape mismatch.
#[cfg(test)]
fn d3b_claim_index(claim: ContinuationEnvironmentClaim) -> u32 {
    match claim {
        ContinuationEnvironmentClaim::CurrentLexical {
            nearest_alias_index, ..
        } => nearest_alias_index,
        ContinuationEnvironmentClaim::EntryFrame { declared_slot, .. } => declared_slot,
    }
}

/// Replace a claim's index, preserving its environment and every identity field.
/// **Mutation support only** — see [`d3b_claim_index`].
#[cfg(test)]
fn d3b_replace_claim_index(
    claim: ContinuationEnvironmentClaim,
    index: u32,
) -> ContinuationEnvironmentClaim {
    match claim {
        ContinuationEnvironmentClaim::CurrentLexical {
            emission_owner,
            producer_result_origin,
            emission_origin,
            lexical_environment_origin,
            nearest_alias_index: _,
        } => ContinuationEnvironmentClaim::CurrentLexical {
            emission_owner,
            producer_result_origin,
            emission_origin,
            lexical_environment_origin,
            nearest_alias_index: index,
        },
        ContinuationEnvironmentClaim::EntryFrame { frame, .. } => {
            ContinuationEnvironmentClaim::EntryFrame {
                frame,
                declared_slot: index,
            }
        }
    }
}
