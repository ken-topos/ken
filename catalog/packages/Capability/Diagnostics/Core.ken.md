# Capability.Diagnostics.Core

`Capability.Diagnostics.Core` is the origin-neutral floor for located diagnostics. It
records a stable machine-readable code and where the problem came from, while
deliberately leaving rendering, width, and layout to later packages.

## 1. Origins and ranges

`ByteRange` is a neutral half-open range. `Origin` is closed over the four
committed location families: source artifacts, command-line arguments,
environment variables, and configuration key paths.

```ken
data SourceId = MkSourceId Nat

data ByteRange = MkByteRange Nat Nat

fn byte_range_start (range : ByteRange) : Nat =
  match range {
    MkByteRange start end ↦ start
  }

fn byte_range_end (range : ByteRange) : Nat =
  match range {
    MkByteRange start end ↦ end
  }

data Origin =
  SourceOrigin SourceId ByteRange
  | ArgumentOrigin Nat ByteRange
  | EnvironmentOrigin String
  | ConfigKeyOrigin (List String)

fn environment_origin (variable : String) : Origin = EnvironmentOrigin variable

fn config_key_origin (path : List String) : Origin = ConfigKeyOrigin path

fn origin_source_id (origin : Origin) : Option SourceId =
  match origin {
    SourceOrigin source range ↦ Some SourceId source;
    ArgumentOrigin index range ↦ None SourceId;
    EnvironmentOrigin variable ↦ None SourceId;
    ConfigKeyOrigin path ↦ None SourceId
  }

fn origin_argument_index (origin : Origin) : Option Nat =
  match origin {
    SourceOrigin source range ↦ None Nat;
    ArgumentOrigin index range ↦ Some Nat index;
    EnvironmentOrigin variable ↦ None Nat;
    ConfigKeyOrigin path ↦ None Nat
  }

fn origin_byte_range (origin : Origin) : Option ByteRange =
  match origin {
    SourceOrigin source range ↦ Some ByteRange range;
    ArgumentOrigin index range ↦ Some ByteRange range;
    EnvironmentOrigin variable ↦ None ByteRange;
    ConfigKeyOrigin path ↦ None ByteRange
  }

fn origin_range_start (origin : Origin) : Option Nat =
  match origin {
    SourceOrigin source range ↦ Some Nat (byte_range_start range);
    ArgumentOrigin index range ↦ Some Nat (byte_range_start range);
    EnvironmentOrigin variable ↦ None Nat;
    ConfigKeyOrigin path ↦ None Nat
  }

fn origin_range_end (origin : Origin) : Option Nat =
  match origin {
    SourceOrigin source range ↦ Some Nat (byte_range_end range);
    ArgumentOrigin index range ↦ Some Nat (byte_range_end range);
    EnvironmentOrigin variable ↦ None Nat;
    ConfigKeyOrigin path ↦ None Nat
  }
```

## 2. Structured diagnostics

A `DiagnosticCode` is a stable code, not a rendered message. Clients own the
codes they introduce and inject them into this neutral carrier.

```ken
data DiagnosticCode = MkDiagnosticCode String

data Diagnostic = MkDiagnostic Origin DiagnosticCode

fn diagnostic_origin (diagnostic : Diagnostic) : Origin =
  match diagnostic {
    MkDiagnostic origin code ↦ origin
  }

fn diagnostic_code (diagnostic : Diagnostic) : DiagnosticCode =
  match diagnostic {
    MkDiagnostic origin code ↦ code
  }
```

## 3. Checkable validity

Validity remains a plain predicate. Source and argument origins require ordered
ranges, environment names are accepted as opaque names, and configuration key
paths must contain at least one segment.

```ken
fn diagnostic_nat_leq (a : Nat) (b : Nat) : Bool =
  match a {
    Zero ↦ True;
    Suc a2 ↦
      match b {
        Zero ↦ False;
        Suc b2 ↦ diagnostic_nat_leq a2 b2
      }
  }

fn ValidByteRange (range : ByteRange) : Prop =
  Equal Bool (diagnostic_nat_leq (byte_range_start range) (byte_range_end range)) True

fn ValidConfigKeyPath (path : List String) : Prop =
  match path {
    Nil ↦ Bottom;
    Cons key rest ↦ Top
  }

fn ValidOrigin (origin : Origin) : Prop =
  match origin {
    SourceOrigin source range ↦ ValidByteRange range;
    ArgumentOrigin index range ↦ ValidByteRange range;
    EnvironmentOrigin variable ↦ Top;
    ConfigKeyOrigin path ↦ ValidConfigKeyPath path
  }

fn ValidDiagnostic (diagnostic : Diagnostic) : Prop = ValidOrigin (diagnostic_origin diagnostic)
```

## 4. Trust and derivation

All declarations are transparent kernel-checked terms over landed data. The
package defines no renderer, axiom, primitive, postulate, or opaque constant.
