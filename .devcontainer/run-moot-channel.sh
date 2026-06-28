#!/bin/bash
# Channel adapter wrapper — reads CONVO_ROLE and looks up API key
# from .moot/actors.json. Same logic as run-moot-mcp.sh.

ROLE="${CONVO_ROLE:-implementation}"
ACTORS_FILE=".moot/actors.json"

# Find project root (walk up to moot.toml)
PROJECT_ROOT="$(pwd)"
while [ "$PROJECT_ROOT" != "/" ]; do
    [ -f "$PROJECT_ROOT/moot.toml" ] && break
    PROJECT_ROOT="$(dirname "$PROJECT_ROOT")"
done

# Read per-role actor identity from .moot/actors.json. See run-moot-mcp.sh
# for the full rationale — same rule applies to the channel adapter.
#
# SEC-2-C: env-driven Python via single-quoted heredoc — no shell-eval RCE.
if [ -f "$PROJECT_ROOT/$ACTORS_FILE" ]; then
    OUT="$(
        export _PR="$PROJECT_ROOT" _AF="$ACTORS_FILE" _ROLE="$ROLE"
        python3 <<'PYEOF'
import json, os, sys
pr = os.environ['_PR']
af = os.environ['_AF']
role = os.environ['_ROLE']
try:
    with open(os.path.join(pr, af)) as f:
        data = json.load(f)
    entry = data.get('actors', {}).get(role, {})
    print(entry.get('api_key', ''))
    print(entry.get('actor_id', ''))
    print(entry.get('display_name', role))
except Exception:
    print('', file=sys.stderr)
    print('', file=sys.stderr)
    print('', file=sys.stderr)
PYEOF
    )"
    KEY="$(printf '%s\n' "$OUT" | sed -n '1p')"
    AID="$(printf '%s\n' "$OUT" | sed -n '2p')"
    ANAME="$(printf '%s\n' "$OUT" | sed -n '3p')"
    if [ -n "$KEY" ]; then
        export CONVO_API_KEY="$KEY"
    else
        echo "WARNING: No API key for role '$ROLE' in $ACTORS_FILE" >&2
    fi
    [ -n "$AID" ] && export CONVO_AGENT_ID="$AID"
    [ -n "$ANAME" ] && export CONVO_AGENT_NAME="$ANAME"
fi

# Read API URL from moot.toml
if [ -z "$CONVO_API_URL" ] && [ -f "$PROJECT_ROOT/moot.toml" ]; then
    URL="$(
        export _PR="$PROJECT_ROOT"
        python3 <<'PYEOF'
import os, tomllib
with open(os.path.join(os.environ['_PR'], 'moot.toml'), 'rb') as f:
    data = tomllib.load(f)
print(data.get('convo', {}).get('api_url', ''))
PYEOF
    )"
    if [ -n "$URL" ]; then
        export CONVO_API_URL="$URL"
    fi
fi

# Read space ID from moot.toml
if [ -z "$CONVO_SPACE_ID" ] && [ -f "$PROJECT_ROOT/moot.toml" ]; then
    SID="$(
        export _PR="$PROJECT_ROOT"
        python3 <<'PYEOF'
import os, tomllib
with open(os.path.join(os.environ['_PR'], 'moot.toml'), 'rb') as f:
    data = tomllib.load(f)
print(data.get('convo', {}).get('space_id', ''))
PYEOF
    )"
    if [ -n "$SID" ]; then
        export CONVO_SPACE_ID="$SID"
    fi
fi

# Alpha-grade diagnostics: DEBUG-level logs to a per-role file under
# .moot/logs/ in the project root. Bind-mounted to the host so users
# and support can grep and share the logs without docker exec.
# Override with MOOT_LOG_LEVEL=INFO (or higher) once alpha stabilizes.
LOG_DIR="$PROJECT_ROOT/.moot/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/channel-${ROLE}.log"
export MOOT_LOG_LEVEL="${MOOT_LOG_LEVEL:-DEBUG}"
exec python -u -m moot.adapters.channel_runner "$@" 2>> "$LOG_FILE"
