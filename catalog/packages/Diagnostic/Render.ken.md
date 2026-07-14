# Diagnostic.Render

`Diagnostic.Render` is the presentation layer shared by command-line and
configuration clients. It depends on the origin-neutral diagnostic carrier and
the document algebra without adding rendering knowledge to either package.

## 1. Rendering

The stable diagnostic code remains the message payload. Origins contribute a
short family label; clients can inspect the structured origin itself when they
need exact machine-readable indexes and ranges.

```ken
fn diagnostic_code_string (code : DiagnosticCode) : String =
  match code {
    MkDiagnosticCode value ↦ value
  }

fn diagnostic_origin_label (origin : Origin) : String =
  match origin {
    SourceOrigin source range ↦ "source";
    ArgumentOrigin index range ↦ "argument";
    EnvironmentOrigin variable ↦ "environment";
    ConfigKeyOrigin path ↦ "configuration"
  }

fn diagnostic_to_doc (diagnostic : Diagnostic) : Doc =
  Group
    (Concat
      (text_string (diagnostic_origin_label (diagnostic_origin diagnostic)))
      (Concat
        (text_string ":")
        (Concat Line (text_string (diagnostic_code_string (diagnostic_code diagnostic))))))
```

## 2. Trust and derivation

The renderer is a transparent structural projection into `Pretty.Doc`. It adds
no error carrier, primitive, postulate, `Axiom`, or trusted-base entry.
