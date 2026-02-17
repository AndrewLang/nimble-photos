use anyhow::{Result, anyhow};
use bytes::Bytes;
use futures_util::stream;
use std::path::Path;
use tokio::fs;

pub struct PhotoUploadService;

#[derive(Clone, Debug)]
pub struct UploadFilePayload {
    pub file_name: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
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

    pub fn new() -> Self {
        Self {}
    }

    pub async fn parse_multipart_files(
        &self,
        content_type: &str,
        body_bytes: Vec<u8>,
    ) -> Result<Vec<UploadFilePayload>> {
        let boundary = multer::parse_boundary(content_type)?;
        let body_stream =
            stream::once(async move { Ok::<Bytes, std::io::Error>(Bytes::from(body_bytes)) });
        let mut multipart = multer::Multipart::new(body_stream, boundary);

        let mut files = Vec::<UploadFilePayload>::new();
        while let Some(field) = multipart.next_field().await? {
            if field.name() != Some(Self::FILES_FIELD_NAME) {
                continue;
            }

            let incoming_name = field
                .file_name()
                .map(ToString::to_string)
                .unwrap_or_else(|| Self::UNKNOWN_FILE_BASENAME.to_string());
            let content_type = field.content_type().map(|value| value.to_string());
            let bytes = field.bytes().await?.to_vec();
            if bytes.is_empty() {
                continue;
            }

            files.push(UploadFilePayload {
                file_name: Self::sanitize_file_name(&incoming_name),
                content_type,
                bytes,
            });
        }

        Ok(files)
    }

    pub async fn persist_to_storage_temp(
        &self,
        storage_path: &Path,
        files: Vec<UploadFilePayload>,
    ) -> Result<Vec<StoredUploadFile>> {
        let temp_folder = storage_path.join(Self::TEMP_FOLDER_NAME);
        fs::create_dir_all(&temp_folder).await?;

        let mut saved_files = Vec::<StoredUploadFile>::new();
        for file in files {
            let final_file_name = file.file_name.clone();
            let absolute_file_path = temp_folder.join(&final_file_name);
            fs::write(&absolute_file_path, &file.bytes).await?;

            saved_files.push(StoredUploadFile {
                file_name: final_file_name.clone(),
                relative_path: format!("{}/{}", Self::TEMP_FOLDER_NAME, final_file_name),
                byte_size: file.bytes.len(),
                content_type: file.content_type.clone(),
            });
        }

        Ok(saved_files)
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
