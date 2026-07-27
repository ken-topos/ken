---
id: SURF-SPACE-CELLS
title: "The `space` block surface — cells and `becomes` — is unbuilt, while its entire desugaring target (the `State` effect: Get/Put/run_state) is built and live"
status: ready
owner: language
size: M–L
gate: none
depends_on: []
blocks: [EFF-SPACE-ENSURES-PRESTATE]
github: null
origin: Steward measurement 2026-07-27 at `origin/main = aea07d62`, taken while scoping the residual left by EFF-SPACE-ENSURES-PRESTATE (closed Shape B, PR #1115). Filed per COORDINATION §2.
---

> ## ⭐ RELEASED 2026-07-27 to **Team Language** as
> **[`SURF-SPACE-CELLS-P1`](../wp/SURF-SPACE-CELLS-P1.md)**.
>
> ⭐ **This is the node that makes `old` buildable.** `EFF-SPACE-ENSURES-PRESTATE`
> closed as Shape B — `old` fails closed — *because* there was no cell
> environment and no `s_pre`/`s_post` to elaborate against. This node builds the
> cells. The pre-state binding is its **successor**, not its scope.

## 1. The measurement

At `origin/main = aea07d62`. ⛔ Re-derive at point of use.

| piece | spec | built? |
|---|---|---|
| `State S` signature — `Op = Get \| Put S`, `Resp Get = S`, `Resp (Put _) = Unit` | `36 §2.1`, `§4.1` | ✅ `effects/state.rs` — `StateOp s = Get \| Put s`, `resp_state` |
| `get` / `put` / `run_state` declarations | `36 §4.5`, `§4.2` | ✅ `effects/state.rs:571–585` |
| the direct monadic `[State s]` surface | `36 §4.5` | ✅ built |
| **the `space` block — `mut` cells** | `36 §4` | ❌ **absent** |
| **`becomes`** | `36 §4`, `§4.1` | ❌ **absent — zero non-comment occurrences in `crates/`** |
| **cell read / write desugaring** | `36 §4.1` | ❌ absent |
| **one effect label per space** | `36 §4.1` | ❌ absent |
| `becomes`-outside-a-space error | `36 §7.3` class 4 | ❌ absent |

The whole of what `space` does today (`crates/ken-elaborator/src/parser.rs:355`):

```rust
fn parse_space_view_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
    self.advance(); // consume 'space'
    match self.peek().clone() {
        Token::KwProc => self.parse_view_decl(start, true, DefKeyword::Proc),
        other => Err(ElabError::ParseError {
            msg: format!("expected 'proc' after 'space', found {:?}", other), ... }),
    }
}
```

`space` is a **modifier on a `proc` declaration**. The block form in `36 §4` —

```
space Counter {
  mut n : Int = 0
  proc inc () : Unit  visits [Counter] = n becomes n + 1
  proc get () : Int   visits [Counter] = n
}
```

— does not parse. There are no space-block tests and no corpus usage.

## 2. ⭐ Why this is a good node rather than a big one

**The hard half is already built.** `36 §4.1` says a space *desugars to* a
`State` effect, and every piece of that target — the signature, the response
family, `get`, `put`, and the `run_state` fold — is live in
`crates/ken-elaborator/src/effects/state.rs`. This node is the **surface and the
desugaring onto it**, not a new effect and not new kernel machinery.

⛔ **`becomes` is not kernel mutation.** `§4.1`, verbatim: *"So `becomes` is
**not** a kernel mutation — it is a `Get`-then-`Put` on the pure tree."* This
node has **zero kernel and zero trusted-base delta**. If a candidate introduces
a mutable cell into the TCB, the premise has failed.

## 3. The desugaring is given, not designed

`36 §4.1` specifies it verbatim. ⛔ **Do not invent it.**

- state type `S = T₁ × … × T_m` — right-nested Σ / record (`13 §3`), with η so
  cell update reconstructs definitionally;
- one **effect label** per space; every operation `visits [<space>]` uses
  `State S`;
- cell access:

```
cᵢ            (read)   ⤳  bind (perform Get) (λ s. Ret (s.i))
cᵢ becomes e  (write)  ⤳  bind (perform Get) (λ s. perform (Put (s with .i := ⟦e⟧)))
```

where `s with .i := v` is the record/Σ update **reusing every other component**.

## 4. Scope

**IN:** the `space` block surface, `mut` cells, `becomes`, the `§4.1`
desugaring, the per-space effect label, and the `§7.3` class-4 error.

⛔ **OUT:**
- ⛔ **`old` / the pre-state binding.** `OldPreStateUnsupported` **stays**. It is
  the successor's subject and it has its own controls. A candidate that makes
  `old` work as a side effect has silently merged two WPs and left the second's
  controls unwritten.
- ⛔ **A second `State`.** Reuse `effects/state.rs`.
- ⛔ **Kernel / trusted-base changes.**
- ⛔ **`§4.4` concurrency & isolation** (`OQ-Space`) — a separate concern.
- ⛔ **Re-specifying `run_state`.** `§4.5.3` says it *is* `§4.2`'s fold at
  `F = 𝟘`, already built.
