# Ken

Ken is an MIT-licensed, verified software-engineering language designed for a
world in which agents write programs and humans review them.

The scarce resource in that world is not typing code. It is understanding what
the code promises and deciding whether those promises are justified. Ken makes
that boundary part of the language: programs state properties alongside their
implementations, the toolchain proves what it can, and the remaining claims are
identified honestly as tested, delegated, or unknown.

Ken is intended for systems-adjacent through application-level software. It
combines a dependently typed language with a small, permanent proof kernel, a
strict reference interpreter, and a native compiler. The kernel re-checks proof
certificates produced by the rest of the toolchain, so the trusted foundation
remains small enough to audit.

Read [`docs/PRINCIPLES.md`](docs/PRINCIPLES.md) for the project's reasoning
charter and [`spec/00-overview.md`](spec/00-overview.md) for the language
overview.

## Language

Ken brings specifications, proofs, programs, and their trust boundaries into
one readable source language.

- **Dependent types.** Pi and Sigma types, inductive families, predicative
  universes, refinements, and total pattern matching express relationships that
  ordinary type systems leave to tests or comments.
- **Observational equality.** Equality, casts, proof irrelevance in `Omega`,
  quotients, and decidable conversion live behind a compact kernel interface.
- **Proofs and specifications.** Functions can carry requirements and
  guarantees. The elaborator generates obligations, the prover produces
  certificates, and the kernel checks them independently.
- **Honest verification status.** Ken distinguishes claims that are `proved`,
  `tested`, `delegated`, or `unknown` instead of presenting every successful
  build as the same kind of assurance.
- **Everyday data.** Arbitrary-precision and fixed-width numbers, strings,
  bytes, sums, records, collections, recursive data, and nested patterns are
  available in the surface language and packages.
- **Canonical type classes.** Classes support reusable interfaces while a
  coherence discipline keeps instance resolution predictable and readable.
- **Effects and foreign boundaries.** Interaction-tree effects, capabilities,
  information-flow labels, constant-time annotations, and explicit foreign
  interfaces keep environmental assumptions visible.
- **Modules and packages.** Names can be organized, imported, qualified, and
  shared across package boundaries without making source-file layout the
  semantic authority.
- **One canonical format.** `ken fmt` formats both `.ken` source and literate
  `.ken.md` files; `ken fmt --check` makes formatting enforceable in automation.

Ken optimizes the permanent form for reading and checking. Rich mathematical
notation has an ASCII transliteration, proof-relevant choices remain visibly
different from proof-irrelevant facts, and ambiguity is rejected rather than
resolved by hidden convention.

## Catalog

[`catalog/`](catalog/) is the public, executable guide to the language. It is
both reference material and real Ken source checked by the toolchain.

- [`catalog/guide/`](catalog/guide/) teaches the surface language and its proof
  techniques.
- [`catalog/packages/`](catalog/packages/) contains standard packages with
  their laws, proofs, examples, and design rationale.
- Literate `.ken.md` files keep explanations next to the code while preserving
  byte-stable prose during formatting.
- The catalog covers core abstractions, lawful functors and classes, natural
  ordering, sums, maps, parsing, effects, collections, and related proof
  patterns.

The catalog is a good place to begin if you want to see how Ken reads before
diving into the normative specification.

## Interpreter and compiler

Ken has two execution paths with deliberately different roles.

The **reference interpreter** defines program behavior. It evaluates Ken using
strict call-by-value semantics over the content-addressed value model and drives
supported effects through explicit capabilities. The REPL and `ken run` use
this path. When another execution backend disagrees with the interpreter, the
interpreter is the semantic reference.

The **Rust bootstrap compiler** consumes kernel-admitted checked core, erases
proof-only content, lowers executable code to Ken's runtime IR, and uses
Cranelift to produce native artifacts for the supported closed-program subset.
Native results are compared with the interpreter, and build artifacts carry
provenance and trust information. The compiler is not part of the
type-soundness trust root: compilation bugs must not become false proofs.

The compiler and interpreter therefore complement each other. The interpreter
provides the simple, stable meaning of a program; the compiler provides native
execution while remaining accountable to that meaning.

## Command-line tools

The `ken` driver currently provides:

```text
ken check <file>
ken run <file> [-- <arguments>...]
ken fmt [--check] <paths...>
ken repl
ken version
```

`check` elaborates source and verifies literate fence expectations without
running a program. `run` executes a Console-capable entry point through the
reference interpreter. `fmt` canonicalizes plain and literate Ken source.

Full support for building command-line tools is in progress. A full POSIX
interface and Linux ABI support are up next. Until those surfaces land, the
current command runner and native executable path should be read as supported
subsets rather than a complete systems interface.

## Repository map

- [`catalog/`](catalog/) — executable guide and standard packages.
- [`crates/`](crates/) — the kernel, elaborator, prover-facing surface,
  interpreter, runtime/compiler support, content-addressed foundation, and CLI.
- [`spec/`](spec/) — normative language and runtime specification.
- [`conformance/`](conformance/) — black-box conformance cases and seeds.
- [`docs/adr/`](docs/adr/) — architecture decisions.
- [`docs/PRINCIPLES.md`](docs/PRINCIPLES.md) — the reasoning charter.
- [`CLEAN-ROOM.md`](CLEAN-ROOM.md) — provenance and clean-room policy.

## Build

Ken is implemented in Rust. To build the CLI and run the workspace tests:

```bash
cargo build --workspace --locked
cargo test --workspace --locked
cargo run -p ken-cli -- help
```

## License

Ken is licensed under the [MIT License](LICENSE). Programs written in Ken may
use any license their authors choose.
