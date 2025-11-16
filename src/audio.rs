// src/audio.rs

use dioxus::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use rodio::{self, OutputStreamBuilder};

#[cfg(not(target_arch = "wasm32"))]
use std::{fs::File, io::BufReader, path::{Path, PathBuf}, thread, time::Duration};

/// Desktop audio: start playing a file and return whether it started OK.
///
/// This returns Result so the UI can display success / error.
/// Actual playback happens on a background thread.
#[cfg(not(target_arch = "wasm32"))]
pub fn play_audio<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path_buf: PathBuf = path.as_ref().to_path_buf();

    // Just some logging context
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "<unknown cwd>".to_string());
    eprintln!(
        "[audio] Requested: {} (cwd: {})",
        path_buf.display(),
        cwd
    );

    if !path_buf.exists() {
        let msg = format!("[audio] File not found: {}", path_buf.display());
        eprintln!("{}", msg);
        return Err(msg);
    }

    // We do all the rodio setup inside a background thread,
    // so the Dioxus UI thread doesn't block.
    thread::spawn(move || {
        // 1. Open default output stream
        let stream_handle = match OutputStreamBuilder::open_default_stream() {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[audio] Failed to open audio stream: {}", e);
                return;
            }
        };

        // 2. Get mixer reference
        let mixer = stream_handle.mixer();

        // 3. Open file
        let file = match File::open(&path_buf) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "[audio] Failed to open {}: {}",
                    path_buf.display(),
                    e
                );
                return;
            }
        };

        let reader = BufReader::new(file);

        // 4. Start playback using rodio::play (rodio 0.21 style)
        match rodio::play(mixer, reader) {
            Ok(sink) => {
                sink.set_volume(0.4);
                eprintln!("[audio] Playing {}", path_buf.display());

                // Keep the stream & sink alive long enough to hear it.
                // You can tweak this depending on your clip length.
                thread::sleep(Duration::from_secs(2));
            }
            Err(e) => {
                eprintln!(
                    "[audio] Failed to play {}: {}",
                    path_buf.display(),
                    e
                );
            }
        }
    });

    // If we got this far, we successfully *started* playback.
    Ok(())
}

/// No-op stub for web/wasm so the crate still compiles.
#[cfg(target_arch = "wasm32")]
pub fn play_audio<P: AsRef<std::path::Path>>(_path: P) -> Result<(), String> {
    Err("Audio not supported on wasm build yet".to_string())
}
