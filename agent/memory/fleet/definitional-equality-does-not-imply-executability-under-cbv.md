---
scope: fleet
audience: (see scope README) — anyone building a lawful class instance that must
  both PROVE and RUN; anyone whose evaluation test returns Unknown on a term the
  kernel happily accepts
source: SUB-1b implementer §10 retro (evt_6w9yp0yxcxcfa), 2026-07-14
---

# Definitional equality does not imply executability under CBV

**SUB-1b's `DecEq Bytes` had to do two things: PROVE (sound/complete laws) and RUN
(decide raw invalid-UTF-8 bytes at the interpreter).** The obvious expression did
the first and silently failed the second.

```ken
list_deceq_eq  …          -- goes through the DICTIONARY   → interpreter returns Unknown
list_eq UInt8 uint8_deceq_eq  -- goes through the RUNTIME FIELD → executes
```

**The two are DEFINITIONALLY EQUAL. The kernel cannot tell them apart. The
interpreter absolutely can.**

## Why

**A lawful class dictionary carries its proofs as fields.** `DecEq a` is
`{ eq; sound; complete }` — and `sound`/`complete` are *proof terms*.

**Ken is call-by-value. Forcing the dictionary forces its fields — including the
proofs.** A proof term is not something the strict interpreter can evaluate to a
value, so the whole application goes **`Unknown`**.

**Reaching for the runtime field directly (`eq`) never forces the proof fields at
all** — so it computes. **Same meaning to the kernel; different fate under CBV.**

## ★ The tell, and why it is nasty

**Your evaluation test returns `Unknown` on a term that TYPE-CHECKS PERFECTLY.**
Nothing is unsound; nothing is even wrong. The kernel is content. **You will look
for the bug in your proof and it is not there** — the proof is fine, and its
*presence* is the problem.

**And `Unknown` is a legal verdict** ([[a-deferral-is-honest-a-deferral-that-reads-as-delivery-is-not]]),
so a test that merely asserts "no error" **passes**. Only a test that asserts the
**specific decided value** on **concrete input** catches it.

## The rule

**When a lawful instance must also EXECUTE, route the executable path through the
runtime field, not the dictionary** — and **verify it by evaluating on concrete
values**, never by type-checking alone.

**⇒ Write the AC as "decides `True` on these bytes and `False` on those,"
NEVER as "elaborates."** SUB-1b's AC4 (equal / last-byte-differs / empty / prefix
/ invalid-UTF-8 `0xff`) is the right shape: **non-vacuous, concrete, and it would
have gone `Unknown` under the dictionary route.**

Sibling of [[cbv-eliminator-method-laziness]] (the same CBV strictness biting a
different construct) and [[carrier-canonicity-axis-for-lawful-class-laws]].
**The general shape: a proof-carrying value is a fine THING and a bad
COMPUTATION — keep the proofs where the kernel reads them and out of the path the
interpreter walks.**
