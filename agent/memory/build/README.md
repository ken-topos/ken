# build — cross-team build-practice lessons

Loaded by **every** build-team function (leader, implementer, QA) across Kernel,
Verify, Language, Runtime, Ergo, and Foundation, in addition to `fleet` and the
team/role scopes. Practice that applies regardless of which build function you
are: test-authoring discipline, tooling gotchas, and verification habits that
cut across leader/implementer/QA. A lesson specific to one function belongs in
`build/leaders`, `build/qa`, or `build/implementers` instead; a lesson specific
to one team belongs in `teams/<team>`.

| Lesson | One-line |
|---|---|
| [assert-specific-error-variant-not-is-err](assert-specific-error-variant-not-is-err.md) | Assert the specific error variant, not a bare `is_err()` |
| [bundled-changes-need-per-mechanism-isolation-flip](bundled-changes-need-per-mechanism-isolation-flip.md) | Bundled changes need a per-mechanism isolation-flip |
| [eval-only-harness-cant-detect-phantom-arg-staleness](eval-only-harness-cant-detect-phantom-arg-staleness.md) | An eval-only harness can't detect phantom-arg staleness |
| [general-fix-can-conflate-similar-shaped-different-cases](general-fix-can-conflate-similar-shaped-different-cases.md) | A general fix can conflate similar-shaped but different cases |
| [green-vs-green-does-not-confirm-a-fix](green-vs-green-does-not-confirm-a-fix.md) | Green-vs-green does not confirm a fix |
| [ken-cargo-build-lock-wedge](ken-cargo-build-lock-wedge.md) | The ken-cargo + sccache build-lock wedge |
| [named-floor-must-be-grepped-not-assumed](named-floor-must-be-grepped-not-assumed.md) | A named floor must be grepped, not assumed |
| [probe-recursion-depth-before-writing-the-real-test](probe-recursion-depth-before-writing-the-real-test.md) | Probe recursion depth before writing the real test |
| [timeout-does-not-kill-grandchild-cargo-test](timeout-does-not-kill-grandchild-cargo-test.md) | `timeout` doesn't kill a grandchild cargo test binary |
