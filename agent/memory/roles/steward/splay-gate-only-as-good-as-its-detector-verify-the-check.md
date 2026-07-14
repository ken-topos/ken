---
scope: roles/steward
audience: (see scope README)
source: live 2026-07-13 (kenfmt batch-2a AC5 false-green; operator caught the
  over-width splay in the merged catalog)
---

# A style/quality spot-check is only as good as its detector — verify the check
# fires, and target the actual failure mode

When you gate a tool's catalog-wide output (formatter, codegen, canonicalizer)
with a **spot-check** before it freezes into a strict gate, a *clean* result is
worthless unless you have **proven the detector actually fires on a known-bad
example**, and unless you aimed it at the **specific failure mode**, not the
easy cases. A green from a broken check is worse than no check — it launders a
real regression as "verified."

**Live (2026-07-13, kenfmt batch-2a).** I ran an "AC5" splay spot-check on the
catalog re-sweep, reported it GREEN, and published; the operator then found
pervasive **over-width splay** still in the merged catalog (`fusion_law` in
LawfulFunctors rendering `(g : b`⏎`→ c)`, `(f`⏎`c)`, `(map`⏎`a`⏎`c`⏎…), and a
correct re-scan showed **~1,200 splayed regions** across Map/EffectfulClasses/
LawfulClasses/… The re-armed strict gate had **frozen the bad output as
canonical.** Two compounding failures:

1. **The detector didn't fire.** My `awk` scan used `\s` in the regex
   (`^\s*\(?[a-z_]…`). **`\s` is a GNU/PCRE extension, NOT portable awk** —
   under `mawk` it does not mean whitespace, so the pattern matched almost
   nothing on the indented catalog lines → a false **"0 splay runs."** Use
   POSIX classes: **`[[:space:]]`**, never `\s`/`\d`, in `awk`. (Sibling of
   [[awk-length-miscounts-multibyte-wrap-check]] — awk regex/portability traps
   bite spot-checks.)
2. **I aimed at the easy case.** The manual eyeball hit files whose proofs
   **fit** (Map's fitting applications, Validation's short `validation_ap_id e
   d`) — never the **over-width** deep signatures where the defect lived. The
   tool had fixed the *fits-on-one-line* case (real) but the *doesn't-fit* case
   still fill-then-broke-low. Spot-checking the collapse it fixed proved
   nothing about the split it didn't.

**How to apply.** (1) **Prove the check on a known bad.** Before trusting a
"clean" scan, run it against an input you *know* exhibits the defect (an old
pre-fix sample) and confirm it flags it. A detector that reports zero on a
dirty file is broken, not passing. (2) **Target the failure mode.** For a
formatter style defect, inspect the **over-width / doesn't-fit** constructs
(deep signatures, arrow chains, nested applications that exceed the budget) —
that's where break-logic defects live; the fitting cases are the ones the tool
already handles. (3) **Prefer an automated CI gate to a one-shot manual scan**
for any defect class that can regress: a `no-splay` / balance gate in the
strict set is enforced every build; a hand scan is one portability bug away
from silent false-green. (4) When your gate *did* pass but the operator finds
the defect, **own it plainly and re-scan correctly with a verified detector**
before scoping the fix — don't re-approve on the same broken check. Dual of
[[representative-file-review-only-covers-constructs-that-file-has]] and the
[[green-vs-green-does-not-confirm-a-fix]] / verify-the-mechanism cluster: the
representative-review lesson says *cover the missing constructs*; this one adds
*and make sure the instrument you cover them with actually works.*
