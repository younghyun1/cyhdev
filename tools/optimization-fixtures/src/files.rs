//! Private runtime material generated independently of production configuration.

use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};

pub const HTTP_PORT: u16 = 34901;
pub const SMTP_PORT: u16 = 3465;
pub const MANAGEMENT_PORT: u16 = 18555;
pub const PLAYER: &str = "01990000-0000-7000-8000-000000000009";

pub fn validate_runtime(path: &Path) -> anyhow::Result<()> {
    anyhow::ensure!(
        std::env::var("CYHDEV_OPT_DISPOSABLE").as_deref() == Ok("1"),
        "fixture admission is missing"
    );
    anyhow::ensure!(
        path.is_absolute()
            && path.is_dir()
            && path.canonicalize()? == path
            && path.as_os_str().as_encoded_bytes().len() < 70
            && path
                .file_name()
                .is_some_and(|p| p.to_string_lossy().starts_with("cyh-opt-")),
        "runtime must be a short, canonical cyh-opt-* directory outside the checkout"
    );
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .ok_or_else(|| anyhow::anyhow!("workspace root unavailable"))?;
    anyhow::ensure!(
        !path.starts_with(checkout),
        "runtime must be outside the checkout"
    );
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

pub fn command(command: &mut Command) -> anyhow::Result<()> {
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    anyhow::ensure!(status.success(), "fixture command failed with {status}");
    Ok(())
}

pub fn prepare(runtime: &Path) -> anyhow::Result<()> {
    anyhow::ensure!(
        !runtime.join("fixture.json").exists(),
        "fixture runtime has already been prepared"
    );
    for directory in [
        "logs",
        "data/search_index",
        "objects",
        "mail",
        "squaremap/tiles",
    ] {
        fs::create_dir_all(runtime.join(directory))?;
    }
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .ok_or_else(|| anyhow::anyhow!("workspace root unavailable"))?;
    for file in ["new_bundle_ipv4.db", "new_bundle_ipv6.db"] {
        fs::copy(
            checkout.join("rust-be-template").join(file),
            runtime.join(file),
        )?;
    }
    command(
        Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-days",
                "2",
                "-subj",
                "/CN=localhost",
                "-addext",
                "subjectAltName=DNS:localhost,IP:127.0.0.1",
                "-addext",
                "basicConstraints=critical,CA:FALSE",
                "-keyout",
            ])
            .arg(runtime.join("tls-key.pem"))
            .arg("-out")
            .arg(runtime.join("tls-cert.pem")),
    )?;
    command(
        Command::new("openssl")
            .args(["genrsa", "-traditional", "-out"])
            .arg(runtime.join("oidc-key.pem"))
            .arg("2048"),
    )?;
    for file in ["tls-key.pem", "oidc-key.pem"] {
        fs::set_permissions(runtime.join(file), fs::Permissions::from_mode(0o600))?;
    }
    let image = image::RgbImage::from_fn(96, 64, |x, y| {
        image::Rgb([(x * 2) as u8, (y * 3) as u8, 128])
    });
    image.save(runtime.join("image.png"))?;
    fs::write(
        runtime.join("demo.html"),
        "<!doctype html><html><body><h1 id='fixture-demo'>Synthetic optimization demo</h1><script>document.body.dataset.ready='true'</script></body></html>",
    )?;
    fs::write(runtime.join("demo.wasm"), b"\0asm\x01\0\0\0")?;
    fs::copy(
        runtime.join("image.png"),
        runtime.join("squaremap/tiles/fixture.png"),
    )?;
    fs::write(
        runtime.join("squaremap/tiles/players.json"),
        "{\"players\":[]}",
    )?;
    fs::write(runtime.join("squaremap/tiles/markers.json"), "[]")?;
    fs::write(
        runtime.join("squaremap/tiles/settings.json"),
        serde_json::to_vec(&serde_json::json!({"worlds":[
            {"name":"minecraft_overworld","display_name":"minecraft:overworld","type":"normal"},
            {"name":"minecraft_the_nether","display_name":"minecraft:the_nether","type":"nether"},
            {"name":"minecraft_the_end","display_name":"minecraft:the_end","type":"end"}
        ]}))?,
    )?;
    for world in [
        "minecraft_overworld",
        "minecraft_the_nether",
        "minecraft_the_end",
    ] {
        let directory = runtime.join("squaremap/tiles").join(world);
        fs::create_dir_all(&directory)?;
        fs::write(
            directory.join("settings.json"),
            serde_json::to_vec(
                &serde_json::json!({"spawn":{"x":0,"z":0},"zoom":{"max":3,"def":2,"extra":0}}),
            )?,
        )?;
        for zoom in 0..=3 {
            let tiles = directory.join(zoom.to_string());
            fs::create_dir_all(&tiles)?;
            for x in -2..=2 {
                for z in -2..=2 {
                    fs::copy(
                        runtime.join("image.png"),
                        tiles.join(format!("{x}_{z}.png")),
                    )?;
                }
            }
        }
    }
    Ok(())
}

pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn inputs(runtime: &Path) -> anyhow::Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    let mut names: Vec<String> = [
        "image.png",
        "demo.html",
        "demo.wasm",
        "tls-cert.pem",
        "tls-key.pem",
        "oidc-key.pem",
        "new_bundle_ipv4.db",
        "new_bundle_ipv6.db",
        "squaremap/tiles/settings.json",
        "squaremap/tiles/fixture.png",
        "squaremap/tiles/players.json",
        "squaremap/tiles/markers.json",
        "squaremap/tiles/minecraft_overworld/settings.json",
        "squaremap/tiles/minecraft_the_nether/settings.json",
        "squaremap/tiles/minecraft_the_end/settings.json",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for world in [
        "minecraft_overworld",
        "minecraft_the_nether",
        "minecraft_the_end",
    ] {
        for zoom in 0..=3 {
            for x in -2..=2 {
                for z in -2..=2 {
                    names.push(format!("squaremap/tiles/{world}/{zoom}/{x}_{z}.png"));
                }
            }
        }
    }
    for name in names {
        let path = runtime.join(&name);
        let metadata = path.symlink_metadata()?;
        anyhow::ensure!(
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.len() <= 512 * 1024 * 1024,
            "invalid or oversized fixture input"
        );
        let mut source = fs::File::open(path)?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let length = source.read(&mut buffer)?;
            if length == 0 {
                break;
            }
            hash.update(&buffer[..length]);
        }
        result.insert(
            name,
            hash.finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        );
    }
    Ok(result)
}
