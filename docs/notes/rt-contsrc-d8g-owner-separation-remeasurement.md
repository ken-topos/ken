# `D8g` — does the emission owner separate anything? Re-measured after `D8h`–`D8p`

**Answer: no. The emission owner separates no current population.** The
prohibition on building an owner positive stands, and `D8g` proceeds without
one.

This is outcome (1) of the two the release named. It is a measurement, not an
argument, and it is recorded here because a measurement that lives only in a
thread is not a durable deliverable.

- **Measured at** branch tip `04c40d4c`, the accepted `D8f` object.
- **No production edit.** The probe that produced the table was removed and the
  worktree verified byte-clean against `04c40d4c`.

## The question, stated precisely

`D8a`'s selector is five fields:

```
(emission_owner, producer_construct_origin, continuation_origin,
 producer_alternative, recursive_position)
```

The premise under test is that `emission_owner` is load-bearing — that some real
population contains two entries agreeing on the other four fields and
distinguished only by owner.

So the measurement is an **injectivity** one. For each planner population,
compare:

- `sel+owner` — the number of distinct full five-field selectors;
- `sel-owner` — the number of distinct four-field selectors with the owner
  projected away.

They are equal exactly when no two entries differ only by owner. `sel-owner <
sel+owner` is the only shape that retires the prohibition, and it did not occur.

## The population

Derived through the ordinary production path: the static transition planner run
over every witness in reach, with no second owner manufactured and no synthetic
program. Both populations the selector keys are measured — continuation calls
and continuation units.

| program | calls | `sel+owner` | `sel-owner` | units | `sel+owner` | `sel-owner` | distinct owners |
|---|---|---|---|---|---|---|---|
| `d8f` lawful | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `d8f` two-call | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `d8n` | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `d8m` two-occurrence | 2 | 2 | 2 | 2 | 2 | 2 | 1 |
| `d8e` | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `px8tr` | 2 | 2 | 2 | 2 | 2 | 2 | **2** |

`sel+owner == sel-owner` in **every** row, in both populations.

## The nuance that makes this worth writing down

**The owner varies; it does not separate.** `px8tr` carries two genuinely
distinct owners — `Predeclared(3)` and `Specialization(0)` — so the field is not
constant, and a census that only asked "do owners differ anywhere?" would have
answered yes and reached the opposite conclusion. But the two entries carrying
those owners are *already* distinguished by their construct origin, continuation
origin, alternative and recursive position. Projecting the owner away loses
nothing.

⇒ "This field takes more than one value" and "this field distinguishes two
otherwise-identical entries" are different claims, and only the second would
retire the prohibition.

## What would change the answer

A population containing two entries that agree on **all four** of construct
origin, continuation origin, producer alternative and recursive position, and
are emitted by **different owners**. Nothing in reach after `D8h`–`D8p`
constructs one.

That is the shape to look for if this is ever re-asked — not a program with two
owners, which already exists and is not sufficient.

## Scope

The two `d8m` two-occurrence entries and the two `px8tr` entries are same-owner
and different-owner pairs respectively, so the table is not vacuous in either
direction: it contains a program where the owner is constant across two entries
and one where it is not, and the projection is injective in both.
