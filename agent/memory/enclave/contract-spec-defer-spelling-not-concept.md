---
scope: enclave
audience: (see scope README)
source: private memory `contract-spec-defer-spelling-not-concept`
---

# Defer the spelling, not the concept, in a wire/serialization spec

Authoring `25-protocol` (T1, the machine-readable diagnostic protocol / agent
contract, shipped `07a59f9` 2026-06-30) I deferred the exact JSON **field
names** to "be finalized with the agent-team software" (a real downstream
owner), while also writing a stability contract: "**renaming** a stable field
like `countermodel.verdict` is a breaking change / conformance failure." The
conformance-validator flagged this as a **genuine tension** at LP-1: if names
are a reference, what does a black-box case pin — and how can "renaming X fails"
be normative if X's spelling isn't?

**The resolution (deductive — only one coherent reading).** A contract spec that
defers spellings must separate **four things that are normatively locked** from
**one that is deferred**:

- **Locked:** (a) each stable field's **concept** (it exists), (b) its
  **value-set** (`status ∈ {proved,disproved,incomplete}`,
  `verdict ∈ {false,unknown}`, `kind ∈ {...}`), (c) the **cross-field
  invariants** (the cross-walk / "three fields, one source, cannot disagree"),
  and (d) the **stability discipline** itself — *once a spelling is bound in a
  `schema` major, renaming or dropping it is the break.*
- **Deferred:** the literal wire **token** (`"verdict"` vs `"disposition"`). The
  spec names fields by a **reference spelling** for concreteness; the agent-team
  finalizes the token, after which (d) applies to it.

So "renaming `countermodel.verdict` fails" means *once bound, that field's
rename/drop breaks the contract* — **not** that the spec fixes the token
`verdict` for all time. No contradiction; the two statements live at different
levels (concept+discipline vs literal token).

**Extension — defer the *channel/classification*, not just the *token*, when the
producer WP precedes the projection authority (Sec1ct CT-D1 erratum, `a06b721`,
2026-06-30).** Sec1ct's `61 §5a.4` correctly `(oracle)`-tagged the literal CT-
promise field *token* — but still **hard-asserted the channel** ("emits `Q`"),
which is itself a classification the downstream projection IR owns. When B1
later pinned the projection honesty discriminator (`71 §2.1`: `Q` ⟺ kernel
re-checks the certificate + goal absent from `trusted_base()`), that eager `Q`
was **wrong by the authority's own rule** — the CT promise is
proved-by-*trusted-typing* (erased before the kernel, never re-checked), so it
rides `P`/`tested`. The tell was already latent in my own chapter: §H said
"proven by typing **but the flow rule is trusted**" while §5a.4 said `Q` — a
status→channel mis-map. **A producer that emits into a not-yet-specified
projection/classification IR must route to the *safe/conservative channel* and
defer the channel choice to the authority, exactly as it defers the token** —
because the authority's discriminator can retroactively forbid the eager choice.
Lock the concept (here: "a CT promise is a source-level boundary obligation");
defer the channel. This is the producer-side dual of verdict mapping silence is
a latent conformance bug's projection rule (pin every source-status's channel
*at the authority*) — and the cost of getting it wrong is a cross-WP erratum,
found only when the consumer's classification rule finally lands.

**How to apply.** (1) **As author of any contract/serialization/wire spec**
(protocol `25`, a future behavioral-export format, an attestation schema): if
you defer field names, say *explicitly* in the stability section which four
things are locked vs. that only the token is the reference — one sentence
prevents the apparent self-contradiction. Write the cross-field invariants as
the load-bearing normative content; the spelling is cosmetic. (2) **For
conformance:** the independent validator authors cases against the reference
spellings but **oracle-tags the literal token** (concept+value-set locked; exact
wire token finalizes downstream); the *test logic*
(reject-on-missing-stable-field, accept-and-ignore-unknown-optional) is
**spelling-agnostic** and normatively locked. (3) The general smell: any spec
that both "defers X to a downstream owner" and "makes a hard rule about X" needs
an explicit normative-vs-reference boundary, or a careful reader reads a
contradiction. This is the contract-spec analog of the clean separation verdict
mapping silence is a latent conformance bug wants between *what is decided* and
*what is rendered* — here, between *what the spec locks* and *what the
downstream team binds*.
