# System.Resource

`System.Resource` is the checked bracket over the runtime's generation-checked
filesystem resource table. Resource handles are ordinary copyable Ken values:
Ken does not make them affine. Liveness is runtime-enforced and Ward-checked.
An escaped copy is legal, but after its bracket settles every later use returns
`Closed`; insufficient authority returns `RightNotHeld`. The bracket acquires
before its delayed body and settles on normal return, returned error, and a
controlled runtime trap. Trap-primary/cleanup-secondary ordering is currently
exercised by a private caller-controlled runtime fixture; Ken has no
checked-source controlled-trap producer yet, so public checked-Ken reachability
of that face is deferred. The guarantee excludes external process destruction,
abort, fatal signal, and machine failure.

`withResource` is the sole public acquisition route. Its body is a delayed
function, so acquisition happens before the body and `release_if_live`
settlement happens afterward. `release` is deliberately non-idempotent; an
early release invalidates every copy, while the bracket's private finalizer
treats the resulting `Closed` as already settled rather than closing the OS
resource twice.

```ken
fn resource_body_success (e : Type) (a : Type) (value : a) : ResourceBodyResult e a =
  ResourceBodyOk e a value

fn resource_body_failure (e : Type) (a : Type) (error : e) : ResourceBodyResult e a =
  ResourceBodyErr e a error

fn resource_bracket_succeeded
      (e : Type) (a : Type) (outcome : ResourceBracketResult e a)
    : Bool =
  match outcome {
    ResourceBracketOk value ↦ True;
    ResourceBracketBodyError error ↦ False;
    ResourceBracketReleaseError error ↦ False;
    ResourceBracketBodyAndReleaseError body_error release_error ↦ False
  }
```
