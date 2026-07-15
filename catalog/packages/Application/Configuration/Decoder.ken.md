# Application.Configuration.Decoder

`Application.Configuration.Decoder` specializes the shared schema to raw environment and config
key/value pairs. Acquisition remains outside the package: process environment
pairs come from `Capability.Process.Environment`, while config pairs are handed in by the
caller. Keys and values remain plain `Bytes` throughout.

## 1. Local provenance and values

```ken
data EnvConfigOrigin = EnvVariableOrigin String | ConfigEntryOrigin (List String)

fn env_config_origin_to_origin (origin : EnvConfigOrigin) : Origin =
  match origin {
    EnvVariableOrigin name ↦ EnvironmentOrigin name;
    ConfigEntryOrigin path ↦ ConfigKeyOrigin path
  }

fn env_config_issue_diagnostic (issue : SchemaIssue EnvConfigOrigin) : Diagnostic =
  MkDiagnostic
    (env_config_origin_to_origin (schema_issue_origin EnvConfigOrigin issue))
    (MkDiagnosticCode (schema_issue_code EnvConfigOrigin issue))
```

## 2. Plain-Bytes lookup

The landed lawful byte dictionary supplies the key decision. No cached length,
string decoding, or client-local equality is involved.

```ken
fn env_config_entry_key (entry : Prod Bytes Bytes) : Bytes =
  match entry {
    MkProd key value ↦ key
  }

fn env_config_entry_value (entry : Prod Bytes Bytes) : Bytes =
  match entry {
    MkProd key value ↦ value
  }

fn env_config_lookup_choice
      (key : Bytes) (entry : Prod Bytes Bytes) (fallback : Option Bytes)
    : Option Bytes =
  match bytes_deceq_eq (env_config_entry_key entry) key {
    True ↦ Some Bytes (env_config_entry_value entry);
    False ↦ fallback
  }

fn env_config_lookup (key : Bytes) (entries : List (Prod Bytes Bytes)) : Option Bytes =
  match entries {
    Nil ↦ None Bytes;
    Cons entry rest ↦ env_config_lookup_choice key entry (env_config_lookup key rest)
  }

fn env_config_missing_field
      (origin : EnvConfigOrigin) (field : SchemaField)
    : SchemaFieldCheck EnvConfigOrigin Bool =
  schema_check_presence
    EnvConfigOrigin
    Bool
    True
    (MkSchemaIssue EnvConfigOrigin origin "missing-required-field")
    (schema_field_presence field)

fn env_config_field_check
      (origin : EnvConfigOrigin) (entries : List (Prod Bytes Bytes)) (field : SchemaField)
    : SchemaFieldCheck EnvConfigOrigin Bool =
  match env_config_lookup (bytes_encode (schema_field_name field)) entries {
    Some value ↦ schema_field_accept EnvConfigOrigin Bool True;
    None ↦ env_config_missing_field origin field
  }

fn environment_field_origin (name : String) : EnvConfigOrigin = EnvVariableOrigin name

fn config_field_origin (name : String) : EnvConfigOrigin =
  ConfigEntryOrigin (Cons String name (Nil String))

fn environment_field_check
      (entries : List (Prod Bytes Bytes)) (field : SchemaField)
    : SchemaFieldCheck EnvConfigOrigin Bool =
  env_config_field_check (environment_field_origin (schema_field_name field)) entries field

fn config_field_check
      (entries : List (Prod Bytes Bytes)) (field : SchemaField)
    : SchemaFieldCheck EnvConfigOrigin Bool =
  env_config_field_check (config_field_origin (schema_field_name field)) entries field

fn env_config_values
      (fields : List SchemaField) (entries : List (Prod Bytes Bytes))
    : List Bytes =
  match fields {
    Nil ↦ Nil Bytes;
    Cons field rest ↦
      match env_config_lookup (bytes_encode (schema_field_name field)) entries {
        Some value ↦ Cons Bytes value (env_config_values rest entries);
        None ↦ Cons Bytes (list_to_bytes (Nil UInt8)) (env_config_values rest entries)
      }
  }
```

## 3. Schema-driven decoding

Both entry points run the same shared accumulating traversal. Their origin
construction remains local, and only this specialization injects issues into
`Diagnostic`.

```ken
fn env_config_validation
      (fields : List SchemaField)
      (entries : List (Prod Bytes Bytes))
      (checked : SchemaValidation EnvConfigOrigin Bool)
    : Validation (NonEmpty Diagnostic) (List Bytes) =
  match checked {
    Valid values ↦ Valid (NonEmpty Diagnostic) (List Bytes) (env_config_values fields entries);
    Invalid issues ↦
      Invalid
        (NonEmpty Diagnostic)
        (List Bytes)
        (nonempty_map
          (SchemaIssue EnvConfigOrigin)
          Diagnostic
          env_config_issue_diagnostic
          issues)
  }

fn decode_environment_entries
      (schema : Schema) (entries : List (Prod Bytes Bytes))
    : Validation (NonEmpty Diagnostic) (List Bytes) =
  env_config_validation
    (schema_fields schema)
    entries
    (schema_validate EnvConfigOrigin Bool (environment_field_check entries) schema)

fn decode_process_environment
      (schema : Schema) (input : ProcessInput)
    : Validation (NonEmpty Diagnostic) (List Bytes) =
  decode_environment_entries schema (process_environment input)

fn decode_config_entries
      (schema : Schema) (entries : List (Prod Bytes Bytes))
    : Validation (NonEmpty Diagnostic) (List Bytes) =
  env_config_validation
    (schema_fields schema)
    entries
    (schema_validate EnvConfigOrigin Bool (config_field_check entries) schema)

fn env_config_help (schema : Schema) : Doc = schema_help schema
```

## 4. Trust and boundaries

The decoder consumes `process_environment`, shared `Schema`, `Validation`,
`Diagnostic`, and the landed lawful `DecEq Bytes`. It adds no parser, renderer,
location carrier, cached length, primitive, postulate, `Axiom`, or trusted-base
entry. Raw values—including invalid UTF-8—are returned without a `String` hop.
