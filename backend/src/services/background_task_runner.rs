use anyhow::{Result, anyhow};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

use crate::services::TaskDescriptor;

pub struct BackgroundTaskRunner {
    parallelism: usize,
    queue: Arc<Mutex<VecDeque<TaskDescriptor>>>,
    worker_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    running_task_count: Arc<AtomicUsize>,
    queued_task_count: Arc<AtomicUsize>,
    accepting_tasks: Arc<AtomicBool>,
    running_workers: Arc<AtomicBool>,
    shutting_down: Arc<AtomicBool>,
}

impl BackgroundTaskRunner {
    const EMPTY_QUEUE_SLEEP_MILLISECONDS: u64 = 5;

    pub fn new(parallelism: usize) -> Self {
        let worker_parallelism = parallelism.max(1);
        Self {
            parallelism: worker_parallelism,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            worker_handles: Arc::new(Mutex::new(Vec::new())),
            running_task_count: Arc::new(AtomicUsize::new(0)),
            queued_task_count: Arc::new(AtomicUsize::new(0)),
            accepting_tasks: Arc::new(AtomicBool::new(true)),
            running_workers: Arc::new(AtomicBool::new(false)),
            shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn enqueue(&self, task: TaskDescriptor) -> Result<()> {
        if !self.accepting_tasks.load(Ordering::SeqCst) {
            return Err(anyhow!("BackgroundTaskRunner is not accepting new tasks"));
        }

        let mut queue = self
            .queue
            .lock()
            .map_err(|_| anyhow!("Failed to lock task queue"))?;
        queue.push_back(task);
        self.queued_task_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        if self.running_workers.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        self.shutting_down.store(false, Ordering::SeqCst);
        self.accepting_tasks.store(true, Ordering::SeqCst);

        let mut handles = self
            .worker_handles
            .lock()
            .map_err(|_| anyhow!("Failed to lock worker handle pool"))?;

        for _ in 0..self.parallelism {
            let worker = WorkerRuntime {
                queue: Arc::clone(&self.queue),
                running_task_count: Arc::clone(&self.running_task_count),
                queued_task_count: Arc::clone(&self.queued_task_count),
                shutting_down: Arc::clone(&self.shutting_down),
            };

            handles.push(tokio::spawn(async move {
                worker.run().await;
            }));
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.accepting_tasks.store(false, Ordering::SeqCst);
        self.shutting_down.store(true, Ordering::SeqCst);

        let handles = {
            let mut guard = self
                .worker_handles
                .lock()
                .map_err(|_| anyhow!("Failed to lock worker handle pool"))?;
            std::mem::take(&mut *guard)
        };

        for handle in handles {
            let _ = handle.await;
        }

        self.running_workers.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn running_count(&self) -> usize {
        self.running_task_count.load(Ordering::SeqCst)
    }

    pub fn queued_count(&self) -> usize {
        self.queued_task_count.load(Ordering::SeqCst)
    }
}

struct WorkerRuntime {
    queue: Arc<Mutex<VecDeque<TaskDescriptor>>>,
    running_task_count: Arc<AtomicUsize>,
    queued_task_count: Arc<AtomicUsize>,
    shutting_down: Arc<AtomicBool>,
}

impl WorkerRuntime {
    async fn run(&self) {
        loop {
            if let Some(task) = self.try_take_next_task() {
                self.execute_task(task).await;
                continue;
            }

            if self.shutting_down.load(Ordering::SeqCst)
                && self.queued_task_count.load(Ordering::SeqCst) == 0
            {
                break;
            }

            sleep(Duration::from_millis(
                BackgroundTaskRunner::EMPTY_QUEUE_SLEEP_MILLISECONDS,
            ))
            .await;
        }
    }

    fn try_take_next_task(&self) -> Option<TaskDescriptor> {
        let mut queue = self.queue.lock().ok()?;
        let task = queue.pop_front();
        if task.is_some() {
            self.queued_task_count.fetch_sub(1, Ordering::SeqCst);
        }
        task
    }

    async fn execute_task(&self, task: TaskDescriptor) {
        self.running_task_count.fetch_add(1, Ordering::SeqCst);
        let task_name = task.name.clone();
        let join_result = tokio::spawn(async move { task.execute().await }).await;
        match join_result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                log::error!("Background task '{}' failed: {}", task_name, error);
            }
            Err(error) => {
                log::error!("Background task '{}' panicked: {}", task_name, error);
            }
        }
        self.running_task_count.fetch_sub(1, Ordering::SeqCst);
    }
}
