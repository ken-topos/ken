---
scope: teams/kernel
audience: kernel-implementer, kernel-qa, kernel-leader (any WP adding a Term variant)
source: former private memory `zonk_term-must-be-exhaustive-over-term-variants`
  (wp/surface-transport, Map Gap A, merged 19955d8 PR #257); relabeled from
  type:project — it is a durable kernel lesson, not campaign state
related: soundness-walker enumeration (K5 playbook fold), gate-widening exposes
  latent bugs
---

# A new `Term` variant needs every shared exhaustive walker extended

When a WP makes the elaborator **construct** a `Term` variant it never built via
`infer`/`check`/`elab_type` before (a new surface former — `J`, `Eq`, `Cast`,
…), the completeness gap lives in the **shared whole-term traversals**, not at
your construction site. "I zonked my own output" does not help if the shared
traversal itself can't recurse into what you built.

**The trap (concrete):** `MetaCtx::zonk_term` had a catch-all arm
(`other => other.clone()`) covering only `Type`/`Omega`/`Var`/`Pi`/`Lam`/`App`/
`Let`/`Const`. `infer_j`/`infer_eq` zonked their own local copies before an
internal sanity check, but the **returned** term (embedded in the whole
declaration body) relied on the outer `elaborate_decl`'s final
`zonk_term(&body)` pass — which didn't recurse into `Term::J`/`Term::Eq`/etc. So
a `Level::Var` nested inside a new node survived to the load-bearing
`kernel_check` unresolved, surfacing as `TypeMismatch{expected: Type u2, found:
Type 0}` — but only on the genuinely universe-polymorphic case (`cast`'s
`Eq Type A B`), where an unresolved level actually diverges from the harmless
Zero-default.

**How to apply:**

1. On any WP adding a `Term` variant, grep **every** shared whole-term traversal
   with a permissive catch-all — at minimum `zonk_term`, and the soundness-
   relevant walkers `sct.rs::collect_calls` (termination) and
   `foreign.rs::collect_consts_in_tb` (trust accounting) — for the new variant,
   **before** trusting "I handled my own output."
2. The fix is to make the traversal **exhaustive** — remove the catch-all and let
   the compiler force every `Term` variant to be matched — not to patch in just
   the variants you happen to need. Compiler-enforced exhaustiveness catches the
   **next** such gap for free.

Same underlying shape as gate-widening exposing latent bugs in newly-reachable
code: a pre-existing gap in shared code, invisible until something new makes it
reachable.
