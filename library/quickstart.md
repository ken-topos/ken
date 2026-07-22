# Quickstart

This page gets you from a clean checkout to reading one real, checked Ken
program with a trust-aware eye. It does not teach the language — that is
`learn/reading-ken/` (in progress) — it teaches the *loop*:
build the toolchain, check a program, run it, format it, then read what it
actually claims.

## 1. Install and use the current toolchain

Ken's CLI is built from this repository with Cargo (`README.md` §Build):

```bash
cargo build --workspace --locked
cargo run -p ken-cli -- help
```

`cargo run -p ken-cli -- help` prints the CLI's subcommands, including the
three this page walks through next: `check`, `run`, `fmt`.

## 2. Check and run one program

The program is `catalog/guide/decomposition-abstraction.ken.md` — a real,
already-checked catalog artifact (not a page written for this exercise). It
is primarily a design-notes guide; its final fenced block is a genuine,
executable `proc main` that the catalog's fence-checking gate holds to the
same standard as every other Ken program, so it doubles honestly as this
page's runnable example.

**Scope note:** this page's "check and run one program" step is exempt
from the `catalog/packages/` constraint that governs the fragments in
`learn/reading-ken/`. No `catalog/packages/` artifact is itself runnable
today (nothing there has a `proc main`); this page's population is
"an executable demo," a different population from the reading curriculum's
checked fragments. `catalog/guide/decomposition-abstraction.ken.md` is
still a real catalog artifact, not an invented toy — it satisfies that
constraint without forcing a package-fragment substitute that does not
exist.

`ken check` elaborates every fence in a file (including the checked
`` ```ken example `` blocks earlier in the file) without driving IO —
correct for a pure-library entry, but this file *does* have a `proc main`,
so run it instead of just checking it:

```
$ cargo run -p ken-cli -- run catalog/guide/decomposition-abstraction.ken.md
decomposition guide ok
```

If you only want elaboration without driving IO (the right call for a
pure-library package with no `proc main`), the CLI has a dedicated mode:

```
$ cargo run -p ken-cli -- check catalog/guide/decomposition-abstraction.ken.md
```

(exits 0, no output — `ken check` never drives IO even when a file happens
to have a runnable shape, per the CLI's own documented `check`-vs-`run`
contract (`docs/program/07-catalog-style-guide.md` §3); that is a property
of the command, not this file.)

## 3. Format it

```
$ cargo run -p ken-cli -- fmt --check catalog/guide/decomposition-abstraction.ken.md
```

This file is already in canonical form, so `--check` exits 0 with no
diff. Drop `--check` to canonicalize a file you're editing yourself;
`fmt` is idempotent — formatting already-canonical source is a no-op.

## 4. A trust-aware reading exercise

Open `catalog/guide/decomposition-abstraction.ken.md` and look at its final
fenced block:

```ken
program capabilities FS APartial

proc main
      (_input : ProcessInput) (_caps : ProgramCaps APartial)
    : HostIO APartial ExitCode
    visits [Console] =
  host_program APartial (print_line "decomposition guide ok")
```

Answer these before moving on — the point is to practice reading the
declaration itself, not to take this paragraph's word for it:

- **What authority does this program request?** `program capabilities FS
  APartial` is the admission-boundary header's authority manifest — it is
  what the runner reads to mint `ProgramCaps` (`spec/30-surface/
  33-declarations.md` §3.2.1). `_caps` is bound but never used in the
  body — so despite requesting `FS` capability, this particular `main`
  never exercises it. A reviewer should notice an unused capability
  parameter as worth asking about, not wave past it.
- **What does it actually do?** `visits [Console]` declares the *effect
  row* this `main` is checked against (`spec/30-surface/36-effects.md`
  §1), and the body — `print_line` through `host_program` — is exactly a
  Console write. The capability it requests (`FS`) and the effect it
  performs (`Console`) are not the same thing; this file is small enough
  to check that gap by eye.
- **What class of claim is `main` itself?** Trick question — **there is no
  specification claim here to classify.** The four-way status
  (`proved`/`tested`/`delegated`/`unknown`) applies **per specification
  claim** (`spec/20-verification/21-spec-syntax.md` §5.2), and this `main`
  carries no `requires`/`ensures`/`assume`/`test` clause for anything to
  attach to. What steps 2 above actually established: the kernel
  type-checked the declaration (every declaration gets this, proved or
  not — it is not itself a proof of a stated property), and running it
  produced exactly this output on this one invocation — an empirical
  observation, not a `tested` status, since `tested` specifically means a
  visible `assume`/`test`-tagged clause registering a runtime/test and
  generator obligation (§5.2-5.3), which nothing here declares. Calling
  this run "tested" in the spec's sense would overclaim what one ad-hoc
  invocation actually shows.

`introduction.md`'s four-way vocabulary — **proved**, **tested**,
**delegated**, **unknown** — classifies *specification claims*, and this
exercise's point is recognizing when a declaration has none to classify.
The `learn/reading-ken/` fragments (`learn/reading-ken/fragments.md`)
include entries that do carry real proof terms and stated claims, which is
where the four-way distinction has something to bite on.

---

**Grounds this page:** `README.md` §Build (toolchain install/use commands);
`catalog/guide/decomposition-abstraction.ken.md` (the program checked, run,
formatted, and read above — verbatim, not paraphrased); `spec/20-
verification/21-spec-syntax.md` §5.2-5.3 (the four-class verification-status
vocabulary, and the exact `tested` disposition the reading exercise turns
on); `spec/30-surface/33-declarations.md` §3.2.1 (`program capabilities`
and `ProgramCaps` — what the authority manifest means, not just that the
tokens occur); `spec/30-surface/36-effects.md` §1 (the `visits` effect-row
declaration); `docs/program/07-catalog-style-guide.md` §3 (the `ken
check`-vs-`ken run` contract cited in step 2). Authority class: `tutorial`
— this page walks a reader through a procedure; every claim about what a
declaration or a status label *means* is cited to the exact spec section
that defines it, rather than asserted on this page's own authority.

**Normative-language sweep:** this page makes two kinds of claim — what a
cited spec section says a construct means (cited to that section, not
asserted), and what the cited program's own text/behavior directly shows
(checked by running the commands above, not paraphrased from memory).
Neither kind is asserted on this page's own authority. A direct observation
of which tokens occur in the program's text is not, by itself, evidence for
what those tokens *mean* — that distinction is why each meaning claim above
now carries its own spec citation rather than resting on the program text
alone. Grep for an unattributed "must"/"is required to"/"always" outside
those two cases found none at authoring time; re-run that grep against this
file's current text before trusting this sentence, since it is a claim
about the past, not a gate.
