# Wave 4 generation-capability report

This report records what the Ken toolchain at
`9cbc5bffad7418da308278d2ca2523048714c11c` can emit for the fact classes
promised by documentation-program Wave 4. It reports capability only. It does
not propose or implement generators, output formats, or registries.

## Result

| Fact class | Can emit today? | What is missing | Evidence |
|---|---|---|---|
| Exact syntax | No | No syntax-production emitter or generator | S2, S3, G1 |
| CLI | Partial | `help` emits human prose, omits four accepted global option spellings, and has no machine-readable form | S3, C1, C2, C3 |
| Target | No | No target-fact command or generator; `version` identifies the interpreter but emits no target | S2, S3, G1, C4 |
| Public declarations | No | `check` validates declarations but emits no declaration inventory | S2, S3, G1, C5 |
| Symbol index | No | No symbol-index command or generator | S2, S3, G1 |
| Keyword index | No | No keyword-index command or generator | S2, S3, G1 |
| Diagnostic index | No | No diagnostic-index command or generator | S2, S3, G1 |
| Glossary index | No | No glossary command or generator | S2, S3, G1 |

The result means later Wave 4 facts in these classes must be authored and
labelled as authored unless a later implementation adds an observed emitter.
The one partial result is not a generation contract: the human help text is
both incomplete and unsuitable as structured generator input.

## Required finding: global options omitted by help

`ken help` lists the seven subcommands and `fmt [--check]`, but it does not
mention any global option. Runs C2 establish that `--version`, `-V`, `--help`,
and `-h` are nevertheless accepted and exit successfully. The toolchain
reference documents those accepted spellings. The help omission remains a
`ken-cli` finding; this documentation slice does not edit the CLI.

## Exit-status boundary

Observed runs establish non-`run` failures at status 1 and establish that
`run` returns the program's status, including status 37. The source inventory
S1 also contains two status-2 arms for ABI-unavailable errors. No CLI command
in this report reached either arm, so they are **source-declared and
unobserved**. They are not claims in the reader reference.

There is no uniform exit-status rule.

## Command log

All commands below ran from the repository root at the recorded revision.
Empty output is stated explicitly.

### S2 — complete documentation-generator inventory

```console
$ rg --files scripts | rg '/gen-' | sort
scripts/gen-doc-status.sh
scripts/gen-progress.sh
scripts/gen-source-attestations.sh
$ rg -n '^# Usage:|^(ISSUES_DIR|MANIFEST|REVISION_FILE|OUT_FILE|PROPOSED_FILE)=' scripts/gen-doc-status.sh scripts/gen-progress.sh scripts/gen-source-attestations.sh
scripts/gen-source-attestations.sh:20:# Usage:
scripts/gen-source-attestations.sh:31:MANIFEST="$ROOT/library/manifest.toml"
scripts/gen-source-attestations.sh:32:PROPOSED_FILE="$ROOT/library/SOURCE-ATTESTATIONS.proposed"
scripts/gen-progress.sh:12:# Usage:
scripts/gen-progress.sh:20:ISSUES_DIR="$ROOT/docs/program/issues"
scripts/gen-progress.sh:21:OUT_FILE="$ROOT/docs/program/IMPLEMENTATION-PROGRESS.md"
scripts/gen-doc-status.sh:28:# Usage:
scripts/gen-doc-status.sh:36:MANIFEST="$ROOT/library/manifest.toml"
scripts/gen-doc-status.sh:37:REVISION_FILE="$ROOT/library/REVISION"
scripts/gen-doc-status.sh:38:OUT_FILE="$ROOT/library/STATUS.md"
```

Exit status: 0. This is a source inventory, not an observation of generator
behaviour. It closes the three installed generator interfaces and their output
paths: the documentation status, implementation progress, and proposed source
attestation ledger.

### S3 — complete CLI dispatch inventory

```console
$ sed -n '8,44p' crates/ken-cli/src/main.rs
fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();

    let result = match command.as_str() {
        "repl" => repl::run(),
        "run" => match args.next() {
            Some(path) => run::run(Path::new(&path), collect_run_args(args)),
            None => Err("ken run: expected a source file".into()),
        },
        "check" => match args.next() {
            Some(path) => check::run(Path::new(&path)),
            None => Err("ken check: expected a source file".into()),
        },
        "native-build" => native_build::run(args.collect()),
        "fmt" => fmt::run(args.collect()),
        "version" | "--version" | "-V" => {
            print_version();
            Ok(())
        }
        "" | "--help" | "-h" | "help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("ken: unknown subcommand '{other}' — try 'ken help'").into()),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        process::exit(1);
    }
}
```

Exit status: 0. This source inventory closes every top-level dispatch arm; it
does not by itself establish the commands' observed behaviour.

### G1 — observed documentation-generator outputs

```console
$ scripts/gen-doc-status.sh --check
gen-doc-status --check: library/STATUS.md is current.
$ scripts/gen-progress.sh --check
gen-progress --check: OK (/workspaces/ken/.worktrees/doc-author/docs/program/IMPLEMENTATION-PROGRESS.md is up to date)
$ scripts/gen-source-attestations.sh
wrote /workspaces/ken/.worktrees/doc-author/library/SOURCE-ATTESTATIONS.proposed
Review it, then install mechanically:
  mv /workspaces/ken/.worktrees/doc-author/library/SOURCE-ATTESTATIONS.proposed /workspaces/ken/.worktrees/doc-author/library/SOURCE-ATTESTATIONS
```

All three commands exit 0. Their observed outputs agree with S2. None emits
syntax, CLI, target, declaration, symbol, keyword, diagnostic, or glossary
facts.

### C1 — human CLI output

```console
$ target/debug/ken help
ken 0.0.0 — verified topos-oriented language

Usage: ken <subcommand>

Subcommands:
  run <file>    Elaborate and run a Ken source file (Console IO)
  check <file>  Elaborate a Ken source file and verify its fences,
                without driving IO (for pure-library entries)
  native-build <file> <output-dir>
                Build the checked Program I main as a native artifact
  fmt [--check] <paths...>
                Canonicalize Ken source, or check without writing
  repl          Start the interactive REPL (the Little Prover loop)
  version       Print version and kernel information
  help          Print this message
```

Exit status: 0.

### C2: accepted global option aliases

```console
$ target/debug/ken --version
ken 0.0.0 — verified topos-oriented language
kernel 0.0.0
ken reference interpreter (X1)
$ target/debug/ken -V
ken 0.0.0 — verified topos-oriented language
kernel 0.0.0
ken reference interpreter (X1)
```

Both commands exit 0. `target/debug/ken --help` and
`target/debug/ken -h` both reproduce C1 exactly and exit 0.

### C3 — no machine-readable CLI mode

```console
$ target/debug/ken --format=json
ken: unknown subcommand '--format=json' — try 'ken help'
```

Exit status: 1.

### C4 — version output is not target output

```console
$ target/debug/ken version
ken 0.0.0 — verified topos-oriented language
kernel 0.0.0
ken reference interpreter (X1)
$ target/debug/ken target
ken: unknown subcommand 'target' — try 'ken help'
```

Exit statuses: 0 and 1, respectively. The successful output contains no target
triple, operating system, architecture, or backend target fact.

### C5 — checking validates but does not enumerate declarations

```console
$ target/debug/ken check library/guide/decomposition-abstraction.ken.md
$ target/debug/ken declarations
ken: unknown subcommand 'declarations' — try 'ken help'
```

The first command exits 0 with empty output. The second exits 1.

### C6: run arguments and program status

The status-propagation probe, saved as
`target/doc-w4-toolchain/exit-37.ken`, is:

```ken
program capabilities FS APartial

proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  host_exit APartial (Failure 37)
```

Its observed result is:

```console
$ target/debug/ken run target/doc-w4-toolchain/exit-37.ken
$ echo $?
37
```

The run prints nothing. The argument-discrimination probe, saved as
`target/doc-w4-toolchain/arg-status.ken`, is:

```ken
program capabilities FS APartial

proc main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  match input {
    MkProcessInput arguments _environment _cwd |->
      match arguments {
        Nil |-> host_exit APartial (Failure 41);
        Cons _ _ |-> host_exit APartial (Failure 42)
      }
  }
```

It returns status 41 when its `ProcessInput.arguments` is empty and status 42
when it is non-empty:

```console
$ target/debug/ken run target/doc-w4-toolchain/arg-status.ken
$ echo $?
41
$ target/debug/ken run target/doc-w4-toolchain/arg-status.ken -- hello
$ echo $?
42
```

Both runs print nothing. The separator keeps `hello` out of the CLI option
surface and passes it to the Ken program.

### C7 — formatter refusal on a tracked input

```console
$ target/debug/ken fmt --check conformance/challenge/C1-deceq-noncanonical/unsound-deceq-decimal.ken
ken fmt --check: non-canonical: conformance/challenge/C1-deceq-noncanonical/unsound-deceq-decimal.ken
$ cp conformance/challenge/C1-deceq-noncanonical/unsound-deceq-decimal.ken target/doc-w4-toolchain/format-copy.ken
$ target/debug/ken fmt target/doc-w4-toolchain/format-copy.ken
$ target/debug/ken fmt --check target/doc-w4-toolchain/format-copy.ken
```

The first command exits 1. The copy, rewrite, and final check exit 0 with empty
output. The tracked challenge file makes the refusal input inspectable; the
copy keeps the rewrite out of the tracked source tree.

### S1 — source-declared exit-status classes

This inventory does not establish runtime behaviour; it records why the two
unobserved status-2 arms cannot be collapsed into a uniform rule.

```console
$ grep -oE 'process::exit\([0-9a-z_.]+\)' crates/ken-cli/src/main.rs | sort | uniq -c
     29 process::exit(1)
      2 process::exit(2)
      1 process::exit(outcome.exit_status)
```

Exit status: 0.
