# System.Buffer

`System.Buffer` presents the immutable checked view of a runtime-owned bounded
buffer. `BufferWindow` is a request descriptor. `BufferSpan` and
`TransferCount` are constructor-private: only the checked transfer boundary can
mint a live span or a positive, request-bounded count. The public projections
expose scalar positions, lengths, structural attempt budgets, and immutable
copies through `spanBytes`/`freeze`; no pointer, slice, file descriptor, mutable
reference, or backing region crosses the boundary.

Fixed capacity, the single-current-window discipline, and invalidation on
settlement are runtime-enforced and externally tested invariants. They are not
stated as kernel proofs. The checked structural budget and count witnesses used
by `writeAll`, however, are ordinary Ken data and their laws are kernel-checked.

```ken
fn buffer_window (start : Int) (length : Int) : BufferWindow = MkBufferWindow start length

fn span_length (span : BufferSpan) : Int = buffer_span_length span

fn span_attempt_budget (span : BufferSpan) : Nat = buffer_span_budget span

fn transferred (count : TransferCount) : Int = transfer_count_int count

lemma transfer_is_positive (count : TransferCount) : transfer_count_positive_prop count =
  transfer_count_positive count

lemma transfer_is_bounded
      (count : TransferCount)
    : Equal Nat
        (transfer_count_request_budget count)
        (buffer_nat_add (transfer_count_nat count) (transfer_count_remaining count)) =
  proof bounded for transfer_count_request_budget count
```
