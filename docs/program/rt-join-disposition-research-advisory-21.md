# NATIVE-HANDLE-CARRIER hard-stop #21 research advisory

**Status:** research advisory, not a design ruling  
**Question:** whether a join materialized before a later proof of branch
unreachability is an inconsistency or a legitimate compiler phase ordering

## Finding

There is no prior-art consensus that branch elimination must precede join
materialization. Mature compilers support both orders:

- **Pruned construction:** use reachability/liveness known at construction time
  to avoid unnecessary φ nodes or block arguments.
- **Overcomplete construction plus cleanup:** construct a valid CFG and its
  joins, later fold a branch or prove an edge unreachable, then remove the edge,
  dead block, and redundant join inputs.

Consequently, “a join was emitted and later its branch was proved unselected”
is not, by itself, a compiler inconsistency. It is normal if “emitted” means
“materialized in an overcomplete IR.” It is inconsistent if “emitted” means
“proved semantically reachable in the same context” and the later fact claims
that same context is unreachable.

Prior art therefore does not decide the Ken fork from the overlap alone. The
load-bearing question is what fact Ken's `consumed_join_origins` records:
materialization, traversal, or semantic reachability for a particular dynamic
or static context.

## Cranelift

Cranelift's frontend SSA builder explicitly supports incomplete CFG
construction:

- A block is sealed only after all predecessors are known.
- Uses in an unsealed block may create an incomplete block parameter, completed
  when the block is sealed.
- A previously declared predecessor may be removed before sealing.
- `seal_all_blocks` exists for frontends that cannot seal incrementally.

The implementation also treats unreachable SSA construction as tolerable: an
undefined value in unreachable code is initialized because it has no execution
effect. After frontend construction, Cranelift code generation computes the
CFG and dominator tree, eliminates unreachable blocks, and removes constant
φ/block parameters before later optimization.

This is direct evidence that Cranelift does not require all dead edges to be
removed before block/join materialization. It does impose a phase boundary:
frontend predecessor changes must happen before sealing; transformations after
sealing use whole-IR CFG repair and verification rather than the frontend
builder's incremental predecessor API.

Sources:

- [Cranelift `FunctionBuilder`](https://docs.rs/cranelift-frontend/latest/cranelift_frontend/struct.FunctionBuilder.html)
- [Cranelift SSA construction and sealing](https://docs.rs/cranelift-frontend/latest/src/cranelift_frontend/ssa.rs.html)
- [Cranelift unreachable-block elimination](https://docs.wasmtime.dev/api/src/cranelift_codegen/unreachable_code.rs.html)
- [Cranelift compilation context](https://docs.wasmtime.dev/api/cranelift_codegen/struct.Context.html)
- [Braun et al., direct SSA construction](https://pp.ipd.kit.edu/uploads/publikationen/braun13cc.pdf)

## LLVM

LLVM also contains both orders.

`mem2reg` computes live-in blocks before placing φ nodes, explicitly avoiding
φ nodes in blocks where they would be dead. That is pruned construction.

Separately, `SimplifyCFG` operates on already-formed SSA. When it removes an
edge, it calls `BasicBlock::removePredecessor`; that API updates φ nodes before
the predecessor is removed. The pass can replace branches with `unreachable`,
redirect switch edges, merge blocks, and transform φ nodes after CFG
materialization.

Thus LLVM treats “emit then retract” as ordinary, but not as a bookkeeping-only
operation. The CFG edge, φ incoming value, dominator information, and any
dependent analyses must be updated together. A φ incoming block that is no
longer a predecessor is invalid IR; a φ that becomes trivial may be folded.

Sources:

- [LLVM `mem2reg` implementation](https://llvm.org/doxygen/PromoteMemoryToRegister_8cpp_source.html)
- [LLVM `SimplifyCFG` implementation](https://llvm.org/doxygen/SimplifyCFG_8cpp_source.html)
- [LLVM `BasicBlock::removePredecessor`](https://llvm.org/doxygen/classllvm_1_1BasicBlock.html)

## MLIR

MLIR's SSACFG regions use block arguments instead of φ operations. Its language
reference states that non-entry blocks with no incoming successor edge are
unreachable and may be removed without changing semantics.

MLIR's region simplifier first erases unreachable blocks, then runs region DCE
and dead-argument cleanup. Its canonicalizer includes folding a constant
conditional by replacing it with the selected branch and erasing the
conditional operation.

Two qualifications are important:

1. Canonicalization is best-effort and has no fixed universal canonical form.
2. MLIR says pipelines must remain correct if the canonicalizer is removed.

So overcomplete but valid IR is normal; cleanup cannot be the only thing making
an otherwise malformed IR correct.

Sources:

- [MLIR language reference](https://mlir.llvm.org/docs/LangRef/)
- [MLIR region simplification](https://mlir.llvm.org/doxygen/RegionUtils_8cpp_source.html)
- [MLIR canonicalization](https://mlir.llvm.org/docs/Canonicalization/)

## Permissive reference: Lean 4

Lean demonstrates both strategies in one compiler:

- Its LCNF `JpCases` simplifier specializes constructor-known jumps and states
  that, when every jump is specialized, the original join point is eliminated
  as dead code.
- Its LLVM emitter materializes a basic block for each case alternative and a
  basic block for a join-point declaration; the downstream LLVM pipeline may
  subsequently simplify that CFG.

This is approach-level evidence only. No reference expression or implementation
is proposed for Ken.

Local sources:

- `local/refs/lean4/src/Lean/Compiler/LCNF/Simp/JpCases.lean:158-207`
- `local/refs/lean4/src/Lean/Compiler/IR/EmitLLVM.lean:1101-1146`

## Recognized sound pattern and its invariant

“Emit then retract” is a recognized sound pattern when the intermediate state
is either valid IR or a compiler-private construction state with a clearly
defined completion boundary. The replacement for a blanket
“emitted XOR unselected” rule is CFG/SSA consistency:

- every live predecessor has the correctly typed join argument;
- no removed predecessor retains a φ/block-argument input;
- reachable uses remain dominated by their definitions;
- blocks erased as dead are unreachable from the entry under the IR's control
  semantics;
- analyses invalidated by CFG mutation are repaired or recomputed; and
- any path-sensitive selection fact is keyed to the context in which it was
  proved.

A compiler may deliberately enforce a stronger phase contract—such as “static
selection is complete before emission” or “no CFG changes after sealing”—and
reject violations. That is a valid project invariant, but it is not mandated by
SSA correctness or by the surveyed compilers.

## Failure modes

### Eliminate first

This gives smaller IR, fewer joins, and simpler downstream analysis. It is safe
when the reachability proof is complete, monotone, and scoped to the same
context as emission.

It fails when a supposedly global selection is actually context-dependent. A
source occurrence revisited by recursion, specialization, or another path may
select a different branch. Eliminating on the first observation loses a real
predecessor and the value that predecessor contributes to the join.

Ken's landed comments already identify this risk independently of the new
failure: `6a451b45:crates/ken-runtime/src/cranelift_backend/lowering/mod.rs:
733-740,1689-1692` says a recursive producer may revisit one source occurrence
and therefore closes the dead population from the union of reached cases, not
the first constructor.

Early elimination can also discard debug, provenance, profile, exception, or
deoptimization information if those are attached only to the executable
subgraph. Such information may need a separate non-executable carrier even
when removing the code is semantically sound.

### Emit then retract

This tolerates late facts and incomplete traversal order, but cleanup is more
complex. Merely changing a disposition set is insufficient: predecessor
lists, join operands, dominance, use-def chains, exception/deoptimization
metadata, and cached analyses may all need repair.

Retraction after an incremental SSA builder has sealed a block cannot be
performed through an API that assumes the predecessor set is still open. It
requires a later transformation API, reconstruction, or a deliberately delayed
seal. Leaving a dead block in the final IR may be semantically harmless, but
leaving a mismatched join input is not.

## Ken-specific grounding without a ruling

At `6a451b45`:

- `lowering/mod.rs:1649-1679` treats a join already in
  `consumed_join_origins` as an error when a later subtree disposition includes
  it.
- `:1689-1715` records selected match cases and deliberately postpones
  disposition because recursive revisits can extend the reached-case union.
- `:1739-1769` closes all unrecorded cases only at generated-function closure.
- `:1772-1801` records a planned join when lowering enters its source
  occurrence.
- `:1821-1863` requires every planned join to be in exactly one of the consumed
  or dispositioned sets.
- `planning/static_transition.rs:2134-2175` derives all planned joins in a dead
  owner subtree structurally.

Those facts establish that the present check encodes a deliberate exclusive
classification. They do not establish whether entry during lowering is
equivalent to semantic reachability for the newly admitted program shape.

## Advisory

The literature supplies a negative result on the requested direction:
**emission-before-elimination is not generally an error, and
elimination-before-emission is not generally required.** Both are established
compiler designs.

The Architect's direction therefore turns on Ken's phase contract and the exact
meaning/context of the two recorded facts:

- same-context semantic reachability followed by same-context unreachability is
  inconsistent;
- structural materialization followed by proven unreachability is ordinary and
  must be handled as CFG/SSA cleanup;
- facts from different recursive visits or specialization contexts are not
  contradictory and must not be collapsed into one global selection.

This advisory does not determine which of those descriptions fits
`StaticOriginId(1000)`.
