use crate::prelude::*;
use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt, stream};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub struct PhotoUploadService {
    max_file_size: u64,
}

#[derive(Clone, Debug)]
pub struct StoredUploadFile {
    pub file_name: String,
    pub relative_path: String,
    pub byte_size: usize,
    pub content_type: Option<String>,
}

impl PhotoUploadService {
    const FILES_FIELD_NAME: &'static str = "files";
    const TEMP_FOLDER_NAME: &'static str = ".temp";
    const UNKNOWN_FILE_BASENAME: &'static str = "upload";
    const DEFAULT_MAX_FILE_SIZE: u64 = 64 * 1024 * 1024;

    pub fn new(max_file_size: u64) -> Self {
        Self {
            max_file_size: if max_file_size == 0 {
                Self::DEFAULT_MAX_FILE_SIZE
            } else {
                max_file_size
            },
        }
    }

    pub async fn persist_multipart_to_storage_temp(
        &self,
        content_type: &str,
        body_bytes: Vec<u8>,
        storage_path: &Path,
    ) -> Result<Vec<StoredUploadFile>> {
        let boundary = multer::parse_boundary(content_type)?;
        let body_stream =
            stream::once(async move { Ok::<Bytes, std::io::Error>(Bytes::from(body_bytes)) });
        let mut multipart = multer::Multipart::new(body_stream, boundary);

        let temp_folder = storage_path.join(Self::TEMP_FOLDER_NAME);
        fs::create_dir_all(&temp_folder).await?;

        let mut saved_files = Vec::<StoredUploadFile>::new();
        while let Some(field) = multipart.next_field().await? {
            if field.name() != Some(Self::FILES_FIELD_NAME) {
                continue;
            }

            let incoming_name = field
                .file_name()
                .map(ToString::to_string)
                .unwrap_or_else(|| Self::UNKNOWN_FILE_BASENAME.to_string());
            let content_type = field.content_type().map(|value| value.to_string());
            let sanitized_name = Self::sanitize_file_name(&incoming_name);
            let (final_file_name, absolute_file_path) =
                self.allocate_unique_path(&temp_folder, &sanitized_name).await?;

            // Stream each multipart field directly to disk to keep memory usage flat.
            let bytes_written = self
                .write_stream_to_file(field.into_stream(), &absolute_file_path)
                .await
                .with_context(|| {
                    format!("failed to persist upload '{}'", absolute_file_path.display())
                })?;

            if bytes_written == 0 {
                let _ = fs::remove_file(&absolute_file_path).await;
                continue;
            }

            log::debug!(
                "Stored upload '{}' ({} bytes)",
                final_file_name,
                bytes_written
            );

            saved_files.push(StoredUploadFile {
                file_name: final_file_name.clone(),
                relative_path: format!("{}/{}", Self::TEMP_FOLDER_NAME, final_file_name),
                byte_size: bytes_written as usize,
                content_type,
            });
        }

        Ok(saved_files)
    }

    async fn write_stream_to_file<S>(&self, mut stream: S, path: &Path) -> Result<u64>
    where
        S: futures_util::Stream<Item = Result<Bytes, multer::Error>> + Unpin,
    {
        let mut file = File::create_new(path).await?;
        let mut bytes_written = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            bytes_written = bytes_written
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| anyhow!("uploaded file size overflow"))?;

            if bytes_written > self.max_file_size {
                drop(file);
                let _ = fs::remove_file(path).await;
                return Err(anyhow!(
                    "uploaded file exceeds max allowed size of {} bytes",
                    self.max_file_size
                ));
            }

            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        Ok(bytes_written)
    }

    async fn allocate_unique_path(
        &self,
        temp_folder: &Path,
        sanitized_name: &str,
    ) -> Result<(String, PathBuf)> {
        let candidate_name = Path::new(sanitized_name);
        let stem = candidate_name
            .file_stem()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or(Self::UNKNOWN_FILE_BASENAME);
        let ext = candidate_name
            .extension()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty());

        for _ in 0..8 {
            let suffix = Uuid::new_v4().simple().to_string();
            let final_name = match ext {
                Some(ext) => format!("{stem}_{suffix}.{ext}"),
                None => format!("{stem}_{suffix}"),
            };
            let path = temp_folder.join(&final_name);

            if !fs::try_exists(&path).await.unwrap_or(false) {
                return Ok((final_name, path));
            }
        }

        Err(anyhow!("failed to allocate unique upload file name"))
    }

    fn sanitize_file_name(file_name: &str) -> String {
        let base_name = Path::new(file_name)
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| Self::UNKNOWN_FILE_BASENAME.to_string());
        let sanitized = base_name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric()
                    || character == '.'
                    || character == '-'
                    || character == '_'
                {
                    character
                } else {
                    '_'
                }
            })
            .collect::<String>();
        if sanitized.is_empty() {
            Self::UNKNOWN_FILE_BASENAME.to_string()
        } else {
            sanitized
        }
    }

    pub fn require_content_type<'a>(&self, content_type: Option<&'a str>) -> Result<&'a str> {
        content_type.ok_or_else(|| anyhow!("Missing content-type header"))
    }
}
