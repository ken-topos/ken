# `RT-CONTSRC-PRODUCER-LOCAL` `D8o` — ambient body-authority census

A complete production read/write census of the two ambient fields on
`Lowering`, taken at `3a63fe64`, before the `D8o` repair. Test-only sites are
excluded; every production site is listed, and every reader is classified by
what it does when the field is absent.

## `defining_emission_owner`

*"The emission owner of the context currently being defined."*

### Writers

| site | value | body kind |
|---|---|---|
| `core.rs:874` | `None` | construction |
| `units.rs:2100` | `Some(Specialization(context.enclosing))` | generated-context body |
| `units.rs:3172` | `Some(Predeclared(unit.function))` | ordinary unit body |

**There is no writer for a specialization body.** `define_continuation_bodies`
sets neither field, so throughout it both hold whatever the previously defined
body left behind.

### Readers

| site | what it does when absent | class |
|---|---|---|
| `core.rs:6788` `claim_and_call_continuation` | refuses | fail-closed |
| `core.rs:9766` carried-invocation retarget | refuses | fail-closed |
| `core.rs:8398` `D8i` construction guard | compares against it | compare |
| `core.rs:8426` `D8j` claim guard | compares against it | compare |
| `core.rs:9373` `composed_recursive_argument_binding` | `Ok(None)` — keeps the ordinary route | **decline** |
| `mod.rs:10054` synthesized constructor occurrence | `occurrence: None` | **decline** |
| `mod.rs:10489` synthesized dynamic reconciliation | `Ok(())` | **decline** |
| `mod.rs:10560` synthesized aggregate occurrence | `Ok(None)` | **decline** |

## `defining_unit`

*"The exact unit currently being defined, so `D3`'s owner check compares
against a fact supplied independently of the token."*

### Writers

| site | value | body kind |
|---|---|---|
| `core.rs:873` | `None` | construction |
| `units.rs:2099` | `Some(context.raw_owner)` | generated-context body |
| `units.rs:3170` | `Some(unit.function)` | ordinary unit body |

Again **no writer for a specialization body**.

### Readers

| site | what it does when absent | class |
|---|---|---|
| `core.rs:6777` `claim_and_call_continuation` | refuses | fail-closed |
| `core.rs:6352` `d5a_trace` | prints it | test-only |

## What the census shows

Two things, and the second is the one that matters.

**The residue is real.** Both fields survive the body that set them. A
specialization body therefore reads the ordinary unit body or generated context
that happened to be defined before it.

**The four DECLINE sites are why this is not loud.** A fail-closed reader turns
residue into a refusal, which is survivable — someone sees it. A *declining*
reader treats a stale owner as a live one and simply answers differently: the
composed binding site looks a target up under the wrong owner, or skips; the
three synthesized-aggregate sites attribute or drop an occurrence. None of them
refuses, and none of them is wrong in a way a green suite can show.

⇒ The repair is a body-lifetime binding, not extra validation at the readers.
Validation at a decline site cannot distinguish "no body is being defined" from
"a body is being defined and left the wrong value here" — only the writer knows.
