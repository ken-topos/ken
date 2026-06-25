# Concrete grammar

> Status: **DRAFT v0**. Proposal-level (OQ-syntax). An EBNF sketch of the
> surface against the brace form (layout, `31 §6`, desugars to this). Normative
> intent: the *productions that exist* (what can be written), not their exact
> spelling.

## 1. Compilation units and declarations

```
unit   ::= module_hdr? import* decl*
module_hdr ::= "module" ConId "{"  -- or a file-level implicit module
import ::= "import" ModPath ("as" ConId)? ("(" name ("," name)* ")")?
        |  "use" ModPath  -- bring names into scope unqualified

decl ::=
    "view" ident binder* (":" type)? effects? contract* "=" expr  -- function
  | "let"  ident (":" type)? "=" expr  -- value
  | "type" ConId tyvar* "=" type  -- alias / refinement
  | "record" ConId tyvar* "{" field ("," field)* "}" derive?  -- product
  | "data" ConId tyvar* "=" ctor ("|" ctor)* derive?  -- sum / inductive
  | "foreign" ident ":" type foreign_spec  -- FFI (38)
  | "space" ConId "{" decl* "}"  -- state region (36)
  | spec_decl  -- prove / law (20)
  | fixity_decl  -- infixl/r N op

binder  ::= "(" ident+ ":" type ")" | "{" ident+ ":" type "}"   -- {…} implicit
field   ::= ident ":" type
ctor    ::= ConId arg_types?                       -- e.g.  Cons a (List a)
arg_types ::= type+ | "{" field ("," field)* "}"   -- positional or named
effects ::= "visits" "[" ConId ("," ConId)* "]"    -- effect row (36)
contract::= "requires" expr | "ensures" expr       -- (20)
derive  ::= "derive" "(" ConId ("," ConId)* ")"    -- Eq, Show, … (33)
```

## 2. Types

```
type ::=
    "(" ident ":" type ")" "->" type  -- dependent function (Π)
  | type "->" type  -- non-dependent arrow
  | "(" ident ":" type ")" "×" type  -- dependent pair (Σ)
  | "{" ident ":" type "|" expr "}"  -- refinement (12 §5, 34)
  | ConId atype*  -- type application (List Int, Vec a n)
  | "Type" level?  -- a universe (12)
  | "forall" tyvar+ "." type  -- explicit polymorphism (usually implicit)
  | tyvar | atype
atype ::= ConId | tyvar | "(" type ")"
```

Universe levels are usually inferred (`12 §4`); `Type` means `Type ℓ` for an
inferred `ℓ`. Implicit arguments `{…}` are inserted by elaboration (`39`).

## 3. Expressions

```
expr ::=
    "λ" binder+ "." expr | "\\" binder+ "->" expr  -- lambda
  | expr expr  -- application (left assoc)
  | expr binop expr  -- operators (declared fixity)
  | "let" ident (":" type)? "=" expr "in" expr  -- local binding
  | "if" expr "then" expr "else" expr  -- = match on Bool
  | "match" expr ("," expr)* "{" arm+ "}"  -- pattern match (34)
  | expr "." ident | expr ".1" | expr ".2"  -- field / projection
  | "(" expr ("," expr)* ")"  -- tuple / pair / grouping
  | "{" field_assign ("," field_assign)* "}"  -- record literal
  | expr "@" expr  -- path application (15)
  | literal | ident | ConId | "(" operator ")"
  | "(" expr ":" type ")"  -- type ascription
arm  ::= pattern ("if" expr)? "=>" expr  -- guard optional
field_assign ::= ident "=" expr | ident  -- punning allowed
```

## 4. Patterns

```
pattern ::=
    ConId pattern*           -- constructor (uppercase ⇒ constructor, 31 §2)
  | ident                    -- variable binder (lowercase)
  | "_"                      -- wildcard
  | literal                  -- literal pattern
  | "(" pattern ("," pattern)* ")"  -- tuple/pair
  | "{" field_pat ("," field_pat)* "}"  -- record pattern
  | pattern "as" ident       -- as-pattern
  | pattern "|" pattern       -- or-pattern (same binders)
```

Pattern matching compiles to nested `elim_D` with exhaustiveness and
reachability checking (`34`, `39`).

## 5. Specifications

Spec forms (`../20-verification/21-spec-syntax.md`) are grammar too:

```
spec_decl ::= "prove" ident ":" type
            | "law" ConId "(" tyvar ")" "{" field ("," field)* "}"
contract is part of `view` (§1); refinements `{x:A|φ}` are types (§2).
```

## 6. Precedence and associativity (defaults)

`->` and `×` right-associative; application binds tightest; `@` (path app)
tight; user operators take declared fixity (`infixl`/`infixr`/`infix N`, default
`infixl 9`). `:` (ascription) loosest. The exact table is OQ-syntax; the
*existence* of declared fixity is not.

## 7. What WS-L must deliver here

A parser for the above producing a surface AST for elaboration (`39`), with the
layout rule (`31 §6`) feeding it. The grammar's *coverage* (dependent function
and pair types, refinements, `data`/`record`, `match`, contracts, FFI, effect
rows) is the normative part; token spelling is OQ. Conformance:
`../../conformance/surface/grammar/` — round-trip parse/print and coverage of
each production.
