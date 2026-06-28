#!/usr/bin/env bash
# Mint a short-lived GitHub App installation token for the Integrator — the
# federation's sole GitHub identity (04-git-and-integration.md §3). Prints the
# token to stdout; nothing is persisted.
#
# Reads (from the operator-mounted secrets dir, never the repo):
#   /home/node/.secrets/github-app-private-key.pem   the App private key (.pem)
#   /home/node/.secrets/github-app.env               GITHUB_APP_ID + GITHUB_APP_INSTALLATION_ID
#
# Usage (Integrator only):
#   export GH_TOKEN="$(.devcontainer/mint-gh-token.sh)"
#   gh auth setup-git           # once — makes git reuse GH_TOKEN for github.com
#   git push / gh pr / gh ...   # token is valid ~1h; re-run before a long gap
#
# Requires: openssl, curl, python3 (all present in the devcontainer).
set -euo pipefail

SEC="${KEN_SECRETS_DIR:-/home/node/.secrets}"
PEM="$SEC/github-app-private-key.pem"
ENV_FILE="$SEC/github-app.env"

[ -r "$PEM" ]      || { echo "mint-gh-token: missing key at $PEM" >&2; exit 1; }
[ -r "$ENV_FILE" ] || { echo "mint-gh-token: missing $ENV_FILE (GITHUB_APP_ID, GITHUB_APP_INSTALLATION_ID)" >&2; exit 1; }
# shellcheck source=/dev/null
. "$ENV_FILE"
: "${GITHUB_APP_ID:?set GITHUB_APP_ID in $ENV_FILE}"
: "${GITHUB_APP_INSTALLATION_ID:?set GITHUB_APP_INSTALLATION_ID in $ENV_FILE}"

b64url() { openssl base64 -A | tr '+/' '-_' | tr -d '='; }
now="$(date +%s)"
header="$(printf '%s' '{"alg":"RS256","typ":"JWT"}' | b64url)"
# iat backdated 60s for clock skew; exp 9 min (<10 min GitHub max).
payload="$(printf '{"iat":%d,"exp":%d,"iss":"%s"}' "$((now - 60))" "$((now + 540))" "$GITHUB_APP_ID" | b64url)"
sig="$(printf '%s' "$header.$payload" | openssl dgst -sha256 -sign "$PEM" -binary | b64url)"
jwt="$header.$payload.$sig"

curl -fsS -X POST \
  -H "Authorization: Bearer $jwt" \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/app/installations/${GITHUB_APP_INSTALLATION_ID}/access_tokens" \
  | python3 -c "import sys, json; print(json.load(sys.stdin)['token'])"
