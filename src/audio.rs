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
        let mut wav = std::env::temp_dir();
        wav.push(format!(
            "lang-sprint-tts-{}-{}.wav",
            std::process::id(),
            stamp
        ));

        // espeak-ng [--path <dir>] -v <voice> -s 150 -w <file> <text>
        let mut cmd = std::process::Command::new("espeak-ng");
        if let Some(dir) = espeak_data_dir() {
            cmd.arg("--path").arg(dir);
        }
        cmd.arg("-v")
            .arg(voice)
            .arg("-s")
            .arg("150")
            .arg("-w")
            .arg(&wav)
            .arg(&text);

        match cmd.status() {
            Ok(s) if s.success() => {
                if let Ok(stream_handle) = OutputStreamBuilder::open_default_stream() {
                    let mixer = stream_handle.mixer();
                    match File::open(&wav) {
                        Ok(file) => match rodio::play(&mixer, BufReader::new(file)) {
                            Ok(sink) => {
                                sink.set_volume(volume);
                                sink.sleep_until_end();
                            }
                            Err(e) => eprintln!("[tts] play failed: {e}"),
                        },
                        Err(e) => eprintln!("[tts] couldn't open synthesized wav: {e}"),
                    }
                }
            }
            Ok(s) => eprintln!("[tts] espeak-ng exited with {s} for voice {voice:?}"),
            Err(e) => eprintln!("[tts] couldn't run espeak-ng (is it installed?): {e}"),
        }

        let _ = std::fs::remove_file(&wav);
        cleanup_id(&id);
    });
}

/// App's language string -> espeak-ng voice code. Add a line per language;
/// unknown -> None -> silent.
#[cfg(not(target_arch = "wasm32"))]
fn espeak_voice(lang: &str) -> Option<&'static str> {
    match lang {
        "georgian" => Some("ka"),
        "russian" => Some("ru"),
        _ => None,
    }
}

/// A private, app-owned espeak data dir, IF it exists and looks valid (this is
/// where a future "Install voices" download would land). `None` -> use the
/// system install's default data path.
#[cfg(not(target_arch = "wasm32"))]
pub fn espeak_data_dir() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })?;
    let dir = base.join("lang-sprint").join("espeak-ng-data");
    // treat as present only if the shared phoneme table is there
    dir.join("phontab").exists().then_some(dir)
}

/// Is the espeak-ng engine runnable at all?
#[cfg(not(target_arch = "wasm32"))]
pub fn espeak_present() -> bool {
    std::process::Command::new("espeak-ng")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Can we actually synthesize THIS language? (engine present AND its voice/data
/// resolvable, honoring the private dir). `-q` = process the text, emit no audio.
#[cfg(not(target_arch = "wasm32"))]
pub fn voice_available(lang: &str) -> bool {
    let Some(voice) = espeak_voice(lang) else {
        return false;
    };
    let mut cmd = std::process::Command::new("espeak-ng");
    if let Some(dir) = espeak_data_dir() {
        cmd.arg("--path").arg(dir);
    }
    cmd.arg("-v").arg(voice).arg("-q").arg("x");
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
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
