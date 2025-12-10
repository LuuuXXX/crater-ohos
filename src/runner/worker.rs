use crate::crates::Crate;
use crate::experiments::Experiment;
use crate::prelude::*;
use crate::results::TestResult;
use crate::toolchain::Toolchain;
use std::sync::{Arc, Mutex, Condvar, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::Duration;

// Placeholder for rustwide Workspace
pub struct Workspace;

/// Trait for recording progress of test runs
pub trait RecordProgress: Send + Sync {
    fn record_progress(
        &self,
        ex: &Experiment,
        krate: &Crate,
        toolchain: &Toolchain,
        log: &[u8],
        result: &TestResult,
        version: Option<(&Crate, &Crate)>,
    ) -> Fallible<()>;
}

/// Worker that executes tasks
pub(super) struct Worker<'a> {
    name: String,
    workspace: &'a Workspace,
    ex: &'a Experiment,
    disk_space_watcher: Arc<DiskSpaceWatcher>,
}

impl<'a> Worker<'a> {
    pub(super) fn new(
        name: String,
        workspace: &'a Workspace,
        ex: &'a Experiment,
        disk_space_watcher: Arc<DiskSpaceWatcher>,
    ) -> Self {
        Worker {
            name,
            workspace,
            ex,
            disk_space_watcher,
        }
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn run(&self) -> Fallible<()> {
        info!("Worker {} started", self.name);

        // Placeholder implementation
        // In real implementation, this would:
        // 1. Get next task from queue
        // 2. Execute the task
        // 3. Record results
        // 4. Check disk space
        // 5. Repeat until no more tasks

        Ok(())
    }
}

/// Monitors disk space and pauses workers if threshold is exceeded
pub(super) struct DiskSpaceWatcher {
    interval: Duration,
    threshold: f32,
    worker_count: usize,
    idle_workers: AtomicUsize,
    permanent_idle_workers: AtomicUsize,
    stop_requested: AtomicBool,
    condvar: Condvar,
    mutex: Mutex<()>,
}

impl DiskSpaceWatcher {
    pub(super) fn new(interval: Duration, threshold: f32, worker_count: usize) -> Arc<Self> {
        Arc::new(DiskSpaceWatcher {
            interval,
            threshold,
            worker_count,
            idle_workers: AtomicUsize::new(0),
            permanent_idle_workers: AtomicUsize::new(0),
            stop_requested: AtomicBool::new(false),
            condvar: Condvar::new(),
            mutex: Mutex::new(()),
        })
    }

    pub(super) fn stop(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        self.condvar.notify_all();
    }

    pub(super) fn run(&self, _workspace: &Workspace) {
        info!(
            "Disk space watcher started (interval: {:?}, threshold: {:.0}%)",
            self.interval,
            self.threshold * 100.0
        );

        loop {
            if self.stop_requested.load(Ordering::SeqCst) {
                break;
            }

            // Check disk space
            // In real implementation, this would check actual disk usage
            // For now, using a conservative placeholder value for safety
            let disk_usage = 0.0; // Placeholder - always below threshold

            if disk_usage > self.threshold {
                warn!(
                    "Disk usage above threshold: {:.0}% > {:.0}%",
                    disk_usage * 100.0,
                    self.threshold * 100.0
                );
                // Would pause workers here
            }

            // Wait for next check or stop signal
            let guard = self.mutex.lock().unwrap();
            let _ = self
                .condvar
                .wait_timeout(guard, self.interval)
                .unwrap();
        }

        info!("Disk space watcher stopped");
    }

    pub(super) fn worker_idle(&self, permanent: bool) {
        if permanent {
            let count = self.permanent_idle_workers.fetch_add(1, Ordering::SeqCst) + 1;
            debug!("Worker permanently idle ({}/{})", count, self.worker_count);
        } else {
            let count = self.idle_workers.fetch_add(1, Ordering::SeqCst) + 1;
            debug!("Worker idle ({}/{})", count, self.worker_count);
        }
    }

    pub(super) fn worker_active(&self) {
        let count = self.idle_workers.fetch_sub(1, Ordering::SeqCst) - 1;
        debug!("Worker active (idle: {}/{})", count, self.worker_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_space_watcher_creation() {
        let watcher = DiskSpaceWatcher::new(Duration::from_secs(30), 0.8, 4);
        assert_eq!(watcher.threshold, 0.8);
        assert_eq!(watcher.worker_count, 4);
        assert_eq!(watcher.idle_workers.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_worker_idle_tracking() {
        let watcher = DiskSpaceWatcher::new(Duration::from_secs(30), 0.8, 4);

        watcher.worker_idle(false);
        assert_eq!(watcher.idle_workers.load(Ordering::SeqCst), 1);

        watcher.worker_idle(false);
        assert_eq!(watcher.idle_workers.load(Ordering::SeqCst), 2);

        watcher.worker_active();
        assert_eq!(watcher.idle_workers.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_permanent_idle_tracking() {
        let watcher = DiskSpaceWatcher::new(Duration::from_secs(30), 0.8, 4);

        watcher.worker_idle(true);
        assert_eq!(watcher.permanent_idle_workers.load(Ordering::SeqCst), 1);

        watcher.worker_idle(true);
        assert_eq!(watcher.permanent_idle_workers.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_stop_watcher() {
        let watcher = DiskSpaceWatcher::new(Duration::from_secs(30), 0.8, 4);
        assert!(!watcher.stop_requested.load(Ordering::SeqCst));

        watcher.stop();
        assert!(watcher.stop_requested.load(Ordering::SeqCst));
    }
}
