//! Synthetic same-user terrain reads and loopback management acknowledgements.

use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
    sync::Semaphore,
};

pub async fn serve(runtime: PathBuf) -> anyhow::Result<()> {
    tokio::try_join!(
        unix(runtime.clone(), false),
        unix(runtime.clone(), true),
        crate::management::serve(runtime)
    )?;
    Ok(())
}

async fn unix(runtime: PathBuf, control: bool) -> anyhow::Result<()> {
    let path = runtime.join(if control {
        "control.sock"
    } else {
        "world.sock"
    });
    anyhow::ensure!(!path.exists(), "fixture socket already exists");
    let listener = UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    let admission = Arc::new(Semaphore::new(4));
    loop {
        let (stream, _) = listener.accept().await?;
        let permit = match admission.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let runtime = runtime.clone();
        tokio::spawn(async move {
            let result = tokio::time::timeout(Duration::from_secs(5), async {
                let mut reader = BufReader::new(stream.take(4097));
                let mut request = String::new();
                reader.read_line(&mut request).await?;
                anyhow::ensure!(
                    request.len() <= 4096 && request.ends_with('\n'),
                    "invalid socket request"
                );
                let response = if control {
                    let hidden_path = runtime.join("player-hidden.json");
                    if request == "STATUS\n" {
                        let hidden = std::fs::read_to_string(&hidden_path)?;
                        anyhow::ensure!(
                            matches!(hidden.as_str(), "0" | "1"),
                            "invalid player visibility state"
                        );
                        format!("OK\n{} {hidden}\n", crate::files::PLAYER)
                    } else if request == format!("HIDE {}\n", crate::files::PLAYER)
                        || request == format!("SHOW {}\n", crate::files::PLAYER)
                    {
                        std::fs::write(
                            &hidden_path,
                            if request.starts_with("HIDE ") {
                                "1"
                            } else {
                                "0"
                            },
                        )?;
                        "OK\n".into()
                    } else {
                        "ERROR\n".into()
                    }
                } else {
                    format!("{}\n", terrain(serde_json::from_str(&request)?, &runtime)?)
                };
                let mut stream = reader.into_inner().into_inner();
                stream.write_all(response.as_bytes()).await?;
                stream.shutdown().await?;
                Ok::<_, anyhow::Error>(())
            })
            .await;
            if !matches!(result, Ok(Ok(()))) {
                eprintln!("synthetic Minecraft socket rejected a request");
            }
            drop(permit);
        });
    }
}

fn terrain(request: Value, runtime: &std::path::Path) -> anyhow::Result<Value> {
    let kind = request["kind"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing kind"))?;
    let world = request["world"].as_str().unwrap_or("minecraft:overworld");
    let worlds = [
        ("minecraft:overworld", "default", -64, 319),
        ("minecraft:the_nether", "nether", 0, 255),
        ("minecraft:the_end", "end", 0, 255),
    ];
    let (_, preset, min_y, max_y) = worlds
        .iter()
        .find(|(name, _, _, _)| *name == world)
        .ok_or_else(|| anyhow::anyhow!("unknown dimension"))?;
    let now = chrono::Utc::now().timestamp_millis();
    if kind == "prediction_context" {
        let x = request["chunk_x"].as_i64().unwrap_or(0);
        let z = request["chunk_z"].as_i64().unwrap_or(0);
        return Ok(
            json!({"kind":kind,"world":world,"world_id":crate::files::PLAYER,"sampled_at_ms":now,"seed":42,"preset":"default",
            "profile_revision":"0000000000000001","coverage":(0..8).flat_map(|cx|(0..8).map(move|cz|json!({"chunk_x":x+cx,"chunk_z":z+cz,"state":"ungenerated"}))).collect::<Vec<_>>()}),
        );
    }
    if kind == "seed_profile" {
        let visibility = std::fs::read_to_string(runtime.join("visibility.json"))?;
        anyhow::ensure!(
            matches!(visibility.as_str(), "true" | "false"),
            "invalid visibility state"
        );
        let visible = visibility == "true";
        return Ok(
            json!({"kind":"seed_profile","world":world,"world_id":crate::files::PLAYER,"sampled_at_ms":now,"seed":42,"preset":preset,
            "profile_revision":if visible {"0000000000000001"} else {"0000000000000002"},
            "world_border":{"min_x":-4096,"min_z":-4096,"max_x":4095,"max_z":4095},
            "visibility":[{"kind":"rectangle","min_x":if visible {-4096} else {2048},"min_z":if visible {-4096} else {2048},"max_x":4095,"max_z":4095}]}),
        );
    }
    let mut reply = json!({"kind":kind,"world":world,"sampled_at_ms":now,"scanned_chunks":0,"missing_chunks":0,"truncated":false,
        "worlds":[],"blocks":[],"cells":[],"structures":[],"matches":[]});
    match kind {
        "catalog" => {
            reply["world"] = Value::Null;
            reply["worlds"] = json!(worlds.iter().map(|(world,_,min,max)|json!({"id":world,"name":format!("Synthetic {world}"),"map_id":world.replace(':',"_"),"min_y":min,"max_y":max})).collect::<Vec<_>>());
            reply["blocks"] = json!(["minecraft:stone", "minecraft:diamond_ore"]);
        }
        "area" | "blocks" => {
            let width = request["width"].as_i64().unwrap_or(1);
            let height = request["height"].as_i64().unwrap_or(1);
            anyhow::ensure!(
                (1..=8).contains(&width) && (1..=8).contains(&height) && width * height <= 64,
                "oversized terrain region"
            );
            let chunk_x = request["chunk_x"].as_i64().unwrap_or(0);
            let chunk_z = request["chunk_z"].as_i64().unwrap_or(0);
            anyhow::ensure!(
                (-1_875_000..=1_875_000).contains(&chunk_x)
                    && (-1_875_000..=1_875_000).contains(&chunk_z),
                "terrain coordinates outside the world"
            );
            let x = chunk_x * 16;
            let z = chunk_z * 16;
            reply["scanned_chunks"] = json!(width * height);
            if kind == "area" {
                let y = request["y"]
                    .as_i64()
                    .unwrap_or(64)
                    .clamp(i64::from(*min_y), i64::from(*max_y));
                let biome = match *preset {
                    "nether" => "minecraft:nether_wastes",
                    "end" => "minecraft:the_end",
                    _ => "minecraft:plains",
                };
                reply["cells"] = json!(
                    (0..width * 4)
                        .flat_map(|cx| (0..height * 4)
                            .map(move |cz| json!({"x":x+cx*4,"z":z+cz*4,"y":y,"biome":biome})))
                        .collect::<Vec<_>>()
                );
            } else {
                reply["matches"] =
                    json!([{"x":x,"z":z,"y":request["min_y"].as_i64().unwrap_or(0)}]);
            }
        }
        _ => anyhow::bail!("unsupported terrain fixture command"),
    }
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    #[test]
    fn terrain_rejects_unknown_dimensions_oversized_regions_and_missing_visibility() {
        let runtime = std::path::Path::new("/nonexistent-cyh-opt-fixture");
        for request in [
            json!({"kind":"area","world":"unknown"}),
            json!({"kind":"area","width":9,"height":9}),
            json!({"kind":"area","chunk_x":i64::MAX}),
            json!({"kind":"seed_profile"}),
        ] {
            assert!(super::terrain(request, runtime).is_err());
        }
    }
}
