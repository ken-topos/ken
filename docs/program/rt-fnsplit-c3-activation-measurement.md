# `RT-FNSPLIT-C3-ACTIVATION` — measurements, findings and the honest ledger

Bound to `origin/main = af8b442a`, frame blob `0ef6226c`, node blob `2bd558f0`.
All six `§1` fixed-input blobs re-measured on this base and all six matched.

## ⭐ Enabler probed BEFORE planning around it

`cc` / `gcc` / `ld` are present, and `package_starter_executable_artifact`
already **links and runs** the executable — asserting `smoke.stdout == "42\n"` —
inside `ken-runtime`'s lib suite on this box. ⇒ `AC-1`'s real linked-starter run
is available **locally**, not CI-only, and ⭐ **those smoke-run tests are exactly
`AC-6`'s pre-existing-positive population.** ⛔ No new fixture family stands in
for them.

## ⛔⛔ FIVE FINDINGS THE CONTROLS TAUGHT — none came from review

### 1 — the persistent image is STORE-lifetime, and the first cut had it
per-activation

`AC-2` — two activations alive **simultaneously** — panicked on the second
with the landed guard's own words: *"reserve before publish: growing a table
moves it under the pointer."* ⭐ That guard is `BoundaryRegion::reserve`
refusing exactly
what `§3` forbids, and it was right. ⇒ Split into `BoundaryStoreBindingV1::open`
(once per store) and `BoundaryActivationV1::begin` (once per activation), which
is `§3a` read correctly. ⚠ It also makes *"an activation cannot widen its own
limits"* structural: the profile lives in the binding.

### 2 — ⛔ `AC-3`(b) SEGFAULTED INSTEAD OF REDDENING

Bypassing `publish` left a null base that every header accessor dereferenced ⇒
**SIGSEGV, taking the whole test binary with it.** ⭐ **A crash is not a loud
failure** — it destroys every other test's result in the shard and names
nothing. ⇒ Publication is a checked state, the accessors return `Option` and
never dereference a null base, and an unpublished activation **has no services
pointer to give**. Re-run after hardening: `M-B` reds all five by assertion.

### 3 — a `dead_code` warning was telling the truth

The owned native-arena box was never read: every check went through the services
record, which could have agreed with itself while pointing at storage the
activation does not own. ⇒ `owned_native_arena_address` is a second independent
surface — ⭐ two surfaces agreeing, ⛔ not one read twice.

### 4 — ⛔ `D7` IS NOT "DELETE TWO STRUCTS"

`ken_print_exported_int` is **a second implementation of `Int` rendering AND of
the export's canonicality checks**, re-deriving in C what
`NativeIntArenaV1::decode_final_export` already decides. ⇒ The layout copy could
not be removed without moving the rendering, and the renderer was landed
**first**, because `AC-6`'s population asserts exact stdout and a rendering off
by one byte reds tests written by someone else.

⚠ The part that would have been got wrong is the padding asymmetry — the
most-significant limb unpadded, every lower limb padded to 16 hex digits — so it
has its own positive control. A renderer that padded every limb passes the
single-limb case **and both sign cases**, failing only on a multi-limb value.

### 5 — the non-process starter linked NO archive at all

It passed `None`, and correctly so: the old stub declared the native-`Int`
layout itself and needed no Rust symbol. ⇒ That is exactly what `D7` removed, so
the archive stopped being optional there. ⭐ **The pre-existing smoke positives
said so by failing to LINK rather than by failing to run** — 38 red, every one
*"undefined reference to `ken_boundary_store_v1_open`"*. That is `AC-6` working.

## ⚠ A RESOURCE COST OF THIS DESIGN, measured rather than forecast

`D1`+`D7` swap a small host archive for the **74 MB** `libken_runtime.a` in
every linked starter, and the packaging corpus links roughly ten per run.
Combined with `temp_output_dir` never cleaning up, `/tmp` reached **100 % full**
(15,525 `ken-runtime-*` directories, 7.6 GB) and three tests failed with
`/usr/bin/ld: final link failed: No space left on device`.

⛔ **Those three failures were the disk, not the change** — triaged on the
*error production raised*, not on the test names. Reclaimed only directories
older than an hour, so no in-flight run of any seat was touched.

⚠ **Measured after reclaiming: one full `-p ken-runtime` run consumes ~700 MB of
`/tmp`.** ⇒ This is a standing cost, not a one-off, and it is the kind of thing
that will present as an unrelated flake to whoever hits it next.

## The ledger — ⛔ stated as a partition, not as a summary

| AC | state |
|---|---|
| `AC-1` real linked starter run | ✅ the pre-existing smoke positives link, run and assert exact stdout |
| `AC-2` two activations distinct | ✅ Rust **and** across the C ABI |
| `AC-3`(a) old host-only link | ✅ production mutation ⇒ every C ABI symbol undefined at link |
| `AC-3`(b) bypass publish | ✅ red ×5 — ⚠ SIGSEGV before hardening |
| `AC-3`(c) native-as-boundary | ✅ red by **pointer identity**, ⛔ no dereference |
| `AC-5` no private C layouts | ✅ removal + build/link fact — ⚠ **residual below** |
| `AC-6` pre-existing positives | ✅ green, and it is the pre-existing population |
| `AC-7` refusal before packaging | ✅ refused at `ResourceProfile` **and no executable written** |
| `D5` profile in package identity | ✅ eight limits perturbed separately, eight distinct identities |
| `AC-4` eight limits | ✅ **six by real generated-code fixtures, two proved UNREACHABLE** — see below |

### ✅ `AC-4` — six cells measured, two proved unreachable

⛔ **The earlier return called this a residual. It was an acceptance gap** — the
governing frame carries no deferment clause — and the leader was right to stand
the return down.

**Six cells** are exercised by a **real generated-code requester**: persistent
nodes · words · data bytes · limbs, and invocation nodes · words. Each fills its
region **to** the authorized ceiling and then asks for one more, and each asserts
the refusal is attributed to **its own** `(scope, resource)`.

⚠ **The fill-to-ceiling step is not decoration, and the control taught it.** The
first cut requested `limit + 1` from an empty region; the request was refused, so
**no count was bumped**, and asking *"which resource is at its limit?"* named
nothing — `persistent words` failed with every live count still zero. ⇒ A
refused request leaves no trace, so the region must be brought to its ceiling
first.

⭐ **Attribution is a comparison between two independent things** — the region's
live count, which emitted code bumped, and the authorized limit the deployment
wrote. ⛔ Not a re-reading of `BOUNDARY_ERR_CAPACITY`, which names nothing and
would make one control claim to be eight.

#### ⛔⛔ The other two cells are UNREACHABLE BY THE ADMITTED RELATION

`BOUNDARY_TAG_CLASS_RELATION` gives the **invocation** arena exactly two lanes —
`(InvocationBorrowed, BorrowedOpaque)` and `(InvocationHostResult, HostResult)` —
and **neither class carries a data body or magnitude limbs**. `Bytes` / `String`
/ `Int` are admitted only under `PersistentGround`, which indexes the persistent
region. ⇒ **No emitted requester can consume an invocation data byte or an
invocation limb**, so those two ceilings are unreachable rather than untested.

⭐ Measured on both sides: the relation is asked directly, **and** an emitted
attempt to claim a data body under an invocation lane is refused with a status
that is ⛔ *not* `BOUNDARY_ERR_CAPACITY`.

⚠ **This is a property of the relation AS LANDED, and the control is written to
go RED if that changes** — `M-AC4c` admits `(InvocationHostResult, Bytes)` and
the control fails saying the two cells now owe fixtures. ⛔ It is a live guard,
not a comment.

| mutation | outcome |
|---|---|
| `M-AC4a` attribution always names the first cell | red — *"attributed to invocation nodes instead of persistent nodes"* |
| `M-AC4b` the persistent region metered against the invocation limits | red |
| `M-AC4c` admit `(InvocationHostResult, Bytes)` — ⭐ the future event the guard exists for | red, naming the newly reachable cells |

### ⚠ `AC-5`'s residual, as a partition with its discriminator

⛔ **Not** *"C has no copies."* What is established is:

- **removed, by name:** the two `KenNativeIntArenaV1` declarations, the
  `KenNativeBigEntryV1` declaration, `KenNativeInvocationV1`,
  `ken_int_arena_destroy`, `ken_print_exported_int`, and the stack construction
  in both `main`s;
- **enforced, by link:** the stub references nine `ken_activation_v1_*` /
  `ken_boundary_store_v1_*` symbols it cannot satisfy without the runtime
  archive — `AC-3`(a) demonstrates that.

⇒ ⛔ **Removing two known copies does not prove a third was never added.** The
discriminator that *would* catch one is the build/link fact above: a private C
copy of a layout the stub also obtains from the owner would compile and link, so
⚠ **it is review-enforced, not mechanically guarded**, and the only layout C is
permitted to know is the resource profile.
