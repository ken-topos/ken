# teams/foundation — Foundation-team lessons

Loaded by the Foundation ring — `foundation-leader`, `foundation-implementer`,
`foundation-qa` — in addition to `fleet`, `build/`, and the function scope
(`build/leaders` · `build/implementers` · `build/qa`).

For lessons specific to the foundation layer: the standard-library catalog
packages, lawful classes, and the shipped `.ken` corpus that downstream proofs
ride on.

**The recurring hazard on this team is corpus-wide reach.** A change to a
catalog package is validated by oracles living in crates the WP never touches, so
a targeted per-crate run cannot see them and they surface as red CI at publish —
after review, after the merge Decision, at the most expensive moment. Enumerate
every test that globs the directory you are adding to.

| Lesson | One-line |
|---|---|
| _(none recorded yet)_ | |

**An empty scope is a normal state, not a defect.** Record a lesson at the
broadest scope where every reader must apply it; a genuinely cross-cutting one
gets a `scope:` frontmatter tag rather than a copy.
