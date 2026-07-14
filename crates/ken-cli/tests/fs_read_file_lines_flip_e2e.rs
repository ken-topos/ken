//! fs-read-file-lines-flip D5 — end-to-end acceptance driving the REAL
//! `ken` CLI binary (subprocess) against a real ABI-shaped program. No test in
//! this file hand-constructs a cap value at any `EvalVal`/`apply` site — the
//! cap originates inside `ken-cli`'s `ProgramCaps APartial` mint-and-bind path.
//!
//! The declared `ProgramCaps APartial` carries `Cap APartial`; a missing-file arm
//! surfaces a total `Err(NotFound)`, never a panic.
//!
//! **effect-composition update (AC6 asterisk retirement):** `main` now
//! genuinely composes `[FS]` and `[Console]` in ONE `bind`-sequenced,
//! tagged `HostIO` coproduct — the
//! program itself prints each line via `[Console]` (`printLines`). Also no
//! test in this file hand-constructs a `Coproduct`/
//! `InL`/`InR` value (AC7's producer-grep, `effect-composition-conformance.md`
//! §2) — `inject_l`/`inject_r` are elaborated from the surface `.ken` source
//! above. On a missing file, the application reports the exact `IOError`
//! through Console and returns `Failure 1`; the runner only maps `ExitCode`.

use std::path::PathBuf;
use std::process::Command;

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The D1 `lines` helper + an ABI-shaped `main` reading `path` through the
/// declaration-minted `ProgramCaps APartial`.
fn program_src(path: &str) -> String {
    format!(
        r#"program capabilities FS APartial
fn isNewline (c : Char) : Bool = eq_int (charToInt c) 10

fn consFirst (c : Char) (acc : List (List Char)) : List (List Char) =
  match acc {{
    Nil           |-> Cons (List Char) (Cons Char c (Nil Char)) (Nil (List Char)) ;
    Cons seg rest |-> Cons (List Char) (Cons Char c seg) rest
  }}

fn splitNL (xs : List Char) : List (List Char) =
  match xs {{
    Nil       |-> Cons (List Char) (Nil Char) (Nil (List Char)) ;
    Cons c cs |->
      match isNewline c {{
        True  |-> Cons (List Char) (Nil Char) (splitNL cs) ;
        False |-> consFirst c (splitNL cs)
      }}
  }}

fn dropTrailingEmpty (segs : List (List Char)) : List (List Char) =
  match segs {{
    Nil |-> Nil (List Char) ;
    Cons seg rest |->
      match rest {{
        Nil |->
          match seg {{
            Nil      |-> Nil (List Char) ;
            Cons _ _ |-> Cons (List Char) seg (Nil (List Char))
          }} ;
        Cons _ _ |-> Cons (List Char) seg (dropTrailingEmpty rest)
      }}
  }}

fn mapListCharToString (segs : List (List Char)) : List String =
  match segs {{
    Nil           |-> Nil String ;
    Cons seg rest |-> Cons String (list_char_to_string seg) (mapListCharToString rest)
  }}

fn lines (s : String) : List String =
  mapListCharToString (dropTrailingEmpty (splitNL (string_to_list_char s)))

const Compose (r : Type) : Type =
  HostIO {auth} r

proc printLines (xs : List String) : Compose (Result IOError Unit) visits [Console] =
  match xs {{
    Nil |->
      Ret (Coproduct (FSOp {auth}) AmbientOp)
          (resp_coproduct (FSOp {auth}) AmbientOp (fs_resp {auth}) ambient_resp)
          (Result IOError Unit) (Ok IOError Unit MkUnit) ;
    Cons x xs' |->
      bind (Coproduct (FSOp {auth}) AmbientOp)
           (resp_coproduct (FSOp {auth}) AmbientOp (fs_resp {auth}) ambient_resp)
           Unit (Result IOError Unit)
        (host_console {auth} Unit (print_line x))
        (\_ . printLines xs')
  }}

proc app (cap : Cap {auth}) : Compose (Result IOError Unit) visits [FS, Console] =
  bind (Coproduct (FSOp {auth}) AmbientOp)
       (resp_coproduct (FSOp {auth}) AmbientOp (fs_resp {auth}) ambient_resp)
       (Result FileError Bytes) (Result IOError Unit)
    (inject_l (FSOp {auth}) AmbientOp (fs_resp {auth}) ambient_resp (Result FileError Bytes)
      (read_bytes {auth} cap (bytes_encode "{path}")))
    (\r .
      match r {{
        Err e    |-> match e {{
          MkFileError operation path kind |->
            Ret (Coproduct (FSOp {auth}) AmbientOp)
                (resp_coproduct (FSOp {auth}) AmbientOp (fs_resp {auth}) ambient_resp)
                (Result IOError Unit) (Err IOError Unit kind)
        }} ;
        Ok bytes |->
          match bytes_decode bytes {{
            Err _ |-> Ret (Coproduct (FSOp {auth}) AmbientOp)
                         (resp_coproduct (FSOp {auth}) AmbientOp (fs_resp {auth}) ambient_resp)
                         (Result IOError Unit) (Err IOError Unit (Other 0)) ;
            Ok text |-> printLines (lines text)
          }}
      }})

proc main (_input : ProcessInput) (caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [FS, Console] =
  match caps {{
    MkProgramCaps cap |->
      bind (Coproduct (FSOp APartial) AmbientOp)
           (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
           (Result IOError Unit) ExitCode
        (app cap)
        (\r .
          match r {{
            Err e |->
              match e {{
                NotFound |-> host_program_then APartial (print_line "NotFound") (Failure 1) ;
                PermissionDenied |-> host_program_then APartial (print_line "PermissionDenied") (Failure 1) ;
                CapabilityDenied |-> host_program_then APartial (print_line "CapabilityDenied") (Failure 1) ;
                BrokenPipe |-> host_program_then APartial (print_line "BrokenPipe") (Failure 1) ;
                Interrupted |-> host_program_then APartial (print_line "Interrupted") (Failure 1) ;
                AlreadyExists |-> host_program_then APartial (print_line "AlreadyExists") (Failure 1) ;
                InvalidInput |-> host_program_then APartial (print_line "InvalidInput") (Failure 1) ;
                IsDirectory |-> host_program_then APartial (print_line "IsDirectory") (Failure 1) ;
                NotDirectory |-> host_program_then APartial (print_line "NotDirectory") (Failure 1) ;
                NotEmpty |-> host_program_then APartial (print_line "NotEmpty") (Failure 1) ;
                Unsupported |-> host_program_then APartial (print_line "Unsupported") (Failure 1) ;
                Other errno |-> host_program_then APartial (print_line "Other") (Failure 1)
              }} ;
            Ok _ |-> host_exit APartial Success
          }})
  }}
"#,
        auth = "APartial",
        path = path,
    )
}

/// Write `src` to a fresh file under a per-test tmp dir and run it through
/// the real `ken` binary (subprocess), from the workspace root (the fixture
/// path is workspace-root-relative). Returns `(stdout, stderr, success)`.
fn run(name: &str, src: &str) -> (String, String, bool) {
    let tmp_dir = std::env::temp_dir().join("ken-fs-flip-e2e");
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let path = tmp_dir.join(format!("{name}.ken"));
    std::fs::write(&path, src).expect("write program");

    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&path)
        .current_dir(workspace_root())
        .output()
        .unwrap_or_else(|e| panic!("{name}: failed to spawn `ken run`: {e}"));

    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

/// The declared `ProgramCaps APartial` is sufficient for `ReadFile`, so the driver
/// reaches the fixture and the application prints its lines.
#[test]
fn m_suff_apartial_reads_fixture() {
    let src = program_src("conformance/fs/fixtures/three-lines.txt");
    let (stdout, stderr, success) = run("m_suff", &src);
    assert!(success, "M-suff must succeed; stderr: {stderr}");
    assert_eq!(
        stdout, "alpha\nbeta\ngamma\n",
        "M-suff must print the exact fixture lines"
    );
}

/// AC6 (totality/fail-closed, missing-file arm): a sufficient cap on an
/// ABSENT path reaches the syscall and surfaces a total `Err(NotFound)` —
/// never a panic, never a false success.
#[test]
fn missing_file_surfaces_total_not_found() {
    let src = program_src("conformance/fs/fixtures/does-not-exist.txt");
    let (stdout, stderr, success) = run("missing_file", &src);
    assert!(
        !success,
        "missing file must fail (exit non-zero), got stdout: {stdout:?}"
    );
    assert_eq!(
        stdout, "NotFound\n",
        "application must report through Console"
    );
    assert!(
        stderr.is_empty(),
        "runner must not render app results: {stderr}"
    );
}
