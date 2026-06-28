#!/usr/bin/env bash
# ken local LLM proxy — multi-provider dispatch for the BUILD tiers only
# (mootup_harness_sdk.llm_proxy). Routes by model prefix:
#   accounts/fireworks/* -> Fireworks (GLM)   deepseek-* -> DeepSeek
# The Opus enclave does NOT use this proxy — it runs direct on the Anthropic
# subscription (OAuth) — so no anthropic-api-key-upstream is needed here, and
# subscription tokens never transit the proxy (D-MBT-PROXY guardrail).
#
# Idempotent via pidfile; safe to call on every container start.
set -euo pipefail

PIDFILE="/tmp/llm-proxy.pid"
LOGFILE="/tmp/llm-proxy.log"

if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE" 2>/dev/null)" 2>/dev/null; then
    echo "llm-proxy already running (pid=$(cat "$PIDFILE"))" >&2
    exit 0
fi
rm -f "$PIDFILE"

# Operator-managed secrets (absent file = that provider disabled).
[ -r /home/node/.secrets/llm-proxy-secret ] && export LLM_PROXY_SHARED_SECRET="$(cat /home/node/.secrets/llm-proxy-secret)"
[ -r /home/node/.secrets/deepseek-api-key ]  && export DEEPSEEK_API_KEY="$(cat /home/node/.secrets/deepseek-api-key)"
[ -r /home/node/.secrets/fireworks-api-key ] && export FIREWORKS_API_KEY="$(cat /home/node/.secrets/fireworks-api-key)"
export LLM_PROXY_PORT="${LLM_PROXY_PORT:-8090}"

nohup python3 -m mootup_harness_sdk.llm_proxy.main > "$LOGFILE" 2>&1 &
echo $! > "$PIDFILE"
echo "llm-proxy started (pid=$(cat "$PIDFILE"); port=$LLM_PROXY_PORT; log=$LOGFILE)" >&2
