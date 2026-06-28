#!/usr/bin/env python3
"""
originality-scan.py — clean-room leakage flagging aid (CLEAN-ROOM.md gate).

Detects shared word-sequences (k-gram shingles) between Ken's spec prose and a
reference repository, to flag possible close-paraphrase / transcription of a
(copyleft) source into the spec. This is a FLAGGING AID, not a verdict: it finds
lexical overlap; a human reviewer decides whether a flagged span is genuine
copying or incidental shared domain vocabulary. Long matched runs are the
suspicious ones; isolated short matches over technical phrases are expected.

Usage:
  scripts/originality-scan.py SPEC_DIR REF_DIR [REF_DIR ...] [options]

  e.g.  scripts/originality-scan.py spec local/refs/smtcoq --report scan.md
        scripts/originality-scan.py spec local/refs/spot local/refs/jif --fail 0.04

Options:
  --k N          shingle size in words (default 8). Smaller = more sensitive +
                 noisier; 8 catches paraphrased fragments while ignoring most
                 incidental phrases.
  --report FILE  write the markdown report here (default: stdout).
  --top N        show the N most-overlapping spec files in detail (default 15).
  --fail F       exit non-zero if any spec file's overlap ratio >= F, or any
                 matched run is >= --fail-run shingles (for use as a CI gate).
  --fail-run N   run length that trips --fail regardless of ratio (default 6).
  --ref-ext ...  extra reference file extensions to scan (comma-separated).
  --quiet        suppress the progress line.

Scope/limits: catches LEXICAL overlap only (shared word runs after lowercasing
and dropping punctuation), so deep reword-everything paraphrase will slip past —
that is what the human review in the gate is for. Run it on the *copyleft* refs
(the ones that taint the MIT channel); permissive refs do not require it.
"""
import argparse, os, re, sys, zlib

TOKEN = re.compile(r"[a-z0-9]+")
# Reference file types worth scanning (source comments + docs carry the
# copyrightable prose most at risk of being paraphrased into a prose spec).
REF_EXTS = {
    ".md", ".txt", ".rst", ".tex", ".org",
    ".v", ".lean", ".agda", ".hs", ".ml", ".mli", ".fst", ".fsti",
    ".rs", ".c", ".cc", ".cpp", ".h", ".hpp", ".py", ".go", ".java",
    ".scala", ".js", ".ts",
}
SKIP_DIRS = {".git", "node_modules", "target", "_build", "build", "dist",
             ".stack-work", "vendor", "third_party"}


def read_text(path):
    try:
        with open(path, "r", encoding="utf-8", errors="strict") as f:
            return f.read()
    except (UnicodeDecodeError, OSError):
        return None  # binary or unreadable — skip


def tokens_with_lines(text):
    """Yield (token, lineno) for lowercased alphanumeric word tokens."""
    out = []
    for lineno, line in enumerate(text.splitlines(), 1):
        for m in TOKEN.finditer(line.lower()):
            out.append((m.group(), lineno))
    return out


def shingle_hash(toks):
    return zlib.crc32(" ".join(toks).encode())


def walk_files(root, exts):
    for dp, dns, fns in os.walk(root):
        dns[:] = [d for d in dns if d not in SKIP_DIRS]
        for fn in fns:
            if os.path.splitext(fn)[1].lower() in exts:
                yield os.path.join(dp, fn)


def build_ref_index(ref_dirs, k, exts, quiet):
    """hash(shingle) -> 'relpath:line' (first occurrence)."""
    index = {}
    nfiles = 0
    for ref in ref_dirs:
        base = os.path.dirname(ref.rstrip("/")) or "."
        for path in walk_files(ref, exts):
            text = read_text(path)
            if text is None:
                continue
            nfiles += 1
            if not quiet and nfiles % 500 == 0:
                sys.stderr.write(f"\r  indexed {nfiles} ref files...")
                sys.stderr.flush()
            toks = tokens_with_lines(text)
            words = [t for t, _ in toks]
            rel = os.path.relpath(path, base)
            for i in range(len(words) - k + 1):
                h = shingle_hash(words[i:i + k])
                if h not in index:
                    index[h] = f"{rel}:{toks[i][1]}"
    if not quiet:
        sys.stderr.write(f"\r  indexed {nfiles} ref files; "
                         f"{len(index)} distinct {k}-shingles.\n")
    return index


def scan_spec(spec_dir, index, k):
    """Return per-file results: matched runs and overlap ratio."""
    results = []
    for path in walk_files(spec_dir, {".md"}):
        text = read_text(path)
        if text is None:
            continue
        toks = tokens_with_lines(text)
        words = [t for t, _ in toks]
        nsh = max(0, len(words) - k + 1)
        if nsh == 0:
            continue
        matched = []  # (shingle_index, ref_loc)
        for i in range(nsh):
            ref = index.get(shingle_hash(words[i:i + k]))
            if ref is not None:
                matched.append((i, ref))
        # Merge consecutive matched shingle indices into runs.
        runs = []
        for i, ref in matched:
            if runs and i == runs[-1]["end"] + 1:
                runs[-1]["end"] = i
            else:
                runs.append({"start": i, "end": i, "ref": ref})
        for r in runs:
            r["line"] = toks[r["start"]][1]
            span = words[r["start"]:r["end"] + k]
            r["len"] = r["end"] - r["start"] + 1  # matched shingles in the run
            r["words"] = r["end"] - r["start"] + k  # word span
            r["text"] = " ".join(span)
        ratio = len(matched) / nsh
        longest = max((r["len"] for r in runs), default=0)
        results.append({
            "file": os.path.relpath(path, spec_dir),
            "shingles": nsh, "matched": len(matched),
            "ratio": ratio, "longest": longest, "runs": runs,
        })
    results.sort(key=lambda r: (r["longest"], r["ratio"]), reverse=True)
    return results


def render(results, k, top, fail, fail_run):
    L = []
    flagged = [r for r in results
               if r["ratio"] >= fail or r["longest"] >= fail_run] if fail is not None else []
    L.append(f"# Originality scan (k={k} word shingles)\n")
    L.append("Flagging aid only — long matched *runs* are the suspicious ones; "
             "short matches over shared domain vocabulary are expected. A human "
             "reviewer decides. See `CLEAN-ROOM.md`.\n")
    tot_sh = sum(r["shingles"] for r in results)
    tot_m = sum(r["matched"] for r in results)
    L.append(f"**Corpus:** {len(results)} spec files, {tot_sh} shingles; "
             f"{tot_m} matched the reference "
             f"({(tot_m/tot_sh if tot_sh else 0):.2%}).\n")
    if fail is not None:
        L.append(f"**Gate:** fail if overlap ratio >= {fail:.2%} or a run >= "
                 f"{fail_run} shingles. **{len(flagged)} file(s) over threshold.**\n")
    L.append("\n## Per-file overlap (ranked)\n")
    L.append("| spec file | shingles | matched | overlap | longest run |")
    L.append("|---|--:|--:|--:|--:|")
    for r in results:
        mark = " ⚠" if (fail is not None and (r["ratio"] >= fail or r["longest"] >= fail_run)) else ""
        L.append(f"| {r['file']}{mark} | {r['shingles']} | {r['matched']} | "
                 f"{r['ratio']:.2%} | {r['longest']} |")
    L.append("\n## Top matched spans (review these)\n")
    shown = 0
    for r in results:
        runs = sorted(r["runs"], key=lambda x: x["len"], reverse=True)
        runs = [x for x in runs if x["len"] >= 2] or runs[:1]
        if not r["matched"]:
            continue
        L.append(f"\n### {r['file']}  (overlap {r['ratio']:.2%}, "
                 f"longest run {r['longest']})\n")
        for x in runs[:6]:
            L.append(f"- L{x['line']} · {x['len']} shingles · ref `{x['ref']}`\n"
                     f"  > {x['text']}")
        shown += 1
        if shown >= top:
            break
    return "\n".join(L) + "\n", flagged


def main():
    ap = argparse.ArgumentParser(description="Clean-room originality flagging aid.")
    ap.add_argument("spec_dir")
    ap.add_argument("ref_dirs", nargs="+")
    ap.add_argument("--k", type=int, default=8)
    ap.add_argument("--report")
    ap.add_argument("--top", type=int, default=15)
    ap.add_argument("--fail", type=float, default=None)
    ap.add_argument("--fail-run", type=int, default=6)
    ap.add_argument("--ref-ext", default="")
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()

    exts = set(REF_EXTS)
    for e in filter(None, a.ref_ext.split(",")):
        exts.add(e if e.startswith(".") else "." + e)

    for d in [a.spec_dir, *a.ref_dirs]:
        if not os.path.isdir(d):
            sys.exit(f"not a directory: {d}")

    index = build_ref_index(a.ref_dirs, a.k, exts, a.quiet)
    if not index:
        sys.exit("reference index empty — nothing to scan against "
                 "(are the ref dirs populated? did clone.sh run?)")
    results = scan_spec(a.spec_dir, index, a.k)
    report, flagged = render(results, a.k, a.top, a.fail, a.fail_run)

    if a.report:
        with open(a.report, "w") as f:
            f.write(report)
        if not a.quiet:
            sys.stderr.write(f"  report -> {a.report}\n")
    else:
        sys.stdout.write(report)

    if a.fail is not None and flagged:
        sys.exit(2)


if __name__ == "__main__":
    main()
