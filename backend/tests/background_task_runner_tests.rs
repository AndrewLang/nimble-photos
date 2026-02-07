use nimble_photos::services::{BackgroundTaskRunner, TaskDescriptor};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Barrier;
use tokio::time::{Duration, Instant, sleep};

async fn wait_until_counter(counter: &AtomicUsize, expected: usize, timeout: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if counter.load(Ordering::SeqCst) >= expected {
            return true;
        }
        sleep(Duration::from_millis(5)).await;
    }
    false
}

#[tokio::test]
async fn minimal_example_enqueue_and_execute_task() {
    let runner = BackgroundTaskRunner::new(2);
    runner.start().expect("failed to start runner");

    let completed_count = Arc::new(AtomicUsize::new(0));
    let completed_count_for_task = Arc::clone(&completed_count);
    runner
        .enqueue(TaskDescriptor::new("example-task", async move {
            completed_count_for_task.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }))
        .expect("failed to enqueue task");

    let completed = wait_until_counter(&completed_count, 1, Duration::from_secs(2)).await;
    assert!(completed);

    runner.stop().await.expect("failed to stop runner");
}

#[tokio::test]
async fn tasks_execute_in_parallel_up_to_configured_limit() {
    let runner = BackgroundTaskRunner::new(2);
    runner.start().expect("failed to start runner");

    let completed_count = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(2));

    for task_index in 0..4 {
        let completed_count_for_task = Arc::clone(&completed_count);
        let barrier_for_task = Arc::clone(&barrier);
        runner
            .enqueue(TaskDescriptor::new(
                format!("parallel-task-{task_index}"),
                async move {
                    barrier_for_task.wait().await;
                    sleep(Duration::from_millis(80)).await;
                    completed_count_for_task.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            ))
            .expect("failed to enqueue task");
    }

    let started = Instant::now();
    let completed = wait_until_counter(&completed_count, 4, Duration::from_secs(3)).await;
    assert!(completed);
    assert!(started.elapsed() < Duration::from_millis(320));

    runner.stop().await.expect("failed to stop runner");
}

#[tokio::test]
async fn stop_finishes_running_tasks_and_rejects_new_tasks() {
    let runner = BackgroundTaskRunner::new(1);
    runner.start().expect("failed to start runner");

    let completed_count = Arc::new(AtomicUsize::new(0));
    let completed_count_for_task = Arc::clone(&completed_count);
    runner
        .enqueue(TaskDescriptor::new("graceful-stop-task", async move {
            sleep(Duration::from_millis(100)).await;
            completed_count_for_task.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }))
        .expect("failed to enqueue task");

    runner.stop().await.expect("failed to stop runner");

    assert_eq!(completed_count.load(Ordering::SeqCst), 1);
    assert_eq!(runner.running_count(), 0);
    assert_eq!(runner.queued_count(), 0);

    let enqueue_after_stop = runner.enqueue(TaskDescriptor::new("rejected-task", async move { Ok(()) }));
    assert!(enqueue_after_stop.is_err());
}
