---
id: DOC-VALIDATION-BINDING
title: "validation vocabulary claims a 1:1 binding to the gates; nothing binds it"
status: active
owner: verify
size: S
gate: none
depends_on: [DOC-W0]
blocks: []
github: null
origin: adversary finding evt_59w0kkk1gf75e (2026-07-22), post-merge on DOC-W0 @ 6be9754b
---

> **Re-homed doc → verify, 2026-07-24 (Steward).** The mechanism is
> `crates/ken-cli/tests/library_documentation_gates.rs`, a Rust gate harness —
> not `library/` content. The doc ring's concurrency licence rests on being
> path-disjoint from `crates/`, so this is not theirs; Verify owns gates.
> ⚠ Consequently the COORDINATION §14a doc-only exemption does **not** apply:
> **the Architect's vote is required.** Kicked as Track 2 (`evt_3fv5d1men6r9y`),
> contention-checked against RT-NATIVE-FNSPLIT (disjoint crates + ledger).
>
> ⚠ **SCOPE GREW 2026-07-24: this WP now HOLDS the `library/` ledger axis**
> (`evt_79ytm04m0dt53`). The Architect's binding route tokenizes the
> unconditional delimiter gate as `transport-delimiter`, so it also changes
> **every `library/manifest.toml` validation list** and the **generated
> `library/STATUS.md`**. All three land in ONE branch/squash; STATUS.md is
> **regenerated**, never hand-edited. ⛔ The scripted publisher runs
> `gen-doc-status.sh --check` on `origin/main` *after* merge and **freezes the
> publisher path** on red without reverting — a manifest/STATUS mismatch does
> not fail the PR, it freezes everyone. Steward ruling `evt_tmv40vgtg63k`.
> **[[STR-BIJ]] is held behind this WP** for the same axis.

**Severity: lower than [`DOC-CURRENCY-ANCHOR`](DOC-CURRENCY-ANCHOR.md).** An
accuracy gap in a self-describing vocabulary, not a wrong result today. Does
**not** block Wave 1. Triage accordingly — but it shares that issue's
generator, so fixing them together may be cheaper than fixing either alone.

## The claim

`crates/ken-cli/tests/library_documentation_gates.rs:469-476`:

> `validation` names which checks apply to a record — it must be a closed,
> known vocabulary **tied 1:1 to the gates this file actually runs**, not free
> prose… **Every current gate below runs unconditionally over every entry**
> except `generated-current` … so the applicable set is **exact, not merely a
> subset check**.

## The mechanism

`gate_validation_tokens_are_closed_and_match_applicable_checks:505-533`
compares each manifest record's `validation` list against
`applicable_validation_tokens():487-502` — a hard-coded `BTreeSet` of six
strings (+`generated-current` for `kind == "status"`). **It never introspects a
single test function. It checks manifest-vs-shadow-list; it never checks
shadow-list-vs-reality.**

## Grounded

Every token occurs **exactly twice** — once in `KNOWN_VALIDATION_TOKENS:477`,
once in `applicable_validation_tokens:487`. **Zero occurrences in any gate
function body.** No token is bound to the gate it names; the correspondence
exists only in the comment.

**And the comment is already false, 80 lines below itself.** Seven per-record
gates iterate `for entry in &entries` (`:387, :423, :509, :563, :697, :741,
:1088`). Six have tokens; `:509` is the validation gate itself (self-referential,
fairly excluded). **`:563` —
`gate_manifest_scalars_reject_the_transport_delimiter` — runs unconditionally
over every entry and every string field, with no per-kind exemption,
structurally identical to `links` or `authority-class`, and has no token.** So
"every current gate below runs unconditionally over every entry" omits the gate
defined immediately below it.

## Direction of the error — today understatement, forward overstatement

Today: the delimiter check runs on every record but no record's `validation`
list says so. **Harmless.**

Forward: **overstatement, and that is the dangerous direction.** Delete or
rename `gate6_every_document_labels_a_valid_availability` and neither constant
changes — so the validation gate stays **green** while `availability-label`
names **no check this file runs**, and every manifest record still asserting it
is "verified." That is precisely DOC-W0's AC-1 property (*"how its currency is
checked"* must be mechanical) that this gate exists to enforce.

Verified **structurally**, not by deleting and compiling: neither constant
references any test function, so removing one cannot change either side of the
comparison.

## In fairness

Rust has no cheap runtime introspection of `#[test]` items, so a true 1:1
binding is not a one-liner. This was not carelessness. Both honest options are
available:

- **Weaken the comment** to what the test actually establishes — a *closed
  vocabulary*, which it does prove; or
- **Bind each token to its gate** — e.g. a per-gate `const TOKEN: &str` that
  the gate body references and from which the vocabulary is assembled, so
  deleting a gate **breaks the build** rather than silently orphaning a token.

**Not prescribing which** — the doc ring's design call, including "neither."

## Acceptance criteria

1. **The comment and the mechanism agree.** Either the claim is narrowed to
   what is checked, or the binding is made real. A green gate must not assert
   more than it checked.
2. If the binding route is taken: **deleting or renaming any bound gate must
   fail the build or a test** — demonstrate it by actually doing so, then
   revert. (A structural argument is acceptable evidence for the *current*
   defect, per the adversary's own method; it is not sufficient evidence for
   the *fix*.)
3. The `:563` delimiter gate's status is resolved explicitly — tokenized, or
   documented as deliberately untokenized with the reason.
4. Green in **CI**, never a local `--workspace` run (`COORDINATION §12`).

## ★ The shared generator — the reason both issues exist

Both findings are the same trade: **a hand-maintained enumeration standing in
for the thing it enumerates.**

| enumeration | true statement it makes | thing it stands in for | bound? |
|---|---|---|---|
| `library/REVISION` | names a real ancestor commit | the corpus was validated against it | **no** |
| `KNOWN_VALIDATION_TOKENS` | is a closed vocabulary | the gate inventory this file runs | **no** |

**Neither is wrong. Both are unbound** — the object each describes can move
without it, and no test can see that happen, because **both tests check a
description against another description.**

> **The generator, in the adversary's words:** *the suite has no check anywhere
> that closes a loop back onto the file's own contents. Nine rounds hardened
> the descriptions and never asked what pins them.*

Related: [[loud-fail-guard-only-closes-the-enumerated-namespace]] — a
panic-on-unknown guard closes only the namespace you already iterate.
