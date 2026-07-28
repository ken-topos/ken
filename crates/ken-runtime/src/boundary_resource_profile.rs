//! **`RT-FNSPLIT-C3-ACTIVATION` `D4` — the deployment-supplied capacity
//! authority for boundary storage.**
//!
//! ⭐ **This type exists because the authority it names did not.** The prior
//! arena ruling asserted that *"the existing store/plan capacity authority
//! grants storage"*; measurement found that phrase has no referent —
//! `BoundaryRegion::reserve(nodes, words, data, limbs)` takes four
//! **caller-supplied** numbers, and no landed function derives them from a plan,
//! a store, or a program. ⇒ Someone has to create that authority, and `§3c`
//! rules that it is **deployment resource policy**, ⛔ not compiler semantics
//! and ⛔ not an emitter-derived formula.
//!
//! ## The eight quantities, and why exactly eight
//!
//! Two regions — the **invocation** arena and the **persistent** image — each
//! metering four resources: nodes, child words, data bytes, and native-`Int`
//! limbs. ⭐ **Eight is not a design choice made here**: it is exactly what the
//! two existing reserve operations consume, so the profile is total over what
//! the runtime can actually be asked to reserve, with nothing left implicit.
//!
//! ## ⛔ No default, and that is enforced by the compiler rather than by review
//!
//! [`BoundaryResourceProfileV1`] deliberately has **no `Default` impl** and
//! **no partial constructor**, and its limits are named public fields. ⇒ Every
//! construction site must write all eight numbers out, and there is no
//! `..Default::default()` to hide behind. ⚠ A `new()` taking four same-typed
//! `usize` positionals would have been the transposition hazard this shape
//! removes: swapping *words* and *data bytes* would compile, run, and be wrong.
//!
//! ⛔ **The emitter may validate and carry a selected profile; it may not
//! invent, widen, or silently default one.** Absence is a configuration refusal
//! **before packaging or activation** — ⛔ never a linked executable that starts
//! and then declines to run generated code.
//!
//! ## ⚠ Zero is a legal limit
//!
//! A limit of `0` is **explicit and finite**, and means *"no room for this
//! resource"*. It is a deployment's right to say so, and the first construction
//! that needs the resource fails loudly naming it. ⛔ Zero is not treated as
//! "unset" — conflating the two is how a default sneaks back in.

use std::fmt;

/// Which of the two boundary regions a limit governs.
///
/// ⛔ Closed, with no catch-all. A third region would be a compile error at
/// [`Self::ALL`] and at every exhaustive `match` over this type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum BoundaryResourceScope {
    /// The per-activation arena. Dies with the activation.
    Invocation,
    /// The store-owned persistent image, which outlives every activation.
    Persistent,
}

impl BoundaryResourceScope {
    /// Every scope, in declaration order.
    pub const ALL: [BoundaryResourceScope; 2] = [
        BoundaryResourceScope::Invocation,
        BoundaryResourceScope::Persistent,
    ];

    /// The name this scope reports in a failure. ⭐ Part of the contract:
    /// `AC-4` requires an exhaustion to name the exact scope.
    pub const fn name(self) -> &'static str {
        match self {
            BoundaryResourceScope::Invocation => "invocation",
            BoundaryResourceScope::Persistent => "persistent",
        }
    }
}

impl fmt::Display for BoundaryResourceScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Which metered resource a limit governs.
///
/// ⛔ Closed, with no catch-all, and the four are exactly the four counts
/// `BoundaryRegion::reserve` consumes. ⭐ Adding a fifth metered table to the
/// region without extending this enum is a compile error, which is the point:
/// an unmetered table is storage no deployment authorized.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum BoundaryResource {
    /// Region nodes.
    Nodes,
    /// Child words, parallel to the field-name table.
    Words,
    /// Bytes in the data table, backing `Bytes` and `String` contents.
    DataBytes,
    /// `u64` limbs backing a region-limbed `Int`'s magnitude.
    NativeIntLimbs,
}

impl BoundaryResource {
    /// Every resource, in the order `BoundaryRegion::reserve` takes them.
    ///
    /// ⭐ **The order is load-bearing** — see
    /// [`BoundaryRegionLimitsV1::as_reserve_arguments`], which is the one place
    /// that knows it.
    pub const ALL: [BoundaryResource; 4] = [
        BoundaryResource::Nodes,
        BoundaryResource::Words,
        BoundaryResource::DataBytes,
        BoundaryResource::NativeIntLimbs,
    ];

    /// The name this resource reports in a failure.
    pub const fn name(self) -> &'static str {
        match self {
            BoundaryResource::Nodes => "nodes",
            BoundaryResource::Words => "words",
            BoundaryResource::DataBytes => "data bytes",
            BoundaryResource::NativeIntLimbs => "native-Int limbs",
        }
    }
}

impl fmt::Display for BoundaryResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One region's four limits.
///
/// ⛔ Public named fields and no `Default`: a construction site names all four
/// or does not compile.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoundaryRegionLimitsV1 {
    /// Region nodes.
    pub nodes: usize,
    /// Child words.
    pub words: usize,
    /// Data-table bytes.
    pub data_bytes: usize,
    /// Native-`Int` limbs.
    pub native_int_limbs: usize,
}

impl BoundaryRegionLimitsV1 {
    /// This region's limit for one resource.
    ///
    /// ⛔ Exhaustive, no `_` arm — a resource added to
    /// [`BoundaryResource::ALL`] without a limit here is a compile error rather
    /// than a silent zero.
    pub const fn limit(self, resource: BoundaryResource) -> usize {
        match resource {
            BoundaryResource::Nodes => self.nodes,
            BoundaryResource::Words => self.words,
            BoundaryResource::DataBytes => self.data_bytes,
            BoundaryResource::NativeIntLimbs => self.native_int_limbs,
        }
    }

    /// The four limits in the **positional order**
    /// `BoundaryRegion::reserve(nodes, words, data, limbs)` takes them.
    ///
    /// ⭐⭐ **The one place that knows the mapping between this type's named
    /// fields and that function's positional parameters.** ⛔ Spelling the
    /// four out at each reserve call site would be a second answer to the
    /// question *"which number is which"*, and the two would drift silently —
    /// a transposition compiles and runs.
    pub const fn as_reserve_arguments(self) -> (usize, usize, usize, usize) {
        (self.nodes, self.words, self.data_bytes, self.native_int_limbs)
    }
}

/// **The versioned, deployment-supplied boundary resource profile.**
///
/// ⛔ No `Default`, no partial constructor, no widening. See the module doc for
/// why that is structural rather than a convention.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoundaryResourceProfileV1 {
    /// Limits for the per-activation arena.
    pub invocation: BoundaryRegionLimitsV1,
    /// Limits for the store-owned persistent image.
    pub persistent: BoundaryRegionLimitsV1,
}

/// The profile schema version this type implements.
///
/// ⭐ Carried so a recorded package provenance can be read back and rejected by
/// a runtime that does not implement its schema, rather than being reinterpreted
/// under a layout it does not have.
pub const BOUNDARY_RESOURCE_PROFILE_VERSION: u32 = 1;

impl BoundaryResourceProfileV1 {
    /// One limit, by the `(scope, resource)` pair.
    ///
    /// ⭐ Total over `ALL × ALL` — which is what lets `AC-4`'s eight cases be
    /// written as a closed product rather than as eight hand-copied lookups
    /// that can disagree with each other.
    pub const fn limit(self, scope: BoundaryResourceScope, resource: BoundaryResource) -> usize {
        self.region(scope).limit(resource)
    }

    /// One region's limits.
    ///
    /// ⛔ Exhaustive, no `_` arm.
    pub const fn region(self, scope: BoundaryResourceScope) -> BoundaryRegionLimitsV1 {
        match scope {
            BoundaryResourceScope::Invocation => self.invocation,
            BoundaryResourceScope::Persistent => self.persistent,
        }
    }
}

/// **A capacity limit was reached, named by exactly which one.**
///
/// ⭐ `AC-4` requires at-limit-plus-one to *"fail loudly naming that exact
/// scope"*. ⚠ The emitted-code status for exhaustion is a single
/// [`crate::boundary_value::BOUNDARY_ERR_CAPACITY`], which names nothing — so a
/// shared *"capacity exhausted"* assertion across eight limits is **one control
/// claiming to be eight**. This type is what makes the eight distinguishable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoundaryCapacityExhaustedV1 {
    /// Which region ran out.
    pub scope: BoundaryResourceScope,
    /// Which metered resource ran out.
    pub resource: BoundaryResource,
    /// The authorized limit for that `(scope, resource)`.
    pub limit: usize,
    /// What was asked for, which is strictly greater than [`Self::limit`].
    pub requested: usize,
}

impl fmt::Display for BoundaryCapacityExhaustedV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "boundary capacity exhausted: {} {} limit is {}, requested {}",
            self.scope, self.resource, self.limit, self.requested
        )
    }
}

/// **⛔ A profile was required and none was supplied.**
///
/// ⭐ A distinct type from [`BoundaryCapacityExhaustedV1`] on purpose. `AC-7`
/// requires absence to be a refusal **before packaging or activation**, and
/// requires its control to *distinguish refusal-to-package from
/// refusal-at-run* — those are different observations and only one is
/// permitted. ⇒ Giving them one error type would make the two indistinguishable
/// at exactly the seam where the distinction is the acceptance criterion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoundaryResourceProfileMissingV1 {
    /// What the caller was trying to do when the profile turned out to be
    /// absent.
    pub during: BoundaryResourceProfileStage,
}

/// When a missing profile was detected. ⛔ Closed, no catch-all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoundaryResourceProfileStage {
    /// Building a package. ⭐ The **only** permitted point of refusal.
    Packaging,
    /// Starting an activation. ⚠ Permitted for the JIT caller, which has no
    /// packaging step.
    Activation,
}

impl fmt::Display for BoundaryResourceProfileMissingV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let during = match self.during {
            BoundaryResourceProfileStage::Packaging => "packaging",
            BoundaryResourceProfileStage::Activation => "activation",
        };
        write!(
            f,
            "no boundary resource profile was supplied; refused at {during}. \
             A profile is deployment resource policy and has no default"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// A profile whose eight numbers are all **distinct**, so that any lookup
    /// returning the wrong one is visible.
    ///
    /// ⚠ ⛔ **A fixture with equal limits cannot detect a transposition** — the
    /// `(scope, resource)` lookup would return the right number for the wrong
    /// reason, and every assertion below would pass on a broken table.
    fn distinct_profile() -> BoundaryResourceProfileV1 {
        BoundaryResourceProfileV1 {
            invocation: BoundaryRegionLimitsV1 {
                nodes: 11,
                words: 22,
                data_bytes: 33,
                native_int_limbs: 44,
            },
            persistent: BoundaryRegionLimitsV1 {
                nodes: 55,
                words: 66,
                data_bytes: 77,
                native_int_limbs: 88,
            },
        }
    }

    /// ⭐ **`AC-4`'s precondition: the eight limits are eight, and each is
    /// reachable by its own `(scope, resource)` pair.**
    ///
    /// **MEASURED:** the product `ALL × ALL` has eight members and the profile
    /// returns a distinct value for each.
    /// **CLAIMED:** each limit governs its named region and resource.
    /// **THE GAP:** ⛔ that the *runtime* consults the right one when it
    /// reserves — this is the lookup table, not its consumer. That is `S2`'s
    /// obligation and `AC-4`'s real subject; ⛔ do not read this as `AC-4`
    /// discharged.
    ///
    /// Promise class: **durable invariant** — a relation over the closed
    /// product, so it survives any extension that keeps the limits distinct.
    #[test]
    fn the_eight_limits_are_eight_and_each_is_separately_addressable() {
        let profile = distinct_profile();
        let mut seen = BTreeSet::new();
        let mut pairs = 0;
        for scope in BoundaryResourceScope::ALL {
            for resource in BoundaryResource::ALL {
                pairs += 1;
                assert!(
                    seen.insert(profile.limit(scope, resource)),
                    "{scope} {resource} returns a limit another pair already \
                     returned, so the two are not separately addressable"
                );
            }
        }
        assert_eq!(pairs, 8, "the scope x resource product is not eight");
        assert_eq!(seen.len(), 8);
    }

    /// ⛔ **The named fields and the `(scope, resource)` lookup agree.**
    ///
    /// ⚠ Without this, the lookup could be internally consistent and still
    /// disagree with the field a deployment actually wrote — the two are
    /// different surfaces and only one of them is what the caller filled in.
    #[test]
    fn the_lookup_agrees_with_the_named_field_a_deployment_wrote() {
        let profile = distinct_profile();
        use BoundaryResource::*;
        use BoundaryResourceScope::*;
        assert_eq!(profile.limit(Invocation, Nodes), profile.invocation.nodes);
        assert_eq!(profile.limit(Invocation, Words), profile.invocation.words);
        assert_eq!(
            profile.limit(Invocation, DataBytes),
            profile.invocation.data_bytes
        );
        assert_eq!(
            profile.limit(Invocation, NativeIntLimbs),
            profile.invocation.native_int_limbs
        );
        assert_eq!(profile.limit(Persistent, Nodes), profile.persistent.nodes);
        assert_eq!(profile.limit(Persistent, Words), profile.persistent.words);
        assert_eq!(
            profile.limit(Persistent, DataBytes),
            profile.persistent.data_bytes
        );
        assert_eq!(
            profile.limit(Persistent, NativeIntLimbs),
            profile.persistent.native_int_limbs
        );
    }

    /// ⭐⭐ **The reserve-argument order is the one place the positional
    /// mapping lives, and it is measured against the names.**
    ///
    /// ⚠ **This is the transposition control.** `BoundaryRegion::reserve` takes
    /// four bare `usize`s; swapping *words* and *data bytes* at a call site
    /// compiles, runs, and silently authorizes the wrong table. Routing every
    /// call through [`BoundaryRegionLimitsV1::as_reserve_arguments`] means there
    /// is exactly one mapping to get wrong, and this asserts it.
    #[test]
    fn the_reserve_argument_order_matches_the_named_fields() {
        let limits = distinct_profile().invocation;
        assert_eq!(limits.as_reserve_arguments(), (11, 22, 33, 44));
        let (nodes, words, data_bytes, native_int_limbs) = limits.as_reserve_arguments();
        assert_eq!(nodes, limits.nodes);
        assert_eq!(words, limits.words);
        assert_eq!(data_bytes, limits.data_bytes);
        assert_eq!(native_int_limbs, limits.native_int_limbs);
        // The positional order and the inventory order are the SAME order —
        // `BoundaryResource::ALL` documents itself as being in reserve order,
        // and that claim is checked here rather than trusted.
        let positional = [nodes, words, data_bytes, native_int_limbs];
        for (index, resource) in BoundaryResource::ALL.into_iter().enumerate() {
            assert_eq!(
                positional[index],
                limits.limit(resource),
                "`BoundaryResource::ALL` is not in reserve-argument order at \
                 position {index} ({resource})"
            );
        }
    }

    /// ⛔ **An exhaustion names the exact scope and resource** — the whole
    /// reason `AC-4` can be eight controls rather than one.
    #[test]
    fn an_exhaustion_names_its_scope_and_resource_distinguishably() {
        let mut messages = BTreeSet::new();
        for scope in BoundaryResourceScope::ALL {
            for resource in BoundaryResource::ALL {
                let failure = BoundaryCapacityExhaustedV1 {
                    scope,
                    resource,
                    limit: 7,
                    requested: 8,
                };
                let rendered = failure.to_string();
                assert!(rendered.contains(scope.name()));
                assert!(rendered.contains(resource.name()));
                assert!(
                    messages.insert(rendered),
                    "{scope} {resource} renders the same message as another \
                     pair, so a test cannot tell which limit fired"
                );
            }
        }
        assert_eq!(messages.len(), 8);
    }

    /// ⛔ **A missing profile and an exhausted one are different observations**,
    /// and `AC-7` turns on being able to tell them apart — as well as on telling
    /// refusal-to-package from refusal-at-run.
    #[test]
    fn a_missing_profile_is_distinguishable_from_exhaustion_and_names_its_stage() {
        let packaging = BoundaryResourceProfileMissingV1 {
            during: BoundaryResourceProfileStage::Packaging,
        };
        let activation = BoundaryResourceProfileMissingV1 {
            during: BoundaryResourceProfileStage::Activation,
        };
        assert_ne!(packaging, activation);
        assert!(packaging.to_string().contains("packaging"));
        assert!(activation.to_string().contains("activation"));
        assert!(!packaging.to_string().contains("activation"));

        let exhausted = BoundaryCapacityExhaustedV1 {
            scope: BoundaryResourceScope::Invocation,
            resource: BoundaryResource::Nodes,
            limit: 0,
            requested: 1,
        };
        // ⭐ The discriminator that matters: a zero limit is an EXPLICIT
        // deployment choice, so exhausting it must not be reported as an
        // absent profile. Conflating the two is how a default sneaks back in.
        assert!(!exhausted.to_string().contains("no boundary resource profile"));
    }

    /// ⚠ **Zero is a legal, explicit limit** — it means "no room", not "unset".
    #[test]
    fn a_zero_limit_is_explicit_and_is_not_an_absent_profile() {
        let profile = BoundaryResourceProfileV1 {
            invocation: BoundaryRegionLimitsV1 {
                nodes: 0,
                words: 0,
                data_bytes: 0,
                native_int_limbs: 0,
            },
            persistent: BoundaryRegionLimitsV1 {
                nodes: 1,
                words: 1,
                data_bytes: 1,
                native_int_limbs: 1,
            },
        };
        for resource in BoundaryResource::ALL {
            assert_eq!(profile.limit(BoundaryResourceScope::Invocation, resource), 0);
            assert_eq!(profile.limit(BoundaryResourceScope::Persistent, resource), 1);
        }
        assert_eq!(profile.invocation.as_reserve_arguments(), (0, 0, 0, 0));
    }
}
