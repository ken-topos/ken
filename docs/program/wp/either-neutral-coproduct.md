# Either · rename the neutral coproduct `Sum`/`InL`/`InR` → `Either`/`Left`/`Right`

**Owned by the Steward** (frame); **home: Runtime** (effect-composition machinery
owner — `eval.rs` peel logic + `effects::state` hand-built inductives), coordinate
the elaborator-prelude touch. **Operator-ruled design change** (2026-07-10, Pat):
the user-facing neutral coproduct is spelled **`Either a b = Left a | Right b`**,
the idiom every functional programmer knows — NOT `Sum a b = InL a | InR b`
("`Sum` reads as addition to everyone outside a narrow category-theory
population"). Logged as judgment call **L5** (supersedes L4's neutral-coproduct
point). This is a **trust-root prelude declared-type change**, landed through the
full ring + Architect gate, **logged prominently** for the operator.

## The decision (operator, corrects L4)

L4 subsumed `Either` into `Result` on a structural-isomorphism argument. That was
imprecise on the axis that matters for **humans-read**: `Either`/`Left`/`Right`
and `Result`/`Ok`/`Err` carry **different reader semantics** —

- **`Result e a = Err e | Ok a`** — names encode a **fallible computation**; one
  side is *the error*. Stays distinct, uncontested (Rust/F#/Elm all keep a named
  error type separate for call-site clarity).
- **`Either a b = Left a | Right b`** — a **neutral disjunction** ("one of two
  types"); the error reading is a *convention* (left-biased), not intrinsic.

So `Either` never really duplicated `Result` — it duplicates the **neutral
coproduct slot**, which Ken fills today with the category-theoretic `Sum`/`InL`/
`InR`. The operator ruled: that slot should be the idiomatic `Either`/`Left`/
`Right`. **One neutral coproduct, idiomatically named** (subsume-don't-proliferate
holds — we are *renaming*, not adding a third spelling); `Result` stays separate.

## Grounded blast radius (all use sites — this WP must hit every one)

`Sum`/`InL`/`InR` is **hand-built** (a real `Decl::Inductive`, kernel-rechecked)
in `crates/ken-elaborator/src/effects/state.rs::declare_sum`, registered as globals
in `prelude.rs:270-273`. It is **non-dependent** (`Sum a b = InL a | InR b`,
`a b : Type0`) and serves BOTH effect composition (`Sum (StateOp s) f`, the `⊕`
container) and value-level neutral sums. Use sites (grep-confirmed):

- `crates/ken-elaborator/src/effects/state.rs` — `declare_sum` (the inductive) +
  `injectL`/`injectR` + `get`/`put`'s hand-baked `InL`.
- `crates/ken-elaborator/src/prelude.rs` — global registration (`:271-273`) +
  the `Sum a b = InL a | InR b` doc block (`:157-176`).
- `crates/ken-interp/src/eval.rs` — the D3.2 coproduct **peel** logic (`:1840`+,
  strips `InL`/`InR` to the innermost base tag). The trickiest site — get it right.
- Effect-composition tests: `effect_composition_resp_sum_acceptance.rs`,
  `effect_composition_synthetic_peel_probe.rs`,
  `effect_composition_state_console_e2e.rs`, `fs_read_file_lines_flip_e2e.rs`, and
  any sibling under `crates/ken-interp/tests/` / `crates/ken-cli/tests/`.
- Spec: `spec/30-surface/36-effects.md` (effect composition) +
  `spec/30-surface/34-data-match.md` (the landed L4 erratum note + three list-sites
  — **rewrite-and-restore, NOT find-replace**; see "Downstream reconciles" for the
  pinned ACs — the L4 subsume erratum is superseded by L5).

**Zero `crates/ken-kernel` delta** (confirmed — `Sum`/`InL`/`InR` never touch the
kernel crate; the hand-built inductive IS kernel-rechecked, but renaming the
type/ctors changes only the elaborator's decl, which the kernel re-checks
identically). **No back-compat alias needed — "no users but us"** (operator), so a
clean atomic rename, not a deprecation shim. **No `Left`/`Right`/`Either` name
collision** (grep-confirmed absent today).

## Open technical sub-question (Architect / build-team call, NOT pre-decided)

`Sum` is hand-built in `effects::state`, but it is **non-dependent** — so it may
now be expressible as a plain surface `data Either a b = Left a | Right b` in the
prelude (like `Option`/`Result`), whose params default to `Type0` (the DS-8
finding) — exactly what the effect use needs. **Probe it:** if a surface `data`
declaration is a clean drop-in (reflect-don't-extend — fewer hand-built inductives
in the TCB surface), prefer it and register the effect machinery against the
surface-declared globals; if there's a real reason it must stay hand-built (a
universe/dependent constraint the probe surfaces), keep it hand-built and just
rename. Either way the mandate — **one coproduct named `Either`/`Left`/`Right`** —
is unchanged; only the declaration site is the open axis. Ground it, don't assume.

## Boundary / constraints

- **Pure rename, zero semantic change.** The coproduct's structure is identical;
  the kernel re-checks the renamed inductive identically. If the change appears to
  need any *behavioral* edit beyond renaming (+ the optional surface-migration),
  STOP and hand back — it shouldn't.
- **Zero kernel-crate delta; zero new `Axiom`/`postulate`/`declare_primitive`.**
  The inductive stays a `declare_inductive` (kernel-rechecked), never downgraded.
- **AC — completeness of the rename:** a residual `git grep -nw -E 'Sum|InL|InR'`
  over `crates/` + `spec/` shows **no** surviving reference to the old coproduct
  (excluding unrelated `Sum` uses — e.g. numeric addition, `resp_sum` if it is
  kept as a distinct name; scope those explicitly). `Either`/`Left`/`Right`
  resolve as globals; a new value-level acceptance test constructs and matches on
  `Either Int String` (`Left`/`Right`).
- **AC — effect composition intact:** the full effect-composition suite (all
  `effect_composition_*` + the FS/console e2e) green pre- and post-rename, zero
  regression. The `eval.rs` peel logic strips `Left`/`Right` identically.
- **Note `resp_sum`:** the response-family helper `resp_sum` (`prelude.rs:279`)
  is named for the coproduct — decide (Architect) whether it renames to
  `resp_either` for consistency or stays (it is an internal effect helper, lower
  stakes). Flag the call in the handback.

## Downstream reconciles (named, this WP or fast-follow)

- **The L4 `Either`-subsume erratum is SUPERSEDED, not amendable by find-replace
  (Architect catch `evt_60ahxgw3vpnqn` — AC, not optional).** It **already landed**
  (`main @ dcc34ed`, PR #446) *before* the operator reopened, so it can't be
  withdrawn — L5's spec touch must reverse it in place. The erratum's whole thesis
  — *"`Either` is **not** a distinct type; subsumed by `Result`; no first-party
  `Either` is declared"* — is exactly what L5 reverses. A mechanical `Sum a b`→
  `Either a b` swap yields a note that **asserts and denies `Either`'s existence in
  one breath**. So the spec touch must, as pinned ACs:
  - **(i) REWRITE** the `34-data-match.md` subsume note to the L5 truth: `Either a
    b = Left a | Right b` is a **distinct declared neutral coproduct** (`Left`/
    `Right`); `Result e a = Err e | Ok a` is a **distinct error sum** (`Err`/`Ok`);
    both honest first-party sums, different reader semantics. No "subsumed"/"no
    first-party Either" language survives.
  - **(ii) RESTORE `Either`** at the three list-sites the erratum dropped it from —
    `README.md:42`, `34-data-match.md:5`, `34-data-match.md:633` — since under L5
    `Either` is declared again and belongs in those lists.
  - **(iii) The `:56` "ordinary prelude `data` decls" claim is CONDITIONED on
    sub-question (a):** if `Either` migrates to a surface `data`, restore it to
    that true set ("`Result`, `Option`, `Either` are ordinary prelude `data`
    decls"); if it stays **hand-built**, do NOT — phrase it as "a prelude type
    (hand-built like `ITree`)", parallel to how `Sum` was described. Match the
    as-built declaration site, don't over-claim.

  The Architect gate verifies the note is **coherently rewritten**, not swapped.
- **DS-3** (in-flight, `Option`/`Result` combinators) is **unaffected** — Result
  and Option are separate; do not block or expand it. `Either` combinators
  (`either`/`mapLeft`/`mapRight`) are a possible **follow-on** once the type lands,
  not this WP.
- **Package naming:** DS-3's recommended `Data/Sums/Sums.ken` home may want to be
  `Data/Sums` still (Option/Result/Either are all "sums") or reconsidered — minor,
  Foundation's call at DS-3, not a blocker here.

## Gate

Ring: Runtime build → runtime-qa independent re-derivation (run the full
effect-composition suite + the new `Either` value-level acceptance yourself) →
**@architect gate** (fidelity: one coproduct, idiomatic names, `Result` untouched;
soundness: zero-kernel-crate-delta, inductive stays kernel-rechecked, zero new
`Axiom`; + rule the hand-built-vs-surface-`data` sub-question and the
`resp_sum`/`resp_either` naming) → `git_request` to Steward. **CI-gated** (touches
`crates/` + `spec/`). Own retro. Resource discipline (`CARGO_BUILD_JOBS=2`, scoped
`-p` tests — the effect suite spans ken-elaborator/ken-interp/ken-cli). Flag every
judgment call for the operator's log — especially the declaration-site decision.
