use adjust_core::{process_image, AdjustParams, RgbImageF32};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

pub struct ExportJob {
    pub save_path: PathBuf,
    /// Full-resolution base image (applied layers already baked in).
    pub base: Arc<RgbImageF32>,
    pub params: AdjustParams,
}

pub enum ExportResult {
    Success {
        save_path: PathBuf,
        elapsed_ms: u128,
        width: usize,
        height: usize,
    },
    Failed {
        save_path: PathBuf,
        error: String,
    },
}

pub struct BackgroundExporter {
    sender: Sender<ExportJob>,
    receiver: Receiver<ExportResult>,
    active_jobs: usize,
}

impl BackgroundExporter {
    pub fn new() -> Self {
        let (job_tx, job_rx) = channel::<ExportJob>();
        let (res_tx, res_rx) = channel::<ExportResult>();

        thread::spawn(move || {
            while let Ok(job) = job_rx.recv() {
                let start = Instant::now();
                let processed = process_image(&job.base, &job.params);

                // Keeps 16-bit depth and alpha when the output format supports them.
                match processed.save(&job.save_path) {
                    Ok(_) => {
                        let elapsed_ms = start.elapsed().as_millis();
                        let _ = res_tx.send(ExportResult::Success {
                            save_path: job.save_path,
                            elapsed_ms,
                            width: processed.width,
                            height: processed.height,
                        });
                    }
                    Err(e) => {
                        let _ = res_tx.send(ExportResult::Failed {
                            save_path: job.save_path,
                            error: e.to_string(),
                        });
                    }
                }
            }
        });

        Self {
            sender: job_tx,
            receiver: res_rx,
            active_jobs: 0,
        }
    }

    pub fn queue_export(&mut self, job: ExportJob) {
        self.active_jobs += 1;
        let _ = self.sender.send(job);
    }

    pub fn poll_results(&mut self) -> Vec<ExportResult> {
        let mut results = Vec::new();
        while let Ok(res) = self.receiver.try_recv() {
            if self.active_jobs > 0 {
                self.active_jobs -= 1;
            }
            results.push(res);
        }
        results
    }

    pub fn active_jobs(&self) -> usize {
        self.active_jobs
    }
}
