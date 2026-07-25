# `RT-FNSPLIT-B2V` — per-pin evasion attempts

One row per pin, as the frame's evasion AC requires. **Branch:**
`wp/RT-FNSPLIT-B2V-executable-value-abi`, base `origin/main` = `aecdb001`.

> ⛔ **Failing to find a witness is evidence about the witnesses you could
> think of, never about the property.** Every "not found" row below says which
> surface was searched, so the limit is inherited rather than hidden.

## ⛔ TWO MORE PINS WERE FALSE GREENS

Found by QA — and by generalizing QA's finding rather than fixing its instance.

**Runtime QA blocked `c3f6da02`** with a compile-preserving mutation I had
not attempted: `LAST_TAG` from `8` to `8 + 1`. **Every boundary test stayed
green (13/13).** The emitted-code negative probed the single byte `0xFF`, so
tag `9` became accepted by `define_resolve` and nothing asked about it.

★ **The Rust-side twin already swept all 256 bytes.** The discipline was applied
to one side of the same property and not the other — which is precisely how a
per-candidate reminder gets satisfied by the control you were thinking hardest
about. *Sampling one witness is not a sweep, and it was my own table that called
the tag set "closed".*

⇒ **Generalizing the defect class rather than fixing the reported instance found
a second one.** `AC-9`'s inventory pin asserted properties of
`BOUNDARY_LOCAL_HELPERS` — its length, its uniqueness — and **never asked the
emitter anything**. Renaming a helper at its `declare` site left it green.
**The row below that claimed a name swap reddens was FALSE, and I wrote it.**

| pin | mutation | before | after |
|---|---|---|---|
| tag closure at the emitted interface | `LAST_TAG + 1` | 🔴 green (13/13) | 🟢 reddens: *"emitted `class` admitted tag byte 9"* |
| `AC-9` inventory | rename one helper at its `declare` site | 🔴 green | 🟢 reddens, naming `ken_boundary_owner_RENAMED` |

**Both repairs interrogate the artifact instead of the declaration of intent:**
the tag pin sweeps all 256 bytes through the emitted helpers and derives its
expectations from `from_bits`/`referent_owner` — a *different* expression of the
rule than the CLIF's threshold comparisons, so the two must agree rather than
one restating the other; the inventory pin reads the module's actual declared
`ken_boundary_*` symbols. ⚠ Both repairs are **test-only** — every diff hunk
sits inside `mod tests`, and the production bytes QA reviewed are unchanged.

## The one that succeeded — and what it cost

| | |
|---|---|
| **pin** | `b2v_ac3_the_lowered_boundary_disposition_has_no_wildcard_arm` |
| **first form** | `!region.contains("_ =>")` |
| **evasion** | `unhandled => ProtocolOnly { .. }` — a **binding** catch-all |
| **compiles?** | ✅ yes (unreachable-pattern warning only) |
| **pin verdict** | 🔴 **GREEN with the catch-all in place — the evasion won** |

★ A binding catch-all silences exhaustiveness exactly like `_` while matching
no `_ =>` substring. **The pin was a claim about one spelling where the
property is about the shape of every arm** — a *granularity* error, which is
the diagnosis that says to enumerate rather than to add the second spelling to
a list that is open at the top.

**Repair:** every line carrying `=>` in the region must begin with `Lowered::`
or a `|` continuation. Verified as a non-degenerate pair — with the evasion
applied the repaired pin reddens **and names the offending arm**; with it
removed the pin greens.

## Every pin

| pin | evasion attempted | compiles? | verdict |
|---|---|---|---|
| `AC-1` tag set closed (**Rust**) | mint a word with tag byte `9`..`255` | ✅ | 🟢 refused — `from_bits` is a closed `match` with `_ => None`, and the test sweeps **all 256 bytes** rather than a sample |
| `AC-1` tag set closed (**emitted**) | `LAST_TAG + 1` | ✅ | 🔴 **DEFEATED — QA's finding.** The emitted negative sampled one byte. Repaired with a 256-byte sweep through the emitted helpers; verified as a non-degenerate pair |
| `AC-1` list/enum drift | add a `BoundaryTag` variant, leave `ALL` alone | ✅ | 🟢 `ALL`'s declared length `9` reddens; the sweep's `byte < ALL.len()` boundary reddens too |
| `AC-2` no value-specialization | pass a seed value into the construction site | ❌ **does not compile** | 🟢 **strongest row — the surface is closed, not merely unwatched.** `boundary_value` imports no `NativeSeedEnvironment` and no environment vector, so there is nothing in scope to specialize from |
| `AC-2` magnitude boundary | claim the immediate range one value wider | ✅ | 🟢 reddens — the case list tests `MAX`, `MAX+1`, `MIN`, `MIN-1`, not typical magnitudes |
| `AC-3` no wildcard | `_ =>` / `unhandled =>` | ✅ | 🔴 **defeated in first form; repaired — see above** |
| `AC-3` demote a live arm | move `Lowered::Constructor` into `FailClosedForbidden` | ✅ | 🟢 reddens — the fail-closed block is checked **positionally**, not by counting arms |
| `AC-3` second dispatch | add a second `boundary_disposition` with a wildcard | ✅ | 🟢 reddens on the single-dispatch assertion |
| `AC-4`/`AC-5` template read | `field` returns a baked-in word | ✅ | 🟢 **M1** — reddens `left: 7, right: -3`. Discriminating: the head=`7` case still **passed**, so the catch came only from running one probe against three values |
| `AC-4`/`AC-5` constant table | `host_payload` selects a fixed arm | ✅ | 🟢 **M2** — reddens `left: 11, right: 22`, one test, no collateral |
| `AC-6` wrong referent owner | collapse `referent_owner()` to one constant | ✅ | 🟢 **M3** — reddens `left: 1, right: 2` |
| `AC-6` degenerate pair | — | — | 🟢 guarded: the test asserts the two owners **differ**, so an oracle that collapsed them fails instead of agreeing with itself |
| `AC-7` escape | return `OK` for invocation-owned words | ✅ | 🟢 reddens; and a malformed word must report `ERR_TAG`, **not** `ERR_ESCAPE` — a corrupt word and a lifetime violation are different answers |
| `AC-7` vacuity | — | — | 🟢 positive control on the permitted side: persistent and immediate words must return `OK`, so "everything refused" cannot pass |
| `AC-9` inventory | add a 14th helper | ✅ | 🟢 reddens — the count is derived from `BOUNDARY_LOCAL_HELPERS`, so the list and the emission must move together |
| `AC-9` name swap | rename a helper at its `declare` site | ✅ | 🔴 **DEFEATED — the row here previously claimed this reddened, and it did not.** The pin only read the list, never the module. Repaired to compare the module's declared `ken_boundary_*` symbols against the permitted set |
| `AC-1` unreachable tag | keep a tag no disposition produces | ✅ | 🔴 **found by inspection, not by a pin** — `ImmediateCapability` / `ImmediateResource` were in the closed set and unreachable. Fixed by removing them (`f934d233`); ⚠ **recorded as a residual: no mechanical check enforces tag reachability**, so a future unreachable tag is review-caught, not CI-caught |

## Mutation provenance

⚠ Provenance was established by **content delta** (`sha256` + `git diff
--numstat`), never by re-counting an anchor. That mattered: the single-line
anchor `let child = b.ins().load(types::I64, MemFlags::trusted(), address, 0);`
occurs **twice** in `boundary_value_clif.rs` — in `define_field` and in
`define_host_payload`. A naive anchor count would have been ambiguous about
which site was mutated; each mutation was therefore applied through a
uniqueness-asserted multi-line block.

All three restored byte-identically, each verified with `git diff --quiet`
(exit 0). ⚠ `git diff --stat` always exits 0 and is not an emptiness test.

## `AC-8` — the census, predicted before measuring

| pin | predicted | measured |
|---|---|---|
| `correspondence_adds_no_emitted_unit_to_the_production_census` | unchanged | ✅ unchanged |
| `the_backend_production_surface_inventory_is_closed` | unchanged, 12 modules / list 13 | ✅ unchanged |
| `px8i_…identical_local_helper_clif` (`LOCAL_HELPER_COUNT = 6`) | unchanged | ✅ unchanged |
| `lowering/mod.rs` census row | stays `0`/`0`/`0` | ✅ stays `0`/`0`/`0` |

⇒ The frame's original *"the pin you will trip"* prediction was **false for
this placement**, reported before building rather than discovered as a green.
Every landed census is scoped to `cranelift_backend/**`; `native_int_clif.rs`
already declares 8 functions and appears in none of them. **A pin's silence is
scoped to the question it asks**, so their silence about a sibling file is not
evidence — which is why `AC-9`'s boundary-helper census exists at all.
