# Cold-context evaluation protocol

## Exit predicate

`agent_core_ready(run)` holds when a genuinely cold coding agent, given only
the selected pack and task fixture:

1. produces a correct, reviewable result for every supported task;
2. refuses every unsupported or unproved request at the named boundary;
3. loads no file outside the selected pack unless a module prerequisite or
   cited authority requires it;
4. invents no syntax, capability, package, command, or proof; and
5. cites the authority used for each language or capability claim.

Any invented syntax or capability makes `agent_core_ready(run)` false,
regardless of correctness on the other tasks.

## Cold-seat precondition

`cold(seat, run)` means the seat has not seen the task fixture, expected result,
or any earlier result from this suite in its current context. Record the seat
identifier and fresh-context evidence. A seat is cold only once for a fixture;
after a module fix, use a semantically equivalent held-back variant with a new
identifier.

## Running the suite

1. Choose the pack declared by each task in `tasks.toml`.
2. Start a seat satisfying `cold(seat, run)`.
3. Provide only the task prompt, fixture, `README.md`, selected pack manifest,
   and its transitive includes.
4. Record every additional file load.
5. Preserve the full answer and cited paths.
6. Record all four axes independently in the run artifact, using the fields
   `correctness`, `unnecessary_loads`, `inventions`, and `cited_authority`.
7. Evaluate `agent_core_ready(run)`; never summarize the run as a pass rate.

## The seven tasks

| ID | Required observation |
|---|---|
| `explain-contract` | explain a small program's contract and trust posture |
| `write-pure-law` | write and check a pure function with one real law |
| `repair-proof-terminal` | distinguish and repair `Proved` versus `Refl` |
| `find-package-by-task` | select a catalog package by task, not guessed name |
| `write-effectful-boundary` | include both the effect row and capability supply |
| `refuse-unsupported` | refuse an unsupported or unproved request honestly |
| `diagnose-layers` | distinguish parse, elaboration, kernel, and runtime failures |

`tasks.toml` supplies prompts, pack selection, and expected properties. It does
not contain full reference answers, so the ordinary run input does not preload
the answer.

## Four independent axes

- **Correctness:** `correct`, `partially-correct`, or `incorrect`, with
  task-specific evidence.
- **Unnecessary file loads:** an integer count plus the exact paths and why
  each was unnecessary.
- **Invented syntax or capabilities:** a list. It must be empty; any item fails
  the whole suite.
- **Cited authority:** `complete`, `partial`, or `missing`, with exact cited
  paths.

The corpus is a bounded oracle. A green suite does not establish behavior no
task exercises.
