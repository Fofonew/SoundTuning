#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

const PLIST_KEY: &str = "info-plist-path";
const MICROPHONE_KEY: &str = "<key>NSMicrophoneUsageDescription</key>";
const MIN_DESCRIPTION_CHARACTERS: usize = 20;

fn manifest_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn info_plist_path() -> PathBuf {
    let manifest = std::fs::read_to_string(manifest_directory().join("Cargo.toml")).unwrap();
    let macos_section = manifest
        .split("[package.metadata.packager.macos]")
        .nth(1)
        .unwrap();
    let line = macos_section
        .lines()
        .find(|line| line.trim_start().starts_with(PLIST_KEY))
        .unwrap();
    let value = line.split_once('=').unwrap().1.trim().trim_matches('"');
    manifest_directory().join(value)
}

#[test]
fn macos_bundle_declares_the_microphone_usage() {
    let path = info_plist_path();
    assert!(path.is_file(), "{}", path.display());
    let plist = std::fs::read_to_string(path).unwrap();
    let after_key = &plist[plist.find(MICROPHONE_KEY).unwrap() + MICROPHONE_KEY.len()..];
    let value = after_key
        .trim_start()
        .strip_prefix("<string>")
        .and_then(|rest| rest.split_once("</string>"))
        .map(|(value, _)| value.trim())
        .unwrap();
    assert!(
        value.chars().count() >= MIN_DESCRIPTION_CHARACTERS,
        "{value}"
    );
}
