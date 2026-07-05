---
scope: fleet
audience: (see scope README)
source: private memory `cleanroom-is-role-discipline-not-host`
---

# Clean-room protection is a role discipline, not a model-host property

Operator correction (2026-06-30): the clean-room policy "was not about the host
per se, but a coincidence — clean-room discipline coincides with access, but
there is no causal relation. It just so happened that the enclave (Opus-class)
roles had reference access. There's no intrinsic reason for it."

**The error I made:** I argued an enclave role (spec-author/conformance-
validator) is "riskier to move to a different host" because a third-party host
creates a copyleft-leakage surface, and that the Steward is "uniquely safe to
move off-host" because it reads no references. That treats the **host** as the
leakage variable. Wrong: the current host already sees everything the enclave
reads — any other host is no different on leakage grounds.

**The real safeguard is the DISCIPLINE, host-agnostic:** copyleft
named-not-read, never vendored, the leakage-recheck. It travels with the
reference-reading **role**, whatever model sits in the seat. Access is **role**-
based (enclave reads refs under discipline; implementers/QA/leaders never read
refs at all — `CLEAN-ROOM.md`), and the role↔tier↔host mapping is incidental.

**How to apply:** model/host reassignment is a **pure capability × stakes**
optimization — don't weight it by host on clean-room grounds. A spec-author on
any host follows named-not-read/no-vendor exactly as the Opus one does; the
leakage-recheck stays on whatever seat owns it, any host. (The fleet is
currently all-Anthropic — Opus 4.8 enclave / Sonnet 5 build — so this is a
dormant principle, not a live plan; it governs *if* a seat ever moves hosts.)
See reviewers review branches not prs.
