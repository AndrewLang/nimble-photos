use crate::prelude::*;
use anyhow::Result;
use tokio::sync::broadcast::error::RecvError;

pub struct PhotoService;

impl PhotoService {
    pub fn new(services: Arc<ServiceProvider>) -> Self {
        let event_bus = services.get::<EventBusService>();
        let mut receiver = event_bus.subscribe();

        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        if let Err(error) = PhotoService::handle_event(Arc::clone(&services), event).await {
                            log::error!("PhotoService event handler failed: {:?}", error);
                        }
                    }
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(skipped)) => {
                        log::warn!("PhotoService event subscription lagged by {}", skipped);
                    }
                }
            }
        });

        Self
    }

    async fn handle_event(services: Arc<ServiceProvider>, event: AppEvent) -> Result<()> {
        if event.topic != EventNames::IMAGES_PROCESSED {
            return Ok(());
        }

        let timeline_repo = services.get::<Repository<TimelineDay>>();
        timeline_repo.sync().await?;
        log::info!("Handled event '{}' and refreshed timeline_days", event.topic);

        Ok(())
    }
}
