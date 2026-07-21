#!/usr/bin/env python3
"""Q2a — mechanical risk scan over the Rust test suite.

Produces the REVIEW QUEUE for Track Q of
docs/program/11-test-suite-and-ci-remediation.md, from the risk patterns in
research/qa-conformance-to-rust-test-guidelines.md §5.

    ⚠ THIS EMITS A REVIEW QUEUE, NOT A DEFECT LIST.

That distinction is the advisory's own and it is load-bearing. Every hit here
is a test whose promise class cannot be settled by looking at its syntax --
it is a question for a human/agent reviewer, not a finding. A hit is very
often correct code: a count literal that IS the contract, an `is_err()` whose
variant genuinely does not matter, a timing assert in a real perf lane. The
scan cannot tell those from their failing lookalikes, which is exactly why
its output is a queue to triage (Q2b) rather than a worklist to fix.

Conversely, a clean scan does NOT mean a test is well-formed. These are
syntactic smells only. A test can be perfectly shaped and still assert
nothing that matters -- see the advisory's reachability and oracle-
independence gates, neither of which is detectable from source text.

Usage:
    scripts/qa-risk-scan.py                 # summary to stdout
    scripts/qa-risk-scan.py --tsv           # one row per (test, pattern)
    scripts/qa-risk-scan.py --json          # same, as JSON
    scripts/qa-risk-scan.py --self-test     # verify detectors on known sites
"""

import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATES = os.path.join(REPO, "crates")

# ── Risk patterns, keyed to advisory §5 ──────────────────────────────────
#
# Each detector is (id, advisory section, human label, compiled regex).
# Deliberately conservative: these fire on SHAPE. Tuning them to reduce hits
# would defeat the purpose -- a queue that misses the ambiguous cases is
# worse than one that over-collects, because the whole point is to surface
# what syntax cannot settle.

PATTERNS = [
    (
        "R1-derived-count",
        "5.1",
        "count literal asserted directly (may freeze a census)",
        # assert_eq!(x.len(), 13) / assert_eq!(13, x.count()) / == 13
        re.compile(
            r"assert_eq!\s*\([^;]*?\.(?:len|count)\(\)\s*,\s*\d+"
            r"|assert_eq!\s*\(\s*\d+\s*,[^;]*?\.(?:len|count)\(\)"
            r"|\.(?:len|count)\(\)\s*==\s*\d+",
            re.S,
        ),
    ),
    (
        "R2-broad-outcome",
        "5.2",
        "outcome asserted without naming the variant",
        re.compile(
            r"assert!\s*\([^;]*?\.is_err\(\)"
            r"|assert!\s*\([^;]*?\.is_ok\(\)"
            r"|matches!\s*\([^;]*?Err\s*\(\s*_\s*\)"
            r"|,\s*Err\s*\(\s*_\s*\)\s*\)",
            re.S,
        ),
    ),
    (
        "R3-disabled",
        "5.3",
        "ignored / placeholder / unimplemented",
        re.compile(r"#\[ignore|todo!\s*\(|unimplemented!\s*\("),
    ),
    (
        "R4-source-text",
        "5.4",
        "asserts over Rust source text rather than a mechanism",
        re.compile(
            r"include_str!"
            r"|read_to_string\s*\([^;)]*\.rs"
            r"|\.contains\s*\(\s*\"(?:fn |pub |struct |enum |impl |unsafe )",
            re.S,
        ),
    ),
    (
        "R5-timing-env",
        "5.5",
        "wall-clock or ambient-environment coupling",
        re.compile(
            r"Instant::now|\.elapsed\(\)|thread::sleep"
            r"|Duration::from_(?:secs|millis)"
            r"|env::var|env::current_dir|std::env::"
        ),
    ),
]

# A test NAME that encodes a quantity is the stateful-name half of §5.1: the
# name outlives the number and becomes a false claim the compiler cannot
# catch. Checked against the name, not the body.
NUMBER_WORDS = (
    r"zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|"
    r"thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty"
)
STATEFUL_NAME = re.compile(rf"_(?:{NUMBER_WORDS})_|_\d+_(?!bit|byte)", re.I)

TEST_ATTR = re.compile(r"#\[(?:tokio::)?test\b")
FN_NAME = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")


def rust_files():
    """Every .rs file under crates/ -- integration tests AND unit tests.

    Both matter: ken-runtime and ken-host have NO tests/ directory at all,
    yet hold 286 and 46 tests respectively in #[cfg(test)] modules. Scanning
    only tests/*.rs would silently miss 17% of the suite while looking
    complete -- the exact shape of failure this program keeps hitting.
    """
    for root, dirs, files in os.walk(CRATES):
        dirs[:] = [d for d in dirs if d != "target"]
        for f in files:
            if f.endswith(".rs"):
                yield os.path.join(root, f)


def split_tests(text):
    """Yield (name, body, line_no) per #[test] function.

    Body runs to the next #[test] attribute, which over-reads a trailing
    helper fn into the previous test. Accepted deliberately: over-attribution
    puts an extra test in a review queue, while under-attribution drops one
    silently. Queues tolerate false positives; they cannot tolerate misses.
    """
    marks = [m.start() for m in TEST_ATTR.finditer(text)]
    for i, start in enumerate(marks):
        end = marks[i + 1] if i + 1 < len(marks) else len(text)
        chunk = text[start:end]
        m = FN_NAME.search(chunk)
        if not m:
            continue
        yield m.group(1), chunk, text.count("\n", 0, start) + 1


def scan():
    rows = []
    total = 0
    for path in rust_files():
        try:
            text = open(path, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        if not TEST_ATTR.search(text):
            continue
        rel = os.path.relpath(path, REPO)
        crate = rel.split(os.sep)[1] if os.sep in rel else "?"
        for name, body, line in split_tests(text):
            total += 1
            hits = [(pid, sec, lbl) for pid, sec, lbl, rx in PATTERNS
                    if rx.search(body)]
            if STATEFUL_NAME.search(name):
                hits.append(("R6-stateful-name", "5.1",
                             "test name encodes a quantity"))
            for pid, sec, lbl in hits:
                rows.append({"crate": crate, "file": rel, "line": line,
                             "test": name, "pattern": pid,
                             "section": sec, "label": lbl})
    return rows, total


# ── Self-test: validate detectors against sites the advisory already named ──
#
# §10 of the advisory lists concrete sites it found by hand. Those are an
# INDEPENDENT ORACLE for this scan: a detector that cannot rediscover what a
# human already found is not trustworthy on the tests nobody has read.
# Running the scan and eyeballing the output would only prove it emits rows.
SELF_TEST = [
    ("dependent_match_wstyle_acceptance", "R2-broad-outcome",
     "advisory §10.2 -- subsuming Err(_) assertion"),
    ("b1_exact_denotation_alphabet", None,
     "advisory §10 cites this file; scan must at least parse it"),
]


def self_test(rows):
    by_file = defaultdict(set)
    for r in rows:
        by_file[os.path.basename(r["file"])].add(r["pattern"])
    ok = True
    for stem, want, why in SELF_TEST:
        fname = stem + ".rs"
        found = by_file.get(fname)
        if found is None:
            print(f"  FAIL {stem}: file produced no rows at all ({why})")
            ok = False
        elif want and want not in found:
            print(f"  FAIL {stem}: expected {want}, got {sorted(found)} ({why})")
            ok = False
        else:
            print(f"  ok   {stem}: {sorted(found)}")
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tsv", action="store_true")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()

    rows, total = scan()

    if a.self_test:
        print(f"self-test against advisory §10 sites ({total} tests scanned):")
        sys.exit(0 if self_test(rows) else 1)

    if a.json:
        print(json.dumps(rows, indent=2))
        return
    if a.tsv:
        print("crate\tfile\tline\ttest\tpattern")
        for r in sorted(rows, key=lambda r: (r["crate"], r["file"], r["line"])):
            print(f"{r['crate']}\t{r['file']}\t{r['line']}\t{r['test']}\t"
                  f"{r['pattern']}")
        return

    flagged = {(r["file"], r["test"]) for r in rows}
    print(f"tests scanned : {total}")
    print(f"tests flagged : {len(flagged)}  "
          f"({100.0 * len(flagged) / total:.1f}% -- a REVIEW QUEUE, "
          f"not a defect count)")
    print(f"total hits    : {len(rows)}  (a test can hit several patterns)")
    print("\nby pattern:")
    for pid, n in Counter(r["pattern"] for r in rows).most_common():
        sec = next(r["section"] for r in rows if r["pattern"] == pid)
        lbl = next(r["label"] for r in rows if r["pattern"] == pid)
        print(f"  {n:5}  {pid:20} §{sec}  {lbl}")
    print("\nby crate (flagged tests / hits):")
    fc = Counter(f.split(os.sep)[1] for f, _ in flagged)
    hc = Counter(r["crate"] for r in rows)
    for crate, n in fc.most_common():
        print(f"  {n:5} / {hc[crate]:<5}  {crate}")


if __name__ == "__main__":
    main()
