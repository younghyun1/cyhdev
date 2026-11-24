//! Language and font packs that the browser frame fetches lazily.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{TaskError, TaskResult};

/// Pack codes of the ten non-English game languages, as in the application's
/// `GameLanguage::code`; English text is embedded and has no pack.
const LANGUAGE_CODES: [&str; 10] = [
    "fr", "de", "es", "pt-br", "pl", "ru", "tr", "ja", "ko", "zh-hans",
];

/// File stems of the script font packs under `assets/fonts/`. The embedded
/// IBM Plex faces are compiled into the application and are never staged.
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

/// Splits `<code>.<build>.bitcode.zst` into its code and decimal build ID.
/// The shape mirrors the path check of the browser host, so a name accepted
/// here is a name the frame can fetch.
fn parse_pack_name(name: &str) -> Option<(&str, &str)> {
    let (code, build) = name.strip_suffix(".bitcode.zst")?.split_once('.')?;
    let code_ok =
        (2..=16).contains(&code.len()) && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'-');
    let build_ok = (1..=20).contains(&build.len()) && build.bytes().all(|b| b.is_ascii_digit());
    (code_ok && build_ok).then_some((code, build))
}

/// Lists the language packs to stage: exactly one file per known language
/// code, all from the same build. Files for other codes are ignored, matching
/// the desktop build script; two builds of one language or a mix of builds
/// would pair a pack with the wrong dataset, so they are errors.
pub(super) fn language_packs(directory: &Path) -> TaskResult<Vec<PathBuf>> {
    let entries = fs::read_dir(directory).map_err(|error| {
        TaskError(format!(
            "EU5 language packs are missing: cannot read {}: {error}; generate them with the application's language-pack command",
            directory.display()
        ))
    })?;
    let mut found: Vec<(String, String, PathBuf)> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            TaskError(format!("could not list {}: {error}", directory.display()))
        })?;
        let file_name = entry.file_name();
        let Some((code, build)) = file_name.to_str().and_then(parse_pack_name) else {
            continue;
        };
        if LANGUAGE_CODES.contains(&code) && entry.path().is_file() {
            found.push((code.to_owned(), build.to_owned(), entry.path()));
        }
    }
    let mut selected = Vec::with_capacity(LANGUAGE_CODES.len());
    let mut build_id: Option<&str> = None;
    for code in LANGUAGE_CODES {
        let mut matches = found.iter().filter(|(found_code, ..)| found_code == code);
        let (Some((_, build, path)), None) = (matches.next(), matches.next()) else {
            return Err(TaskError(format!(
                "{} must hold exactly one `{code}.<build>.bitcode.zst`; remove stale packs or regenerate them",
                directory.display()
            )));
        };
        if build_id.is_some_and(|expected| expected != build.as_str()) {
            return Err(TaskError(format!(
                "language packs in {} come from different builds; regenerate them together",
                directory.display()
            )));
        }
        build_id = Some(build.as_str());
        selected.push(path.clone());
    }
    Ok(selected)
}

/// Resolves one script font pack, which must be a non-empty regular file: the
/// desktop build script stages an empty file for a missing face, and an empty
/// file served to the browser would only fail later in the frame.
fn font_source(source: &Path, name: &str) -> TaskResult<PathBuf> {
    let path = source.join("assets/fonts").join(format!("{name}.ttf"));
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() && metadata.len() > 0 => Ok(path),
        _ => Err(TaskError(format!(
            "EU5 font pack {} is missing or empty; generate the font packs with the application's font-pack command",
            path.display()
        ))),
    }
}

fn copy_asset(from: &Path, to: &Path) -> TaskResult<()> {
    fs::copy(from, to).map(drop).map_err(|error| {
        TaskError(format!(
            "could not stage {} to {}: {error}",
            from.display(),
            to.display()
        ))
    })
}

/// Copies the language packs unchanged into `lang/` and the script fonts into
/// `fonts/`, the two directories the frame fetches from lazily.
pub(crate) fn stage_packs(source: &Path, application: &Path) -> TaskResult<()> {
    let packs = language_packs(&source.join("assets/lang"))?;
    let fonts = SCRIPT_FONTS
        .iter()
        .map(|name| font_source(source, name).map(|path| (path, format!("{name}.ttf"))))
        .collect::<TaskResult<Vec<_>>>()?;
    for (directory, files) in [
        (
            "lang",
            packs
                .into_iter()
                .filter_map(|path| {
                    let name = path.file_name()?.to_str()?.to_owned();
                    Some((path, name))
                })
                .collect::<Vec<_>>(),
        ),
        ("fonts", fonts),
    ] {
        let target = application.join(directory);
        fs::create_dir_all(&target).map_err(|error| {
            TaskError(format!(
                "could not create staged EU5 directory {}: {error}",
                target.display()
            ))
        })?;
        for (path, name) in files {
            copy_asset(&path, &target.join(name))?;
        }
    }
    Ok(())
}

/// Requires every script font pack to be present and non-empty.
pub(super) fn script_fonts(source: &Path) -> TaskResult<()> {
    for name in SCRIPT_FONTS {
        font_source(source, name)?;
    }
    Ok(())
}
