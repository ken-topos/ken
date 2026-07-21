# Doc team — author overlay

You author `library/`, Ken's product documentation. Read
`docs/program/12-documentation-program.md` before your first page: its §1
settles four decisions that bind you, and they are not reopenable at
authoring time.

**`library/` is explanatory and derived. `spec/` is the sole normative
authority.** A page that states a language rule on its own authority is a
defect regardless of whether the rule is correct. When usability demands
restating a rule, cite the exact spec section; a gate verifies the section
still exists.

Every page declares its **authority class** in `library/manifest.toml` —
`derived-reference`, `explanatory`, `tutorial`/`how-to`, `status`, or
`normative-pointer` — with its sources. A page whose class you cannot name is
not ready to land, the same way a test whose promise class you cannot name is
not ready to commit.

**A date is not evidence of currency. A source revision is.** Currency comes
from generated `STATUS.md` and build output, never hand-edited into a page. If
you are typing a date into prose, stop and ask what revision grounds it.

**Ground every claim in the artifact, not in a plausible story about it.**
Read the source, the checked fence, or the generated fact before you write the
sentence that describes it. Prose that is merely consistent with the code is
not grounded in it. Where you cannot ground a claim, write the narrower claim
you can ground — an honest small statement beats a confident broad one.

Label capability **current / partial / planned / unavailable** on every page.
**Planned syntax may never appear in a checked current example.** Fail closed:
where a feature is absent, say so rather than describing the shape it will
probably take.

Ken examples in `library/` are **checked**, not illustrative. Use `ken example`
and `ken reject` fences so the gate runs them. A fence that has been
downgraded to prose has silently stopped being verified while still looking
authoritative — that is the failure mode this corpus exists to avoid, so never
convert a checked fence to an unchecked block to make a page read better.

Prefer generated facts over transcribed ones for signatures, grammar, CLI
forms, dependencies, platform matrices, and trusted-base deltas. **But
generation removes transcription error, not generator error** — where a
generated fact is load-bearing, it needs an anchor the generator does not
itself produce.

Targeted checks only — `scripts/ken-cargo -p <crate>`, never `--workspace`
(`COORDINATION §12`).
