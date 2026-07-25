---
name: write-ken
description: Select the smallest checked Ken product-context pack before writing, proving, reviewing, or diagnosing Ken source.
scope: tools
model: claude-sonnet-5
---

# Write Ken

You are about to write, prove, review, or diagnose Ken source. Before changing
a line, load the product-context selection protocol at
`library/agents/README.md`, then select exactly one pack from
`library/agents/packs/` whose triggers match the task and whose exclusions do
not.

Use this routing:

| Task | Pack |
|---|---|
| explain or review existing source | `read-review` |
| write a pure program and law | `write-pure` |
| write an effectful boundary | `write-effectful` |
| find, use, or author a catalog package | `author-package` |
| repair a proof without adding trust | `repair-proof` |
| diagnose a parse-to-runtime failure | `diagnose` |

Load pack dependencies first and module includes in their listed order. Follow
a module prerequisite only when the task reaches it. Do not load the entire
agent library by default.

If no pack matches, or a pack excludes the requested work, stop and report the
unsupported boundary. In particular, this wave has no FFI/platform pack. Do
not invent a package, primitive, effect, capability, proof, command, or syntax
to bridge a missing module.

The selected modules contain Ken product facts. This skill retains the
repository workflow trigger: after the product task is complete, follow the
role and team playbooks already active in the session for checks, review, and
handoff. A mid-session edit to this skill does not update a running seat's
registered skill body; read this file directly when verifying the in-fleet
path.

Route Findings instead of leaving them in task notes: gaps in authoring
guidance go to the relevant `catalog/guide/` strand; recurring sugar candidates
and confusing manual elaboration go to Ergo; reusable `def`/`theorem`/`prop`
artifacts go into the catalog; kernel-reduction defects go to Kernel through
the enclave.
