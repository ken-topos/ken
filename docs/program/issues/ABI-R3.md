---
id: ABI-R3
title: "generated operation inventory derived from catalog structure — a new operation must be a build break"
status: draft
owner: runtime
size: M
gate: none
depends_on: [PX8]
blocks: [ABI-REVOKE, ABI-M1]
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## ✅ FRAMED 2026-07-27 — shovel-ready
>
> **Frame:** `docs/program/wp/ABI-R3-derived-operation-inventory.md`, inputs
> pinned by blob at `origin/main = 012aa56d`. Owner **Runtime**, size **M**.
>
> ### ⭐⭐ The gap, measured — it is narrower and sharper than this node says
>
> The catalog's closure is **real**: `HOST_EFFECT_ABI_V1_CATALOG` is *generated*
> by `crates/ken-host/build.rs:288` from the data file `effect_abi_v1.catalog`,
> and two landed tests cross-check the generated catalog, the Rust registry, the
> canonical observer, and the consumer set. Adding an operation to one side and
> not the other **does** fail.
>
> ⭐ **But all of those sets are derived from `HostOpV1::ALL`, and `ALL` is a
> hand-written `[Self; 25]` that nothing ties to the `HostOpV1` enum.** Add a
> variant to the enum and stop: it compiles, `ALL` still type-checks at 25, every
> downstream set agrees with every other — because they are all downstream of
> `ALL`. **Nothing fails.** ⇒ The enum is the unwritten surface.
>
> Three silent defaults follow: `availability()` (`effect_v1.rs:73`) returns
> `RepresentedUnavailable`, `is_ambient()` (`:97`) returns `false`, and `ALL`
> omits it — all via `matches!`/`else`, none a compile error.
>
> ⭐ **The correct mechanism is already in the same file**:
> `FsOpenModeV1::required_right` (`:574`) is an exhaustive `match`, so a new
> variant there **is** a build break. Same file, two enums, opposite discipline.
>
> ### ⛔ And the one guard on completeness is a tautology
>
> `effect_v1.rs:2916` asserts `HostOpV1::ALL.len() == 25`. `ALL` is declared
> `[Self; 25]`, so `.len()` is a compile-time constant — this asserts `25 == 25`
> and has never been able to fail. It is also exactly the anti-pattern §4 of the
> program names (*"named memberships and properties, never total counts"*), and a
> count is defeated by a compensating duplicate.
>
> ⇒ **`AC-1` is the whole WP:** add a throwaway 26th variant, change nothing
> else, and the build must fail — with the control first run against the
> *current* tree to show it passes today.
>
> ### ⚠ On the `PX8 -> ABI-R3` edge
>
> §7 of the program argues every sequencing edge it asserts **except this one**,
> and this WP adds no operations and needs none of PX8's behavior. ⚠ The edge
> stands until the Architect or operator moves it — it costs nothing today,
> because `effect_v1.rs` is contended by the in-flight [[PX8-ERRID-ALLOC]] and
> Runtime has two WPs queued ahead regardless. Raise it only if this becomes the
> thing Runtime is waiting on.
>
> ⭐ **Status stays `draft` because `depends_on` is unmet**, not because it is
> unframed.

## Objective

Generated inventory of **operation identity, availability, rights,
request/reply schema, and differential fixture per operation**, derived from the
catalogs own structure so that **adding an operation is a build break**.

## ★ This is the load-bearing node of Track R

It is the **same mechanism `SEAL-2` built for carrier producers**, applied to the
operation catalog: an enumeration **derived from structure** rather than restated
by hand. Every later track adds operations, and each one is a chance for a
hand-maintained list to drift.

⛔ **Tests assert NAMED memberships and properties, never total counts.** A count
is only as good as its window; a named membership survives the window changing.
