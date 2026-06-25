//! One per-user data directory for everything the app persists (settings, the
//! evidence log, downloaded espeak voices). Cross-platform:
//!   Windows      -> %APPDATA%\lang-sprint
//!   Linux/macOS  -> $XDG_DATA_HOME/lang-sprint, else ~/.local/share/lang-sprint
//!
//! Use this everywhere instead of hand-rolling $HOME paths, so a Windows build
//! actually finds a place to write.

use std::path::PathBuf;

pub fn data_root() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("lang-sprint"))
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .map(|base| base.join("lang-sprint"))
    }
}
