# ArgParse

`ArgParse` specializes the landed cursor, decoder, diagnostic, validation, and
document packages to explicit command-line specifications. Argument bytes are
never decoded: names are matched one byte at a time and option values remain
the original `Bytes` values.

## 1. Explicit specifications and results

The v1 surface is intentionally explicit. A command owns its option,
positional, and subcommand descriptions; that same value drives parsing and
help generation.

```ken
data OptionMode = FlagOption | ValueOption

data OptionSpec = MkOptionSpec String (Option String) OptionMode String

data PositionalSpec = MkPositionalSpec String Bool

data CommandSpec = MkCommandSpec String String (List OptionSpec) (List PositionalSpec)

data ProgramSpec = MkProgramSpec String String (List CommandSpec)

data ParsedArgument =
  ParsedFlag String
  | ParsedOption String Bytes
  | ParsedPositional String Bytes

data ParsedCommand = MkParsedCommand String (List ParsedArgument)

fn option_name (spec : OptionSpec) : String =
  match spec {
    MkOptionSpec name short mode description ↦ name
  }

fn option_short_name (spec : OptionSpec) : Option String =
  match spec {
    MkOptionSpec name short mode description ↦ short
  }

fn option_mode (spec : OptionSpec) : OptionMode =
  match spec {
    MkOptionSpec name short mode description ↦ mode
  }

fn option_description (spec : OptionSpec) : String =
  match spec {
    MkOptionSpec name short mode description ↦ description
  }

fn positional_name (spec : PositionalSpec) : String =
  match spec {
    MkPositionalSpec name required ↦ name
  }

fn positional_required (spec : PositionalSpec) : Bool =
  match spec {
    MkPositionalSpec name required ↦ required
  }

fn command_name (spec : CommandSpec) : String =
  match spec {
    MkCommandSpec name description options positionals ↦ name
  }

fn command_description (spec : CommandSpec) : String =
  match spec {
    MkCommandSpec name description options positionals ↦ description
  }

fn command_options (spec : CommandSpec) : List OptionSpec =
  match spec {
    MkCommandSpec name description options positionals ↦ options
  }

fn command_positionals (spec : CommandSpec) : List PositionalSpec =
  match spec {
    MkCommandSpec name description options positionals ↦ positionals
  }

fn program_name (spec : ProgramSpec) : String =
  match spec {
    MkProgramSpec name description commands ↦ name
  }

fn program_description (spec : ProgramSpec) : String =
  match spec {
    MkProgramSpec name description commands ↦ description
  }

fn program_commands (spec : ProgramSpec) : List CommandSpec =
  match spec {
    MkProgramSpec name description commands ↦ commands
  }
```

## 2. Decoder-backed byte matching

Expected names come from the explicit spec. The actual side stays `ArgBytes`:
the decoder walks `arg_cursor_ops`, `bytes_at` supplies each byte, and equality
is `uint8_to_int` followed by `eq_int`.

```ken
fn argparse_byte_matches_char (actual : UInt8) (expected : Char) : Bool =
  eq_int (uint8_to_int actual) (charToInt expected)

fn argparse_name_decoder (expected : List Char) : Decoder ArgCursor ArgLocation Bool =
  match expected {
    Nil ↦ decoder_pure ArgCursor ArgLocation Bool True;
    Cons first rest ↦ decoder_bind
      ArgCursor
      ArgLocation
      UInt8
      Bool
      (decoder_satisfy
        ArgCursor
        UInt8
        ArgLocation
        arg_cursor_ops
        (λactual. argparse_byte_matches_char actual first))
      (λignored. argparse_name_decoder rest)
  }

fn argparse_single_cursor (argument : ArgBytes) : ArgCursor =
  arg_cursor_start (Cons ArgBytes argument (Nil ArgBytes))

fn argparse_matches_chars (argument : ArgBytes) (expected : List Char) : Bool =
  match argparse_name_decoder expected (argparse_single_cursor argument) {
    DecoderFailed err ↦ False;
    Decoded matched rest ↦
      match cursor_remaining ArgCursor UInt8 ArgLocation arg_cursor_ops rest {
        Zero ↦ True;
        Suc remaining ↦ False
      }
  }

fn argparse_has_prefix_chars (argument : ArgBytes) (expected : List Char) : Bool =
  match argparse_name_decoder expected (argparse_single_cursor argument) {
    DecoderFailed err ↦ False;
    Decoded matched rest ↦ True
  }

fn argparse_long_chars (name : String) : List Char =
  list_append Char (string_to_list_char "--") (string_to_list_char name)

fn argparse_short_chars (name : String) : List Char =
  list_append Char (string_to_list_char "-") (string_to_list_char name)

fn argparse_option_matches (argument : ArgBytes) (spec : OptionSpec) : Bool =
  match argparse_matches_chars argument (argparse_long_chars (option_name spec)) {
    True ↦ True;
    False ↦
      match option_short_name spec {
        None ↦ False;
        Some short ↦ argparse_matches_chars argument (argparse_short_chars short)
      }
  }

fn argparse_find_option (argument : ArgBytes) (specs : List OptionSpec) : Option OptionSpec =
  match specs {
    Nil ↦ None OptionSpec;
    Cons spec rest ↦
      match argparse_option_matches argument spec {
        True ↦ Some OptionSpec spec;
        False ↦ argparse_find_option argument rest
      }
  }

fn argparse_find_command (argument : ArgBytes) (specs : List CommandSpec) : Option CommandSpec =
  match specs {
    Nil ↦ None CommandSpec;
    Cons spec rest ↦
      match argparse_matches_chars argument (string_to_list_char (command_name spec)) {
        True ↦ Some CommandSpec spec;
        False ↦ argparse_find_command argument rest
      }
  }
```

## 3. Located diagnostics and accumulating validation

Every token check is independent. `validation_ap` combines its result with the
recursive tail using `NonEmpty Diagnostic`'s lawful semigroup, so two bad
tokens remain two diagnostics in encounter order.

```ken
fn argparse_diagnostic (index : Nat) (start : Nat) (end : Nat) (code : String) : Diagnostic =
  MkDiagnostic (ArgumentOrigin index (MkByteRange start end)) (MkDiagnosticCode code)

fn argparse_error
      (a : Type) (index : Nat) (start : Nat) (end : Nat) (code : String)
    : Validation (NonEmpty Diagnostic) a =
  Invalid
    (NonEmpty Diagnostic)
    a
    (NonEmptyCons Diagnostic (argparse_diagnostic index start end code) (Nil Diagnostic))

fn argparse_valid_argument
      (value : ParsedArgument)
    : Validation (NonEmpty Diagnostic) ParsedArgument =
  Valid (NonEmpty Diagnostic) ParsedArgument value

fn argparse_cons_validations
      (head : Validation (NonEmpty Diagnostic) ParsedArgument)
      (tail : Validation (NonEmpty Diagnostic) (List ParsedArgument))
    : Validation (NonEmpty Diagnostic) (List ParsedArgument) =
  validation_ap
    (NonEmpty Diagnostic)
    (Semigroup_instance_NonEmpty Diagnostic)
    (List ParsedArgument)
    (List ParsedArgument)
    (validation_map
      (NonEmpty Diagnostic)
      ParsedArgument
      (List ParsedArgument → List ParsedArgument)
      (Cons ParsedArgument)
      head)
    tail

fn argparse_missing_positionals
      (positionals : List PositionalSpec) (index : Nat)
    : Validation (NonEmpty Diagnostic) (List ParsedArgument) =
  match positionals {
    Nil ↦ Valid (NonEmpty Diagnostic) (List ParsedArgument) (Nil ParsedArgument);
    Cons spec rest ↦
      match positional_required spec {
        False ↦ argparse_missing_positionals rest index;
        True ↦ argparse_cons_validations
          (argparse_error ParsedArgument index Zero Zero "missing-positional")
          (argparse_missing_positionals rest index)
      }
  }

fn argparse_parse_tokens
      (options : List OptionSpec)
      (positionals : List PositionalSpec)
      (arguments : List ArgBytes)
      (index : Nat)
    : Validation (NonEmpty Diagnostic) (List ParsedArgument) =
  match arguments {
    Nil ↦ argparse_missing_positionals positionals index;
    Cons argument rest ↦
      match argparse_find_option argument options {
        Some spec ↦
          match option_mode spec {
            FlagOption ↦ argparse_cons_validations
              (argparse_valid_argument (ParsedFlag (option_name spec)))
              (argparse_parse_tokens options positionals rest (Suc index));
            ValueOption ↦
              match rest {
                Nil ↦ argparse_cons_validations
                  (argparse_error
                    ParsedArgument
                    index
                    Zero
                    (arg_length argument)
                    "missing-option-value")
                  (argparse_missing_positionals positionals (Suc index));
                Cons value more ↦ argparse_cons_validations
                  (argparse_valid_argument (ParsedOption (option_name spec) (arg_bytes value)))
                  (argparse_parse_tokens options positionals more (Suc (Suc index)))
              };
          };
        None ↦
          match argparse_has_prefix_chars argument (string_to_list_char "--") {
            True ↦ argparse_cons_validations
              (argparse_error
                ParsedArgument
                index
                (Suc (Suc Zero))
                (arg_length argument)
                "unknown-option")
              (argparse_parse_tokens options positionals rest (Suc index));
            False ↦
              match positionals {
                Nil ↦ argparse_cons_validations
                  (argparse_error
                    ParsedArgument
                    index
                    Zero
                    (arg_length argument)
                    "unexpected-positional")
                  (argparse_parse_tokens options positionals rest (Suc index));
                Cons positional more ↦ argparse_cons_validations
                  (argparse_valid_argument
                    (ParsedPositional (positional_name positional) (arg_bytes argument)))
                  (argparse_parse_tokens options more rest (Suc index))
              }
          }
      }
  }

fn argparse_parsed_command
      (name : String) (parsed : Validation (NonEmpty Diagnostic) (List ParsedArgument))
    : Validation (NonEmpty Diagnostic) ParsedCommand =
  (validation_map
    (NonEmpty Diagnostic)
    (List ParsedArgument)
    ParsedCommand
    (MkParsedCommand name)
    parsed)

fn argparse_run
      (specification : ProgramSpec) (arguments : List ArgBytes)
    : Validation (NonEmpty Diagnostic) ParsedCommand =
  match arguments {
    Nil ↦ argparse_error ParsedCommand Zero Zero Zero "missing-subcommand";
    Cons subcommand rest ↦
      match argparse_find_command subcommand (program_commands specification) {
        None ↦ argparse_error
          ParsedCommand
          Zero
          Zero
          (arg_length subcommand)
          "unknown-subcommand";
        Some spec ↦ argparse_parsed_command
          (command_name spec)
          (argparse_parse_tokens
            (command_options spec)
            (command_positionals spec)
            rest
            (Suc Zero))
      }
  }
```

## 4. Help derived from the spec

Help is a pure fold from `CommandSpec` to `Doc`. Names, descriptions, modes,
positionals, and subcommands all come from that one value.

```ken
fn argparse_option_value_chars (spec : OptionSpec) : List Char =
  match option_mode spec {
    FlagOption ↦ Nil Char;
    ValueOption ↦ string_to_list_char " <value>"
  }

fn argparse_option_chars (spec : OptionSpec) : List Char =
  list_append
    Char
    (string_to_list_char "  --")
    (list_append
      Char
      (string_to_list_char (option_name spec))
      (list_append
        Char
        (argparse_option_value_chars spec)
        (list_append
          Char
          (string_to_list_char "  ")
          (list_append
            Char
            (string_to_list_char (option_description spec))
            (string_to_list_char "\n")))))

fn argparse_options_chars (specs : List OptionSpec) : List Char =
  match specs {
    Nil ↦ Nil Char;
    Cons spec rest ↦ list_append Char (argparse_option_chars spec) (argparse_options_chars rest)
  }

fn argparse_positional_chars (spec : PositionalSpec) : List Char =
  match positional_required spec {
    True ↦ list_append
      Char
      (string_to_list_char "  <")
      (list_append
        Char
        (string_to_list_char (positional_name spec))
        (string_to_list_char ">\n"));
    False ↦ list_append
      Char
      (string_to_list_char "  [")
      (list_append
        Char
        (string_to_list_char (positional_name spec))
        (string_to_list_char "]\n"))
  }

fn argparse_positionals_chars (specs : List PositionalSpec) : List Char =
  match specs {
    Nil ↦ Nil Char;
    Cons spec rest ↦ list_append
      Char
      (argparse_positional_chars spec)
      (argparse_positionals_chars rest)
  }

fn argparse_subcommand_chars (spec : CommandSpec) : List Char =
  list_append
    Char
    (string_to_list_char "  ")
    (list_append
      Char
      (string_to_list_char (command_name spec))
      (list_append
        Char
        (string_to_list_char "  ")
        (list_append
          Char
          (string_to_list_char (command_description spec))
          (string_to_list_char "\n"))))

fn argparse_subcommands_chars (specs : List CommandSpec) : List Char =
  match specs {
    Nil ↦ Nil Char;
    Cons spec rest ↦ list_append
      Char
      (argparse_subcommand_chars spec)
      (argparse_subcommands_chars rest)
  }

fn command_help (spec : CommandSpec) : Doc =
  Text
    (list_append
      Char
      (string_to_list_char (command_name spec))
      (list_append
        Char
        (string_to_list_char " ")
        (list_append
          Char
          (string_to_list_char (command_description spec))
          (list_append
            Char
            (string_to_list_char "\nOptions:\n")
            (list_append
              Char
              (argparse_options_chars (command_options spec))
              (list_append
                Char
                (string_to_list_char "Positionals:\n")
                (argparse_positionals_chars (command_positionals spec))))))))

fn program_help (spec : ProgramSpec) : Doc =
  Text
    (list_append
      Char
      (string_to_list_char (program_name spec))
      (list_append
        Char
        (string_to_list_char " ")
        (list_append
          Char
          (string_to_list_char (program_description spec))
          (list_append
            Char
            (string_to_list_char "\nSubcommands:\n")
            (argparse_subcommands_chars (program_commands spec))))))
```

## 5. Trust and derivation

All declarations are transparent structural terms over CC1–CC6a. The package
adds no parser carrier, error carrier, renderer, cached-length carrier, byte
equality primitive, postulate, `Axiom`, or trusted-base entry. Raw argv values
cross the parser only as `Bytes` obtained from the existing `ArgBytes` carrier.
