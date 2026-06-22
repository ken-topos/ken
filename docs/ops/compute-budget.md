# Compute budget & build discipline

Ken is developed by many parallel agents. Until hardware grows, the dev machine
is an **8-core / 16 GB laptop**, and the binding constraint is the **Rust build**
— specifically concurrent **linking**, which is the worst RAM spike. The failure
mode to prevent: several build teams each running `cargo build --workspace -j8` at
once → dozens of rustc processes + simultaneous link spikes → swap death / OOM.

The strategy has four levers, in order of impact.

## 1. Cap build concurrency machine-wide (the big one)

Build parallelism *multiplies* with agent parallelism, so the cap must be global,
not per-agent. Agents build and test **only** through `scripts/ken-cargo`, which
holds a machine-wide lock:

```
scripts/ken-cargo build -p ken-kernel
scripts/ken-cargo test  -p ken-kernel
```

- `KEN_BUILD_SLOTS` (default **1**) = how many cargo invocations run at once
  across the whole machine. **1 is correct for 16 GB** — one build at a time, with
  `-j6` from `.cargo/config.toml`, leaves cores for the OS, the agent harness, and
  a scoped test. Raise it (and use GNU `sem`) when you add RAM/cores.
- Never run raw `cargo build`/`cargo test` in an agent — it bypasses the lock.

## 2. Dedup compilation across agents — shared cache + registry

N agents on different branches otherwise each recompile the *same* dependencies
and re-download the registry. `source scripts/ken-env.sh` enables:

- **`sccache`** shared compiler cache (20 GB cap): a dependency compiled by one
  agent is reused by all. Biggest multi-agent win. Implies `CARGO_INCREMENTAL=0`
  (incremental and sccache conflict; cross-agent reuse wins here).
- **Shared `CARGO_HOME`**: one crate index + source cache for all agents.
- **Per-worktree `target/`** stays local (isolation); sccache does the sharing.
  Do *not* share `CARGO_TARGET_DIR` — concurrent cargo contends on its lock and
  branch differences thrash it.

If agents run in separate containers/worktrees, the harness must mount
`SCCACHE_DIR` and `CARGO_HOME` as **shared volumes** — otherwise the cache isn't
actually shared.

## 3. Make each build cheaper

In `Cargo.toml` (committed): dev-profile debuginfo is minimized —
`line-tables-only` for our crates, **off for dependencies** and build scripts.
This cuts RAM, link time, and `target/` size sharply while keeping backtraces.

Optional, highest-value local add-on: install **`mold`** (Linux) or **`lld`**
(macOS) and uncomment the linker block in `.cargo/config.toml`. Linking is the
serial bottleneck and the main OOM source; mold is dramatically faster and
lighter. Left off by default so a fresh clone always builds.

## 4. Push heavy work to CI; keep the laptop incremental

The laptop does **scoped, incremental** work; GitHub Actions (off-laptop compute)
does the **comprehensive** work.

- Agents build/test the **touched crate** (`-p <crate>`), not `--workspace`.
- **Full-workspace build, the conformance suite, and any release/LTO build run in
  CI**, not on the laptop. The Integrator relies on CI being green (per
  `../program/04-git-and-integration.md`), not on local full builds. LTO/`--release` are
  RAM-killers — never run N× locally.

## 5. Phase the teams (don't run all 8 hot)

The roadmap is dependency-ordered; exploit it. On this box, keep **~2–3 teams
active concurrently** — early on that's Foundation → Spec → Kernel, not the whole
federation. Each running agent process (harness + runtime) also costs RAM
independent of builds, so idle teams should be paused, not left resident. The
token-ring already keeps ~one agent active per team; throttle the *cross-team*
parallelism here.

> Inference (Opus/GLM/DeepSeek) runs on remote APIs, so it does **not** consume
> laptop CPU — the laptop's cores are for builds/tests and the harness only.

## Python tooling (if any)

Any Python test tooling (e.g. a conformance runner) inverts the profile: no build
cost, heavy *test-time* CPU. Cap it explicitly — never `pytest -n auto` (that's
all cores × every agent). Pin a small worker count (`pytest -n 2`) and run it
through the same global lock if it's CPU-heavy.

## Scaling path (when hardware grows)

Raise in this order: `KEN_BUILD_SLOTS` (2, 4, …, with `sem`), then per-build
`jobs`, then number of concurrently-active teams. Add a dedicated build/cache box
(shared `sccache` server, beefier CI) before turning the full federation hot.
