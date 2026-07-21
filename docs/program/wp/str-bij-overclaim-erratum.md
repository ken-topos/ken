# STR-BIJ — the `String`/`List Char` "bijection" over-claim (adversary A1 + A2)

**Owner:** Spec enclave · **Size:** S · **Risk:** low (wording only; zero
semantic change) · **Gate:** none — honesty erratum on landed prose ·
**Deps:** none

**Origin:** adversary findings **A1** (spec normative over-claim) and **A2**
(catalog title over-claim), both CONFIRMED. The adversary itself supplied the
refutation that keeps this out of soundness — see §4.

## 1. Objective

The corpus repeatedly justifies `String`'s canonicity by calling the
`string_to_list_char` / `list_char_to_string` pair a **bijection**. It is not
one. It is a **retraction**, and that is all the landed axiom states. Correct
every site to the property that is actually landed and actually needed.

**Nothing about Ken's behavior changes.** No code, no law, no instance, no
test verdict. This WP moves words into agreement with the mechanism.

## 2. The defect, grounded

`list_char_to_string` **NFC-normalizes** — stated by the very spec file that
later calls the pair a bijection
(`spec/30-surface/37-strings-collections.md:90`):

| Conversion | Totality | Meaning |
|---|---|---|
| `List Char → String` | total | encode UTF-8, **then NFC-normalize** + intern |

A normalizing map is not injective, so the pair cannot be a bijection.
**Repro** — both elements are valid `Char` per `37 §2.4`
(`U+0301` COMBINING ACUTE and `U+00E9` are both Unicode scalar values):

```
cs₁ = [U+0065, U+0301]     ("e" + combining acute)
cs₂ = [U+00E9]             ("é" precomposed)

l2s cs₁  ≡  l2s cs₂        — NFC identifies them ⇒ l2s is NOT injective
s2l (l2s cs₁) ≡ cs₂ ≠ cs₁  — the List-Char-side round trip is NOT the identity
```

The landed certificate agrees, and says so in its own body
(`catalog/packages/Data/Text/StringBijection.ken.md:13-14`) — the one postulate
is the **String**-side direction only:

```ken
axiom string_to_list_char_retraction
    : (text : String) → Equal String (list_char_to_string (string_to_list_char text)) text
```

`s2l` is injective (derived at `:16` as `string_to_list_char_injective`);
`l2s` is surjective. Neither is a bijection, and the reverse-direction lemma
a reader would expect from the word "bijection" **does not exist on `main`**.

## 3. ★ The inventory is larger than reported — and it has a ROOT

The two reported sites are the representative case, not the property
(COORDINATION §7). Grounded at `origin/main @ 61a78620`, the real inventory is
**7 sites**, of which **two were never reported** and **one is the root every
other site cites**:

| # | Site | Text | Class |
|---|---|---|---|
| **R** | `docs/adr/0010-lawful-deceq-requires-canonical-carrier.md:61` | "the round-trip is the identity on scalar sequences" | **ROOT** — ambiguous; reads as the false direction |
| 1 | `spec/30-surface/37-strings-collections.md:188` | "`s2l`/`l2s` are a round-trip bijection on scalar sequences" | **A1** — normative |
| 2 | `catalog/packages/Data/Text/StringBijection.ken.md:1` | H1 "String/List-Char **bijection** certificate" | **A2** — contradicts its own §1 heading, "**Retraction** certificate" |
| 3 | `catalog/packages/Data/Collections/Derived.ken.md:1389` | "round trip is a bijection on scalar sequences, ADR 0010 §2" | propagation |
| 4 | `catalog/packages/Capability/Parsing/Numeric.ken.md:6` | "crossing the opaque `String`/`List Char` bijection" | propagation |
| 5 | `catalog/packages/Data/Text/Codec.ken.md:96` | "the unrelated `String`/`List Char` bijection boundary" | **NOT REPORTED** |
| 6 | `conformance/surface/collections/seed-collections.md:656` | "the `s2l`/`l2s` round-trip is a bijection on scalar sequences, ADR 0010 §2" | **NOT REPORTED — and it is in `conformance/`** |

**Two consequences the reported pair alone would have hidden:**

**(a) Site 6 puts the erratum in `conformance/`, which pulls a CV vote**
(§14 diff-scope). Had this landed as the reported two-site fix, the corpus
would have kept a normative bijection claim inside the conformance seed — the
exact artifact that is supposed to pin the contract.

**(b) Site R is why the corpus is uniformly wrong.** Sites 1, 3 and 6 each
cite **ADR 0010 §2** as their authority. The ADR does **not** use the word
"bijection" — it says the round trip "is the identity on scalar sequences,"
which names the **`List Char` side**, the direction that is false. Every
downstream author read that phrase, correctly inferred "both directions," and
wrote "bijection." **Fixing the six leaves without the root lets the claim
re-grow from the citation.** A corpus can be internally consistent and
collectively wrong when it inherits one imprecise root.

**⇒ The ADR sentence rides this branch.** ADR 0010's recorded deciders are
"Architect (soundness ruling), Steward (commissioned the record)", so the
amendment is in the Architect's lane. **The enclave does not draft the
replacement ADR sentence unilaterally** — propose it and let the Architect
rule the wording as part of the §14 vote (that is the vote it already casts;
this adds no edge and no extra pass).

## 4. ★ What is NOT wrong — do not "fix" the mechanism

**This is a wording defect, not a soundness hole, and the adversary's own
refutation is why.** Read this section before touching anything:

- The consumer of the claim is the soundness of **codepoint-wise `eq`** and
  the deliverability of `DecEq String` / `Ord String` by transport.
- That consumer needs exactly one property: **`s2l` is injective** — if
  `s2l a ≡ s2l b` then `a ≡ b`.
- Injectivity is **landed**, derived from the retraction axiom as
  `string_to_list_char_injective`.
- A bijection is **strictly stronger than the consumer needs.**

⇒ `String` **is** canonical w.r.t. `List Char`, `DecEq String`/`Ord String`
transport **is** soundly deliverable, and the non-canonical-carrier trap
(ADR 0010 §1, `deceq-on-noncanonical-carrier-inhabits-bottom`) is **NOT**
tripped. **Every conclusion in the corpus stands; only the stated reason for
it was too strong.** An over-claimed *justification* whose consumer only needs
the weaker landed property is a wording defect — check the consumed property
before escalating something like this to soundness.

## 5. ⛔ Negative inventory — three "bijection"s that must NOT be swept

The word is used in **unrelated and correct** senses elsewhere. A
grep-and-replace over the corpus will corrupt them. Leave these alone:

| Site | Sense | Verdict |
|---|---|---|
| `spec/30-surface/33-declarations.md:131` | catalog **path ⇔ import** identity map | **correct — leave** |
| `conformance/surface/modules/seed-modules.md:44,264,275,867` | same path⇔file bijection (`07-catalog-style-guide.md §13`) | **correct — leave** |
| `spec/10-kernel/18a-primitive-registry.md:325` | the **store** round trip, which F1 establishes in *both* directions byte-identically | **correct — leave** |

`catalog/guide/proof-techniques.ken.md:349` reproduces the certificate but
**never uses the word** — verified. It needs no edit; confirm rather than
assume.

## 6. Fixed inputs — settled, do not reopen

1. **Fix the words; change no mechanism.** No `.ken` fence body, no axiom, no
   lemma, no instance, no test may change. If a wording fix appears to require
   a semantic change, **stop and escalate** — that means this frame is wrong.
2. **The replacement claim is the landed one:** the pair is a **retraction**
   (`l2s ∘ s2l ≡ id` on `String`), from which `s2l` **injectivity** follows,
   and injectivity is what canonicity and the `DecEq`/`Ord` transport consume.
   Do not substitute a different strong word ("isomorphism", "equivalence") —
   they are wrong the same way.
3. **All 7 sites, one branch, one Decision** (§14(4)). The combined diff-scope
   touches `spec/` + `conformance/` + `catalog/` + `docs/adr/` and therefore
   correctly pulls **CV** alongside the Architect. Do **not** split the ADR or
   the conformance seed onto its own branch — that is precisely the split that
   has dropped pieces before.
4. **The ADR sentence is proposed, not imposed** (§3).

## 7. Acceptance criteria

1. **All 7 sites in §3 corrected**, each stating the retraction/injectivity
   property rather than a bijection.
2. **The A2 title/body contradiction is gone.** `StringBijection.ken.md:1` no
   longer asserts something its own §1 heading denies.
3. **The negative inventory (§5) is untouched** — verify explicitly, and
   report the count of remaining "bijection" occurrences with a one-line
   justification for each survivor. A silent post-fix grep hit reads as a miss.
4. **Zero mechanism delta.** `git diff origin/main...HEAD` touches no `.rs`
   file, and no `ken` fence body changes. State this in the PR body.
5. **The catalog still elaborates.** `StringBijection.ken.md` is consumed by
   three elaborator integration suites (§8); prose-only edits must not
   perturb them. Targeted: `scripts/ken-cargo test -p ken-elaborator --test
   cc2_text_codec_numeric_acceptance`. **Never a local `--workspace`**
   (COORDINATION §12) — workspace-green means **green in CI**.
6. **The ADR amendment is recorded as an amendment**, with its date and the
   reason, not silently rewritten. An accepted ADR that changes without a note
   is a worse honesty defect than the one being fixed.

## 8. Deferred on the record: the FILE is still named `StringBijection`

The filename — and therefore the module path `Data.Text.StringBijection` —
carries the same over-claim, and a reader meets the **name** before the prose.
Renaming it is **out of scope here**, for a grounded reason, not by oversight:

- `07-catalog-style-guide.md §13` makes path ⇔ import an **identity map**, so
  renaming the file **renames the module**.
- Three `ken-elaborator` integration suites hard-code **both** the path and the
  dotted name: `cc2_text_codec_numeric_acceptance.rs:21,44,141,242,260,280`,
  `cc7_argparse_acceptance.rs:26,53`, `cc8_env_config_decoder_acceptance.rs:26,55`.

That converts a docs-only erratum into a **crates-touching change with a
whole-harness consumer sweep and a CI gate** — a different WP with a different
risk profile. Holding a two-line honesty fix hostage to it would be the wrong
trade. **It is filed, not forgotten:** the Steward carries it as a follow-on,
to be sequenced with the `research/catalog-package-taxonomy-proposal.md` moves
that already contemplate relocating this file. A deferral is honest; a
deferral that reads as delivery is not — so §7(3)'s survivor report must
name the filename as a known, accepted survivor.

## 9. Guardrails — do not reopen

- **Do not weaken the conclusions.** Canonicity, `DecEq String`/`Ord String`
  deliverability, and codepoint-wise `eq` soundness all **stand** (§4). A fix
  that hedges them has over-corrected and is wrong in the other direction.
- **Do not grep-and-replace.** §5 lists three correct uses of the word.
- **Do not add a reverse-direction axiom.** The temptation is to make the word
  true by postulating `s2l (l2s cs) ≡ cs`. That statement is **false** (§2's
  repro) — postulating it would inhabit `Bottom`. This is the one way this WP
  could do real damage; it is the reason §4 exists.
- **Do not rename the file** (§8).
- **Every anchor here is perishable.** All line numbers were measured at
  `origin/main @ 61a78620`. Re-verify at pickup; if a fixed input is false
  against the landed code, **say so and escalate — do not quietly build
  around it.**
