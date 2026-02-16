use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

pub struct TaskDescriptor {
    pub name: String,
    task_future: Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>,
}

impl TaskDescriptor {
    pub fn new<F>(name: impl Into<String>, task_future: F) -> Self
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            name: name.into(),
            task_future: Box::pin(task_future),
        }
    }

    pub async fn execute(self) -> Result<()> {
        self.task_future.await
    }
}
