//! fs-read-file-lines-flip D5 — end-to-end acceptance driving the REAL
//! `ken` CLI binary (subprocess) against a real ABI-shaped program. No test in
//! this file hand-constructs a cap value at any `EvalVal`/`apply` site — the
//! cap originates inside `ken-cli`'s `ProgramCaps` mint-and-bind path.
//!
//! The fixed `ProgramCaps` grant carries `Cap APartial`; a missing-file arm
//! surfaces a total `Err(NotFound)`, never a panic.
//!
//! **effect-composition update (AC6 asterisk retirement):** `main` now
//! genuinely composes `[FS]` and `[Console]` in ONE `bind`-sequenced,
//! `inject_l`/`inject_r`-tagged `ITree (Coproduct (FSOp a) ConsoleOp) …` — the
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
/// fixed `ProgramCaps` grant.
fn program_src(path: &str) -> String {
    format!(
        r#"
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
  ITree (Coproduct (FSOp {auth}) ConsoleOp)
        (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
        r

proc printLines (xs : List String) : Compose (Result IOError Unit) visits [Console] =
  match xs {{
    Nil |->
      Ret (Coproduct (FSOp {auth}) ConsoleOp)
          (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
          (Result IOError Unit) (Ok IOError Unit MkUnit) ;
    Cons x xs' |->
      bind (Coproduct (FSOp {auth}) ConsoleOp)
           (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
           Unit (Result IOError Unit)
        (inject_r (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp Unit (print_line x))
        (\_ . printLines xs')
  }}

proc app (cap : Cap {auth}) : Compose (Result IOError Unit) visits [FS, Console] =
  bind (Coproduct (FSOp {auth}) ConsoleOp)
       (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
       (Result IOError Bytes) (Result IOError Unit)
    (inject_l (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp (Result IOError Bytes)
      (read_bytes {auth} cap (bytes_encode "{path}")))
    (\r .
      match r {{
        Err e    |-> Ret (Coproduct (FSOp {auth}) ConsoleOp)
                        (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
                        (Result IOError Unit) (Err IOError Unit e) ;
        Ok bytes |-> printLines (lines (bytes_decode bytes))
      }})

proc main (_input : ProcessInput) (caps : ProgramCaps)
  : HostIO ExitCode visits [FS, Console] =
  match caps {{
    MkProgramCaps cap |->
      bind (Coproduct (FSOp APartial) ConsoleOp)
           (resp_coproduct (FSOp APartial) ConsoleOp (fs_resp APartial) console_resp)
           (Result IOError Unit) ExitCode
        (app cap)
        (\r .
          match r {{
            Err e |->
              match e {{
                NotFound |-> host_program_then (print_line "NotFound") (Failure 1) ;
                PermissionDenied |-> host_program_then (print_line "PermissionDenied") (Failure 1) ;
                CapabilityDenied |-> host_program_then (print_line "CapabilityDenied") (Failure 1) ;
                BrokenPipe |-> host_program_then (print_line "BrokenPipe") (Failure 1) ;
                Interrupted |-> host_program_then (print_line "Interrupted") (Failure 1) ;
                Other |-> host_program_then (print_line "Other") (Failure 1)
              }} ;
            Ok _ |-> host_exit Success
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

/// The fixed `ProgramCaps` grant is sufficient for `ReadFile`, so the driver
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
