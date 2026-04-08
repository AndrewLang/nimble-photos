pub use super::image_process_context::ImageProcessContext;
use super::image_process_step::ImageProcessStep;
use crate::entities::StorageLocation;
use crate::services::background_task_runner::BackgroundTaskRunner;
use crate::services::event_bus_service::EventBusService;
use crate::services::image_process_steps::{
    CategorizeImageStep, ComputeHashStep, ExtractExifStep, GeneratePreviewStep,
    GenerateThumbnailStep, PersistMetadataStep,
};
use crate::services::photo_upload_service::StoredUploadFile;
use crate::services::task_descriptor::TaskDescriptor;

use anyhow::Result;
use nimble_web::Configuration;
use nimble_web::ServiceProvider;
use serde_json::{Value as JsonValue, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct ImageProcessPipelineContext {
    pub services: Arc<ServiceProvider>,
    pub configuration: Configuration,
}

impl ImageProcessPipelineContext {
    pub fn new(services: Arc<ServiceProvider>, configuration: Configuration) -> Self {
        Self {
            services: services.clone(),
            configuration,
        }
    }

    pub fn get_service<T: Send + Sync + 'static>(&self) -> Arc<T> {
        self.services.get::<T>()
    }
}

#[derive(Clone, Debug)]
pub struct ImageProcessPayload {
    pub storage: StorageLocation,
    pub relative_path: String,
    pub file_name: String,
    pub byte_size: usize,
    pub content_type: Option<String>,
}

impl ImageProcessPayload {
    pub fn from_upload(storage: StorageLocation, file: StoredUploadFile) -> Self {
        log::debug!(
            "Creating ImageProcessPayload for storage {} file {} {}",
            storage.path,
            file.relative_path.clone(),
            file.file_name,
        );
        Self {
            storage,
            relative_path: file.relative_path,
            file_name: file.file_name,
            byte_size: file.byte_size,
            content_type: file.content_type,
        }
    }

    pub fn source_path(&self) -> PathBuf {
        self.storage
            .normalized_path()
            .join(Path::new(&self.relative_path))
    }

    pub fn working_directory(&self) -> PathBuf {
        self.storage.normalized_path()
    }
}

#[derive(Clone)]
pub struct ImageProcessPipeline {
    runner: Arc<BackgroundTaskRunner>,
    event_bus: Arc<EventBusService>,
    steps: Vec<Arc<dyn ImageProcessStep>>,
    services: Arc<ServiceProvider>,
}

impl ImageProcessPipeline {
    pub fn new(context: ImageProcessPipelineContext) -> Self {
        let runner = context.get_service::<BackgroundTaskRunner>();
        let event_bus = context.get_service::<EventBusService>();

        let steps: Vec<Arc<dyn ImageProcessStep>> = vec![
            Arc::new(ComputeHashStep::new(context.services.clone())),
            Arc::new(ExtractExifStep::new(context.services.clone())),
            Arc::new(GenerateThumbnailStep::new(context.services.clone())),
            Arc::new(GeneratePreviewStep::new(context.services.clone())),
            Arc::new(CategorizeImageStep::new(context.services.clone())),
            Arc::new(PersistMetadataStep::new(context.services.clone())),
        ];

        Self {
            runner,
            event_bus,
            steps,
            services: Arc::clone(&context.services),
        }
    }

    pub fn enqueue_files(
        &self,
        storage: StorageLocation,
        files: Vec<StoredUploadFile>,
    ) -> Result<()> {
        for file in files {
            let request = ImageProcessPayload::from_upload(storage.clone(), file);
            self.enqueue_request(request)?;
        }
        Ok(())
    }

    pub async fn process(&self, request: ImageProcessPayload) -> Result<()> {
        self.run_steps(request).await
    }

    fn enqueue_request(&self, request: ImageProcessPayload) -> Result<()> {
        let pipeline = self.clone();
        let task_name = format!("image-process-{}-{}", request.storage.id, request.file_name);
        self.runner
            .enqueue(TaskDescriptor::new(task_name, async move {
                let completion = json!({
                    "storageId": request.storage.id,
                    "storagePath": request.storage.path,
                    "fileName": request.file_name,
                    "relativePath": request.relative_path,
                });

                if let Err(error) = pipeline.run_steps(request).await {
                    pipeline.emit_images_processed_if_idle(completion);
                    log::error!("Image process pipeline failed: {:?}", error);
                    return Err(error);
                }

                pipeline.emit_images_processed_if_idle(completion);
                Ok(())
            }))
    }

    async fn run_steps(&self, request: ImageProcessPayload) -> Result<()> {
        log::trace!(
            "Starting pipeline for storage {} file {}",
            request.storage.id,
            request.file_name
        );

        let mut context = ImageProcessContext::new(request, self.services.clone());
        for step in &self.steps {
            step.execute(&mut context).await?;
            if !context.can_continue() {
                log::debug!(
                    "Stopping image process pipeline for {} because can_continue is false",
                    context.source_path().display()
                );
                break;
            }
        }
        Ok(())
    }

    fn emit_images_processed_if_idle(&self, last_completed: JsonValue) {
        if self.runner.queued_count() != 0 || self.runner.running_count() != 1 {
            return;
        }

        self.event_bus.emit(
            "images.processed",
            json!({
                "queuedCount": 0,
                "runningCount": 0,
                "lastCompleted": last_completed,
            }),
        );
    }
}
