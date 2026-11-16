#!/bin/bash
# ntfy kanalını dinləyir; "lock" mesajı gələndə Mac-i kilidləyir.
TOPIC="$(tr -d '[:space:]' < "$HOME/.securitylock/topic")"

while true; do
  # Hər dövrdə yalnız indidən sonrakı mesajlar alınır (köhnə əmrlər təkrar işləməsin).
  # --max-time: bağlantı səssiz ölərsə, saatda bir yenidən qoşulur.
  curl -sN --connect-timeout 10 --max-time 3600 \
    "https://ntfy.sh/${TOPIC}/raw?since=$(date +%s)" 2>/dev/null |
  while IFS= read -r line; do
    msg="$(printf '%s' "$line" | tr -d '[:space:]')"
    if [ "$msg" = "lock" ]; then
      pmset displaysleepnow
    fi
  done
  sleep 10
done
