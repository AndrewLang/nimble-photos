use async_trait::async_trait;
use nimble_web::entity::hooks::{EntityHooks, RequestContext};
use nimble_web::result::{HttpError, Result as HttpResult};
use std::marker::PhantomData;
use uuid::Uuid;

use crate::services::IdGenerationService;

pub trait HasOptionalUuidId {
    fn current_id(&self) -> Option<Uuid>;
    fn set_id(&mut self, id: Uuid);
}

pub struct EnsureUuidIdHooks<T>(PhantomData<T>);

impl<T> EnsureUuidIdHooks<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[async_trait]
impl<T> EntityHooks<T> for EnsureUuidIdHooks<T>
where
    T: HasOptionalUuidId + Send + Sync,
{
    async fn before_insert(&self, context: &RequestContext, entity: &mut T) -> HttpResult<()> {
        let generator = context
            .services()
            .resolve::<IdGenerationService>()
            .ok_or_else(|| HttpError::new(500, "IdGenerationService is not registered"))?;

        if entity.current_id().is_none() {
            entity.set_id(generator.generate());
        }

        Ok(())
    }
}
