# SecurityLock

A tiny background app (Rust) that instantly locks your Windows PC in two ways:

1. **Panic corner:** slam the mouse cursor into the **bottom-right corner** of the screen and the PC locks. Moving to the corner at normal speed does nothing.
2. **Remote lock from an iPhone:** one tap in the Shortcuts app, on any network (home Wi-Fi, mobile data, anywhere). No VPN required.

It is lightweight: about 10 MB of RAM and near-zero CPU when idle. It runs at below-normal priority, opens no console window, and **opens no ports** on your computer.

> The app can only **lock** the machine. It cannot unlock it or run any other command.

## How it works

Remote locking uses the free [ntfy.sh](https://ntfy.sh) pub/sub service:

```text
iPhone Shortcut --HTTPS--> ntfy.sh <--HTTPS (outbound)-- Windows app --> lock
```

The app listens on a random, secret topic. When a `lock` message arrives on it, Windows locks. No router configuration or port forwarding is needed.

## Install (Windows)

Requires Rust ([rustup.rs](https://rustup.rs)).

```powershell
git clone <repo-url>
cd SecurityLock
cargo build --release
```

The binary is created at `target\release\win_iphone_guard.exe`.

### Start automatically at login

In PowerShell (copies the exe to a fixed folder and adds a Startup shortcut):

```powershell
$dir = Join-Path $env:LOCALAPPDATA 'SecurityLock'
New-Item -ItemType Directory -Force $dir | Out-Null
Copy-Item .\target\release\win_iphone_guard.exe $dir -Force
$exe = Join-Path $dir 'win_iphone_guard.exe'

$lnk = Join-Path ([Environment]::GetFolderPath('Startup')) 'SecurityLock.lnk'
$s = (New-Object -ComObject WScript.Shell).CreateShortcut($lnk)
$s.TargetPath = $exe; $s.WorkingDirectory = $dir; $s.WindowStyle = 7; $s.Save()

Start-Process $exe
```

On first run the app creates `ntfy_topic.txt` next to the exe. Its content is your secret topic:

```powershell
Get-Content "$env:LOCALAPPDATA\SecurityLock\ntfy_topic.txt"
```

To stop the app, end the `win_iphone_guard.exe` process in Task Manager.

## iPhone Shortcut

1. Open **Shortcuts** → create a new shortcut → add the **Get Contents of URL** action.
2. Set the URL (method **GET**, no body):

   ```text
   https://ntfy.sh/<YOUR-SECRET-TOPIC>/publish?message=lock
   ```

3. Add it to the Home Screen, or bind it to the **Action Button** or **Back Tap**.
4. For automatic locking: **Automation → Leave** (when you leave a location) → run the shortcut with **Run Immediately**.

The message usually arrives within a few seconds.

## macOS

The Mac version needs no Rust: it is a small shell script in [mac/](mac/). Run `bash mac/install.sh`. It generates a random topic, installs a LaunchAgent that starts at login, and prints the URL for your iPhone Shortcut. On the Mac, set **System Settings → Lock Screen → Require password → Immediately**.

> The macOS scripts have not been tested on a real Mac yet.

## Security

- The topic is a 128-bit random value. **Keep it secret**: it is the key that triggers the lock.
- Anyone who knows the URL can only **lock** your PC, nothing more. The app recognizes nothing but the exact text `lock`.
- Messages older than 120 seconds are ignored (replay protection).
- Messages pass through the third-party ntfy.sh server (over HTTPS). For more privacy, run your own ntfy server and change the `NTFY_SERVER` constant in `src/main.rs`.
- If the URL leaks: delete `ntfy_topic.txt` and restart the app to get a new topic, then update the URL in your Shortcut.

## Configuration

Constants at the top of [src/main.rs](src/main.rs):

| Constant | Meaning |
| --- | --- |
| `CORNER_PX` | Size of the corner zone, in pixels |
| `FAST_WINDOW_MS`, `FAST_MIN_DISTANCE_PX` | "Fast slam" rule: the cursor must travel at least this many pixels within this time window before reaching the corner |
| `NTFY_SERVER` | ntfy server to use |
| `NTFY_MAX_AGE_SECS` | Maximum accepted message age |

On multi-monitor setups only the bottom-right corner of the **primary monitor** is watched.

## Why no Bluetooth proximity lock?

An iPhone connects to Windows over classic Bluetooth, and Windows exposes no signal strength (distance) for classic Bluetooth. Its BLE advertisements are anonymous and use constantly rotating addresses. A reliable proximity lock was not achievable, so the project uses the Shortcut approach instead.
