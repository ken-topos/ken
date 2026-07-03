# Read a file line by line

Read a text file and process it line by line.

Reference: <https://rosettacode.org/wiki/Read_a_file_line_by_line>

## Status

**Blocked — genuine capability gap, see `KNOWN-GAP.md`.** `read_bytes` has
no real runtime reduction in `ken-interp` (only `print_line`/
`string_to_list_char`/`list_char_to_string` are wired); any Ken program
touching file I/O is stuck. This is exactly the axis this example was
chosen to probe (`[FS]` effect, streaming-under-totality) — the gap it
surfaced (no FS wiring at all) is more fundamental than "streaming is
awkward."

Routed to language-leader / Steward; the fix (a real `read_bytes`
reduction in `ken-interp`) is its own properly-gated capability WP, not
patched here.
