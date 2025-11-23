// src/audio.rs
// Desktop audio playback for Dioxus using rodio 0.21.x
// - prevents multiple concurrent plays of the same file
// - exposes is_playing(path) for UI

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
use rodio::OutputStreamBuilder;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    collections::HashSet,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
};

#[cfg(not(target_arch = "wasm32"))]
static PLAYING_FILES: Lazy<Mutex<HashSet<PathBuf>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Desktop: fire-and-forget playback of a WAV/OGG/etc file.
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
        let mut set = PLAYING_FILES
            .lock()
            .expect("PLAYING_FILES mutex poisoned");

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
                cleanup_playing(&abs);
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
                cleanup_playing(&abs);
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
        cleanup_playing(&abs);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn cleanup_playing(path: &PathBuf) {
    if let Ok(mut set) = PLAYING_FILES.lock() {
        set.remove(path);
    }
}

/// Query from the UI: is this path currently playing?
#[cfg(not(target_arch = "wasm32"))]
pub fn is_playing<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let set = PLAYING_FILES
        .lock()
        .expect("PLAYING_FILES mutex poisoned");

    set.contains(&abs)
}

// ─── WASM STUBS ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn play_audio<P: AsRef<std::path::Path>>(_path: P) {
    // no-op for now (web would use <audio> or JS)
}

#[cfg(target_arch = "wasm32")]
pub fn is_playing<P: AsRef<std::path::Path>>(_path: P) -> bool {
    false
}
