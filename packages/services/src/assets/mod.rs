mod local;
pub use local::LocalStorageDriver;

use image::imageops::FilterType;
use image::ImageFormat;
use uuid::Uuid;

#[derive(Debug)]
pub struct StorageError(pub String);

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum StorageDriver {
    Local(LocalStorageDriver),
}

impl StorageDriver {
    pub async fn put(&self, key: &str, data: Vec<u8>) -> Result<(), StorageError> {
        match self {
            StorageDriver::Local(d) => d.put(key, data),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        match self {
            StorageDriver::Local(d) => d.get(key),
        }
    }

    pub async fn delete_dir(&self, prefix: &str) -> Result<(), StorageError> {
        match self {
            StorageDriver::Local(d) => d.delete_dir(prefix),
        }
    }

    pub fn url(&self, key: &str) -> String {
        match self {
            StorageDriver::Local(d) => d.url(key),
        }
    }
}

/// Variants: (name, max_longest_edge; None = original size)
const VARIANTS: &[(&str, Option<u32>)] = &[
    ("thumbnail", Some(200)),
    ("small", Some(640)),
    ("medium", Some(1280)),
    ("large", Some(1920)),
    ("original", None),
];

/// Decode image, produce 5 WebP variants, store via driver. Returns original byte count.
pub async fn process_and_store(
    data: &[u8],
    asset_id: Uuid,
    driver: &StorageDriver,
) -> Result<u64, String> {
    let original_size = data.len() as u64;
    let img = image::load_from_memory(data).map_err(|e| format!("decode image: {e}"))?;

    for (variant_name, max_edge) in VARIANTS {
        let resized = if let Some(max) = max_edge {
            let (w, h) = (img.width(), img.height());
            if w > *max || h > *max {
                img.resize(*max, *max, FilterType::Lanczos3)
            } else {
                img.clone()
            }
        } else {
            img.clone()
        };

        let mut buf = Vec::new();
        resized
            .write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::WebP)
            .map_err(|e| format!("encode webp {variant_name}: {e}"))?;

        let key = format!("{asset_id}/{variant_name}.webp");
        driver
            .put(&key, buf)
            .await
            .map_err(|e| format!("store {variant_name}: {e}"))?;
    }

    Ok(original_size)
}
