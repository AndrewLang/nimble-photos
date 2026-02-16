use super::image_pipeline::ImageProcessPayload;
use crate::models::property_map::PropertyMap;

use nimble_web::di::ServiceProvider;

use std::any::Any;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ImageProcessContext {
    payload: ImageProcessPayload,
    source_path: PathBuf,
    properties: PropertyMap,
    services: Arc<ServiceProvider>,
}

impl ImageProcessContext {
    pub(super) fn new(payload: ImageProcessPayload, services: Arc<ServiceProvider>) -> Self {
        let source_path = payload.source_path();

        Self {
            payload,
            source_path,
            properties: PropertyMap::new(),
            services,
        }
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, name: impl Into<String>, value: T) {
        self.properties.insert::<T>(value).alias(name);
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.properties.get::<T>()
    }

    pub fn get_by_alias<T: Any + Send + Sync>(&self, alias: &str) -> Option<&T> {
        self.properties.get_by_alias::<T>(alias)
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.properties.get_mut::<T>()
    }

    pub(super) fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn services(&self) -> Arc<ServiceProvider> {
        Arc::clone(&self.services)
    }

    pub fn payload(&self) -> &ImageProcessPayload {
        &self.payload
    }

    pub fn properties(&self) -> &PropertyMap {
        &self.properties
    }
}
