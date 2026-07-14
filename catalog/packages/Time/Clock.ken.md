# Time.Clock

`wall_now` reads the ambient process wall clock as a structural `Instant`.
The `Int` payload is nanoseconds relative to the Unix epoch, so applications
can inspect and persist the value without an opaque carrier or conversion
primitive.

A wall clock can move in either direction when the host adjusts civil time.
This package therefore declares no ordering or monotonicity law. Programs that
need that guarantee require the separate session-shaped monotonic-clock design;
they must not assume it for `wall_now`.

```ken
fn instant_nanoseconds (instant : Instant) : Int =
  match instant {
    MkInstant nanoseconds ↦ nanoseconds
  }

fn replace_instant_nanoseconds (instant : Instant) (nanoseconds : Int) : Instant =
  match instant {
    MkInstant _old ↦ MkInstant nanoseconds
  }
```
