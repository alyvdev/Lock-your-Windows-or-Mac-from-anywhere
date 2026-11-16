# SecurityLock

Windows kompüterini iki yolla dərhal kilidləyən kiçik arxa plan proqramı (Rust):

1. **Panika küncü:** siçan kursorunu ekranın **sağ aşağı küncünə sürətlə çırpanda** kompüter kilidlənir. Normal sürətlə küncə getmək kilidləmir.
2. **iPhone-dan uzaqdan kilid:** iPhone Shortcuts-dan bir toxunuşla, Wi-Fi və ya mobil internet fərqi olmadan. VPN lazım deyil.

Proqram resursu az işlədir: təxminən 10 MB RAM, boş vaxtda CPU demək olar ki, 0. Aşağı prioritetlə işləyir, konsol pəncərəsi açmır və kompüterdə **heç bir port açmır**.

> Proqram yalnız **kilidləyir**. Kompüteri açmaq və ya başqa əmr icra etmək mümkün deyil.

## Necə işləyir

Uzaqdan kilid [ntfy.sh](https://ntfy.sh) pulsuz mesaj servisi ilə işləyir:

```
iPhone Shortcut --HTTPS--> ntfy.sh <--HTTPS (çıxan bağlantı)-- Windows proqramı --> kilid
```

Proqram təsadüfi gizli kanalı dinləyir. Kanala `lock` mesajı gələndə Windows kilidlənir. Router ayarı və ya port yönləndirmə lazım deyil.

## Quraşdırma (Windows)

Rust ([rustup.rs](https://rustup.rs)) lazımdır.

```powershell
git clone <repo-url>
cd SecurityLock
cargo build --release
```

Hazır proqram `target\release\win_iphone_guard.exe` ünvanında yaranır.

### Avtomatik başlama

PowerShell-də (exe-ni sabit qovluğa köçürür və Windows açılanda işə salır):

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

İlk işə düşəndə `ntfy_topic.txt` faylı yaranır. Gizli kanal adınız budur:

```powershell
Get-Content "$env:LOCALAPPDATA\SecurityLock\ntfy_topic.txt"
```

Proqramı dayandırmaq üçün Task Manager-də `win_iphone_guard.exe` prosesini bitirin.

## iPhone Shortcut

1. **Shortcuts** → yeni shortcut → **Get Contents of URL** əməliyyatı.
2. URL (method **GET**, body yoxdur):
   ```
   https://ntfy.sh/<SİZİN-KANAL-ADI>/publish?message=lock
   ```
3. Ana ekrana əlavə edin və ya **Action Button** / **Back Tap** ilə bağlayın.
4. Avtomatik kilid üçün: **Automation → Leave** (məkandan çıxanda) → shortcut-u **Run Immediately** ilə işə salın.

Mesaj bir neçə saniyəyə çatır.

## macOS

Mac üçün Rust lazım deyil: [mac/](mac/) qovluğunda kiçik skript var. `bash mac/install.sh` işlədin, o təsadüfi kanal yaradır, Mac açılanda işləyən xidmət qurur və iPhone üçün linki göstərir. Mac-də **System Settings → Lock Screen → Require password → Immediately** olmalıdır.

## Təhlükəsizlik

- Kanal adı 128 bitlik təsadüfi dəyərdir. **Gizli saxlayın**, o, kompüteri kilidləmək açarıdır.
- Linki bilən şəxs yalnız kompüteri **kilidləyə** bilər, daha heç nə. Proqram yalnız dəqiq `lock` mətnini tanıyır.
- 120 saniyədən köhnə mesajlar rədd edilir.
- Mesajlar üçüncü tərəf serveri olan ntfy.sh-dan keçir (HTTPS ilə). Daha yüksək məxfilik üçün öz ntfy serverinizi qura və `src/main.rs`-dəki `NTFY_SERVER` sabitini dəyişə bilərsiniz.
- Link sızsa: `ntfy_topic.txt` faylını silib proqramı yenidən başladın, yeni kanal yaranacaq, Shortcut-da linki yeniləyin.

## Tənzimləmələr

[src/main.rs](src/main.rs) başlığındakı sabitlər:

| Sabit | Mənası |
|---|---|
| `CORNER_PX` | Künc zonasının ölçüsü (piksel) |
| `FAST_WINDOW_MS`, `FAST_MIN_DISTANCE_PX` | "Sürətli çırpma" şərti: küncə çatmazdan əvvəlki bu müddətdə bu qədər piksel qət edilməlidir |
| `NTFY_SERVER` | ntfy serveri |
| `NTFY_MAX_AGE_SECS` | Mesajın maksimum yaşı |

Çoxmonitorlu sistemdə proqram yalnız **əsas monitorun** sağ aşağı küncünü izləyir.

## Niyə Bluetooth yoxdur?

iPhone Windows-a klassik Bluetooth ilə qoşulur və Windows klassik Bluetooth üçün siqnal gücü (məsafə) vermir. BLE reklamları isə adsızdır və ünvanı daim dəyişir. Ona görə yaxınlığa əsaslanan kilid etibarlı alınmadı və Shortcut yoluna keçildi.
