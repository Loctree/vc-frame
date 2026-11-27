//!
//! # This module contain everything you'll need to access local system paths
//! containing configuration and layouts

use crate::consts::{SYSTEM_DEFAULT_CONFIG_DIR, ZELLIJ_PROJ_DIR};

use std::{path::Path, path::PathBuf};

#[cfg(not(windows))]
use crate::home_unix as platform;
#[cfg(windows)]
use crate::home_windows as platform;

#[cfg(not(test))]
/// Goes through a predefined list and returns the first usable config dir.
///
/// Prefer a directory that already has `config.kdl` (so an empty
/// `~/.config/vc-frame` cannot shadow the frontier install). Fall back to the
/// first directory that merely exists.
pub fn find_default_config_dir() -> Option<PathBuf> {
    let dirs: Vec<PathBuf> = default_config_dirs().into_iter().flatten().collect();
    dirs.iter()
        .find(|p| p.join("config.kdl").is_file())
        .cloned()
        .or_else(|| dirs.into_iter().find(|p| p.exists()))
}

#[cfg(test)]
pub fn find_default_config_dir() -> Option<PathBuf> {
    None
}

/// Order in which config directories are checked.
///
/// `home` first, then the installed Vibecrafted Frame config directory,
/// then the vibecrafted frontier install path, then XDG and the
/// system default. Env `VC_FRAME_CONFIG_DIR` is handled by the CLI layer before
/// this list is consulted.
pub fn default_config_dirs() -> Vec<Option<PathBuf>> {
    default_config_dirs_from(home_config_dir(), Some(xdg_config_dir()))
}

fn default_config_dirs_from(home: Option<PathBuf>, xdg: Option<PathBuf>) -> Vec<Option<PathBuf>> {
    let vibecrafted = home.as_deref().and_then(vibecrafted_config_dir_from_home);
    let frontier = home.as_deref().and_then(frontier_config_dir_from_home);
    vec![
        home,
        vibecrafted,
        frontier,
        xdg,
        Some(Path::new(SYSTEM_DEFAULT_CONFIG_DIR).to_path_buf()),
    ]
}

fn vibecrafted_config_dir_from_home(home: &Path) -> Option<PathBuf> {
    // home_config_dir returns ~/.config/vc-frame — climb to ~/.config
    home.parent().map(|cfg| cfg.join("vibecrafted/vc-frame"))
}

fn frontier_config_dir_from_home(home: &Path) -> Option<PathBuf> {
    // home_config_dir returns ~/.config/vc-frame — climb to ~/.config
    home.parent()
        .map(|cfg| cfg.join("vetcoders/frontier/vc-frame"))
}

/// Installed Vibecrafted Frame config directory: `~/.config/vibecrafted/vc-frame`.
/// Used when vc-frame is launched directly without wrapper env overrides.
pub fn vibecrafted_config_dir() -> Option<PathBuf> {
    platform::home_config_dir().map(|home| vibecrafted_config_dir_from_home(&home).unwrap_or(home))
}

/// Operator frontier install: `~/.config/vetcoders/frontier/vc-frame`.
/// Present on machines launched through vibecrafted; must beat an empty
/// legacy home dir so configless GUI sessions land on real chrome.
pub fn frontier_config_dir() -> Option<PathBuf> {
    platform::home_config_dir().map(|home| frontier_config_dir_from_home(&home).unwrap_or(home))
}

/// Looks for an existing dir, uses that, else returns a
/// dir matching the config spec.
pub fn get_default_data_dir() -> PathBuf {
    [xdg_data_dir(), platform::system_data_dir()]
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(xdg_data_dir)
}

pub fn xdg_config_dir() -> PathBuf {
    ZELLIJ_PROJ_DIR.config_dir().to_owned()
}

pub fn xdg_data_dir() -> PathBuf {
    ZELLIJ_PROJ_DIR.data_dir().to_owned()
}

pub fn home_config_dir() -> Option<PathBuf> {
    platform::home_config_dir()
}

pub fn try_create_home_config_dir() {
    platform::try_create_home_config_dir()
}

pub fn system_data_dir() -> PathBuf {
    platform::system_data_dir()
}

pub fn get_layout_dir(config_dir: Option<PathBuf>) -> Option<PathBuf> {
    config_dir.map(|dir| dir.join("layouts"))
}

pub fn default_layout_dir() -> Option<PathBuf> {
    find_default_config_dir().map(|dir| dir.join("layouts"))
}

pub fn get_theme_dir(config_dir: Option<PathBuf>) -> Option<PathBuf> {
    config_dir.map(|dir| dir.join("themes"))
}

pub fn default_theme_dir() -> Option<PathBuf> {
    find_default_config_dir().map(|dir| dir.join("themes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_dirs_places_vibecrafted_before_frontier() {
        let home = PathBuf::from("/Users/tester/.config/vc-frame");
        let xdg = PathBuf::from("/Users/tester/.config/vc-frame-xdg");
        let dirs: Vec<PathBuf> = default_config_dirs_from(Some(home.clone()), Some(xdg.clone()))
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(
            dirs,
            vec![
                home,
                PathBuf::from("/Users/tester/.config/vibecrafted/vc-frame"),
                PathBuf::from("/Users/tester/.config/vetcoders/frontier/vc-frame"),
                xdg,
                PathBuf::from(SYSTEM_DEFAULT_CONFIG_DIR),
            ]
        );
    }
}
