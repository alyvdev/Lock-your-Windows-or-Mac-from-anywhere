// Terminal pəncərəsi açılmır (arxa planda səssiz işləyir)
#![windows_subsystem = "windows"]

use rdev::{listen, Event, EventType};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::synchapi::CreateMutexW;
use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
use winapi::um::winbase::BELOW_NORMAL_PRIORITY_CLASS;
use winapi::um::winuser::{
    GetSystemMetrics, LockWorkStation, SetProcessDPIAware, SM_CXSCREEN, SM_CYSCREEN,
};

// --- TƏNZİMLƏMƏLƏR ---
// Hər yerdən (mobil internet daxil) kilid əmri üçün ntfy serveri
const NTFY_SERVER: &str = "ntfy.sh";
// iPhone bu mətni göndərməlidir
const NTFY_LOCK_MESSAGE: &str = "lock";
// Bundan köhnə mesajlar etibarsız sayılır (təkrar oynatmaya qarşı), saniyə
const NTFY_MAX_AGE_SECS: u64 = 120;

// Sağ aşağı künc: kənardan neçə piksel daxilində sayılır
const CORNER_PX: f64 = 5.0;
// Küncə çatmazdan əvvəlki bu müddətdə (ms) ...
const FAST_WINDOW_MS: u128 = 250;
// ... kursor ən azı bu qədər piksel məsafə qət etməlidir (sürətli çırpma)
const FAST_MIN_DISTANCE_PX: f64 = 600.0;

fn lock_windows() {
    unsafe {
        LockWorkStation();
    }
}

/// Gizli dəyəri oxuyur; yoxdursa təsadüfi yaradıb exe-nin yanındakı fayla yazır.
fn load_or_create_token(file_name: &str) -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(file_name)))
        .unwrap_or_else(|| file_name.into());

    if let Ok(t) = std::fs::read_to_string(&path) {
        let t = t.trim().to_string();
        if t.len() >= 16 {
            return t;
        }
    }
    let mut token = String::new();
    for _ in 0..2 {
        token.push_str(&format!("{:016x}", RandomState::new().build_hasher().finish()));
    }
    let _ = std::fs::write(&path, &token);
    token
}

/// ntfy kanalını dinləyir (yalnız çıxan HTTPS bağlantısı). "lock" mesajı gələndə kilidləyir.
fn ntfy_listener(topic: String) {
    use std::io::{BufRead, BufReader};
    use std::time::{SystemTime, UNIX_EPOCH};

    let now_secs = || SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(90)) // ntfy ~45 san-dən bir "keepalive" göndərir
        .build();
    let mut last_lock = Instant::now() - Duration::from_secs(10);
    let mut wait = 5u64;

    loop {
        let url = format!("https://{}/{}/json?since={}", NTFY_SERVER, topic, now_secs());
        if let Ok(resp) = agent.get(&url).call() {
            wait = 5;
            for line in BufReader::new(resp.into_reader()).lines() {
                let Ok(line) = line else { break };
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
                if v["event"] != "message" || v["message"].as_str().map(str::trim) != Some(NTFY_LOCK_MESSAGE) {
                    continue;
                }
                let age = now_secs().saturating_sub(v["time"].as_u64().unwrap_or(0));
                if age <= NTFY_MAX_AGE_SECS && last_lock.elapsed() > Duration::from_secs(3) {
                    last_lock = Instant::now();
                    lock_windows();
                }
            }
        }
        // Bağlantı kəsildi və ya alınmadı: gözləyib yenidən qoşul (maks. 60 san)
        std::thread::sleep(Duration::from_secs(wait));
        wait = (wait * 2).min(60);
    }
}

fn main() {
    // Proqram artıq işləyirsə (adlı mutex mövcuddur), ikinci nüsxə dərhal çıxır
    let name: Vec<u16> = "Local\\SecurityLockWinIphoneGuard\0".encode_utf16().collect();
    let _mutex = unsafe {
        let h = CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr());
        if h.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }
        h
    };

    // Koordinatların fiziki piksellərlə uyğun gəlməsi üçün (DPI miqyası)
    let (screen_w, screen_h) = unsafe {
        // Arxa plan proses: digər proqramlarla CPU üçün yarışmasın
        SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
        SetProcessDPIAware();
        (
            GetSystemMetrics(SM_CXSCREEN) as f64,
            GetSystemMetrics(SM_CYSCREEN) as f64,
        )
    };
    println!("Sistem aktivdir. Ekran: {} x {} px", screen_w, screen_h);

    // Hər yerdən kilid üçün ntfy dinləyicisi
    let topic = load_or_create_token("ntfy_topic.txt");
    std::thread::spawn(move || ntfy_listener(topic));

    // Siçan: sağ aşağı küncə SÜRƏTLİ çırpma (əsas axında işləyir)
    let mut last_trigger = Instant::now() - Duration::from_secs(10);
    let mut trail: VecDeque<(Instant, f64, f64)> = VecDeque::new();

    if let Err(error) = listen(move |event: Event| {
        if let EventType::MouseMove { x, y } = event.event_type {
            let now = Instant::now();
            trail.push_back((now, x, y));
            while let Some(&(t, _, _)) = trail.front() {
                if now.duration_since(t).as_millis() > FAST_WINDOW_MS {
                    trail.pop_front();
                } else {
                    break;
                }
            }

            let in_corner = x >= (screen_w - CORNER_PX) && y >= (screen_h - CORNER_PX);
            if in_corner && last_trigger.elapsed() > Duration::from_secs(3) {
                let (_, ox, oy) = trail[0];
                let dist = ((x - ox).powi(2) + (y - oy).powi(2)).sqrt();
                if dist >= FAST_MIN_DISTANCE_PX {
                    last_trigger = Instant::now();
                    lock_windows();
                }
            }
        }
    }) {
        eprintln!("Siçan dinləyici xətası: {:?}", error);
    }
}
