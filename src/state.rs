// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-12)
// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{collections::VecDeque, time::Duration};

use crate::gpu::Snapshot;

pub struct GpuVisual {
    pub history: VecDeque<u32>,
    pub shown_util: f64,
    pub shown_memory: f64,
}

impl Default for GpuVisual {
    fn default() -> Self {
        Self {
            history: VecDeque::with_capacity(240),
            shown_util: 0.0,
            shown_memory: 0.0,
        }
    }
}

pub struct AppState {
    pub snapshot: Snapshot,
    pub visuals: Vec<GpuVisual>,
    pub elapsed: Duration,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub selected_gpu: usize,
    pub gpu_navigation_active: bool,
    pub selected_process: usize,
    pub process_navigation_active: bool,
    pub notes: String,
    pub note_buffer: String,
    pub note_editor: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            snapshot: Snapshot::default(),
            visuals: Vec::new(),
            elapsed: Duration::ZERO,
            error: None,
            notice: None,
            selected_gpu: 0,
            gpu_navigation_active: false,
            selected_process: 0,
            process_navigation_active: false,
            notes: String::new(),
            note_buffer: String::new(),
            note_editor: false,
        }
    }

    pub fn set_snapshot(&mut self, snapshot: Snapshot) {
        self.selected_gpu = self.selected_gpu.min(snapshot.gpus.len().saturating_sub(1));
        self.selected_process = self
            .selected_process
            .min(snapshot.processes.len().saturating_sub(1));
        self.visuals
            .resize_with(snapshot.gpus.len(), GpuVisual::default);
        for (sample, visual) in snapshot.gpus.iter().zip(&mut self.visuals) {
            visual
                .history
                .push_back(sample.utilization.unwrap_or(0).min(100));
            if visual.history.len() > 240 {
                visual.history.pop_front();
            }
        }
        self.snapshot = snapshot;
        self.error = None;
    }

    pub fn animate(&mut self, delta: Duration) {
        self.elapsed += delta;
        let seconds = delta.as_secs_f64().min(0.1);
        let ease = 1.0 - (-seconds * 9.0).exp();
        for (sample, visual) in self.snapshot.gpus.iter().zip(&mut self.visuals) {
            let util = sample.utilization.unwrap_or(0) as f64;
            let memory = match (sample.memory_used, sample.memory_total) {
                (Some(used), Some(total)) if total > 0 => used as f64 / total as f64 * 100.0,
                _ => 0.0,
            };
            visual.shown_util += (util - visual.shown_util) * ease;
            visual.shown_memory += (memory - visual.shown_memory) * ease;
        }
    }
}
