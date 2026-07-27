# Filesystem authority manifests

A filesystem procedure states both parts of its authority surface in its
signature: the authority-indexed capability is an explicit value, and the
effect row names the world interaction. The authority-polymorphic reader below
uses the landed `read_bytes` producer without minting or wrapping a capability.

```ken
proc capability_read
      (a : Auth) (cap : Cap a) (path : Bytes)
    : FS a (Result FileError Bytes)
    visits [FS] =
  read_bytes a cap path

proc full_authority_write
      (cap : Cap AFull) (path : Bytes)
      (policy : CreatePolicy) (contents : Bytes)
    : FS AFull (Result FileError Unit)
    visits [FS] =
  write_file AFull cap path policy contents
```

`full_authority_write` makes the authority index load-bearing: `Cap AFull` and
`Cap APartial` are distinct opaque types. The package's acceptance test supplies
each in turn through otherwise identical callers; only the `AFull` caller
elaborates.

## Deliberately absent management operations

`attenuate`, `revoke`, and `strengthen` are `UnboundName` by design, not omitted
examples. Authority cannot be altered or amplified by Ken source
(`spec/60-security/62-authority.md §3.2`), and raw revocation and attenuation
operate over non-Ken-visible grant identities in the trusted host/runner
(`spec/60-security/62-authority.md §4`). The typed filesystem API preserves
that boundary (`spec/30-surface/38-ffi-io.md §1.3.1`). This entry therefore
accepts a capability only as an input and defines no constructor, producer,
wrapper, or management binding for `Cap`.

## Checked surface and host complement

The catalog can exhibit and check:

- an explicit `Cap a` parameter beside the declared `[FS]` effect row;
- authority-index separation between `Cap AFull` and `Cap APartial`;
- rejection of a filesystem program whose boundary supplies no FS capability;
- acceptance of the identical program when its boundary supplies that
  capability.

The complement is host/runner-side and cannot honestly appear as checked
catalog Ken: capability minting; raw attenuation and its child-lineage update;
transitive revocation over non-Ken-visible grant identities; admission
linearization; OS-operation denial and settlement; and audit-record emission.
Those properties require runtime identities or host actions that Ken source
cannot name. Their absence here preserves the authority boundary rather than
leaving the exemplar incomplete.
