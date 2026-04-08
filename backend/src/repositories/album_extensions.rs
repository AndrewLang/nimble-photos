use crate::prelude::*;

#[async_trait]
pub trait AlbumExtensions {}

#[async_trait]
impl AlbumExtensions for Repository<Album> {}

#[async_trait]
pub trait AlbumPhotoExtensions {
    async fn add_photos_to_album(
        &self,
        album_id: Uuid,
        photo_ids: &[Uuid],
    ) -> Result<u32, PipelineError>;
    async fn remove_photos_from_album(
        &self,
        album_id: Uuid,
        photo_ids: &[Uuid],
    ) -> Result<u32, PipelineError>;
}

#[async_trait]
impl AlbumPhotoExtensions for Repository<AlbumPhoto> {
    async fn add_photos_to_album(
        &self,
        album_id: Uuid,
        photo_ids: &[Uuid],
    ) -> Result<u32, PipelineError> {
        let query = QueryBuilder::<AlbumPhoto>::new()
            .filter("album_id", FilterOperator::Eq, Value::Uuid(album_id))
            .build();

        let photo_ids_set: HashSet<Uuid> = self
            .all(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .into_iter()
            .map(|item| item.id)
            .collect();

        let entities = photo_ids
            .iter()
            .filter(|photo_id| !photo_ids_set.contains(photo_id))
            .map(|photo_id| AlbumPhoto::new(album_id, *photo_id))
            .collect::<Vec<_>>();

        let mut added = 0;
        for entity in entities {
            if self.insert(entity).await.is_ok() {
                added += 1;
            }
        }
        Ok(added)
    }

    async fn remove_photos_from_album(
        &self,
        album_id: Uuid,
        photo_ids: &[Uuid],
    ) -> Result<u32, PipelineError> {
        let mut removed = 0;
        let query = QueryBuilder::<AlbumPhoto>::new()
            .filter("album_id", FilterOperator::Eq, Value::Uuid(album_id))
            .filter(
                "photo_id",
                FilterOperator::In,
                Value::List(photo_ids.iter().copied().map(Value::Uuid).collect()),
            )
            .build();

        let items = self
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        for item in items.items {
            if self
                .delete(&item.id)
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            {
                removed += 1;
            }
        }

        Ok(removed)
    }
}

#[async_trait]
pub trait AlbumCommentExtensions {}

impl AlbumCommentExtensions for Repository<AlbumComment> {}
