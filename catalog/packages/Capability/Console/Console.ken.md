# `Console` — ordinary text-output helpers

This package keeps text policy above the byte-exact Console ABI. The built-in
`write` operation accepts bytes without encoding or newline behavior; these
four helpers choose UTF-8 encoding, select stdout or stderr, and make the
line-ending choice explicit. They are ordinary kernel-checked Ken definitions
with zero `trusted_base()` delta.

```ken
proc print (text : String) : IO (Result IOError Unit) visits [Console] =
  write Stdout (bytes_encode text)

proc printLine (text : String) : IO (Result IOError Unit) visits [Console] =
  write
    Stdout
    (bytes_concat
      (bytes_encode text)
      (bytes_encode (list_char_to_string (Cons Char (10 : Int) (Nil Char)))))

proc eprint (text : String) : IO (Result IOError Unit) visits [Console] =
  write Stderr (bytes_encode text)

proc eprintLine (text : String) : IO (Result IOError Unit) visits [Console] =
  write
    Stderr
    (bytes_concat
      (bytes_encode text)
      (bytes_encode (list_char_to_string (Cons Char (10 : Int) (Nil Char)))))
```

The helpers preserve `write`'s total `Result IOError Unit`; broken pipes remain
named values visible to callers rather than host exceptions.
