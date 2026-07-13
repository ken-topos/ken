# WP B4 — `.ken.md` splicing + prose byte-identity (kenfmt build)

Owner: **Language team**. Single build lane (**build**; the layout contract
already exists — WP S `spec/30-surface/31-lexical.md §Literate .ken.md source`,
and the layout engine is **B3**). Consumes **B1** (`FormattableSource`), **B2**
(token-kind canon), and **B3** (`layout::format_ken` — the string→canonical
pipeline). Design source of truth: **WP S** (the four fenced roles + the
parse-first exemption, already normative — no new design ruling). Size **S**.
Base: `origin/main @ 549bf8d9` (B3 landed; re-verify cites at pickup).

## Objective

Extend the canonical formatter to **literate `.ken.md` files**: format every
recognized Ken **fence body** in place through the B3 layout engine, splice the
results back into the document, and guarantee that **every byte of Markdown
outside recognized fence bodies and their fence markers is identical** to the
input. This is the last build WP before the capstone; it adds **no new layout
mechanism** — it reuses B3's `format_ken` per fence body and B2's token canon for
the narrow non-parseable exemption. The only real build is a **data-model
extension of the literate extractor** so the splicer sees every fence's role +
body range + marker spans (the current `KenMdExtraction` drops `ken ignore` and
retains no role/opener spans).

## Fixed inputs — SETTLED (WP S `§Literate .ken.md source`), do NOT reopen

WP S `spec/30-surface/31-lexical.md` already rules every disposition below. B4
**implements**; it does not re-decide. Verbatim contract:

- **Exactly four fenced roles**, classified byte-simply at column zero: `ken`,
  `ken ignore`, `ken reject`, `ken example`. A canonical opener is column-0,
  exactly three backticks immediately followed by `ken`, and — when a role word
  is present — **exactly one** ASCII space before it. A canonical closer is
  exactly three backticks at column zero. (This is already what the landed
  extractor `crates/ken-elaborator/src/literate.rs::classify_fence_opener`
  admits; a non-canonical `ken`-tagged opener is an **extraction-time error**
  (`UnrecognizedRole`), never a reformattable input — so there is nothing for B4
  to normalize in the markers. **Preserve fence markers byte-identical.**)
- **Recognized fence bodies are formatted in place.** Adjacent fences are **not
  joined**; declarations are **not moved between fences**; **roles are not
  changed**; fences are **not reordered**.
- **Parse-first, role-gated exemption (the single disposition that decides body
  treatment):**
  - **A body that parses** — in **all four roles** — receives the **full
    canonical form** (B3 layout via `format_ken`).
  - **The ONLY layout exemption**: an intentionally **incomplete `ken ignore`**
    body **or** an intentionally **syntax-erroring `ken reject`** body **that
    cannot be parsed**. Such a body receives **token-kind-aware canonicalization
    only** (B2's canonical spellings over the tokens recognizable without
    guessing structure); **its layout and protected regions remain unchanged.**
  - **A parse failure in a `ken` or `ken example` body is a hard error, not an
    exemption** — `ken` tangles into the module and `ken example` must elaborate,
    so both must parse. Only `ken ignore` / `ken reject` may legally be
    non-parseable, and only they fall back to token-canon. **No other fence or
    source region is exempt.**
- **Prose + markers are inviolate.** Every byte outside recognized fence bodies
  **and** their fence markers is identical — the prose-identity soundness AC.

## Scope

- `crates/ken-elaborator` only. Two pieces:
  1. **Extractor data-model extension** (`src/literate.rs`): expose, per
     recognized fence, its **role**, **body byte-range** (original-`src`
     offsets), and **opener/closer marker spans** — **including `ken ignore`**,
     which today is folded into prose (`FenceState::ProseFence`) and records no
     range. Do this **additively** (a new richer accessor / struct field);
     **do not change** `extract_ken_md`'s existing `source` / `compiled_ranges` /
     `reject_ranges` / `example_ranges` contract that elaboration depends on, and
     do not change the blanking behavior that feeds the compiler. The formatter
     reads the new per-fence view; elaboration keeps its current view.
  2. **The splicer** (the `.ken.md` formatter path, sibling to B3's `.ken`
     path): for each recognized fence body, produce its canonical replacement
     (per the parse-first rule), then splice replacements back **last-range-first
     (descending byte offset)** so each splice leaves the earlier offsets valid,
     and reassemble the document.

### Out of scope (later kenfmt WPs / capstone)

- **No one-time catalog reformat.** B4 runs its gate **read-only** over the
  literate corpus (`catalog/**/*.ken.md`, `spec/**` literate files if any,
  `examples/rosetta/**`); the actual rewrite of `.ken.md` files to canonical form
  is **capstone C** (atomic, strict gate), not B4.
- **No change to B1/B2/B3.** Consume `FormattableSource`, the token canon, and
  `format_ken` **read-only**. B4 adds the `.ken.md` orchestration around them; it
  does not touch the layout algebra, the printers, or the paren rules.
- **No new fence roles, no CommonMark attribute parsing, no indented/tilde
  fences.** The four column-0 backtick roles are the whole surface (WP S).
- **No `ken fmt --check` CI wiring / catalog rewrite** — capstone C.

## The preservation gate at B4 (read carefully)

B4's gate is **B3's three-part property, applied per fence body, PLUS the
prose-identity invariant**, run **continuously over the whole literate corpus,
read-only**:

1. **Prose + marker byte-identity (the soundness AC).** Mask every recognized
   fence **body** range in both the original and the formatted document (leaving
   prose, blank lines, and the fence markers themselves in place); the masked
   remainders must be **byte-identical**. Equivalently: concatenating every
   non-body segment (prose + openers + closers) of the output reproduces the
   original's non-body segments **exactly**. A single changed prose byte fails
   the WP.
2. **Per-body parse-preservation** — for each **parseable** body, the spliced
   canonical form parses to an AST equal to the body's, modulo trivia / spans /
   sanctioned aliases (B3's AC1, inherited).
3. **Idempotence (byte)** — `fmt(fmt(doc)) == fmt(doc)` byte-exact over the whole
   document (markers, prose, and every body), and each token-canon-only exempt
   body is itself idempotent.

## Acceptance criteria

- **AC1 — extractor exposes every fence.** A per-fence view carries `{role, body
  range, opener span, closer span}` for **all four roles including `ken
  ignore`**; the existing `extract_ken_md` contract (compiler `source`,
  `compiled_ranges`, `reject_ranges`, `example_ranges`) is **unchanged** and its
  tests stay green. Assert a fixture containing one fence of **each** role plus a
  non-`ken` (other-language) fence and bare prose, and confirm the view's ranges
  and roles.
- **AC2 — parse-first body treatment.** A parseable body in **each** of the four
  roles receives the **full B3 canonical form** (`format_ken`); a deliberately
  **incomplete `ken ignore`** body and a deliberately **syntax-erroring `ken
  reject`** body receive **token-canon only** (layout + protected regions
  unchanged); a **non-parseable `ken` or `ken example`** body is a **hard error**
  (assert the error, per `assert-specific-error-variant`, not merely `is_err`).
- **AC3 — splice correctness.** Replacements are applied **last-range-first**;
  a fixture with **multiple** fences of differing formatted-vs-original lengths
  reassembles with every body in its right place and **no offset drift** (assert
  by round-tripping the reassembled document through the extractor and matching
  body count/roles).
- **AC4 — prose + marker byte-identity (soundness).** The fence-masked comparison
  (gate #1) holds over a literate fixture with **long prose paragraphs between
  fences, blank-line runs, an other-language fence, and both a long `ken` body
  and a `ken ignore` fragment**; every non-body byte is identical. Add an
  **adversarial** fixture: prose that textually resembles a fence opener but is
  **not** column-0 / not canonical (must stay prose, untouched).
- **AC5 — idempotence + no-op stability.** `fmt(fmt(doc)) == fmt(doc)` byte-exact
  over the corpus; an **already-canonical** `.ken.md` is a **byte no-op** (formats
  to itself).
- **AC6 — the gate + build.** Prose-identity + per-body parse-preservation +
  idempotence green **read-only over the whole literate corpus**
  (`catalog/**/*.ken.md`, `examples/rosetta/**`, any literate `spec/**`).
  `scripts/ken-cargo test -p ken-elaborator` green **AND** literal `cargo build
  --workspace --locked && cargo test --workspace --locked` green (the literal
  locked CI is a first-class QA gate — a `.ken.md`/rosetta fixture lives in a
  wrapper-skipped crate; see `ken-cargo-workspace-does-not-match-ci-locked`).
  `git diff --check` clean; scope = `crates/ken-elaborator` (+ tests) only;
  **zero** kernel/prelude/semantics/Cargo/lock/`trusted_base()` delta
  (tool-internal formatter).

## Review

**Architect-terminal** (owns the kenfmt B-series contracts). The soundness AC is
**prose + marker byte-identity** and **per-body AST-preservation** — a formatter
that alters a byte of prose, moves a declaration between fences, changes a role,
or changes a body's parsed meaning is catastrophic. Team QA runs the four-part
gate (prose-identity + parse-preservation + idempotence + literal locked CI) over
the **whole** literate corpus, not a sample. No spec sub-lane and **no CV
conformance companion** is required: WP S is the single normative home for the
literate rules, and the golden `conformance/surface/formatting/seed-canonical-
format.md` already covers `.ken` layout; a `.ken.md` splicing golden is optional
and, if added, is CV's call — coordinate, don't block on it. `git_request` to
Steward; honesty-gated publish.

## Do-not-reopen guardrails

- **Prose + markers are inviolate** — byte-identity outside recognized bodies is
  the soundness AC; never rewrite a marker, join fences, move a declaration
  between fences, reorder fences, or change a role.
- **Parse-first, role-gated** — parseable → full B3 form (all four roles);
  non-parseable **only** in `ken ignore`/`ken reject` → token-canon only; a
  non-parseable `ken`/`ken example` is a hard error. Do not invent a per-role
  layout table; the disposition is parseability + the role gate.
- **Splice last-range-first** — descending byte offset; never recompute offsets
  forward after an edit changes a length.
- **Consume B1/B2/B3 read-only** — no re-lex, no re-canonicalize, no layout-rule
  change; B4 is orchestration + the extractor data-model extension only.
- **Additive extractor change** — do not alter `extract_ken_md`'s existing
  compiler-facing contract or blanking; elaboration must stay byte-for-byte
  unaffected.
- **Read-only gate, not the reformat** — the one-time `.ken.md` rewrite is
  capstone C; B4's gate runs read-only over the corpus.
- **Surface/tool-internal only** — zero TCB delta.

## Notes

- **B3's entry point is `layout::format_ken(source: &str) -> Result<String,
  ElabError>`** (verify at pickup — it is the string→canonical pipeline; `Ok` =
  canonical text, `Err` = parse/elab failure that the parse-first rule keys on).
  B4 calls it per fence body; it does **not** re-implement layout.
- **The token-canon-only fallback reuses B2's canonical-spelling path** (the same
  one `format_ken` uses internally after parsing); for a non-parseable body,
  apply it **lexically** over the recognizable tokens **without** structural
  layout — verify B2 exposes a token-level canonicalizer usable off the parse
  path, and if it does not, the minimal such surface is the only new code the
  exemption needs (flag to Steward if it forces more than a thin wrapper).
- **`ken ignore` today records no range** (folded into prose) — the AC1 extractor
  extension is what makes ignore bodies visible to the token-canon exemption;
  without it an `ignore` fragment would pass through as prose (untouched), which
  is *almost* the same observable result but is **not** WP-S-faithful (S mandates
  token-canon over an ignore body's recognizable tokens). Build AC1 so the
  exemption actually runs on ignore bodies.
- **Capstone C is next and needs the catalog freeze** — B4 is the last read-only
  build WP; when B4 lands, the Steward opens the catalog-quiet window for C
  (tool + whole-catalog `.ken`+`.ken.md` rewrite + strict `ken fmt --check` gate,
  one atomic merge).
