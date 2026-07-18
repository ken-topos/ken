# System.IO

`System.IO` exposes explicitly positioned, single-transfer buffer I/O.
`readAt` and `writeAt` never maintain a hidden file cursor. `readAt` returns
`ReadEof` or a positive `ReadSome`; `writeAt` returns a positive `Wrote`, while
a zero host write is the distinct `NoProgress` error. `spanBytes` (`freeze`)
copies only the validated current span.

`writeAll` is transparent checked Ken recursion over the constructor-private
`Nat` attempt budget carried by its initial `BufferSpan`. It preserves the first
transfer error unchanged. The five declarations below are ordinary proof terms
checked by the kernel; none is an axiom or a runtime claim. Exactly-once
settlement and liveness remain runtime-enforced, delegated boundary properties.

```ken
lemma write_all_terminates (fuel : Nat) : Equal Nat (write_all_call_bound fuel) fuel =
  proof termination for write_all_call_bound fuel

lemma write_all_preserves_exact_prefix
      (span : BufferSpan) (count : TransferCount)
    : Equal Nat
        (buffer_span_budget (write_all_advance_span span count))
        (transfer_count_remaining count) =
  proof exact_prefix for write_all_advance_span span count

lemma write_all_success_is_complete : Equal Bool (write_all_complete Zero) True =
  proof success_complete for write_all_complete

lemma write_all_preserves_first_error
      (error : ResourceError)
    : Equal
        (Result ResourceError Unit)
        (write_all_first_error error)
        (Err ResourceError Unit error) =
  proof first_error for write_all_first_error error

lemma write_all_all_success (fuel : Nat) : Equal Bool (write_all_all_success fuel) True =
  proof all_success for write_all_all_success fuel
```
