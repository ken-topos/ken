---
scope: build
audience: (see scope README)
source: private memory `named-floor-must-be-grepped-not-assumed`
---

# A named floor must be grepped, not assumed

A frame doc naming a floor to transport/bottom-out-at ("bottoms out at
`DecEq Int`/`Num Int`'s audited-delta `Axiom` leaves") is a claim that thing
exists *right now*, not a design intent. Parallel phrasing ("X/Y's leaves")
reads as one verified unit even when only half of it is real — `DecEq Int`
existed, `Num Int` (no `class Num` anywhere, only prose in `minimality.md`
claiming "Lc, landed") did not.

**Why:** this was the SECOND WP in a row (after K7) where a named-but-
nonexistent floor was the actual blocker, not a hard proof. On K7 the "floor"
was a kernel capability assumed present; on `wp/lawful-classes-lane` it was a
whole typeclass (`Num`) assumed landed. Both times, grounding before writing
code caught it in minutes; discovering it mid-build (after committing to an
approach) would have cost far more and produced a half-built, unlandable
deliverable.

**How to apply:** before writing the first line of code against a frame's
"bottoms out at / references / builds on top of X," `git grep` X (the class, the
postulate, the decl) at the cited location. If the frame cites two things in one
breath ("`A`/`B`'s leaves"), grep both independently — don't let shared phrasing
imply shared verification. Escalate a missing floor immediately rather than
trying to build the floor yourself unless explicitly asked to (scope discipline
— see wp scope hold dont force a workaround if that memory exists, otherwise:
hold and ask, don't unilaterally author a prerequisite class/WP).

**Variant — a CAPABILITY fork ("can the current surface express X as a derived
def?") is answered by grepping a LANDED PRECEDENT of the exact shape, not by
reasoning about expressibility abstractly (L3-strings slice-2 fork, 2026-07-03,
`evt_4k1yqah3yvpds`).** Steward asked me to settle A(derive over
`elim_List`/`elim_Nat`) vs B(native prims) for the List-combinator floor
(`append`/`take`/`drop`/`nth`/`list_eq`/`list_compare`), the deciding question
being "is recursion-via-elim expressible or a surface gap needing a prerequisite
WP?" The abstract answer (eliminators internalise structural recursion, so yes)
is right but not *sufficient* — the honest confirm was `git grep` on
`l3a_acceptance.rs`, which showed the exact recursion shapes already landed and
SCT-passing: `map`/`fold` (single-list), `zip` (two-list = the `list_eq` shape),
`unfoldUpTo` (Nat-fuel = the `take`/`drop` shape), `insert`+`sort`
(element-comparison threaded through List recursion). A landed precedent
confirms BOTH halves of a capability in one O(1) check — the elaborator path
(`match` → `Term::Elim` lowering) AND the termination gate (`sct_check` accepts
the shape) — which abstract reasoning about the type theory cannot (the SCT
might reject a shape that "should" typecheck). **Apply:** when a
derive-vs-native / build-vs- prerequisite fork turns on "is X expressible in the
current surface," grep the tests/packages for a landed def of the same
recursion/elaboration shape before ruling; the precedent de-risks the capability
confirm and tells you whether the "hidden floor" is real work or ordinary
derived defs. Zero-TCB-delta corollary: derived
(`declare_recursive_group`+`sct_check`+`declare_def`) beats native prims on
subsume-don't-proliferate whenever the shape is expressible. Same core
discipline as the floor case — grep the named thing, don't assume it — applied
to a capability rather than a decl.

**Variant — a TRANSPORTED class instance carries only a SUBSET of methods; grep
the instance body, not the class concept (L3-strings-surface `compare` fork,
2026-07-03, `evt_1stp9sspm6ag8`).** My derivation table named
`compare a b = list_compare (Ord Char).compare … : Ordering` — but `Ord Char` is
transported from `Ord_instance_Int` and carries only
`leq/refl/antisym/ trans/total` (NO `compare` method), and no
`Ordering`/`OrdResult` type exists globally (ES2 retired it from the prelude).
Two nonexistent constructs in one table cell, caught by spec-author's grep, not
mine. **Apply:** a derivation/ elaboration table that names `(Instance).method`
or a result TYPE must grep the instance's actual field set + the type's actual
declaration — a transported instance mirrors only the fields it transported, and
a type "sanctioned" may be LOCAL-only, not global. Companion judgment-lesson (so
you don't wrongly block/defer a satisfiable request as a lock violation): read a
"do-not-reopen" input by its ACTUAL bounds — a lock forbidding "re-derive X" is
honored by *repackaging* landed primitives
(`compareChar = if eqChar then Eq else if leqChar then Lt else Gt` packages the
landed 2-way ops into 3-way, not a re-derivation), and a prelude-*retirement* is
honored by *local* declaration where genuinely-needed (grounded by a landed
local precedent), neither is a reopen. A checked `data` inductive is
zero-`trusted_base()`-delta (only postulates/primitives grow the TCB), so
introducing a local result type costs no trust. **Root-cause sharpening (3
strikes in the L3-strings arc, 2026-07-03): I conflate DELIVERABLE with
DELIVERED for lawful-class instances.** Strike 1 was existence
(`(Ord Char).compare` — method doesn't exist); strikes 2+3 were STATUS — ADR
0010 said `DecEq Char` "instances landed" (only the `eqChar` view is) and
`Ord String` "transports now" (no lawful instance landed; it's
deliverable-but-unbuilt, still needs lex law proofs). Canonicity makes a lawful
instance *sound to construct*, and my prose slides `deliverable → delivered`.
**Fix: before any present-tense availability claim about a lawful-class
instance/method (`Ord String`/`DecEq String`/…), grep for the actual
`instance …`/method decl — `deliverable` and `landed` are different words.**
Healthy version of the pattern: strike 3 surfaced during my own review of the
fix to strike 2 and folded pre-vote — when a correction targets an over-claim I
authored, the same region likely holds its siblings (grep-the-region-fold-all on
my own prose).

**What to DO once grounding catches a stale premise mid-review** — resolve it
yourself only if the fix is structurally determined, escalate to the lane owner
if it would contradict a locked input, and mind the fold-vs-track timing
against the Decision's SHA anchor — is a separate discipline with its own
evidence; see [[mid-review-fix-inline-escalate-or-track]].

**Variant — a spec's BUILDABILITY claim ("in-WP, unblocked, real proof terms
this WP") is a claim about what the ELABORATOR can construct TODAY; grep the
proof-construction capability, and don't transfer a concrete-scrutinee proof
idiom to an abstract-scrutinee setting (52-map §5 erratum, 2026-07-03).**
**Status check before reuse:** the two capability gaps below (no
`Term::J`/`Cast` construction in `elab.rs`; the induction-mechanism nullary
gate) are the 2026-07-03 ground truth — both have since moved (`elab.rs` now
constructs `Term::J`/`Term::Cast` at several sites). Re-grep the current
elaborator before citing either gap as still open; the *method* — grep the
construction capability, don't reason abstractly — is what survives, not the
specific verdict.

I claimed the Map §5 induction proofs were "real proof terms, this WP, unblocked,"
borrowing the `Ord Bool` idiom (case-split + reduce + `refl`/`tt`/`absurd`). But
`Ord Bool`'s scrutinees are CONCRETE constructors (`True`/`False`) that reduce,
whereas the Map inductions hit an ABSTRACT `leq k k'` (variable keys) that is
irreducibly STUCK — closing it needs a transport (`J`/`cast`) step, a DIFFERENT
elaborator capability that isn't landed (`check_match_dependent` admits only
`Term::Var` scrutinees; `elab.rs` builds no `Term::J`/`Cast`; `Refl` only checks
pre-existing convertibility). Architect caught it via foundation-implementer's
build-time refuse-`Axiom`-and-escalate; a buildability over-claim (the math is
SOUND — it's not a soundness defect), forcing a 10-site whole-doc erratum that
split the laws Branch-A (comparison-free, buildable now) vs Branch-B
(stuck-`leq`, transport-gated deferred). **Apply:** before writing a spec's
"buildable now / unblocked / real proof terms this WP" *scope* claim, grep the
landed elaborator for the CONSTRUCTION capability the proof needs (dependent
match / transport / `J` / `cast`) — a concrete-carrier proof precedent does NOT
license the same idiom over abstract scrutinees (the K1.5
stress-test-against-an-abstract-scrutinee lesson, applied to proof AUTHORING).
Buildability = a claim about the elaborator-that-exists-now, orthogonal to
mathematical soundness; the discriminating grep is "can the elaborator emit this
proof term," not "is the theorem true." Sibling of soundness AC static vs
runtime face (deliverable ≠ delivered).

**Two-walls sharpening (SAME 52-map arc — the FIX itself recurred, 2026-07-03):
a "buildable" ruling can miss a SECOND capability wall on a DIFFERENT axis —
ground EVERY axis the proof's construction touches.** My erratum *fixing* the
transport over-claim ITSELF kept `toList`-ordered as "buildable now," trusting a
per-proof ruling that cleared the `leq`/transport axis (Gap A) but not the
induction-mechanism axis (**Gap B**: `check_match_dependent`'s nullary-only
gate — as of 2026-07-03, a constructor-arity check in `elab.rs` — blocked ALL
hypothesis-narrowing `List`/`Tree` induction; re-grep the current gate before
citing this as still-live, the match-compilation code has since moved).
foundation-implementer's **empirical 2-line build-attempt** caught it — a
reviewer (and I) reasoning from
one axis structurally can't see a wall on another. **Apply:** enumerate every
construction capability the proof needs (dependent-match / transport / each
eliminator) and grep EACH; when buildability is live, prefer the
**build-attempt** (or a foundation grounding) over any reasoning-only "it should
typecheck" — and don't re-draft a buildability claim until the split is settled
at ground truth, never guessed (guessing is the exact error the erratum fixes).
**Honest-diagnosis corollary:** frame a buildability gap as "the elaborator
**lags** the spec" (a build-completeness shortfall against ALREADY-SPECIFIED
behavior — point at the spec § + the existing kernel/former as the named build
target, e.g. `34 §Dependent-motive-recovery` / the kernel's landed
`Term::J`/`Cast`), not "the spec over-asks" — more honest AND it gives the
deferral a target instead of reading as a spec hole.

**Variant — grep a name you DEFINE, not just names you reference (CAT-3/CAT-4
`Perm` three-way collision, 2026-07-04).** Every variant above is about a floor
you *build atop*; the dual failure is *binding a new definition to a name
already taken by a landed global*. My CAT-3 `57 §3.1` ruled
`Perm := count-equality` and grounded carefully against `Perm_rel` (the
truncation *mechanism* it dodged) — but never grepped the bare umbrella `Perm`,
which is a landed prelude global (`crates/ken-elaborator/src/prelude.rs`,
`Perm := ‖Perm_rel‖`, a *different, proof-relevant* definition — line numbers
drift, grep the symbol). Three-way `37 §6` ↔ `57 §3.1` ↔
prelude-global collision, **build-affecting** (the verified sort's AC6 binds
`Perm`), surfaced only a WP later — CAT-4's `58` re-referenced "the `Perm`
move," CV caught it → a queued `permCount`-rename / AC6-repoint follow-up.
**Apply: before pinning a NAME to a new definition, `git grep '"<Name>"'` the
prelude/globals AND the spec — a grep for the *variant* (`Perm_rel`) is not a
grep for the *umbrella* (`Perm`). Naming is a define-site grounding step, not
only a reference-site one; a re-used name with a divergent definition is a
latent collision that detonates a WP later, in someone else's reconcile.**

Sibling of laundered citation authority (a citation naming a specific id/status
is a claim about *now*, verify at source) and kernel backed claim grep the
emission not the name (names are not mechanisms — grep the producer, not the
label).
