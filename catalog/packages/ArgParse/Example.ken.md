# ArgParse example — `forge`

This separate client package is the Milestone-C worked example. `forge` has two
subcommands, value options, flags, and positionals. Its parser and help are
thin applications of `ArgParse`; adding an option changes both behaviors from
the single specification value.

```ken
const forge_build_options : List OptionSpec =
  Cons
    OptionSpec
    (MkOptionSpec "verbose" (Some String "v") FlagOption "show build steps")
    (Cons
      OptionSpec
      (MkOptionSpec "output" (Some String "o") ValueOption "write raw output bytes")
      (Nil OptionSpec))

const forge_build_positionals : List PositionalSpec =
  Cons PositionalSpec (MkPositionalSpec "input" True) (Nil PositionalSpec)

const forge_build_spec : CommandSpec =
  MkCommandSpec "build" "compile one input" forge_build_options forge_build_positionals

const forge_inspect_options : List OptionSpec =
  Cons
    OptionSpec
    (MkOptionSpec "format" (None String) ValueOption "select the report format")
    (Cons
      OptionSpec
      (MkOptionSpec "quiet" (Some String "q") FlagOption "suppress headings")
      (Nil OptionSpec))

const forge_inspect_positionals : List PositionalSpec =
  Cons PositionalSpec (MkPositionalSpec "artifact" True) (Nil PositionalSpec)

const forge_inspect_spec : CommandSpec =
  MkCommandSpec "inspect" "inspect one artifact" forge_inspect_options forge_inspect_positionals

const forge_spec : ProgramSpec =
  MkProgramSpec
    "forge"
    "build and inspect artifacts"
    (Cons CommandSpec forge_build_spec (Cons CommandSpec forge_inspect_spec (Nil CommandSpec)))

fn forge_parse (arguments : List ArgBytes) : Validation (NonEmpty Diagnostic) ParsedCommand =
  argparse_run forge_spec arguments

const forge_help : Doc = program_help forge_spec

fn forge_diagnostic_doc (diagnostic : Diagnostic) : Doc = diagnostic_to_doc diagnostic
```

The caller constructs certified `ArgBytes` at the process boundary. Parsing
never converts their payloads to `String`; callers may explicitly choose
`Text.Codec` after a raw `ParsedOption` or `ParsedPositional` is returned.
