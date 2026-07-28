# A seat can stop receiving deliveries entirely — clean composer, `Enter` is a no-op

**Measured 2026-07-28** on `runtime-leader`, which sat through **three** events
addressed to it, in a thread it was already in, and ran no turn for ~7 minutes
while its implementer waited on a hand-off only it could perform.

## ⛔ This is a THIRD shape, and the first two repairs do not fix it

| shape | what the composer holds | repair |
|---|---|---|
| stranded delivery | `[Pasted Content N chars]` blocks | bare `Enter` |
| freshly compacted seat | empty | a typed tmux prompt |
| **not receiving at all** | **placeholder text only** | **a typed tmux prompt** |

⛔ **A bare `Enter` is a no-op here** — there is nothing queued to release. Sending
it and seeing no change is **not** evidence the seat is fine; it is evidence
there was never anything in the composer. ⚠ Do not read a quiet pane after an
`Enter` as "delivered and considered."

## ⭐ The diagnostic — a FROZEN counter while events arrive

The seat renders normally: footer, model line, cwd. What discriminates it is the
**finished-turn counter not changing across two sweeps** while events land in a
thread it belongs to.

```
sweep 1   Worked for 1m 09s     <- events at 14:50:57, 14:52:28 land after this
sweep 2   Worked for 1m 09s     <- identical: it never ran
```

⇒ **Frozen counter + arriving events + clean composer = not receiving.** ⚠ Check
the composer *before* concluding it, or you will mistake this for the strand
shape and "fix" it with an `Enter` that does nothing.

## ✅ The repair

`tmux send-keys -l '<short prompt>'`, then a **separate** `Enter`. Name the event
ids it missed and the acts it owes; ⛔ do not restate their content — make it read
them, so its context comes from the channel and not from your paraphrase. The
seat began working within 9 seconds and posted its hand-off 90 seconds later.

## ⚠ It is silent from BOTH ends

The senders' events posted successfully and the seat looks merely quiet. ⛔ Nobody
gets an error. ⇒ The only party who can see it is whoever compares **event
arrival times against pane turn activity** — which, in this federation, is the
Steward's sweep.

⭐ **The blocked seat is what surfaces it.** Here the implementer said plainly *"I
am not taking the branch until Runtime Leader hands the turn to me by name"* — a
correct, well-behaved seat idling on an edge nobody was going to traverse. Treat
a well-formed "I am waiting on X" from a healthy seat as a **probe pointing at
X**, not as an all-clear.
