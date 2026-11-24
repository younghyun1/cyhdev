//! Language and font pack staging against disposable source trees.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use super::{NEXT_DIRECTORY, TestResult};
use crate::eu5_web::{require_checkout, stage_packs};

const LANGUAGE_CODES: [&str; 10] = [
    "fr", "de", "es", "pt-br", "pl", "ru", "tr", "ja", "ko", "zh-hans",
];
const SCRIPT_FONTS: [&str; 8] = [
    "plex-sans-kr",
    "plex-sans-jp",
    "sans-sc",
    "sans-tc",
    "noto-sans-arabic",
    "noto-sans-devanagari",
    "noto-sans-bengali",
    "noto-sans-telugu",
];

/// Every file a complete EU5 checkout holds, relative to the submodule root.
pub(super) fn required_files() -> Vec<String> {
    let mut files = vec![
        "Cargo.toml".to_owned(),
        "Cargo.lock".to_owned(),
        "web/index.html".to_owned(),
        "assets/eu5-locations.bitcode.zst".to_owned(),
        "assets/eu5-indexes.bitcode.zst".to_owned(),
    ];
    files.extend(
        LANGUAGE_CODES
            .iter()
            .map(|code| format!("assets/lang/{code}.26547371.bitcode.zst")),
    );
    files.extend(
        SCRIPT_FONTS
            .iter()
            .map(|name| format!("assets/fonts/{name}.ttf")),
    );
    files
}

/// Writes a complete source tree whose file contents are unique per path.
pub(super) fn write_source(source: &Path) -> TestResult {
    for relative in required_files() {
        let path = source.join(&relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, format!("content of {relative}\n"))?;
    }
    Ok(())
}

/// A unique scratch directory removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("cyhdev-eu5-stage-test-{}-{id}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }

    fn source(&self) -> PathBuf {
        self.0.join("source")
    }

    fn application(&self) -> PathBuf {
        self.0.join("application")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.0);
    }
}

fn staging_error(scratch: &Scratch) -> Result<String, Box<dyn std::error::Error>> {
    match stage_packs(&scratch.source(), &scratch.application()) {
        Ok(()) => Err("incomplete pack set was staged".into()),
        Err(error) => Ok(error.to_string()),
    }
}

fn names(directory: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(directory)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}

#[test]
fn staging_copies_every_language_pack_and_script_font_unchanged() -> TestResult {
    let scratch = Scratch::new()?;
    write_source(&scratch.source())?;
    // Files that are not packs for a known language must stay behind.
    let lang = scratch.source().join("assets/lang");
    fs::write(lang.join("zh.26547371.bitcode.zst"), "unknown code\n")?;
    fs::write(lang.join("README.md"), "notes\n")?;
    fs::write(lang.join("ko.latest.bitcode.zst"), "not a build id\n")?;
    fs::write(
        scratch.source().join("assets/fonts/plex-mono-regular.ttf"),
        "embedded face\n",
    )?;

    stage_packs(&scratch.source(), &scratch.application())?;

    let mut expected_packs = LANGUAGE_CODES
        .iter()
        .map(|code| format!("{code}.26547371.bitcode.zst"))
        .collect::<Vec<_>>();
    expected_packs.sort();
    assert_eq!(names(&scratch.application().join("lang"))?, expected_packs);
    let mut expected_fonts = SCRIPT_FONTS
        .iter()
        .map(|name| format!("{name}.ttf"))
        .collect::<Vec<_>>();
    expected_fonts.sort();
    assert_eq!(names(&scratch.application().join("fonts"))?, expected_fonts);
    for name in &expected_packs {
        assert_eq!(
            fs::read(scratch.application().join("lang").join(name))?,
            fs::read(lang.join(name))?
        );
    }
    assert_eq!(
        fs::read(scratch.application().join("fonts/sans-sc.ttf"))?,
        fs::read(scratch.source().join("assets/fonts/sans-sc.ttf"))?
    );
    Ok(())
}

#[test]
fn missing_or_empty_script_font_is_rejected_with_its_path() -> TestResult {
    let scratch = Scratch::new()?;
    write_source(&scratch.source())?;
    let font = scratch.source().join("assets/fonts/noto-sans-telugu.ttf");
    fs::write(&font, "")?;
    assert!(staging_error(&scratch)?.contains("noto-sans-telugu.ttf is missing or empty"));
    fs::remove_file(&font)?;
    assert!(staging_error(&scratch)?.contains("noto-sans-telugu.ttf is missing or empty"));
    Ok(())
}

#[test]
fn missing_language_pack_directory_or_pack_is_rejected() -> TestResult {
    let scratch = Scratch::new()?;
    write_source(&scratch.source())?;
    fs::remove_file(scratch.source().join("assets/lang/ko.26547371.bitcode.zst"))?;
    assert!(staging_error(&scratch)?.contains("exactly one `ko.<build>.bitcode.zst`"));

    fs::remove_dir_all(scratch.source().join("assets/lang"))?;
    assert!(staging_error(&scratch)?.contains("language packs are missing"));
    Ok(())
}

#[test]
fn stale_or_mixed_pack_builds_are_rejected() -> TestResult {
    let scratch = Scratch::new()?;
    write_source(&scratch.source())?;
    let lang = scratch.source().join("assets/lang");
    fs::write(lang.join("de.26547372.bitcode.zst"), "stale\n")?;
    assert!(staging_error(&scratch)?.contains("exactly one `de.<build>.bitcode.zst`"));

    fs::remove_file(lang.join("de.26547371.bitcode.zst"))?;
    assert!(staging_error(&scratch)?.contains("different builds"));
    Ok(())
}

#[test]
fn checkout_requires_packs_and_fonts_next_to_the_data_bundles() -> TestResult {
    let scratch = Scratch::new()?;
    let root = scratch.0.join("root");
    let source = root.join("vendor/eu5-location-filter");
    write_source(&source)?;
    require_checkout(&root)?;

    fs::remove_file(source.join("assets/fonts/sans-tc.ttf"))?;
    match require_checkout(&root) {
        Ok(()) => return Err("checkout without a font pack was accepted".into()),
        Err(error) => assert!(error.to_string().contains("sans-tc.ttf"), "{error}"),
    }
    write_source(&source)?;
    fs::remove_file(source.join("assets/lang/zh-hans.26547371.bitcode.zst"))?;
    match require_checkout(&root) {
        Ok(()) => return Err("checkout without a language pack was accepted".into()),
        Err(error) => assert!(error.to_string().contains("zh-hans"), "{error}"),
    }
    Ok(())
}
