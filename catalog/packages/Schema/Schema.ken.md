# Schema

`Schema` is the client-independent description shared by command-line and
environment/config decoders. It owns only field identity, presence,
value-shape, documentation metadata, and the two traversals both clients run.
Acquisition, provenance, validation policy, and diagnostic rendering remain in
the clients.

## 1. Description vocabulary

```ken
data SchemaPresence = SchemaRequired | SchemaOptional

data SchemaValueShape = SchemaFlag | SchemaBytes

data SchemaField = MkSchemaField String SchemaPresence SchemaValueShape String

data Schema = MkSchema String String (List SchemaField)

fn schema_field_name (field : SchemaField) : String =
  match field {
    MkSchemaField name presence shape documentation ↦ name
  }

fn schema_field_presence (field : SchemaField) : SchemaPresence =
  match field {
    MkSchemaField name presence shape documentation ↦ presence
  }

fn schema_field_shape (field : SchemaField) : SchemaValueShape =
  match field {
    MkSchemaField name presence shape documentation ↦ shape
  }

fn schema_field_documentation (field : SchemaField) : String =
  match field {
    MkSchemaField name presence shape documentation ↦ documentation
  }

fn schema_name (schema : Schema) : String =
  match schema {
    MkSchema name documentation fields ↦ name
  }

fn schema_documentation (schema : Schema) : String =
  match schema {
    MkSchema name documentation fields ↦ documentation
  }

fn schema_fields (schema : Schema) : List SchemaField =
  match schema {
    MkSchema name documentation fields ↦ fields
  }
```

## 2. Generic accumulating validation

The checker is supplied by each client, so source-specific policy and origin
types stay local. The traversal nevertheless owns the accumulation shape and
visits every field.

```ken
data SchemaIssue origin = MkSchemaIssue origin String

data SchemaFieldCheck origin value =
  SchemaFieldAccepted value
  | SchemaFieldRejected (SchemaIssue origin)

const SchemaValidation (origin : Type) (value : Type) : Type =
  Validation (NonEmpty (SchemaIssue origin)) (List value)

fn schema_field_accept
      (origin : Type) (value : Type) (accepted : value)
    : SchemaFieldCheck origin value =
  SchemaFieldAccepted origin value accepted

fn schema_field_reject
      (origin : Type) (value : Type) (issue : SchemaIssue origin)
    : SchemaFieldCheck origin value =
  SchemaFieldRejected origin value issue

fn schema_check_presence
      (origin : Type)
      (value : Type)
      (optional_value : value)
      (required_issue : SchemaIssue origin)
      (presence : SchemaPresence)
    : SchemaFieldCheck origin value =
  match presence {
    SchemaRequired ↦ schema_field_reject origin value required_issue;
    SchemaOptional ↦ schema_field_accept origin value optional_value
  }

fn schema_issue_origin (origin : Type) (issue : SchemaIssue origin) : origin =
  match issue {
    MkSchemaIssue at code ↦ at
  }

fn schema_issue_code (origin : Type) (issue : SchemaIssue origin) : String =
  match issue {
    MkSchemaIssue at code ↦ code
  }

fn schema_validation_cons
      (origin : Type)
      (value : Type)
      (head : SchemaFieldCheck origin value)
      (tail : SchemaValidation origin value)
    : SchemaValidation origin value =
  match head {
    SchemaFieldAccepted accepted ↦
      match tail {
        Valid rest ↦ Valid
          (NonEmpty (SchemaIssue origin))
          (List value)
          (Cons value accepted rest);
        Invalid issues ↦ Invalid (NonEmpty (SchemaIssue origin)) (List value) issues
      };
    SchemaFieldRejected issue ↦
      match tail {
        Valid rest ↦ Invalid
          (NonEmpty (SchemaIssue origin))
          (List value)
          (NonEmptyCons (SchemaIssue origin) issue (Nil (SchemaIssue origin)));
        Invalid issues ↦ Invalid
          (NonEmpty (SchemaIssue origin))
          (List value)
          (nonempty_append
            (SchemaIssue origin)
            (NonEmptyCons (SchemaIssue origin) issue (Nil (SchemaIssue origin)))
            issues)
      }
  }

fn schema_validate_fields
      (origin : Type)
      (value : Type)
      (inspect : SchemaField → SchemaFieldCheck origin value)
      (fields : List SchemaField)
    : SchemaValidation origin value =
  match fields {
    Nil ↦ Valid (NonEmpty (SchemaIssue origin)) (List value) (Nil value);
    Cons field rest ↦ schema_validation_cons
      origin
      value
      (inspect field)
      (schema_validate_fields origin value inspect rest)
  }

fn schema_validate
      (origin : Type)
      (value : Type)
      (inspect : SchemaField → SchemaFieldCheck origin value)
      (schema : Schema)
    : SchemaValidation origin value =
  schema_validate_fields origin value inspect (schema_fields schema)
```

## 3. Shared help traversal

```ken
fn schema_presence_chars (presence : SchemaPresence) : List Char =
  match presence {
    SchemaRequired ↦ string_to_list_char "required";
    SchemaOptional ↦ string_to_list_char "optional"
  }

fn schema_shape_chars (shape : SchemaValueShape) : List Char =
  match shape {
    SchemaFlag ↦ Nil Char;
    SchemaBytes ↦ string_to_list_char " <value>"
  }

fn schema_field_label_chars (field : SchemaField) : List Char =
  list_append
    Char
    (string_to_list_char (schema_field_name field))
    (schema_shape_chars (schema_field_shape field))

fn schema_field_detail_chars (field : SchemaField) : List Char =
  list_append
    Char
    (string_to_list_char " [")
    (list_append
      Char
      (schema_presence_chars (schema_field_presence field))
      (string_to_list_char "]  "))

fn schema_field_help_chars (field : SchemaField) : List Char =
  list_append
    Char
    (string_to_list_char "  ")
    (list_append
      Char
      (schema_field_label_chars field)
      (list_append
        Char
        (schema_field_detail_chars field)
        (list_append
          Char
          (string_to_list_char (schema_field_documentation field))
          (string_to_list_char "\n"))))

fn schema_fields_help_chars (fields : List SchemaField) : List Char =
  match fields {
    Nil ↦ Nil Char;
    Cons field rest ↦ list_append
      Char
      (schema_field_help_chars field)
      (schema_fields_help_chars rest)
  }

fn schema_help (schema : Schema) : Doc =
  Text
    (list_append
      Char
      (string_to_list_char (schema_name schema))
      (list_append
        Char
        (string_to_list_char " ")
        (list_append
          Char
          (string_to_list_char (schema_documentation schema))
          (list_append
            Char
            (string_to_list_char "\nFields:\n")
            (schema_fields_help_chars (schema_fields schema))))))
```

## 4. Trust and layering

`Schema` mentions neither client and adds no primitive, postulate, `Axiom`, or
trusted-base entry. Its result and issue carriers are parameterized over client
origin and value types.
