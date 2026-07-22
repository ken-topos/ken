# Quickstart

This page gets you from a clean checkout to reading one real, checked Ken
program with a trust-aware eye. It does not teach the language — that is
`learn/reading-ken/` (in progress, this wave) — it teaches the *loop*:
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
  APartial` declares the capability surface; `_caps` is bound but never
  used in the body — so despite requesting `FS` capability, this
  particular `main` never exercises it. A reviewer should notice an unused
  capability parameter as worth asking about, not wave past it.
- **What does it actually do?** `visits [Console]` is the *effect* this
  `main` is declared to perform, and the body — `print_line` through
  `host_program` — is exactly a Console write. The capability it requests
  (`FS`) and the effect it performs (`Console`) are not the same thing;
  this file is small enough to check that gap by eye.
- **What class of claim is `main` itself?** It is an ordinary kernel-typed
  declaration, not a proof term — nothing here is `proved` in the sense
  `docs/PRINCIPLES.md` §5 and `spec/20-verification/21-spec-syntax.md` §5
  use that word. Elaborating and running it (steps 2 above) is empirical
  evidence that it behaves as shown on this input, which is the `tested`
  reading, not a proof that it always will.

`introduction.md`'s four-way vocabulary — **proved**, **tested**,
**delegated**, **unknown** — is exactly what you just applied. The
`learn/reading-ken/` chapters (this wave, in progress) practice the same
move against library-shaped fragments instead of a design-guide's demo
footer; see `learn/reading-ken/fragments.md` for the fragment set they use
and the mechanism recorded for each one's currency.

---

**Grounds this page:** `README.md` §Build (toolchain install/use commands);
`catalog/guide/decomposition-abstraction.ken.md` (the program checked, run,
formatted, and read above — verbatim, not paraphrased); `spec/20-
verification/21-spec-syntax.md` §5 (the four-class verification-status
vocabulary the reading exercise applies); `docs/program/07-catalog-style-
guide.md` §3 (the `ken check`-vs-`ken run` contract cited in step 2).
Authority class: `tutorial` — this page walks a reader through a procedure;
the one language-status claim it makes (the proved/tested/delegated/unknown
vocabulary) is cited to its spec section rather than asserted on this
page's own authority.

**Normative-language sweep:** every claim about what a declaration *means*
in this page traces to a cited spec section (the verification-status
vocabulary) or is a direct, checked observation of the cited program's own
text (capability/effect/parameter reading) — grep for an unattributed
"must"/"is required to"/"always" outside those two cases found none at
authoring time; re-run that grep against this file's current text before
trusting this sentence, since it is a claim about the past, not a gate.
