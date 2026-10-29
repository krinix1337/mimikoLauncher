use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tauri::{Manager, Runtime};
use tauri_plugin_http::reqwest;

use super::Result;

const CUSTOM_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

/// Where the six faces of a title screen panorama can be downloaded from.
enum PanoramaSource {
    /// Asset index objects, used since 1.14.
    Assets([&'static str; 6]),
    /// Entries of a client jar, given as local header offsets and compressed sizes.
    Jar {
        url: &'static str,
        entries: [(u64, u64); 6],
    },
}

/// Every distinct title screen panorama Minecraft: Java Edition has shipped.
const PANORAMAS: &[(&str, PanoramaSource)] = &[
    (
        "26.3",
        PanoramaSource::Assets([
            "479ad40b906b87eab6d9eaa66f273226988dc2b2",
            "6a84d4afab797aaf9f6ad283ebcd5899493c5c64",
            "a9368ad23ea1b7517cc2fdf5b3d73cec45a045db",
            "4e260704b88f2cd57fbc05abcd1a4264df733f91",
            "17229c85a056fa9e97e9b025fd76e1e081b36317",
            "61331fc97f54d2f7500cdbb14d16bc1244539272",
        ]),
    ),
    (
        "26.2",
        PanoramaSource::Assets([
            "f5e0a5cfbc2e7b1e89094d7882dde106b030ec26",
            "7eed27f04c07c11f28c8500f3bfe768efcce4ccb",
            "cc6832a02808837dd5bc309020054592ef4b58de",
            "d57bf5739d53501e511593220aa730488f1be990",
            "894dbd30f3c467d78629927b6ec89d9838412415",
            "e448b43724a15a06b9e2fe44114d2c203947d607",
        ]),
    ),
    (
        "26.1",
        PanoramaSource::Assets([
            "4a8493454b4307e330501892058541ed0bf228da",
            "9cf7432d7cf6ac9e1d67cb72fb3606860fdbca65",
            "b1b1ffde0727a421f15b736c85b5bc6605ec1cc2",
            "fb5668247ffd51905e3a13eff34b74c9b709e2f4",
            "0f66c27917abf3b05c61c58e79e012b87ff87bcb",
            "81dedcb3d42b47306fde0d47e7388c62792b5fd3",
        ]),
    ),
    (
        "1.21.11",
        PanoramaSource::Assets([
            "af28e43df0b60875348cecd1c1f65e2325b54204",
            "15e3d77a4555455d752fde55b2661c159bee59ba",
            "bc648a153191d8490fbaaa49e89c62240c6b2936",
            "79795659716b1f0ad2b2b3078933acb7c86327d9",
            "64d2b763bb3bcfcaee67ba1b9351b0a83c209ae7",
            "4ef46fd3165dcd588f4b2d945d4aacca9ad90d42",
        ]),
    ),
    (
        "1.21.9",
        PanoramaSource::Assets([
            "0345cc0f9d4e5ddaf1b1c091c8785b28365d094f",
            "5964341a814ba251ab15e4a8c986d045ecb8e799",
            "dbc44494bd40cd75914a9f58a7ad8a6244dc6759",
            "ec17c4fe7e64e499452071698702ecfa8c94f09b",
            "69e16a0585be89de9bd4007a9b9eef330412c9e6",
            "36d4362fb349de7118d4c7143eb41e4dfb37fde7",
        ]),
    ),
    (
        "1.21.6",
        PanoramaSource::Assets([
            "e3e856304400d0c4da3e41cf4954491e0c337e03",
            "89e4e7c845d442d308a6194488de8bd3397f0791",
            "80e51ad5c2febf6fab7fd4416ebf8d7b244ac726",
            "0c55f2465e076e974d4c21fc0320fb4a44c870ef",
            "52c1682d1c7195b377109a73b1e68a614d145d54",
            "24b73002ef685bf6c2364219428f548c0c8b1488",
        ]),
    ),
    (
        "1.21.5",
        PanoramaSource::Assets([
            "4c91acd1304382c9815c9cda9be936a8f1add659",
            "fe231871707578f0e122d0d7c85f467d04c8bb30",
            "2bf90cff6cfb2c2e9d5e74069a08fd3dbfa9f0c3",
            "90662a383543c50f40f1e25927d95b18d8222ea0",
            "01ee504c51dfe11bbdfadbc0445c7c84b728d3a8",
            "2b37830aab6dfb005764829dd2371bd371b87d66",
        ]),
    ),
    (
        "1.21.4",
        PanoramaSource::Assets([
            "11e7261085b99ec9c850a59ed95b6bfbb576c8cc",
            "87a04c89609bad3c46e45420eba4ffddac4be836",
            "cf5c286cb670d4c0ea5fc03728252fe4003b3fbb",
            "8294df934ee4a6373d973cdb5257df793641ba34",
            "60a0380dca096ae055fc1d7d88eebfc33ae94d7b",
            "7d5d238627d3e4171979be59f8c73e8aad5479ae",
        ]),
    ),
    (
        "1.21",
        PanoramaSource::Assets([
            "dca189017c6b77dc6e55ebfb0baca2b645b6e4cd",
            "97d080a6827e54904eb7bbe0f04df20eb9070d1a",
            "d79ff13dd8ff2f7b8d497a7516db1bb2d86d2ab8",
            "b6265425a2c765dd5fc286d5f355a812cad938e2",
            "cec25b0246d77e6f1dcef0d051ccb328ba39227a",
            "74703134a27a57d2de96860bca55d447759c0edc",
        ]),
    ),
    (
        "1.20",
        PanoramaSource::Assets([
            "f228a4964175f885b334033c5e5cf12bf62421d0",
            "28389e89137ba4ed2578c6748b23543759580b06",
            "28ed5175e545a92757ec60a2cc05e4599dd3e96a",
            "29ef924f1bb2a66ec35aa032ca8ab59431a2ebf6",
            "dbd2a426590ecbbbe2ee416d8fa00e1cac800008",
            "344a480830217004a601358cb2bdbb4ccccf88a9",
        ]),
    ),
    (
        "1.19",
        PanoramaSource::Assets([
            "e956e0986b597ec1b7d1db720dc8aec3a0642630",
            "e11e318343cc7dad5d1ffd7f73b64a38c8efd307",
            "10d2bc498bffd73e0f120ba3eb336953db05c713",
            "70ae9c314c2c280af94a08c2403801e120ee4b3c",
            "9519c93477a4394b4ceaaa0f856c7fdd7babd834",
            "9a5729d795b1edfeffaa99506983c8b279880fa9",
        ]),
    ),
    (
        "1.18",
        PanoramaSource::Assets([
            "166d2109d4ba5a5c713fa31df0f07018220449a8",
            "11dda623a695884d57f79b230f5ed0a709817758",
            "03658a18af099f00103fbc2bf5e1dde514e63dff",
            "a28ba83aece09a472178699d706f7aa5747f25bf",
            "75a124e1be0d86c57bac49c7f2d7911486f6e98d",
            "617783a408fca4c7f74af3ad73cb0f9263f01698",
        ]),
    ),
    (
        "1.17",
        PanoramaSource::Assets([
            "81b7228275ad580fd0a47ab4e51061f330df0bd4",
            "4b7e517611dc9a5180201466046c2fe0d1216d00",
            "f4e61ad8551dad48c505b8258841d87ec7df75f5",
            "5d3f8ef1dc0084d49b3a1c1e8b2606028d83cca0",
            "bc07f32c38f0321358c7399dc69f380a1401a888",
            "873c08365fbb20413491b5d782f88aa0459fe8bc",
        ]),
    ),
    (
        "1.16",
        PanoramaSource::Assets([
            "fe531cd13f671fa7d7392d090b93bbeca4ed0029",
            "d8c61356bcb0c2dd087300d2857000078d05e862",
            "781d243e743f1a716ae36d5fdcbd4c80adf8ee0d",
            "98729ec34c8e1bceb9b0ff41a85ebee33eff8b41",
            "6c426128a04ff458f823319e3e68c8bc893d0dda",
            "945630f5471aa307ec6bd6e53e9c1a278ff35f32",
        ]),
    ),
    (
        "1.15",
        PanoramaSource::Assets([
            "5f2e3863ecb4f942315bf65c30083831570065bc",
            "6d6218dbc0344524ffafbe6f63657940029e25ba",
            "f67b109ec66db2100c9580d974937fb14bad0d94",
            "363803ca6e29e3b3a86238086bb4ea06874e8a77",
            "e0ea270a90cb51b0ab135371f3604888a77be739",
            "af7c80d4229733c732623f3b08ffd45110608b13",
        ]),
    ),
    (
        "1.14",
        PanoramaSource::Assets([
            "d75f4a52c3a7333e1676f9d648dbc3a129eaf182",
            "efd77d8aef4530e470dde3ac37d63be6e480f8a0",
            "56422fac8e05210fdccb2ff208070f9218c0da6d",
            "20b8b2f99f0c6f659267366239f61d6a1b8c9156",
            "42b33085867226820f5d6ea60d276fd22be2c0ef",
            "b11528c03e56db38d3a09dc6f0f3fe7bcb4fc0f4",
        ]),
    ),
    (
        "1.13",
        PanoramaSource::Jar {
            url: "https://piston-data.mojang.com/v1/objects/3675ef34dd983d862b366662405e0d9b9654b656/client.jar",
            entries: [
                (12204899, 752949),
                (10660592, 741231),
                (11401930, 708090),
                (12959791, 710148),
                (12110127, 94665),
                (13670046, 1028879),
            ],
        },
    ),
    (
        "classic",
        PanoramaSource::Jar {
            url: "https://launcher.mojang.com/v1/objects/3870888a6c3d349d3771a3e9d16c9bf5e076b908/client.jar",
            entries: [
                (4995748, 64696),
                (5060551, 67009),
                (5127667, 49000),
                (5176774, 71292),
                (5248173, 2321),
                (5250601, 52802),
            ],
        },
    ),
    (
        "april-2026",
        PanoramaSource::Jar {
            url: "https://piston-data.mojang.com/v1/objects/49040e0696c6ade6606645f0065f7aed34ce53d0/client.jar",
            entries: [
                (12685747, 822290),
                (13508147, 659710),
                (14167967, 664890),
                (14832967, 712032),
                (15545109, 617825),
                (16163044, 182079),
            ],
        },
    ),
    (
        "craftmine",
        PanoramaSource::Jar {
            url: "https://piston-data.mojang.com/v1/objects/0f3ed2e39938ccd178981f34f3f62ab7d08b399c/client.jar",
            entries: [
                (14030230, 98163),
                (14128501, 153093),
                (14468423, 145623),
                (13852517, 177145),
                (14281702, 186613),
                (14029770, 352),
            ],
        },
    ),
    (
        "potato",
        PanoramaSource::Jar {
            url: "https://piston-data.mojang.com/v1/objects/b3be1d7926319d4d58f3d8f25d648bf7d1e43970/client.jar",
            entries: [
                (14347509, 892522),
                (15971981, 751208),
                (15240154, 731704),
                (16723312, 702878),
                (17426313, 40520),
                (17466956, 495776),
            ],
        },
    ),
];

async fn backgrounds_dir<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf> {
    let dir = theseus::State::get()
        .await?
        .directories
        .caches_dir()
        .join("backgrounds");
    tokio::fs::create_dir_all(&dir).await?;
    app.asset_protocol_scope()
        .allow_directory(&dir, true)
        .map_err(std::io::Error::other)?;
    Ok(dir)
}

async fn download_range(
    client: &reqwest::Client,
    url: &str,
    start: u64,
    length: u64,
) -> Result<Vec<u8>> {
    let response = client
        .get(url)
        .header(
            reqwest::header::RANGE,
            format!("bytes={start}-{}", start + length - 1),
        )
        .send()
        .await?
        .error_for_status()?;
    if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(std::io::Error::other("Server ignored the byte range").into());
    }
    Ok(response.bytes().await?.to_vec())
}

/// Reads one stored or deflated zip entry straight from a remote jar.
async fn download_jar_entry(
    client: &reqwest::Client,
    url: &str,
    offset: u64,
    compressed_size: u64,
) -> Result<Vec<u8>> {
    let header = download_range(client, url, offset, 30).await?;
    if header.len() < 30 || header[..4] != [0x50, 0x4b, 0x03, 0x04] {
        return Err(std::io::Error::other("Unexpected jar entry header").into());
    }
    let method = u16::from_le_bytes([header[8], header[9]]);
    let name_length = u16::from_le_bytes([header[26], header[27]]) as u64;
    let extra_length = u16::from_le_bytes([header[28], header[29]]) as u64;
    let data = download_range(
        client,
        url,
        offset + 30 + name_length + extra_length,
        compressed_size,
    )
    .await?;
    match method {
        0 => Ok(data),
        8 => {
            let mut inflated = Vec::new();
            flate2::read::DeflateDecoder::new(data.as_slice())
                .read_to_end(&mut inflated)?;
            Ok(inflated)
        }
        _ => Err(std::io::Error::other("Unsupported jar compression").into()),
    }
}

async fn download_face(
    client: &reqwest::Client,
    source: &PanoramaSource,
    face: usize,
) -> Result<Vec<u8>> {
    match source {
        PanoramaSource::Assets(hashes) => {
            let hash = hashes[face];
            Ok(client
                .get(format!("{RESOURCES_URL}/{}/{hash}", &hash[..2]))
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?
                .to_vec())
        }
        PanoramaSource::Jar { url, entries } => {
            let (offset, size) = entries[face];
            download_jar_entry(client, url, offset, size).await
        }
    }
}

fn face_path(dir: &Path, face: usize) -> PathBuf {
    dir.join(format!("panorama_{face}.png"))
}

/// Downloads a Minecraft title screen panorama once and returns the URLs of its six faces
/// in the order front, right, back, left, top, bottom.
#[tauri::command]
pub async fn background_panorama<R: Runtime>(
    app: tauri::AppHandle<R>,
    id: String,
) -> Result<Vec<String>> {
    let (id, source) = PANORAMAS
        .iter()
        .find(|(panorama, _)| *panorama == id)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Unknown panorama",
            )
        })?;
    let dir = backgrounds_dir(&app).await?.join("panorama").join(id);
    if !(0..6).all(|face| face_path(&dir, face).is_file()) {
        let client = reqwest::Client::new();
        let faces = download_faces(&client, source).await?;
        let staging = dir.with_extension("tmp");
        tokio::fs::create_dir_all(&staging).await?;
        for (face, bytes) in faces.into_iter().enumerate() {
            image::guess_format(&bytes).map_err(std::io::Error::other)?;
            tokio::fs::write(face_path(&staging, face), bytes).await?;
        }
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::rename(&staging, &dir).await?;
    }
    (0..6)
        .map(|face| {
            Ok(super::utils::tauri_convert_file_src(&face_path(&dir, face))?
                .to_string())
        })
        .collect()
}

async fn download_faces(
    client: &reqwest::Client,
    source: &PanoramaSource,
) -> Result<Vec<Vec<u8>>> {
    let mut faces = Vec::with_capacity(6);
    let downloads = (0..6).map(|face| download_face(client, source, face));
    for download in downloads {
        faces.push(download.await?);
    }
    Ok(faces)
}

/// Copies an image chosen by the user into the launcher and returns its stored name.
#[tauri::command]
pub async fn background_import<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: PathBuf,
) -> Result<String> {
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .filter(|extension| CUSTOM_EXTENSIONS.contains(&extension.as_str()))
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported image format",
            )
        })?;
    let bytes = tokio::fs::read(&path).await?;
    image::guess_format(&bytes).map_err(std::io::Error::other)?;
    let hash: String = Sha256::digest(&bytes)
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let custom = backgrounds_dir(&app).await?.join("custom");
    tokio::fs::create_dir_all(&custom).await?;
    let name = format!("{hash}.{extension}");
    tokio::fs::write(custom.join(&name), bytes).await?;
    Ok(name)
}

/// Returns the URL of an image previously stored with `background_import`.
#[tauri::command]
pub async fn background_custom_url<R: Runtime>(
    app: tauri::AppHandle<R>,
    name: String,
) -> Result<String> {
    let path = backgrounds_dir(&app).await?.join("custom").join(&name);
    if name.contains(['/', '\\']) || !path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Background image not found",
        )
        .into());
    }
    Ok(super::utils::tauri_convert_file_src(&path)?.to_string())
}
