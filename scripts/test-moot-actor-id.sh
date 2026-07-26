#!/usr/bin/env bash
# Controls for moot-actor-id.sh.
#
# ⛔ THE POINT OF THESE CONTROLS IS THE SAFETY PROPERTY, NOT THE LOOKUP. A test
# that only checks "the right id comes back" is green for a script that also
# prints a credential. So the load-bearing controls here are:
#
#   C4  a synthetic actors file whose api_key values are known sentinels, and the
#       assertion that NEITHER sentinel appears in the output. ⭐ This is a
#       POPULATION-side control: the secret really is present in the input, so a
#       green result means the script excluded it, not that there was nothing to
#       exclude. A test against the real file could pass simply because the script
#       silently failed.
#   C5  a positive control ON C4 — the sentinel IS findable in the input file, so
#       C4's grep is capable of finding it. Without this, C4 passes for any
#       reason, including a broken grep or an unwritten fixture.
#   C6  the output whitelist fires: a doctored record whose actor_id does not
#       match `agt_<id>` must abort with nothing printed.

set -uo pipefail
cd "$(dirname "$0")/.."
SCRIPT=scripts/moot-actor-id.sh
pass=0; fail=0
ok()   { printf '  ✅ %s\n' "$1"; pass=$((pass+1)); }
bad()  { printf '  ⛔ %s\n' "$1"; fail=$((fail+1)); }

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

# A synthetic repo so we never depend on (or read) the real actors file.
mkdir -p "$tmp/repo/.moot" "$tmp/repo/scripts"
( cd "$tmp/repo" && git init -q . )
cp "$SCRIPT" "$tmp/repo/scripts/"
cat > "$tmp/repo/.moot/actors.json" <<'JSON'
{
  "space_id": "spc_test",
  "space_name": "test",
  "api_url": "https://example.invalid",
  "actors": {
    "steward":  { "actor_id": "agt_aaa111", "api_key": "SENTINEL_KEY_ALPHA", "display_name": "steward" },
    "architect":{ "actor_id": "agt_bbb222", "api_key": "SENTINEL_KEY_BETA",  "display_name": "architect" }
  }
}
JSON
run() { ( cd "$tmp/repo" && bash scripts/moot-actor-id.sh "$@" ) 2>&1; }

echo "C1 — known role resolves"
got=$(run steward)
[ "$got" = "steward agt_aaa111" ] && ok "steward -> agt_aaa111" || bad "got: $got"

echo "C2 — several roles, and --ids prints bare ids"
got=$(run --ids steward architect | tr '\n' ',')
[ "$got" = "agt_aaa111,agt_bbb222," ] && ok "--ids ok" || bad "got: $got"

echo "C3 — NEGATIVE CONTROL: unknown role fails, nonzero, no id printed"
got=$( ( cd "$tmp/repo" && bash scripts/moot-actor-id.sh nosuchrole ) 2>&1 ); rc=$?
if [ "$rc" -ne 0 ] && ! printf '%s' "$got" | grep -q 'agt_'; then
  ok "unknown role rejected (rc=$rc), no id leaked"
else
  bad "rc=$rc got: $got"
fi

echo "C4 — ★ SAFETY: no api_key value reaches the output (population-side)"
got=$(run steward architect; run --ids steward architect; run --list)
if printf '%s' "$got" | grep -q 'SENTINEL_KEY'; then
  bad "AN API KEY VALUE APPEARED IN THE OUTPUT"
else
  ok "neither sentinel present in any output mode"
fi

echo "C5 — POSITIVE CONTROL on C4: the sentinel IS findable in the input"
if grep -q 'SENTINEL_KEY_ALPHA' "$tmp/repo/.moot/actors.json" \
   && printf 'x SENTINEL_KEY_ALPHA y' | grep -q 'SENTINEL_KEY'; then
  ok "the secret is really in the input and the grep can find it"
else
  bad "C4 was vacuous — fixture or grep cannot see the sentinel"
fi

echo "C6 — ★ the output whitelist fires on a malformed actor_id"
python3 - "$tmp/repo/.moot/actors.json" <<'PY'
import json,sys
p=sys.argv[1]; d=json.load(open(p))
d["actors"]["steward"]["actor_id"]="not-an-agt-id"
json.dump(d, open(p,"w"))
PY
got=$( ( cd "$tmp/repo" && bash scripts/moot-actor-id.sh steward ) 2>&1 ); rc=$?
if [ "$rc" -ne 0 ] && ! printf '%s' "$got" | grep -q 'not-an-agt-id'; then
  ok "malformed id rejected (rc=$rc) and never printed"
else
  bad "rc=$rc got: $got"
fi

echo "C7 — missing actors file fails closed"
rm -f "$tmp/repo/.moot/actors.json"
got=$( ( cd "$tmp/repo" && bash scripts/moot-actor-id.sh steward ) 2>&1 ); rc=$?
[ "$rc" -ne 0 ] && ok "missing file rejected (rc=$rc)" || bad "rc=$rc got: $got"

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
