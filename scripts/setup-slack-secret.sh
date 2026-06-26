#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/setup-slack-secret.sh webhook
  scripts/setup-slack-secret.sh bot-token [#channel]

Stores the Slack demo credential in macOS Keychain without committing or printing it.

webhook:
  Paste an incoming webhook URL from the Slack app's Incoming Webhooks page.
  Stores service: slack-webhook-url

bot-token:
  Paste a Bot User OAuth Token with chat:write.
  Stores service: slack-bot-token
  Optional #channel prints the matching server env for SLACK_CHANNEL.
EOF
}

mode="${1:-}"
channel="${2:-#coach-records}"

case "$mode" in
  webhook)
    prompt="Paste Slack incoming webhook URL: "
    service="slack-webhook-url"
    pattern='^https://hooks\.slack\.com/services/'
    ;;
  bot-token)
    prompt="Paste Slack bot token (xoxb-...): "
    service="slack-bot-token"
    pattern='^xoxb-'
    ;;
  -h|--help|help|"")
    usage
    exit 0
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

printf "%s" "$prompt" >&2
stty -echo
IFS= read -r secret
stty echo
printf "\n" >&2

if ! [[ "$secret" =~ $pattern ]]; then
  echo "ERROR: input does not look like a valid Slack $mode credential" >&2
  exit 1
fi

security add-generic-password -U -a "$USER" -s "$service" -w "$secret" >/dev/null
echo "Stored Slack credential in Keychain service: $service"

if [ "$mode" = "bot-token" ]; then
  echo "Restart with: SLACK_CHANNEL='$channel' AIRPLANE_WEB_ADDR=127.0.0.1:8099 ./run.sh web"
else
  echo "Restart with: AIRPLANE_WEB_ADDR=127.0.0.1:8099 ./run.sh web"
fi
