use async_trait::async_trait;
use nimble_web::entity::hooks::{EntityHooks, RequestContext};
use nimble_web::result::{HttpError, Result as HttpResult};
use std::marker::PhantomData;
use uuid::Uuid;

use crate::services::IdGenerationService;

pub trait HasOptionalUuidId {
    fn id_slot(&mut self) -> &mut Option<Uuid>;
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

        let id_slot = entity.id_slot();
        if id_slot.is_none() {
            *id_slot = Some(generator.generate());
        }

        Ok(())
    }
}
