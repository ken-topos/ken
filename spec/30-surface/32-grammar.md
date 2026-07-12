# Concrete grammar

> Status: **DRAFT v0**. Proposal-level (OQ-syntax). An EBNF sketch of the
> surface against the brace form (layout, `31 §6`, desugars to this). Normative
> intent: the *productions that exist* (what can be written), not their exact
> spelling.

## 1. Compilation units and declarations

```
unit   ::= module_hdr? import* decl*
module_hdr ::= "module" ConId "{"  -- or a file-level implicit module
import ::= "import" ModPath ("as" ConId)?
                          ("(" import_item ("," import_item)* ")")?
import_item ::= name | name "as" rename
rename ::= name

decl ::=
    "const" ident binder* (":" type)? contract* constraint_clause? "=" expr  -- pure value (36 §1.6)
  | "fn"    ident binder* (":" type)? contract* constraint_clause? "=" expr  -- pure function
  | "proc"  ident binder* (":" type)? effects? contract* constraint_clause? "=" expr  -- effectful / imperative
  | "def" ConId tyvar* "=" type  -- definition: alias / refinement ("type" reserved)
  | "record" ConId tyvar* "{" field (";" field)* "}" derive?  -- product
  | "data" ConId tyvar* "=" simple_ctor ("|" simple_ctor)* derive?  -- simple sum sugar
  | "data" ConId data_param* ":" data_family data_block derive?  -- inductive family
  | "class" ConId binder* "{" class_field (";" class_field)* "}"  -- typeclass (33 §5, ADR 0008)
  | "instance" ConId atype* constraint_clause? "{" field_assign (";" field_assign)* "}"  -- instance (33 §5, §5.4)
  | "prop" ConId tyvar* binder* ":" type prop_block?  -- proposition family / claim shape
  | "lemma" ident binder* ":" type "=" expr  -- standalone checked proof theorem
  | "proof" ident "for" path binder* ":" type "=" expr  -- attached proof theorem
  | "foreign" ident ":" type foreign_spec  -- FFI (38)
  | "space" ConId "{" (cell | decl | becomes)* "}"  -- state region (36)
  | "policy" ConId "{" decl* "}"  -- policy module (65) [OQ-syntax]
  | spec_decl  -- prove / law (20)
  | fixity_decl  -- infixl/r N op

cell    ::= "mut" ident ":" type "=" expr           -- mutable space cell (36 §4)
becomes ::= ident "becomes" expr  -- space cell update (36 §4) [OQ-syntax]
constraint_clause ::= "where" constraint_bind ("," constraint_bind)*  -- shared def-path + instance (33 §5.4)
constraint_bind ::= constraint                        -- bare: auto-named `d<v>` (single tyvar); sole-constraint `d` alias
                  | "(" ident ":" constraint ")"      -- explicit named binder, e.g. `(da : DecEq a)`
constraint  ::= ConId atype+                          -- e.g.  DecEq a
binder  ::= "(" ident+ ":" type ")" | "{" ident+ ":" type "}"   -- {…} implicit
field   ::= ident ":" type
class_field ::= field_purity? ident ":" type
field_purity ::= "const" | "fn" | "proc"  -- checked class-field purity (33 §5.2, 39 §6.0)
data_param ::= binder
data_family ::= type                               -- index telescope ending in Type
data_block ::= "where" "{" data_ctor (";" data_ctor)* "}"
simple_ctor ::= ConId arg_types?                   -- e.g.  Cons a (List a)
data_ctor ::= simple_ctor
           | ConId ":" ctor_type                   -- dependent/GADT-like signature
ctor_type ::= ctor_arg "->" ctor_type | type       -- telescope ending in D params indices
ctor_arg ::= binder | type                         -- named/implicit or anonymous
arg_types ::= type+ | "{" field ("," field)* "}"   -- positional or named
effects ::= "visits" "[" row "]"                   -- effect row (36 §1), proc only
row     ::= ConId ("," ConId)*                      -- concrete row  [FS, Console]
          | ident                                   -- a row variable  [e]  (36 §1.5)
          | ConId ("," ConId)* "|" ident            -- open row  [FS | e]  (concrete head + poly tail)
contract::= "requires" expr | "ensures" expr       -- (20)
derive  ::= "derive" "(" ConId ("," ConId)* ")"    -- DecEq, Show, … (33)
path    ::= ident ("." ident)*                     -- qualified value/module path
prop_block ::= "where" "{" prop_intro (";" prop_intro)* "}"
prop_intro ::= ident ":" type
```

**Proof-claim declarations are ordinary checked terms.** `prop`, `lemma`, and
attached `proof` all elaborate to existing checked terms only. `prop` requires
an `Omega`-valued result and may carry an optional constructor-style
`where` block whose intro helpers elaborate to ordinary proof terms under the
family namespace. `lemma` is a checked theorem in the ordinary module
namespace. `proof` attaches a checked theorem to a resolved subject path, and
the canonical attached name is `subject::proof_name`. None of these forms adds
a new kernel declaration class, a trusted proof table, or ambient proof search.

**Definition keywords are purity-checked (`33 §1`, `36 §1.6`).** `const`/`fn`/
`proc` replace the retired `view`; the grammar admits `binder*` on all, but
the **bidirectional purity check** (`36 §1.6.2`) constrains which is legal:
`const` requires **zero explicit value parameters**, `fn` requires **≥1**, and
only `proc` may carry a `visits` row (or a row variable). A `const`/`fn`
with an effect, or the wrong arity, is a **hard error** — a grammar-accepted but
purity-rejected program, caught at elaboration (`36 §1.6.3`), not by the parser.

**Data declarations have a simple form and an explicit family form (`34 §2`).**
The old `data D a = C A | ...` form remains sugar for a non-indexed family
whose constructors target `D a`; it admits only `simple_ctor`, not
`C : ctor_type`. The explicit form is the normative spelling for dependent
constructors:

```ken
data Vec (A : Type) : Nat -> Type where {
  VNil  : Vec A 0;
  VCons : (n : Nat) -> A -> Vec A n -> Vec A (n+1)
}
```

In a constructor signature, the elaborator peels the leading `->`/dependent
binders as the constructor telescope and requires the final codomain to be an
instance of the declared family at the declared parameters and some index
expressions. Anonymous arrows (`A -> ...`) generate anonymous telescope binders;
`(x : A) -> ...` and `{x : A} -> ...` bind names that may be used by later
argument types and by the result indices. The named-record constructor shorthand
`C { f : A, g : B }` stays sugar for the simple default-result form; record-style
field labels inside an explicit dependent constructor signature are a later
surface refinement, not part of this production.

**Constraint `where` clauses are one shared production (`33 §5.4`).** The
definition path (`const`/`fn`/`proc`) and `instance` share `constraint_clause`
(landed: both routes parse through the one constraint parser). A constraint is
either **bare** — a single-type-variable `C v` auto-named `d<v>`, with the
reserved bare `d` as the sole-constraint alias — or an **explicit named binder**
`(name : C τ)`, required wherever no auto-name exists (compound/multi-argument
constraints, or a same-variable collision that would auto-name twice, which is
**rejected**); the naming contract is normatively `33 §5.4`. Comma is the unified
separator on both paths. Two spelling compatibilities are retained, not new
grammar: the def-path additionally accepts `;` as a separator (legacy, `37 §6`
L3b), and `instance` additionally tolerates one trailing `,` before `{`. The
clause scopes over the declaration's `requires`/`ensures`, its result refinement,
and its body, but not sibling declarations.

## 2. Types

```
type ::=
    "(" ident ":" type ")" "->" type  -- dependent function (Π)
  | type "->" type  -- non-dependent arrow
  | "(" ident ":" type ")" "×" type  -- dependent pair (Σ)
  | "{" ident ":" type "|" expr "}"  -- refinement (12 §5, 34)
  | ConId atype*  -- type application (List Int, Vec a n); also Lazy a, Wrapping[T]
  | type "@" label  -- IFC labeled type A @ ℓ (60-security/61 §3) [OQ-syntax]
  | "Type" level?  -- a universe (12)
  | "forall" tyvar+ "." type  -- explicit polymorphism (usually implicit)
  | tyvar | atype
atype ::= ConId | tyvar | "(" type ")"
label ::= expr | "ct"  -- a lattice label ℓ, or timing-sensitive ct (61 §3,§5a)
```

Universe levels are usually inferred (`12 §4`); `Type` means `Type ℓ` for an
inferred `ℓ`. Implicit arguments `{…}` are inserted by elaboration (`39`). Type
application is shown by juxtaposition (`ConId atype*`); the chapters also use
the bracketed spelling `F[T]` (e.g. `Wrapping[T]`, `35 §3`) — the same
construct, spelling `[OQ-syntax]`.

## 3. Expressions

```
expr ::=
    "λ" binder+ "." expr | "\\" binder+ "->" expr  -- lambda
  | expr expr  -- application (left assoc)
  | expr binop expr  -- operators (declared fixity)
  | "(" ident ":" type ")" "->" expr  -- dependent function type Π (a term; §2, 11 §1)
  | expr "->" expr  -- non-dependent function type (arrow); elaborates to kernel Pi
  | "let" ident (":" type)? "=" expr "in" expr  -- local binding
  | "if" expr "then" expr "else" expr  -- = match on Bool
  | "match" expr ("eqn:" ident)? ("," expr)* "{" arm+ "}"  -- pattern match (34); single-scrutinee eqn: modifier (34 §3.6)
  | expr "." ident | expr ".1" | expr ".2"  -- field / projection
  | path "::" ident  -- canonical attached-proof path
  | proof_ref  -- attached-proof selector atom
  | "(" expr ("," expr)* ")"  -- tuple / pair / grouping
  | "{" field_assign ("," field_assign)* "}"  -- record literal
  | "temporal" "{" expr "}"  -- temporal obligation → Temporal data (72) [OQ-syntax]
  | literal | ident | ConId | "(" operator ")"
  | "(" expr ":" type ")"  -- type ascription
arm  ::= pattern ("if" expr)? ("↦" | "|->") expr  -- guard optional
proof_ref ::= "proof" ident "for" path
field_assign ::= ident "=" expr | ident  -- punning allowed
```

`proof_ref` is a primary expression atom and therefore binds more tightly than
application or any infix operator. Its subject is exactly one `path`; subsequent
expressions are arguments to the selector result. Thus `proof p for s a b`
parses as `((proof p for s) a) b`, and `f proof p for s` parses as
`f (proof p for s)`. The bare atom and its grouped form `(proof p for s)`
produce the identical `Expr::EAttachedProofRef { subject, proof_name }` and
desugar to the same `subject::ident` global, where `ident` is stored as
`proof_name`; parentheses are optional grouping.

Several decided constructs need **no special expression syntax** — they are
ordinary terms over stdlib/library values (spelling still `OQ-syntax`):

- **`declassify`** is a function (`61 §4`: `declassify : Cap → A@ℓ → A@ℓ'`), so
  a call is ordinary application `declassify cap x`.
- **Named-instance passing** is ordinary application — an instance is a
  first-class value, so you pass it as an argument (`sortBy byLength xs`, `33
  §5`, ADR 0008); there is **no** dedicated keyword.
- **`@ct`** is an IFC label, carried on the *type* via `A @ label` (§2), not an
  expression prefix.
- **Wrapping arithmetic** (`+%`/`-%`/`*%`, `35 §3`) are ordinary `binop`s.

**Arrow types in expression position.** Because **types are terms**
(`../10-kernel/11 §1`), a function type is a perfectly good *term* — of type
`Type ℓ` — and may stand **wherever an expression is expected**, not only in a
type annotation. The two `expr`-position arrow forms above (`(x : A) -> B` and
`A -> B`) are the **same construct** as the `type`-position arrows of §2 and
**elaborate identically to the kernel `Pi`** (`39`); no kernel variant is added.
This closes the gap where an arrow type could be *written* in an annotation but
not, e.g., passed as an argument, bound by `let`, or returned (`const f : Type =
Int -> Int`). Two disambiguations, both by existing lookahead:

- The lambda form `\ binder+ -> expr` (and `λ binder+ . expr`) is unambiguous
  with the arrow **type**: a leading `\`/`λ` commits to the lambda, so a `->`
  reached without one is always the function-type constructor.
- `(x : A) -> …` vs. the ascription `(x : A)`: after parsing the parenthesized
  `(ident : type)`, a following `->` selects the dependent arrow; its absence
  leaves an ascription — the same one-token lookahead §2 already uses in type
  position.

The V0 minimal slice (§8) deliberately keeps `->` **type-position only**; that
restriction is V0-local and is lifted here for the full surface.

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
            | "law" ConId "(" tyvar ")" "{" field (";" field)* "}"
contract is part of `fn`/`proc` (§1); refinements `{x:A|φ}` are types (§2).
```

## 6. Precedence and associativity (defaults)

Application binds tightest; `@` (label / annotation prefix) tight; `->` and `×`
are right-associative and looser than the arithmetic operators; user operators
take declared fixity (`infixl`/`infixr`/`infix N`, default `infixl 9`); `:`
(ascription) loosest.

**Arithmetic precedence — resolved (conventional default).** The core arithmetic
operators bind by the conventional levels, so `a + b * c` parses as `a + (b *
c)` and `a - b - c` as `(a - b) - c`:

| Level | Operators | Assoc | Notes |
|---|---|---|---|
| `7` | `*` (and any future `/`) | `infixl` | multiplicative — binds **tighter** |
| `6` | `+`, `-` | `infixl` | additive |

The wrapping variants (`+%`/`-%`/`*%`, `35 §3`) share their base operator's
level (`*%` at 7; `+%`/`-%` at 6). `->` sits **below** all arithmetic (a
function type built from arithmetic operands, e.g. `Vec (n + 1) -> Vec n`,
groups the operands first) and **above** ascription.

This pins the split VAL2 needs (multiplicative tighter than additive, both
left-associative) as Ken's default. The remaining spellings and the levels of
*non-arithmetic* user operators stay `OQ-syntax`; the *existence* of declared
fixity, and this arithmetic ordering, are not.

## 7. What WS-L must deliver here

A parser for the above producing a surface AST for elaboration (`39`), with the
layout rule (`31 §6`) feeding it. The grammar's *coverage* (dependent function
and pair types, refinements, `data`/`record`, `match`, contracts, FFI, effect
rows) is the normative part; token spelling is OQ. Conformance:
`../../conformance/surface/grammar/` — round-trip parse/print and coverage of
each production.

## 8. V0 minimal grammar (the G1 slice)

V0 (the minimal elaborator, `../30-surface/39-elaboration.md §5`) parses
**only** the subset below — the brace form, against the V0 lexer (`31 §8`). It
is a
strict sub-grammar of §1–§3: no `data`/`record`/`module`, no `match`/`if`, no
operators or fixity, no implicit binders `{…}`, no effects/contracts, no
literals. Every V0 argument is explicit.

```
decl   ::= "fn"    ident binder+ (":" type)? "=" expr   -- pure function (≥1 param)
         | "const" ident          (":" type)? "=" expr   -- pure value (0 params)
binder ::= "(" ident+ ":" type ")"                     -- explicit telescope
type   ::= "(" ident ":" type ")" "->" type            -- dependent Π
         | type "->" type                               -- non-dependent arrow
         | "Type" level?                                -- universe (level optional)
         | ConId                                        -- base type by name
expr   ::= ("\" | "λ") ident+ "." expr                  -- lambda
         | expr expr                                    -- application (left assoc)
         | "let" ident (":" type)? "=" expr "in" expr   -- local binding
         | "(" expr ":" type ")"                        -- type ascription
         | ident                                        -- variable
         | ConId                                        -- base type used as a term
         | "Type" level?                                -- universe used as a term
level  ::= NAT                                          -- 0, 1, 2, …
```

The `ConId` and `Type` atoms in `expr` reflect that **types are terms**
(`../10-kernel/11 §1`): a base type and a universe may stand in expression
position (e.g. `let x : Type = Type in x`). In V0, `->`/Π is type-position only,
so a bare function type is not a V0 expression — a deliberate V0-local
restriction the full surface lifts (§3, arrow types in expression position).

`->` is right-associative; application binds tightest; ascription `:` is loosest
(§6). A type-position `ident` (lowercase) is a bound type variable (e.g. the `A`
in `(A : Type) -> A`); a `ConId` (uppercase) is a base type resolved against the
kernel environment (`31 §2`, `39 §5.2`). This is genuinely minimal: enough to
write

```
fn id (A : Type) (x : A) : A = x
fn konst (A : Type) (B : Type) (x : A) (y : B) : A = x   -- 'const' is now a keyword
fn apply (A : Type) (B : Type) (f : (x : A) -> B) (x : A) : B = f x
```

V0 is pure-only (no effects, `§8`), so it needs `fn`/`const` and never `proc`.
The K combinator is renamed `konst` because `const` is now a reserved keyword
(`31 §4`) — the one keyword-collision the migration (D4) must rewrite; no `.ken`
in the corpus names a def `const`/`fn`/`proc`. The landed V0 parser still
spells `view`/`let` until D4 migrates it (perishable; the surface above is the
target).

and have each parse → elaborate (`39 §5`) → kernel-check (`18 §3`). Everything
else in §1–§5 is out of V0 and owned by a later WP (the full surface, Team
Language; specifications, V1).
