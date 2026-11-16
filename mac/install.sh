#!/bin/bash
# Mac üçün quraşdırma: təsadüfi gizli kanal yaradır və Mac açılanda avtomatik işə düşən xidmət qurur.
set -e

DIR="$HOME/.securitylock"
PLIST="$HOME/Library/LaunchAgents/com.securitylock.listener.plist"
SRC="$(cd "$(dirname "$0")" && pwd)"

mkdir -p "$DIR" "$HOME/Library/LaunchAgents"
cp "$SRC/lock-listener.sh" "$DIR/lock-listener.sh"
chmod +x "$DIR/lock-listener.sh"

# Hər istifadəçi üçün özünün təsadüfi kanalı (mövcuddursa dəyişmir)
if [ ! -s "$DIR/topic" ]; then
  openssl rand -hex 16 > "$DIR/topic"
fi
TOPIC="$(tr -d '[:space:]' < "$DIR/topic")"

cat > "$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.securitylock.listener</string>
  <key>ProgramArguments</key>
  <array><string>/bin/bash</string><string>$DIR/lock-listener.sh</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>ProcessType</key><string>Background</string>
  <key>Nice</key><integer>10</integer>
</dict>
</plist>
EOF

# Köhnə nüsxə varsa dayandır, yenisini işə sal
launchctl bootout "gui/$(id -u)" "$PLIST" 2>/dev/null || true
launchctl bootstrap "gui/$(id -u)" "$PLIST"

echo
echo "Quraşdırıldı. iPhone Shortcut-da 'Get Contents of URL' üçün bu linki yazın:"
echo
echo "  https://ntfy.sh/${TOPIC}/publish?message=lock"
echo
echo "ƏHƏMİYYƏTLİ: System Settings > Lock Screen bölməsində"
echo "'Require password after screen saver begins or display is turned off'"
echo "parametrini 'Immediately' edin, əks halda ekran sönər, amma kilidlənməz."
