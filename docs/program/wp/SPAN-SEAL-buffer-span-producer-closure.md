# SPAN-SEAL — seal the `BufferSpan` producer surface

**Owner:** Team Foundation · **Size:** S–M · **Risk:** low mechanism, high
care · **Branch:** `wp/SPAN-SEAL-span-producer-closure` · one branch, one
Decision.

**Blocking:** RT-PARITY is held at `506fa393` until this lands. This is the
prerequisite, not a parallel task.

---

## 1. Objective

Restore the locked `BufferSpan` abstraction: **no public declaration may
produce a `BufferSpan`.** Replace the escaped public producer with a public
*proposition + checked lemma*, keep the advance step private, and point the
published `writeAll` exact-prefix law at **the mechanism `writeAll` actually
executes**.

## 2. The grounded defect

`crates/ken-elaborator/src/prelude.rs:2076` installs, into source globals:

```ken
fn write_all_advance_span (span : BufferSpan) (count : TransferCount) : BufferSpan =
  PrivateBufferSpan (add_int (buffer_span_start span) (transfer_count_int count))
                    (transfer_count_remaining count)
```

It is **absent** from the private-name closure at `prelude.rs:2111+` (verified
by reading the list, not by trusting a line number). So checked source can mint
a `BufferSpan` without naming `PrivateBufferSpan`.

**This violates a locked spec clause.** `spec/30-surface/38-ffi-io.md:356-365`:

> a constructor-private immutable `BufferSpan` for the exact current live
> subrange … User code can neither forge the budget nor choose a different one.

The breach is **not** primarily about arithmetic overflow. The two arguments are
not indexed to the same request, span, or buffer, so source code can combine
**unrelated host-minted values** and choose a new start/budget. That is ordinary
semantic forgery, available at small values. **A `u64` bound check would repair
only the magnitude symptom and leave the abstraction breach intact** — the
Architect ruled this explicitly (`evt_1ppsszssn593s`), and it is the trap to
avoid: the reported issue arrived as an overflow story, and the overflow story
has a cheap fix that does not solve the problem.

### 2a. The second defect underneath it — the law is about a proxy

The real loop constructs its span **inline** at `prelude.rs:2044-2046`:

```ken
private_write_all_fuel a file (add_int file_offset (transfer_count_int count)) buffer
  (PrivateBufferSpan (add_int (buffer_span_start span) (transfer_count_int count))
                     (Suc remaining)) rest
```

`writeAll` **never calls `write_all_advance_span`.** Therefore the published
catalog lemma `write_all_preserves_exact_prefix`
(`catalog/packages/Capability/System/IO.ken.md:19-24`) proves a property of a
*faithful restatement* of the step rather than of the step itself. Sealing the
helper and routing the real recursion through it converts that law from
about-a-proxy to about-the-mechanism. **This is why SPAN-SEAL is a net
strengthening, not a removal.**

> ⚠ **Elaboration hazard — verify, do not argue.** The loop spells the new
> budget `(Suc remaining)`; the helper spells it `transfer_count_remaining
> count`. These are definitionally equal **only inside that match arm**, where
> `remaining` was bound by matching `transfer_count_remaining count` against
> `Suc remaining`. Refactoring the loop to call the shared helper must be shown
> to elaborate and to keep the five laws checking. If it does not elaborate,
> **stop and report** — do not reshape the loop's recursion to force it.

## 3. Consumer inventory (repo-wide, verified)

| Site | Role |
|---|---|
| `crates/ken-elaborator/src/prelude.rs:2076` | the definition |
| `crates/ken-elaborator/src/prelude.rs:2080` | its `proof exact_prefix` |
| `catalog/packages/Capability/System/IO.ken.md:22,24` | **the only** landed source consumer |
| RT-PARITY probe (`506fa393`, unmerged) | held; will be removed there, not here |

Grep across `crates/`, `catalog/`, `conformance/`, `spec/`, `docs/` returns
exactly these. **No spec or conformance file names the helper or its proof.**

⇒ **This is a code repair to conform to an already-locked spec.** It is **not**
a spec amendment and **not** an ADR. No enclave ruling gates the start.

## 4. What is NOT wrong (do not "fix" these)

- **`writeAll`'s public API.** Unchanged. Sealing the helper does not redesign
  it — the helper is not on its execution path.
- **The other four laws** (`terminates`, `success_is_complete`,
  `preserves_first_error`, `all_success`). They are about non-carrier
  observables and stay exactly as they are.
- **`PrivateBufferSpan`'s existing sealing.** Already correct.
- **Native / host code.** `crates/ken-runtime/` and `crates/ken-host/` stay
  **byte-unchanged.**
- **The runtime's own span tracking.** This is a *source-surface* repair only;
  no runtime behavior changes. No wire identity changes.

## 5. Required shape (Architect ruling `evt_1ppsszssn593s`, binding)

1. **Keep the advance step private.** Have `private_write_all_fuel` call a
   single private advance helper rather than duplicating the formula, so the
   law is about the executed mechanism (§2a).
2. **Remove `write_all_advance_span` from source globals** — add it to the same
   private-name closure as `PrivateBufferSpan` / `private_write_all_fuel`.
3. **Expose a proposition, not a carrier producer.** Public, observer-only —
   e.g. `write_all_exact_prefix_prop span count : Prop` whose transparent body
   states the budget equality over the private transition — plus a checked
   lemma proving it. **The catalog wrapper names only that proposition/lemma;
   it must not call or return the private `BufferSpan` transition.** The five
   kernel-checked laws stay human-visible in the catalog.
4. **Pin both closure edges** (see AC-3).

## 6. Acceptance criteria

- **AC-1 — sealed.** Checked source rejects **both** `PrivateBufferSpan` **and**
  `write_all_advance_span`. Both pinned absent from `env.globals`.
- **AC-2 — laws intact and re-aimed.** All five `write_all_*` laws remain real
  checked bodies (`transparent_body(...).is_some()`), the catalog `System.IO`
  fences still elaborate, and the exact-prefix law now concerns the advance step
  `private_write_all_fuel` actually executes.
- **AC-3 — ★ the closure is DERIVED, and derived MODULO DEFINITIONAL EQUALITY.**
  This is the criterion that matters most; read the rationale below before
  implementing it.
  **State of the AC after two blocks — read this as a PROPERTY, and do not
  read any API name below as the specification.**

  In `crates/ken-elaborator/tests/px8f_buffer_io_surface.rs`, derive the set of
  **every public global whose result type is `BufferSpan`** and assert it
  equals an exact expected set (production: **`{}`**). The derivation must be
  closed along **three independent axes**, each of which has already been
  breached once:

  1. **Modulo definitional equality** — weak-head-reduce against the real
     `GlobalEnv` *before* the Pi decision and *after* every codomain step,
     carrying a `Context` and pushing each Pi domain, then compare the
     **reduced** head. (Breach 1: a syntactic head-match let a transparent
     alias through.)
  2. **Over every category of public global** — top-level declarations *and*
     **constructors**, which live in a separate index and are **not**
     reachable through the declaration accessor. (Breach 2: `lookup(*id)?`
     silently dropped every constructor.)
  3. **★ Closed by construction against categories that do not exist yet** —
     if an `env.globals` id resolves to **neither** known category, the test
     must **FAIL LOUDLY**, never `?`-filter it away.

  **Axis 3 is the one that ends this.** Axes 1 and 2 were each found by a
  reviewer *after* the fact, and enumerating categories will keep losing to the
  next unenumerated one. A loud failure on the unknown case converts "a
  category I did not think of passes silently" into "a category I did not think
  of breaks the build" — which is the only version of this AC that is closed
  independently of the author's imagination.

  A newly added `BufferSpan` producer must **fail this test by default**, in
  any category.

  > **⚠ AMENDED 2026-07-21 after CV blocked `8f625666` — the first
  > implementation of this AC satisfied its letter and still admitted a
  > bypass.** A **syntactic** walk (strip raw `Term::Pi` codomains, raw-match
  > `Term::IndFormer{ id == BufferSpan }`) is **not sufficient**. CV
  > demonstrated the hole with a reversible mutation that stayed green:
  >
  > ```ken
  > def BufferSpanAlias = BufferSpan
  > fn escaped_alias (span : BufferSpan) (count : TransferCount)
  >   : BufferSpanAlias = write_all_advance_span span count
  > ```
  >
  > That declaration is public, elaborates, and really does produce a
  > `BufferSpan` through the private transition — yet the derived set stayed
  > `{}`. **Required:** weak-head-reduce the declaration type against the real
  > `GlobalEnv` **before deciding whether it is a `Pi`**, and again **after each
  > codomain step**, then compare the *reduced* head to `BufferSpan`. This
  > closes whole-function-type aliases as well as result aliases. The alias
  > producer above (or an equivalent reversible discriminator) **must appear in
  > the derived set.**
  >
  > **Do not weaken the sealed product surface to accommodate the test.**
- **AC-4 — positive control still reaches.** A checked program still obtains a
  span/count only from `ReadSome` and drives the public consumers (`writeAt`,
  `freeze`, `writeAll`) successfully. AC-1 must not pass by breaking the
  surface.
- **AC-5 — scope.** `crates/ken-runtime/` and `crates/ken-host/` byte-unchanged
  vs `origin/main` (`git diff --stat origin/main...HEAD -- crates/ken-runtime/
  crates/ken-host/` empty). `rustfmt --check` exit 0 on every touched file.
- **AC-6 — no-regression means green in CI**, never a local `--workspace` run.
  Targeted `scripts/ken-cargo` only (`-p ken-elaborator`, plus the named
  integration suites — `--lib` alone is not sufficient for a prelude-surface
  change).

> ### ★ Why AC-3 is written this way — the whole point of this WP
>
> `px8f_buffer_io_surface.rs:59-67` **already** asserts a private-name list, and
> `write_all_advance_span` is not in it. The list was built by enumerating what
> the author remembered to seal, not by deriving what must be sealed — so the
> escaped producer passed a test whose stated purpose was to catch exactly this.
>
> The same defect shape then repeated one layer up: RT-PARITY argued
> BufferFreeze unreachability from *direct constructor-name rejection*, which is
> an enumeration, and the property actually needed was **closure under
> composition of every public producer**. QA, CV and the Steward all read that
> argument's shape — non-degenerate pair, fails-if-premise-breaks — and credited
> it without checking that its discriminator matched its claim. The Architect
> caught it; CV converged independently.
>
> **A better allowlist would leave this defect one addition away from
> recurring.** Derive the set from the elaborated environment so the closure is
> a property, not a memory. Do not substitute a source grep: the prelude is
> Rust-emitted, so grepping `.ken` sources misses it entirely.
>
> **★★ And the defect has now recurred once MORE, one level down — including in
> this AC as I first wrote it.** The first respin derived the set *syntactically*
> and CV blocked it, because a syntactic head-match is **still an enumeration in
> disguise**: it enumerates *spellings* of the result type rather than deciding
> the type's *meaning*, so any transparent alias walks straight past it. My
> original AC-3 said "walk the Pi codomain to the head and compare" — I specified
> a **mechanism** and called it a property, and the implementer built exactly
> what I wrote. **That under-specification is mine, not the ring's.**
>
> The general rule, now three layers deep on this one surface: **"derived" is
> only as strong as *derived modulo what*.** State the equivalence the closure
> must hold under — here, Ken's definitional equality — or a reader will pick
> the cheapest one that satisfies the words.
>
> **★★★ FOURTH occurrence, and the diagnosis of my own drafting.** The Architect
> then blocked the *reduction-correct* oracle because it began each arm with
> `env.env.lookup(*id)?` — and `GlobalEnv::lookup` resolves only top-level
> declarations (`env.rs:342`). Constructors live in `ctor_index` behind
> `GlobalEnv::constructor` (`env.rs:404`), so **every public constructor took
> the `?` path and vanished from the derived set unexamined.**
>
> **This one is squarely mine, twice over.** My AC-3 originally said *"look up
> each `GlobalId` via `GlobalEnv::lookup`"* — I **named an API and called it a
> property**, so the implementer used exactly that API and inherited its blind
> spot. Worse, I had grepped the accessors while drafting and `constructor` at
> `env.rs:404` was **in my own search output**; I wrote only `lookup` into the
> AC anyway.
>
> **So the meta-lesson is about how an AC is written, not about this surface:**
> an acceptance criterion that names a **mechanism** transfers that mechanism's
> blind spots into the deliverable, and it does so invisibly — the
> implementation is *correct against the words*. State the **property**, name
> the **axes it must be closed along**, and require a **loud failure on the
> unhandled case**; let the implementer choose the mechanism. Every layer of
> this defect except the first was introduced by a specification that described
> *how* instead of *what*.

## 7. Guardrails

- **Do not smuggle RT-PARITY work in here**, and do not touch
  `wp/RT-PARITY-interp-native`. Its respin happens after this lands.
- **Do not add a `u64` bound check as the fix** (§2). If you conclude a bound
  check is *additionally* warranted, report it — do not fold it in.
- **If the §5.1 refactor does not elaborate**, stop and report with the grounded
  alternatives rather than reshaping the recursion (§2a hazard).
- One branch, one Decision. Required votes: **QA + Architect §14**, and **CV** —
  the diff changes a published catalog surface, so the diff-scope pulls CV.
- No WP-token identifiers in production source.

## 8. Sequencing

`SPAN-SEAL` merges → RT-PARITY rebases, drops the derived-span test that
depended on the escaped helper, retains its six approved executable narrowing
differentials and internal BufferFreeze dispatch-boundary pins, and reports
source-level BufferFreeze narrowing as structurally unconstructible **on the
landed sealed producer closure** — with both private names pinned absent. No
astronomical-execution case is needed, because the offending public composition
route will no longer exist.
