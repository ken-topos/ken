#!/usr/bin/env bash
# moot-actor-id.sh — resolve role -> actor_id from .moot/actors.json, SAFELY.
#
# ⛔ WHY THIS SCRIPT EXISTS. `.moot/actors.json` holds, per actor, BOTH an
# `actor_id` (needed constantly, safe to read) and an `api_key` (another seat's
# credential, never yours to read). Two seats have leaked a key into a transcript
# by reading the file directly. In BOTH cases the leak happened during SCHEMA
# DISCOVERY, not during the lookup:
#
#   - instance 1: a `sed`/`grep` over the raw file to find a role's id
#   - instance 2: a field-projecting one-liner returned nothing (the author had
#     guessed the key names), so the author dumped the file "just to see the
#     structure" and printed three actor records verbatim, keys included
#
# ⇒ A rule saying "project only the fields you need" does NOT bind the moment that
#   actually leaks, because you cannot write a projection until you know the field
#   names, and the reflex for finding out is to dump the file. This script removes
#   the need to ever open it.
#
# ⭐ SAFETY IS ENFORCED BY AN OUTPUT WHITELIST, NOT BY A BLACKLIST. The script does
#   not scan its output for `api_key`-shaped strings — a negative check like that
#   passes for any reason, including a bug that produced no output at all. Instead
#   every line is required to match `^<role> agt_<id>$` and anything else is a hard
#   failure. Nothing but a role name and an actor id can leave this script.
#
# Usage:
#   scripts/moot-actor-id.sh <role> [<role> ...]   # prints "<role> <actor_id>"
#   scripts/moot-actor-id.sh --ids <role> [...]    # prints bare actor_ids only
#   scripts/moot-actor-id.sh --list                # prints known role names
#
# Exits non-zero on an unknown role, a missing file, or an output line that does
# not match the whitelist. Fail-closed: it prints nothing rather than something
# it cannot vouch for.

set -euo pipefail

die() { printf 'moot-actor-id: %s\n' "$*" >&2; exit 1; }

# The .moot directory lives in the MAIN worktree, not in whichever worktree the
# caller is running from. --git-common-dir resolves to the main .git for every
# linked worktree, so its parent is the main worktree root.
common_dir=$(git rev-parse --git-common-dir 2>/dev/null) \
  || die "not in a git repository"
case "$common_dir" in
  /*) ;;
  *) common_dir="$(pwd)/$common_dir" ;;
esac
root=$(cd "$(dirname "$common_dir")" && pwd)
actors="$root/.moot/actors.json"

[ -f "$actors" ] || die "no actors file at $actors"

mode=roles
case "${1:-}" in
  --list)
    exec python3 -c '
import json,sys
with open(sys.argv[1]) as f:
    a = json.load(f)["actors"]
for name in sorted(a):
    print(name)
' "$actors"
    ;;
  --ids) mode=ids; shift ;;
  -h|--help)
    sed -n '3,30p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
    ;;
  "") die "usage: $0 <role> [<role> ...] | --ids <role> ... | --list" ;;
esac

[ $# -gt 0 ] || die "no role given"

# Project ONLY actor_id, by name. The script never touches any other field, and
# never serializes a record.
out=$(python3 -c '
import json,sys,re
path, mode = sys.argv[1], sys.argv[2]
with open(path) as f:
    actors = json.load(f)["actors"]
missing, lines = [], []
for role in sys.argv[3:]:
    rec = actors.get(role)
    if rec is None or "actor_id" not in rec:
        missing.append(role); continue
    aid = rec["actor_id"]
    if not re.fullmatch(r"agt_[A-Za-z0-9]+", aid):
        print("malformed actor_id for %s" % role, file=sys.stderr); sys.exit(3)
    lines.append(aid if mode == "ids" else "%s %s" % (role, aid))
if missing:
    print("unknown role(s): %s" % ", ".join(missing), file=sys.stderr)
    print("run with --list to see known roles", file=sys.stderr)
    sys.exit(2)
print("\n".join(lines))
' "$actors" "$mode" "$@")

# ⭐ The output whitelist. Fail-closed: an unrecognised line aborts with NOTHING
# printed, rather than passing through something this script cannot vouch for.
if [ "$mode" = ids ]; then
  pattern='^agt_[A-Za-z0-9]+$'
else
  pattern='^[A-Za-z0-9_-]+ agt_[A-Za-z0-9]+$'
fi
while IFS= read -r line; do
  [ -n "$line" ] || continue
  printf '%s' "$line" | grep -qE "$pattern" \
    || die "refusing to print a line that is not <role> <actor_id> (output whitelist)"
done <<< "$out"

printf '%s\n' "$out"
