# Wave 4 terminal residual

This report measures the four remaining Wave 4 reference surfaces and four
indexes at `cb2011e65082132943131fb7ecf1b38f3ea42763`. It measures the
stipulated human-audience corpus for coverage and uses repo-wide source
censuses for mechanism-absence claims. Agent pages are outside the coverage
comparison. The normative specification remains a source of terms and
inventories, not human-audience coverage.

The verdicts distinguish two ways a row can close:

- `none` means the human corpus already delivers the subject;
- `not-producible` means the stable extraction or derivation mechanism needed
  to maintain the promised reference or index does not exist;
- a named gap means grounded material exists and a reader-facing lookup page is
  both useful and honest to author now.

## D0 residual measurement

| Subject | Human material and actual delivery | Shape | Verdict |
|---|---|---|---|
| Verification | `library/guide/proof-techniques.ken.md` §§1–6 teaches proof closure, induction, decidable equality, function equality, termination hazards, and evidence chains. `library/learn/reading-ken/02-types-contracts-and-proofs.md` covers signatures, proof claims, and checked evidence; `03-assurance-and-trust.md` covers verification status, certificates/trust, and stated limits. | Explanatory: the material motivates distinctions and is read as an ordered account rather than a standalone lookup table. | `none` |
| Runtime | `library/learn/reading-ken/06-execution.md` covers execution paths, marked partiality, and the native backend; `04-effects-capabilities-and-authority.md` covers effect rows, capabilities, and the checked-versus-host corpus boundary. | Explanatory: the chapters build execution and authority boundaries through narrative and cross-reading. | `none` |
| Platform | `library/learn/reading-ken/06-execution.md` §“Native Backend” explains the backend boundary but is not a target lookup. The build-time emitter in `crates/ken-host/build.rs` writes target, target OS, backend, audited dependency identities, probed Linux ABI facts, and a manifest hash into `target_abi.rs`; `crates/ken-host/src/lib.rs` exposes the generated `TargetAbi`. It completes only for a Linux target equal to the build host and fails closed before generation for cross-target or non-Linux builds. | The existing human material is explanatory. The emitted facts are lookup-shaped but had no human reference page. | named gap `platform-current-facts`; authored as `library/reference/platform/README.md` with `partial` availability and explicit unavailable lanes. |
| Diagnostics | The stipulated human corpus contains no diagnostics reference. The repo-wide census finds structured V4 diagnostics plus distinct `KernelError`, `RuntimeTrapCode`, and `IoErrorIdentityV1` identities in separate subsystems; it finds no one public registry or derivation interface that enumerates all of them. | No complete human lookup exists. Existing structured diagnostics are subsystem-specific, not a complete public inventory. | `not-producible`: Ken first needs a unified public diagnostic registry or derivation interface. |
| Symbol index | No symbol-index page exists. The merged generation report records no symbol-index command or generator and no public-declaration inventory. | No human index exists to classify. | `not-producible`: Ken first needs a stable public-symbol/declaration inventory and an exporter over it. |
| Keyword index | No keyword-index page exists. `spec/30-surface/31-lexical.md` §4 carries a normative keyword proposal, while `crates/ken-elaborator/src/lexer.rs` carries the accepted token variants and spelling map. The repo-wide census finds these inventories but no exporter that derives a reader index from them. | No human index exists to classify. | `not-producible`: the inventories exist; a maintained extraction/export path does not. |
| Diagnostic index | No diagnostic-index page exists. The diagnostics census above finds several real identity families but no unified public enumeration. | No complete human index exists to classify. | `not-producible`: the same unified public registry or derivation interface must exist before an index can be derived. |
| Glossary index | No glossary page exists in `library/`. `spec/00-overview.md` §8 is the authoritative orientation glossary in a chapter normative for terminology. The repo-wide census finds that maintained source but no extraction path from it into `library/`. | No human lookup index exists to classify. | `not-producible`: the source exists; a maintained extraction/export path does not. |

The named-gap set is exactly `{platform-current-facts}`, and the `D1` page set
is exactly `{library/reference/platform/README.md}`. No row is `reclassify`.
The other five undelivered promises remain mechanism-limited for the narrower
reasons above; neither missing inventories nor a missing target emitter is
claimed.

## D3 mechanism findings

1. Platform facts are generated, but only for the current host-equals-target
   Linux lane. The emitter is not a ratified target-support contract or a
   cross-target inventory. The reader page therefore reports the generated
   fields and labels every other lane unavailable.
2. Diagnostics have multiple structured identities and a V4 projection system,
   but no unified public registry or derivation interface spans the kernel,
   runtime, and host error families.
3. The keyword source inventory and lexer inventory exist. What is missing is
   an exporter that can keep a reader index derived from those maintained
   sources.
4. The authoritative glossary source exists in `spec/00-overview.md` §8. What
   is missing is an extraction path into `library/`, not a term taxonomy.
5. The symbol-index finding is unchanged: no stable public-declaration
   inventory or exporter exists.

These are candidate inputs to later implementation or documentation programs.
This slice does not build the missing mechanisms.

## Evidence log

The two already-closed human coverage rows retain their prior measurement:

```console
$ grep -nH '^## ' library/guide/proof-techniques.ken.md library/learn/reading-ken/02-types-contracts-and-proofs.md library/learn/reading-ken/03-assurance-and-trust.md library/learn/reading-ken/04-effects-capabilities-and-authority.md library/learn/reading-ken/06-execution.md
library/guide/proof-techniques.ken.md:15:## Contents
library/guide/proof-techniques.ken.md:83:## 1. `Proved` vs. `Refl`: the two-way discriminator
library/guide/proof-techniques.ken.md:176:## 2. Induction and motive construction
library/guide/proof-techniques.ken.md:218:## 3. Decidable equality: the `sound`/`complete` pattern
library/guide/proof-techniques.ken.md:268:## 4. `funext` is definitional
library/guide/proof-techniques.ken.md:296:## 5. Non-termination hazards
library/guide/proof-techniques.ken.md:326:## 6. Name endpoints and evidence in proof chains
library/learn/reading-ken/02-types-contracts-and-proofs.md:9:## Signatures
library/learn/reading-ken/02-types-contracts-and-proofs.md:32:## Proof Claims
library/learn/reading-ken/02-types-contracts-and-proofs.md:99:## Checked Evidence
library/learn/reading-ken/03-assurance-and-trust.md:10:## Verification Status
library/learn/reading-ken/03-assurance-and-trust.md:39:## Certificates and Trust
library/learn/reading-ken/03-assurance-and-trust.md:190:## Stated Limits
library/learn/reading-ken/04-effects-capabilities-and-authority.md:10:## Effect Rows
library/learn/reading-ken/04-effects-capabilities-and-authority.md:61:## Capabilities
library/learn/reading-ken/04-effects-capabilities-and-authority.md:131:## Corpus Boundary
library/learn/reading-ken/06-execution.md:12:## Execution Paths
library/learn/reading-ken/06-execution.md:99:## Marked Partiality
library/learn/reading-ken/06-execution.md:153:## Native Backend
```

### Platform census

The search ranges over every tracked path at the exact base. It returned 224
hits across 41 paths and found the build script that the earlier narrow census
missed:

```console
$ git grep -nEi '(target[-_ ]?(abi|fact|manifest|support|matrix|inventory|triple)|cross[-_ ]target|CARGO_CFG_TARGET(_OS)?|OUT_DIR)' cb2011e65082132943131fb7ecf1b38f3ea42763 -- .
```

The production seam is:

```console
$ git grep -nE 'fn write_generated|target_abi\.rs|pub struct TargetAbi|include!\(concat!\(env!\("OUT_DIR"\), "/target_abi\.rs"\)\)' cb2011e65082132943131fb7ecf1b38f3ea42763 -- crates/ken-host/build.rs crates/ken-host/src/lib.rs
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-host/build.rs:456:fn write_generated(
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-host/build.rs:479:        PathBuf::from(env::var("OUT_DIR").unwrap()).join("target_abi.rs"),
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-host/src/lib.rs:60:pub struct TargetAbi {
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-host/src/lib.rs:71:include!(concat!(env!("OUT_DIR"), "/target_abi.rs"));
```

`build.rs:24–30` rejects a non-Linux target or `TARGET != HOST` before the
emitter runs. `build.rs:68–84` selects the `linux_raw` facts and constructs the
manifest; `build.rs:436–481` serializes and writes it. This supports the
partial page without turning the emitter into a support contract.

### Glossary census

The repo-wide search returned 24 hits across 16 paths. It found the maintained
source and no extraction/export implementation:

```console
$ git grep -nEi '(glossary|terminolog(y|ies)|term[-_ ]?(registry|inventory|extract|export|generator)|generate.*glossary|extract.*glossary)' cb2011e65082132943131fb7ecf1b38f3ea42763 -- .
$ git grep -nE '^## 8\. Glossary' cb2011e65082132943131fb7ecf1b38f3ea42763 -- spec/00-overview.md
cb2011e65082132943131fb7ecf1b38f3ea42763:spec/00-overview.md:216:## 8. Glossary (orientation; precise definitions in the cited chapters)
```

### Keyword census

The repo-wide search returned 1,158 hits across 245 paths. It found both
maintained inventories and no extraction/export implementation:

```console
$ git grep -nEi '(keyword|Token::Kw|Kw[A-Z]|generate.*keyword|extract.*keyword|export.*keyword)' cb2011e65082132943131fb7ecf1b38f3ea42763 -- .
$ git grep -nE '^## 4\. Keywords|Token::Kw[A-Z]|Kw[A-Z]' cb2011e65082132943131fb7ecf1b38f3ea42763 -- spec/30-surface/31-lexical.md crates/ken-elaborator/src/lexer.rs
cb2011e65082132943131fb7ecf1b38f3ea42763:spec/30-surface/31-lexical.md:522:## 4. Keywords (proposal)
```

The second command also returns the lexer's `Kw*` token variants at lines
18–64 and its spelling map at lines 432–469. The two inventories are not
treated as an already-generated public index.

### Diagnostics census

The repo-wide search returned 2,210 hits across 392 paths. It found several
real diagnostic systems and identities rather than one complete registry:

```console
$ git grep -nEi '(diagnostic|KernelError|RuntimeTrapCode|IoErrorIdentityV1|error[-_ ]?(registry|catalog|inventory|derivation))' cb2011e65082132943131fb7ecf1b38f3ea42763 -- .
$ git grep -nE 'pub (enum|struct) (KernelError|RuntimeTrapCode|IoErrorIdentityV1)|pub mod diagnostics' cb2011e65082132943131fb7ecf1b38f3ea42763 -- crates/
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-elaborator/src/lib.rs:19:pub mod diagnostics;
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-host/src/effect_v1.rs:2076:pub enum IoErrorIdentityV1 {
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-kernel/src/error.rs:14:pub enum KernelError {
cb2011e65082132943131fb7ecf1b38f3ea42763:crates/ken-runtime/src/ir.rs:894:pub enum RuntimeTrapCode {
```

The wide result includes `crates/ken-elaborator/src/diagnostics.rs` and the
checked `catalog/packages/Capability/Diagnostics/` packages. Neither surface
enumerates the kernel, runtime, and host identities above as one public
registry. The finding is therefore the absence of that unifying derivation
interface, not the absence of structured diagnostics.

Because the only `D1` page displays no Ken syntax, AC-5 requires no build turn.
