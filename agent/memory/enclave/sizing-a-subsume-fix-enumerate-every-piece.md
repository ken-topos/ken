---
scope: enclave
audience: (see scope README)
source: private memory `sizing-a-subsume-fix-enumerate-every-piece`
---

# Sizing a subsume-fix: enumerate every piece the generalization needs

**D4 effect-composition fork on `wp/fs-read-file-lines-flip` build (2026-07-04,
my design/soundness lane).** Steward asked me to size Option 1 ("add a general
`Sum ConsoleOp (FSOp a)` dispatcher to `run_io`") as bolt-on-vs-own-WP. The
trap: `Sum`/`InL`/`InR`/`resp_sum`/`run_state` ALL already exist on
`origin/main` (State-effect-build landed them), so it LOOKS like "reuse the Sum
machinery, three lines." Wrong.

**The move that worked — enumerate every distinct piece the general fix needs,
grep each:**
1. **Response family** — `resp_sum : (s f)(RespF) -> Sum (StateOp s) f -> Type`
   is hardcoded to a `StateOp` FIRST summand. A `Sum ConsoleOp (FSOp a)` tree
   needs a NEW `resp_console_fs`; the existing one gives zero reuse.
2. **Injection/lift** — `print_line`/`read_bytes` produce UN-wrapped
   `ITree ConsoleOp …`/`ITree (FSOp a) …`. There is no `inject`/`lift` morphism
   (`ITree E → ITree (Sum E F)`); the State path HAND-BAKES `InL` into
   `get`/`put`'s bodies. Composing needs new injection machinery.
3. **Top-level dispatcher** — `run_io` matches raw `Write`/`ReadFile` op-tags,
   zero `Sum` awareness; `run_state` (the ONLY Sum-fold) interprets State and
   passes the other summand through UN-RUN. So **no path executes two base
   effects at the top level at all.**

All three missing/hardcoded ⇒ Option 1 is a genuine new capability
(effect-composition WP), NOT a bolt-on. Ruled: close the WP now via the
composition-free option, defer real composition to its own scoped WP.

**The reusable rule.** Sizing a "just generalize / subsume it" fix: DON'T let
the presence of the sibling case's machinery (same type constructors, a related
fold) stand in for reuse. List the distinct pieces the general capability
requires — data/response family, the intro/injection morphism, AND the
eliminator/driver — and grep the LANDED producer for each. Machinery built for
one member of a family is routinely hardcoded to that member (fixed summand,
member-specific handler) and generalizes to nothing. This is the sizing-a-FIX
dual of buildability ruling must ground every axis (there: ground every axis
before ruling a build blocked/buildable; here: ground every piece before ruling
a fix bolt-on/own-WP). Sibling of class dict explicit vs implicit abstract tyvar
(probe, don't accept the blocker) — same discipline, opposite polarity (there
under-claiming a blocker, here under-claiming the cost).
