---
id: CAT-C2
title: "Localized Map/Set key-interface split: a non-canonical carrier becomes a lawful Map/Set key under a weaker key-order dictionary while staying an unlawful Ord key wherever antisym concludes kernel Equal"
status: draft
owner: spec-enclave
size: M
gate: none
depends_on: [SPEC-IDENT-BLESSED]
blocks: []
github: null
origin: "Architect framing instruction evt_7jppg10gk983 (14-spec-mission-alignment-campaign.md §6.2). Track-A conformance census run by the Steward 2026-07-27 returned a hard stop; ruled by Architect Decision dec_72c7f9wr8tr3m (resolved). Frame written at origin/main 7d557560."
---

Frame:
[`../wp/CAT-C2-map-key-interface-split.md`](../wp/CAT-C2-map-key-interface-split.md)
— shovel-ready, inputs pinned by blob at `origin/main = 7d557560`.

⛔ **`status: draft` is deliberate.** The frame is complete and the design is
ruled; what is missing is a **release slot**. Phase 1 goes to the spec enclave,
which is currently building `SPEC-IDENT-BLESSED`. Flip to `ready` when that
merges. ⚠ `depends_on: SPEC-IDENT-BLESSED` records a **scheduling** dependency,
not a technical one — the two WPs share no path.

## The ruling, verbatim — `dec_72c7f9wr8tr3m`, `resolved`

> *"The `C2` Map/Set boundary intentionally flips noncanonical carriers from
> rejected-as-Map-key to accepted-under-the-weaker-key-order interface, while
> the same carrier remains rejected as lawful `Ord` wherever `antisym`
> concludes kernel `Equal`. Row 1 leaves the Map lane and remains only
> `Ord`/ADR-0010 coverage; row 2 splits into `Ord`-reject plus Map/Set-accept;
> row 3 is replaced by an `Axiom`-free, noncanonical, weak-dictionary-only
> discriminator whose `Ord`/`antisym`/`Equal` mutation fails. `KeyEq` is
> derived from mutual `leq`, last representative/value win, `to_list` exposes
> that representative, and structural `Map` equality remains
> representation-sensitive."*

⭐ **The scoping is the ruling.** The `(soundness)` verdict does **not** invert
— it **splits**. The ADR-0010 non-canonical-carrier trap is preserved exactly
where it was load-bearing (`Ord`, where `antisym` concludes kernel `Equal`) and
lifted only where the weaker dictionary makes it inapplicable (Map/Set keying,
where nothing concludes kernel equality).

⛔ **The single most likely defect in this WP is the split silently becoming an
inversion** — dropping the `Ord`-reject half retires the trap everywhere
instead of at one boundary. `AC-1` exists to catch exactly that.

## Why it was a hard stop, and why now is cheap

The spec side was already localized: lookup laws 1–4 are antisym-free, and
`antisym` is load-bearing **only** in the `insert`/`from_list` ⟹ `Distinct`
discharge. But three **live** conformance rows in
`conformance/stdlib/map/seed-map.md` are keyed on that single site, one of them
marked **`(soundness)`**.

⭐ Rows 1 and 3 are marked **"Deferred (buildability)"** — the overwrite proof
(`52 §7d`) is Branch-B and **not yet built**. ⇒ This WP changes **assertions**.
After the overwrite proof lands it becomes a **proof retraction**. That is the
argument for doing it before the Branch-B build, not after.

## Shape

Two phases. **Phase 1 (enclave)** is `spec/` + `conformance/` only — the
key-order dictionary, the derived `KeyEq`, the re-proved `Distinct` discharge,
and the three row dispositions. **Phase 2 (Ergo)** is `catalog/` only — keying
`Map.ken.md` / `Derived.ken.md` on the new dictionary and demonstrating a
working non-canonical-carrier `Map`.

⛔ `Ord.antisym` is **not** weakened; `Ord` adapts to the weaker interface
**forgetfully**, one-way. ⛔ No parallel `CanonicalOrd`. ⛔ `KeyEq` is derived
from `refl`+`trans` or it is wrong — no second equality field, no postulated
compatibility theorem.

⚠ `AC-4` is the load-bearing criterion: row 3's old discriminator (*which
order-faculty each law uses*) **collapses** under `C2`, since both sides become
`refl`/`trans`/`total`. A replacement that restates it is **vacuous and green**,
so the ruling requires the replacement's `Ord`/`antisym`/`Equal` mutation to
**fail** — a real reddening obligation on a different axis.

⛔ `C2` blocks no build lane and the operator has not prioritized it. Queued
work, not urgent work.
