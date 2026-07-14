---
scope: enclave
audience: (see scope README)
source: live 2026-07-13 (operator redirect of the Runtime I-4 ProgramCaps
  launch-grant fork)
---

# Ken owns "what is a valid program," not runtime constraint — capabilities are DECLARED IN the program, not granted at the CLI

When a capability/authority design reaches for a **launch-time grant surface**
(CLI `--allow-fs-write` flags, a default-deny/default-grant posture, an external
grant the runtime compares against), stop: that is the **wrong axis** for Ken at
present. The operator drew a hard line on Runtime I-4:

> *"There are two separate concerns: (1) what the Ken language accepts as a valid
> program, and (2) mechanisms to constrain running programs (SELinux, etc.). Ken
> is concerned only with (1) at present. The capabilities granted to the program
> should be explicitly stated in the program declaration — the same declaration
> that carries the `admits` class/instance roster."*

**The two concerns, and which is Ken's:**
- **(1) Program validity (Ken's, now).** Static, type-level: does this program
  type-check, and does it stay within the authority it **declares**? The
  capability requirement is **stated explicitly in the program declaration**
  (alongside the `admits` class/instance roster), and the type system (e.g. an
  authority-indexed `Cap a` threaded through the ops) enforces the program cannot
  invoke authority it did not declare. This is the whole job.
- **(2) Runtime constraint (NOT Ken's, now).** SELinux, OS sandboxing, a CLI
  grant/deny surface that confines a *running* program below what it declared —
  external mechanisms Ken does not own at present. Do not design a CLI
  launch-grant flag vocabulary or a default-grant/default-deny posture into a Ken
  capability WP; that is concern (2).

**How to apply.** (a) A capability WP's authority **source is the program's own
declaration**, not an external grant — the runner mints exactly what the
declaration states; there is no launch-grant to be "insufficient" against, so a
grant-mismatch failure model is the wrong frame. Recast the failure as **static**
(the type check that the program stays within declared authority) with an op-time
`CapabilityDenied` backstop as defense-in-depth. (b) The declared capability
**composes with the `admits` roster** in one declaration form — design that
integration, not a separate CLI surface. (c) If you catch yourself surfacing a
"default posture / flag vocabulary" fork to the operator, you are on concern (2)
— it is out of scope; don't ask. (d) This is a v1 line ("at present"): external
runtime confinement may enter Ken's scope later, but do not pre-build its surface.

Sibling of the authority-manifest idea (a function's signature is its authority
manifest, spec §62) — but sharper: the manifest is an **explicit declaration**
tied to `admits`, and Ken's enforcement is **program-validity only**. Related:
[[surface-the-seam-need-not-your-preferred-mechanism]] (surface the need, not a
mechanism the operator hasn't asked for).
