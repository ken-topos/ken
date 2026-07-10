//! fs-read-file-lines-flip D5 — end-to-end acceptance driving the REAL
//! `ken` CLI binary (subprocess) against a real elaborated `(cap : Cap …)`
//! program. No test in this file hand-constructs a cap value at any
//! `EvalVal`/`apply` site — the cap originates **inside** `ken-cli`'s
//! `run_file` manifest-read -> mint-exactly -> `apply` path (AC3's
//! producer-grep: grep this file for `EvalVal::Cap`/`EvalVal::Int`/
//! `cap_evalval` — none exist).
//!
//! AC4's discriminating pair: two `main`s, identical except the declared
//! `Auth` index on the cap param. AC6: a missing-file arm surfaces a total
//! `Err(NotFound)`, never a panic.
//!
//! **effect-composition update (AC6 asterisk retirement):** `main` now
//! genuinely composes `[FS]` and `[Console]` in ONE `bind`-sequenced,
//! `inject_l`/`inject_r`-tagged `ITree (Coproduct (FSOp a) ConsoleOp) …` — the
//! program itself prints each line via `[Console]` (`printLines`), not a
//! CLI-side post-render. Also no test in this file hand-constructs a `Coproduct`/
//! `InL`/`InR` value (AC7's producer-grep, `effect-composition-conformance.md`
//! §2) — `inject_l`/`inject_r` are elaborated from the surface `.ken` source
//! above. On a denied/insufficient cap or a missing file, `main` does NOT
//! print (fail-closed) and returns `Err e`; `ken-cli`'s unchanged
//! `render_fs_result` still surfaces the exact `IOError` variant on stderr
//! with a non-zero exit — so the M-insuff/missing-file assertions below are
//! unchanged even though the mechanism (in-program vs. post-hoc printing)
//! flipped. M-suff's stdout assertion is also unchanged: the composed
//! program's own `println!`-per-line `[Console]` output is byte-identical
//! to the old CLI-side render.

use std::path::PathBuf;
use std::process::Command;

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The D1 `lines` helper + a `main` reading `path` under authority `auth`.
/// Identical for every case except `auth`/`path` — the discriminating pair
/// (AC4) and the missing-file arm (AC6) are pure substitutions here.
fn program_src(auth: &str, path: &str) -> String {
    format!(
        r#"
fn isNewline (c : Char) : Bool = eq_int (charToInt c) 10

fn consFirst (c : Char) (acc : List (List Char)) : List (List Char) =
  match acc {{
    Nil           => Cons (List Char) (Cons Char c (Nil Char)) (Nil (List Char)) ;
    Cons seg rest => Cons (List Char) (Cons Char c seg) rest
  }}

fn splitNL (xs : List Char) : List (List Char) =
  match xs {{
    Nil       => Cons (List Char) (Nil Char) (Nil (List Char)) ;
    Cons c cs =>
      match isNewline c {{
        True  => Cons (List Char) (Nil Char) (splitNL cs) ;
        False => consFirst c (splitNL cs)
      }}
  }}

fn dropTrailingEmpty (segs : List (List Char)) : List (List Char) =
  match segs {{
    Nil => Nil (List Char) ;
    Cons seg rest =>
      match rest {{
        Nil =>
          match seg {{
            Nil      => Nil (List Char) ;
            Cons _ _ => Cons (List Char) seg (Nil (List Char))
          }} ;
        Cons _ _ => Cons (List Char) seg (dropTrailingEmpty rest)
      }}
  }}

fn mapListCharToString (segs : List (List Char)) : List String =
  match segs {{
    Nil           => Nil String ;
    Cons seg rest => Cons String (list_char_to_string seg) (mapListCharToString rest)
  }}

fn lines (s : String) : List String =
  mapListCharToString (dropTrailingEmpty (splitNL (string_to_list_char s)))

const Compose (r : Type) : Type =
  ITree (Coproduct (FSOp {auth}) ConsoleOp)
        (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
        r

proc printLines (xs : List String) : Compose (Result IOError Unit) visits [Console] =
  match xs {{
    Nil =>
      Ret (Coproduct (FSOp {auth}) ConsoleOp)
          (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
          (Result IOError Unit) (Ok IOError Unit MkUnit) ;
    Cons x xs' =>
      bind (Coproduct (FSOp {auth}) ConsoleOp)
           (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
           Unit (Result IOError Unit)
        (inject_r (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp Unit (print_line x))
        (\_ . printLines xs')
  }}

proc main (cap : Cap {auth}) : Compose (Result IOError Unit) visits [FS, Console] =
  bind (Coproduct (FSOp {auth}) ConsoleOp)
       (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
       (Result IOError Bytes) (Result IOError Unit)
    (inject_l (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp (Result IOError Bytes)
      (read_bytes {auth} cap (bytes_encode "{path}")))
    (\r .
      match r {{
        Err e    => Ret (Coproduct (FSOp {auth}) ConsoleOp)
                        (resp_coproduct (FSOp {auth}) ConsoleOp (fs_resp {auth}) console_resp)
                        (Result IOError Unit) (Err IOError Unit e) ;
        Ok bytes => printLines (lines (bytes_decode bytes))
      }})
"#,
        auth = auth,
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

/// AC4 M-suff: `main` declares `Cap APartial` (sufficient — read requires
/// `AUTH_PARTIAL`) — the CLI mints exactly that, the driver's `authorizes`
/// gate allows it, the fixture is read.
#[test]
fn m_suff_apartial_reads_fixture() {
    let src = program_src("APartial", "conformance/fs/fixtures/three-lines.txt");
    let (stdout, stderr, success) = run("m_suff", &src);
    assert!(success, "M-suff must succeed; stderr: {stderr}");
    assert_eq!(stdout, "alpha\nbeta\ngamma\n", "M-suff must print the exact fixture lines");
}

/// AC4 M-insuff (the load-bearing negative arm, SEAM-A): `main` declares
/// `Cap ANone` — it still KEEPS its cap param (clears the static face,
/// unlike a no-cap-param `main`), gets a level-0 cap minted + bound,
/// reaches the driver, and is denied at `authorizes` with EXACTLY
/// `CapabilityDenied` — not a bare failure, not `NotFound` (the fixture
/// path exists, isolating the denial from a not-found confound). A
/// full-minting CLI (the precise bug this AC targets) would pass this arm
/// too — that's what makes the pair non-vacuous.
#[test]
fn m_insuff_anone_denied_capabilitydenied_not_notfound() {
    let src = program_src("ANone", "conformance/fs/fixtures/three-lines.txt");
    let (stdout, stderr, success) = run("m_insuff", &src);
    assert!(!success, "M-insuff must fail (exit non-zero), got stdout: {stdout:?}");
    assert_eq!(stdout, "", "M-insuff must print nothing to stdout (fail-closed, never partial success)");
    assert!(
        stderr.contains("CapabilityDenied"),
        "M-insuff must be denied with EXACTLY CapabilityDenied, not e.g. NotFound/a panic; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("NotFound"),
        "the fixture path exists — a NotFound here would mean the denial fired for the wrong reason; stderr: {stderr}"
    );
}

/// AC6 (totality/fail-closed, missing-file arm): a sufficient cap on an
/// ABSENT path reaches the syscall and surfaces a total `Err(NotFound)` —
/// never a panic, never a false success.
#[test]
fn missing_file_surfaces_total_not_found() {
    let src = program_src("APartial", "conformance/fs/fixtures/does-not-exist.txt");
    let (stdout, stderr, success) = run("missing_file", &src);
    assert!(!success, "missing file must fail (exit non-zero), got stdout: {stdout:?}");
    assert_eq!(stdout, "", "missing-file must print nothing to stdout");
    assert!(stderr.contains("NotFound"), "must surface NotFound, not a panic; stderr: {stderr}");
}
