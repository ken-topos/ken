# teams/runtime — Runtime-team lessons

Loaded by the Runtime ring — `runtime-leader`, `runtime-implementer`,
`runtime-qa` — in addition to `fleet`, `build/`, and the function scope
(`build/leaders` · `build/implementers` · `build/qa`).

⚠ **Runtime seating is the standing exception to the tier table**
(`agent/MODELS.md`): **`runtime-implementer` is T1**, `runtime-leader` is T2 —
the implementation is the hard part on this team, not the coordination. **Never
infer a seat's tier from its role here**, and remember that a Runtime WP
legitimately running for **hours** is normal, not a stall.

Domain lessons about the Cranelift backend: the static-transition plane, the
lowering path, function-unit boundaries, and the `RT-NATIVE-FNSPLIT` chain.

| Lesson | One-line |
|---|---|
| [closure-body-is-a-return-successor-not-a-unit-head](closure-body-is-a-return-successor-not-a-unit-head.md) | `TransitionKind::ClosureBody` is a body's return successor, never a unit head — the ruled seeds are `plan.entries` ∪ every `EdgeKind::StaticBody` **target** |

**Adding one:** a lesson belongs here when every Runtime seat must apply it and
it is specific to this backend's structures. A lesson about *how to author a
pin* belongs in `agent/playbooks/tools/pin-a-property.md`; one about review flow
or coordination belongs in `fleet/` or `build/`.
