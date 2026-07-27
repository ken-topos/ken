# CAT-C2 — the localized Map/Set key-interface split

**A non-canonical carrier becomes a lawful `Map`/`Set` key under a weaker
key-order dictionary, while staying an unlawful `Ord` key wherever `antisym`
concludes kernel `Equal`. The flip is scoped to a boundary, not global.**

**Owner:** Phase 1 → spec enclave. Phase 2 → Team Ergo.
**Branch:** `wp/CAT-C2-P1` then `wp/CAT-C2-P2`. **Size:** Phase 1 M, Phase 2 M.
**Risk:** medium-high — a **coupled `spec/` + `conformance/` relaxation** that
changes a row marked `(soundness)`.

**Status:** Steward frame, shovel-ready. ⛔ **Do not release Phase 1 until the
enclave finishes `SPEC-IDENT-BLESSED`.** Phase 2 is gated on Phase 1 merging.

---

## 1. Fixed inputs — measured at `origin/main = 7d557560`

| path | blob |
|---|---|
| `spec/50-stdlib/52-map.md` | `24fb736d3c7f2716a6e36dd8efafda6537947431` |
| `spec/50-stdlib/54-map-verified-laws.md` | `f5e840019e27f2042edcda16ef6c9c7775b016ee` |
| `spec/50-stdlib/58-maps-sets-relations.md` | `ff99acc4fd8cb77556f505f9d7c281a2e87488e9` |
| `spec/50-stdlib/51-lawful-classes.md` | `0ad2ce4b0a496bdd54e421ebf6f3d80c1fd8baf6` |
| `conformance/stdlib/map/seed-map.md` | `9410304c31040f1cd4bf1359313b4fce1d830e05` |

Library sites carrying `antisym` (Phase 2 surface):
`catalog/packages/Data/Collections/Map.ken.md`,
`Data/Collections/Derived.ken.md`, `Data/Text/StringKeys.ken.md`,
`Data/Numeric/Nat/Order.ken.md`, `Core/Classes/LawfulClasses.ken.md`.

---

## 2. Authority — two resolved Architect Decisions

**Framing instruction** (`evt_7jppg10gk983`, transcribed in
`14-spec-mission-alignment-campaign.md §6.2`): *"frame `C2` as a localized
Map/Set key-interface split."*

**The Track-A hard stop and its ruling — `dec_72c7f9wr8tr3m`, `resolved`.**
Verbatim, because the channel is not a durable store:

> *"The `C2` Map/Set boundary intentionally flips noncanonical carriers from
> rejected-as-Map-key to accepted-under-the-weaker-key-order interface, while
> the same carrier remains rejected as lawful `Ord` wherever `antisym`
> concludes kernel `Equal`. Row 1 leaves the Map lane and remains only
> `Ord`/ADR-0010 coverage; row 2 splits into `Ord`-reject plus
> Map/Set-accept; row 3 is replaced by an `Axiom`-free, noncanonical,
> weak-dictionary-only discriminator whose `Ord`/`antisym`/`Equal` mutation
> fails. `KeyEq` is derived from mutual `leq`, last representative/value win,
> `to_list` exposes that representative, and structural `Map` equality remains
> representation-sensitive."*

⭐ **Read the scoping carefully — it is the whole ruling.** The `(soundness)`
verdict does **not** invert. It **splits**: the ADR-0010 trap is preserved
exactly where it was load-bearing (`Ord`, where `antisym` concludes kernel
`Equal`) and lifted only where the weaker dictionary makes it inapplicable
(Map/Set keying, where nothing concludes kernel equality).

---

## 3. The measurement that made this a hard stop

The spec side was already localized. Lookup laws 1–4 (`52 §5.2`, `54 §5.2`) are
**antisym-free** — `refl`/`trans`/`total` only. `antisym` is load-bearing
**only** in the separate `insert`/`from_list` ⟹ `Distinct` **discharge** lemma
(`52 §2.1`/`§5.3`, `54 §4`, `58 antisymLeq`). ⇒ `C2`'s "no `Equal` step" route
touches exactly that one discharge.

The conformance side is what stopped it — three live rows in
`conformance/stdlib/map/seed-map.md`, all keyed on that single site:

| row | line | current verdict | disposition per the ruling |
|---|---|---|---|
| `antisym-equal-sound-over-canonical-key` | 427 | accepts | **leaves the Map lane**; remains only `Ord`/ADR-0010 coverage |
| `noncanonical-key-not-a-lawful-map-key` **(soundness)** | 441 | rejects | **splits** → `Ord`-reject **plus** Map/Set-accept |
| `lookup-laws-need-no-equal-promotion` | 468 | pins localization | **replaced** — see `AC-4`, its discriminator collapses |

⭐ **Timing.** Rows 1 and 3 are marked **"Deferred (buildability)"** — the
overwrite proof (`52 §7d`) is Branch-B and **not yet built**. ⇒ This WP changes
**assertions**, not a landed proof. After the overwrite proof lands it becomes
a **proof retraction**. That is an argument for doing it now.

---

## 4. The interface — fixed by the framing instruction, not open

- **New key-order dictionary:** `leq`, `refl`, `trans`, `total`. ⛔ **No
  theorem from mutual order to kernel `Equal`.**
- **`KeyEq x y := IsTrue (leq x y) ∧ IsTrue (leq y x)`** — **derived**, not an
  independent class. Equivalence follows from `refl`+`trans`;
  order-compatibility **also** from `trans`. ⛔ No second equality field, ⛔ no
  postulated compatibility theorem.
- **Map/Set keying consumes the key-order interface.** Existing `Ord` adapts to
  it by **forgetting `antisym`**, and keeps serving the other census sites
  (comparison equality, compound `Ord` instances, sort-permutation proof).
  ⛔ **`Ord` itself is not weakened** — the adaptation is forgetful, one-way.
- **Route rules:** lookup + overwrite use `KeyEq`, never kernel `Equal`.
  `Distinct` = no two stored entries `KeyEq`-equivalent. `insert`/`from_list`
  discharge `Distinct` **directly** from the overwrite branch plus preorder
  compatibility, with **no `Equal k k'` step**. Overwrite/uniqueness concludes
  **one entry per `KeyEq` class**, ⛔ **not** equality of representatives. ⛔ No
  theorem converts `KeyEq` to `Equal` without explicit canonical `Ord`
  evidence.
- **Observable representative policy, pinned:** **last inserted representative
  and last inserted value win** (already true of `Map.ken.md:108–118`);
  `to_list` exposes **that** representative. Structural kernel equality of two
  `Map`s stays **representation-sensitive**. Any API-level extensional map
  equivalence must compare keys by `KeyEq` and ⛔ may **not** claim
  representatives are kernel-equal.

---

## 5. ⛔ Banned shapes

- ⛔ **Do not weaken `Ord.antisym`.**
- ⛔ **Do not add a parallel `CanonicalOrd` class.**
- ⛔ **Do not reopen** the antisym-free lookup laws (`52 §5.2`, `54 §5.2`),
  `C4`, or `C5`. `§6`'s do-not-reopen list stands.
- ⛔ **Do not add a second equality field or a postulated compatibility
  theorem** — `KeyEq` is derived or it is wrong.
- ⛔ **Do not let the row-2 split become a row-2 inversion.** Dropping the
  `Ord`-reject half is the single most likely defect in this WP: it silently
  retires the ADR-0010 trap everywhere instead of at one boundary.
- ⛔ **Phase 1 does not touch `catalog/`. Phase 2 does not touch `spec/` or
  `conformance/`.**

---

## 6. Deliverables

### Phase 1 — spec enclave (`wp/CAT-C2-P1`)

- **`D1`** — the key-order dictionary and derived `KeyEq` specified in `58`,
  with the `Distinct` discharge in `54 §4` re-proved through the overwrite
  branch with **no `Equal` step**.
- **`D2`** — `52 §2.1` re-scoped: the canonical-carrier obligation belongs to
  `Ord`, not to `Map`. The `antisym → Equal` localization note becomes a
  statement about `Ord` keys.
- **`D3`** — the three conformance rows dispositioned exactly as `§3`'s table
  requires, including **row 2's split into two rows with opposite verdicts**.
- **`D4`** — row 3's replacement: an **`Axiom`-free, non-canonical,
  weak-dictionary-only discriminator**.
- **`D5`** — a **closed** report of every `spec/` and `conformance/` site that
  consumed the `antisym → Equal` step, each with its disposition, complement
  named.

### Phase 2 — Team Ergo (`wp/CAT-C2-P2`), gated on Phase 1 merging

- **`D6`** — `Map.ken.md` / `Derived.ken.md` keyed on the new dictionary;
  `StringKeys`, `Nat/Order`, `LawfulClasses` adapted forgetfully.
- **`D7`** — a **non-canonical-carrier `Map` that works**: the executable
  demonstration that two distinct representatives denoting one value collapse
  to one entry.

---

## 7. Acceptance criteria

- **`AC-1`** — ⭐ **the split is real on both halves.** After `D3`, a
  non-canonical carrier is **rejected** as a lawful `Ord` key **and accepted**
  as a lawful `Map`/`Set` key, and both rows exist. **Control:** name the two
  rows and their opposite verdicts. ⛔ A single row asserting either half alone
  fails this — *"`Decimal` is now a lawful key"* with no surviving `Ord`-reject
  is the banned inversion, not a passing result.

- **`AC-2`** — no route concludes kernel `Equal` from mutual `leq`.
  **Control:** the `Distinct` discharge proof is exhibited with its steps, and
  **no step is `Equal k k'`**. A proof that still needs one means the interface
  is wrong, not that the proof needs a hypothesis.

- **`AC-3`** — the representative policy is observable and pinned.
  **Control:** inserting two `KeyEq`-equivalent, **structurally distinct**
  representatives yields **one** entry, and `to_list` exposes the **last
  inserted** one. ⚠ The two representatives must be structurally distinct or
  the test cannot tell `KeyEq` from `Equal`.

- **`AC-4`** ⭐ **(load-bearing)** — `D4`'s replacement discriminator is
  **causal, not incidental**. The ruling requires it be **`Axiom`-free,
  non-canonical, weak-dictionary-only, and its `Ord`/`antisym`/`Equal`
  mutation must FAIL**. **Control:** exhibit the mutation and its red output.
  ⛔ The old row's discriminator (*which order-faculty each law uses*)
  **collapses** under `C2` — both sides become `refl`/`trans`/`total` — so a
  replacement that merely restates it is **vacuous and green**. The
  replacement must discriminate on a **different axis**.

- **`AC-5`** — `D5`'s inventory is **closed and names its complement**.
  **Control:** show the sites examined **and** those excluded with the reason.
  ⛔ An empty exclusion list while `58 antisymLeq` and the `Ord` comparison /
  compound-instance / sort-permutation sites all exist is a **failed
  measurement**, not a clean one.

- **`AC-6`** (Phase 2) — `D7` runs. **Control:** a `Map` keyed by a carrier
  with two representatives of one value holds **one** entry, and the same
  carrier is still refused where an `Ord` instance concluding kernel `Equal` is
  demanded.

⛔ **No checker or gate** asserting facts about spec or catalog lines (operator
test policy). The weak "reports drift" form is still a gate if it can go red.

---

## 8. Contention and sequencing

**Phase 1 is `spec/` + `conformance/` only.** The enclave is the sole owner of
both. ⚠ It is currently building `SPEC-IDENT-BLESSED` — ⛔ **do not release
Phase 1 until that merges**, and run the handoff gate before the kickoff.

**Phase 2 is `catalog/` only.** ⚠ `Map.ken.md` and `LawfulClasses.ken.md` are
shared with any live catalog work — **re-measure contention at release time**,
not from this frame.

⛔ `C2` blocks no build lane and the operator has not prioritized it. It is
**queued work, not urgent work.**

## 9. Hard stop

⛔ Route to the Steward if: the `Distinct` discharge cannot be closed without an
`Equal` step (the interface is then wrong); or `D5` finds an `antisym → Equal`
consumer **outside** the `Map` discharge (the `§3` localization would be false,
and the whole cost model with it).
