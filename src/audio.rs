// src/audio.rs
// Desktop audio playback for Dioxus using rodio 0.21.x
// - path-based playback (existing)
// - bytes-based playback for embedded WAVs
// - prevents multiple concurrent plays of the same file/ID

// ─── DESKTOP IMPLEMENTATION ────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
use rodio::{self, OutputStreamBuilder};

#[cfg(not(target_arch = "wasm32"))]
use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
};

// De-dup for filesystem paths (old API)
#[cfg(not(target_arch = "wasm32"))]
static PLAYING_FILES: Lazy<Mutex<HashSet<PathBuf>>> = Lazy::new(|| Mutex::new(HashSet::new()));

// De-dup for embedded audio IDs (new API)
#[cfg(not(target_arch = "wasm32"))]
static PLAYING_IDS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Desktop: fire-and-forget playback of a WAV/OGG/etc file from the filesystem.
/// - If the same absolute path is already playing, this call is ignored.
#[cfg(not(target_arch = "wasm32"))]
pub fn play_audio<P: AsRef<Path>>(path: P, volume: f32) {
    // Canonicalize so we don't get dupes like "./foo.wav" vs "foo.wav"
    let path = path.as_ref();
    let abs = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    // De-dup: if this file is already in PLAYING_FILES, bail.
    {
        let mut set = PLAYING_FILES.lock().expect("PLAYING_FILES mutex poisoned");

        if set.contains(&abs) {
            eprintln!("[audio] Already playing: {}", abs.display());
            return;
        }

        set.insert(abs.clone());
    }

    // Offload playback to a background thread so we don't block the UI.
    thread::spawn(move || {
        // Debug info so you can see what’s going on.
        if let Ok(cwd) = std::env::current_dir() {
            println!("Audio: playing {} (cwd: {})", abs.display(), cwd.display());
        } else {
            println!("Audio: playing {}", abs.display());
        }

        // 1. Open default stream
        let stream_handle = match OutputStreamBuilder::open_default_stream() {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[audio] Failed to open audio stream: {}", e);
                cleanup_playing_file(&abs);
                return;
            }
        };

        // 2. Get mixer
        let mixer = stream_handle.mixer();

        // 3. Open file
        let file = match File::open(&abs) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[audio] Failed to open {}: {}", abs.display(), e);
                cleanup_playing_file(&abs);
                return;
            }
        };

        // 4. Play via rodio::play using BufReader<File>
        match rodio::play(&mixer, BufReader::new(file)) {
            Ok(sink) => {
                sink.set_volume(volume);
                // This blocks the *audio thread*, not the UI.
                sink.sleep_until_end();
            }
            Err(e) => {
                eprintln!("[audio] Failed to play {}: {}", abs.display(), e);
            }
        }

        // 5. Remove from PLAYING_FILES so future clicks can replay the letter
        cleanup_playing_file(&abs);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn cleanup_playing_file(path: &PathBuf) {
    if let Ok(mut set) = PLAYING_FILES.lock() {
        set.remove(path);
    }
}

/// Query from the UI: is this path currently playing?
#[cfg(not(target_arch = "wasm32"))]
pub fn is_playing<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let set = PLAYING_FILES.lock().expect("PLAYING_FILES mutex poisoned");

    set.contains(&abs)
}

/// New: play from *embedded bytes*, deduped by a logical ID (e.g. "georgian/a.wav")
#[cfg(not(target_arch = "wasm32"))]
pub fn play_audio_bytes(id: &str, data: &'static [u8], volume: f32) {
    let id_str = id.to_string();

    {
        let mut set = PLAYING_IDS.lock().expect("PLAYING_IDS mutex poisoned");

        if set.contains(&id_str) {
            eprintln!("[audio] Already playing bytes: {id_str}");
            return;
        }

        set.insert(id_str.clone());
    }

    thread::spawn(move || {
        println!("[audio] playing embedded audio: {id_str}");

        // 1. Open default stream (same pattern as path-based version)
        let stream_handle = match OutputStreamBuilder::open_default_stream() {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[audio] Failed to open audio stream: {}", e);
                cleanup_id(&id_str);
                return;
            }
        };

        // 2. Get mixer
        let mixer = stream_handle.mixer();

        // 3. Wrap bytes in Cursor -> BufReader
        let cursor = Cursor::new(data);
        let reader = BufReader::new(cursor);

        // 4. Play via rodio::play using BufReader<Cursor<&[u8]>>
        match rodio::play(&mixer, reader) {
            Ok(sink) => {
                sink.set_volume(volume);
                sink.sleep_until_end();
            }
            Err(e) => {
                eprintln!("[audio] Failed to play embedded {id_str}: {e}");
            }
        }

        // 5. Remove from PLAYING_IDS so it can be replayed
        cleanup_id(&id_str);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn cleanup_id(id: &str) {
    if let Ok(mut set) = PLAYING_IDS.lock() {
        set.remove(id);
    }
}

// ─── ADD TO src/audio.rs ───────────────────────────────────────────────────
//
// Cross-platform whole-word TTS via the espeak-ng CLI (Windows + Linux/macOS).
// Windows specifics handled here: the binary is espeak-ng.exe and the installer
// often doesn't add it to PATH (so we also probe its default location and a
// bundled copy next to our exe), and CREATE_NO_WINDOW stops a console window
// from flashing on every call. Engine + data must be the same version, so a
// private/bundled dir must hold a matched pair — never data alone.

#[cfg(all(not(target_arch = "wasm32"), windows))]
const ESPEAK_EXE: &str = "espeak-ng.exe";
#[cfg(all(not(target_arch = "wasm32"), not(windows)))]
const ESPEAK_EXE: &str = "espeak-ng";

/// Resolve the espeak-ng executable: bundled next to our app, then (Windows) the
/// installer's default location, then PATH.
#[cfg(not(target_arch = "wasm32"))]
fn espeak_bin() -> std::path::PathBuf {
    use std::path::PathBuf;
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join(ESPEAK_EXE);
            if bundled.exists() {
                return bundled;
            }
        }
    }
    #[cfg(windows)]
    {
        for var in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"] {
            if let Some(pf) = std::env::var_os(var) {
                let p = PathBuf::from(pf).join("eSpeak NG").join(ESPEAK_EXE);
                if p.exists() {
                    return p;
                }
            }
        }
    }
    PathBuf::from(ESPEAK_EXE)
}

/// A `Command` for espeak-ng with the resolved binary and, on Windows, no
/// console-window flash.
#[cfg(not(target_arch = "wasm32"))]
fn espeak_command() -> std::process::Command {
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(espeak_bin());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

#[cfg(not(target_arch = "wasm32"))]
pub fn speak(lang: &str, text: &str, volume: f32) {
    let Some(voice) = espeak_voice(lang) else {
        return;
    };
    let text = text.trim().to_string();
    if text.is_empty() {
        return;
    }

    let id = format!("tts:{voice}:{text}");
    {
        let mut set = PLAYING_IDS.lock().expect("PLAYING_IDS mutex poisoned");
        if set.contains(&id) {
            return;
        }
        set.insert(id.clone());
    }

    thread::spawn(move || {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let mut wav = std::env::temp_dir(); // %TEMP% on Windows, /tmp on unix
        wav.push(format!(
            "lang-sprint-tts-{}-{}.wav",
            std::process::id(),
            stamp
        ));

        // Write the text as UTF-8 and feed espeak with -f, so the word reaches
        // the engine intact regardless of console code page / argv quirks.
        let mut txt = std::env::temp_dir();
        txt.push(format!(
            "lang-sprint-tts-{}-{}.txt",
            std::process::id(),
            stamp
        ));
        if let Err(e) = std::fs::write(&txt, text.as_bytes()) {
            eprintln!("[tts] couldn't write text file: {e}");
            cleanup_id(&id);
            return;
        }

        // [--path <dir>] -v <voice> -s 150 -w <wav> -f <txt>
        let mut cmd = espeak_command();
        if let Some(dir) = espeak_data_dir() {
            cmd.arg("--path").arg(dir);
        }
        cmd.arg("-v")
            .arg(voice)
            .arg("-s")
            .arg("150")
            .arg("-w")
            .arg(&wav)
            .arg("-f")
            .arg(&txt);

        match cmd.status() {
            Ok(s) if s.success() => match std::fs::read(&wav) {
                Ok(bytes) => {
                    if let Ok(stream_handle) = OutputStreamBuilder::open_default_stream() {
                        let mixer = stream_handle.mixer();
                        let cursor = std::io::Cursor::new(bytes);
                        match rodio::play(&mixer, std::io::BufReader::new(cursor)) {
                            Ok(sink) => {
                                sink.set_volume(volume);
                                sink.sleep_until_end();
                                std::thread::sleep(std::time::Duration::from_millis(150));
                            }
                            Err(e) => eprintln!("[tts] play failed: {e}"),
                        }
                        let _ = stream_handle;
                    }
                }
                Err(e) => eprintln!("[tts] couldn't read synthesized wav: {e}"),
            },
            Ok(s) => eprintln!("[tts] espeak-ng exited with {s} for voice {voice:?}"),
            Err(e) => eprintln!("[tts] couldn't run espeak-ng (is it installed?): {e}"),
        }

        let _ = std::fs::remove_file(&txt);
        let _ = std::fs::remove_file(&wav);
        cleanup_id(&id);
    });
}

/// App language string -> espeak-ng voice code. Add a line per language.
#[cfg(not(target_arch = "wasm32"))]
fn espeak_voice(lang: &str) -> Option<&'static str> {
    match lang {
        "georgian" => Some("ka"),
        "russian" => Some("ru"),
        _ => None,
    }
}

/// espeak data dir to pass via `--path`: a downloaded copy under our data dir,
/// else a bundled copy next to the app, else `None` (use the engine's own data).
#[cfg(not(target_arch = "wasm32"))]
pub fn espeak_data_dir() -> Option<std::path::PathBuf> {
    if let Some(root) = crate::paths::data_root() {
        let d = root.join("espeak-ng-data");
        if d.join("phontab").exists() {
            return Some(d);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let d = dir.join("espeak-ng-data");
            if d.join("phontab").exists() {
                return Some(d);
            }
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn espeak_present() -> bool {
    espeak_command()
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn voice_available(lang: &str) -> bool {
    let Some(voice) = espeak_voice(lang) else {
        return false;
    };
    let mut cmd = espeak_command();
    if let Some(dir) = espeak_data_dir() {
        cmd.arg("--path").arg(dir);
    }
    cmd.arg("-v").arg(voice).arg("-q").arg("x");
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

// ─── WASM STUBS (unchanged shape) ───────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
pub fn speak(_lang: &str, _text: &str, _volume: f32) {}
#[cfg(target_arch = "wasm32")]
pub fn espeak_data_dir() -> Option<std::path::PathBuf> {
    None
}
#[cfg(target_arch = "wasm32")]
pub fn espeak_present() -> bool {
    false
}
#[cfg(target_arch = "wasm32")]
pub fn voice_available(_lang: &str) -> bool {
    false
}
// ─── WASM STUBS ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn play_audio<P: AsRef<std::path::Path>>(_path: P, _volume: f32) {
    // no-op for now (web would use <audio> or JS)
}

#[cfg(target_arch = "wasm32")]
pub fn is_playing<P: AsRef<std::path::Path>>(_path: P) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
pub fn play_audio_bytes(_id: &str, _data: &'static [u8], _volume: f32) {
    // no-op on web
}

#[cfg(target_arch = "wasm32")]
pub fn speak(_lang: &str, _text: &str, _volume: f32) {}
#[cfg(target_arch = "wasm32")]
pub fn espeak_data_dir() -> Option<std::path::PathBuf> {
    None
}
#[cfg(target_arch = "wasm32")]
pub fn espeak_present() -> bool {
    false
}
#[cfg(target_arch = "wasm32")]
pub fn voice_available(_lang: &str) -> bool {
    false
}
