use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    core::{CRANE_JOB_ID, POWER_HOUR_JOB_ID, POWER_HOUR_SEQUENCE},
    data::{find_job, JobDefinition},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobRecord {
    pub active: bool,
    pub complete: bool,
    pub progress: u32,
}

/// Server-authoritative job state (replaces Godot `JobSystem` autoload).
#[derive(Resource, Debug, Clone)]
pub struct JobSystem {
    pub definitions: Vec<JobDefinition>,
    pub states: HashMap<String, JobRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobActionResult {
    Progressed,
    Completed,
    AlreadyComplete,
    WrongSequence,
    NotActive,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerResult {
    Flipped,
    Completed,
    WrongBreaker,
}

impl JobSystem {
    pub fn from_definitions(definitions: Vec<JobDefinition>) -> Self {
        let states = definitions
            .iter()
            .map(|job| (job.id.clone(), JobRecord::default()))
            .collect();
        Self {
            definitions,
            states,
        }
    }

    /// Clear all job progress (rematch / new tournament).
    pub fn reset_all(&mut self) {
        for state in self.states.values_mut() {
            *state = JobRecord::default();
        }
    }

    pub fn is_active(&self, job_id: &str) -> bool {
        self.states
            .get(job_id)
            .is_some_and(|state| state.active && !state.complete)
    }

    pub fn is_complete(&self, job_id: &str) -> bool {
        self.states
            .get(job_id)
            .is_some_and(|state| state.complete)
    }

    pub fn progress(&self, job_id: &str) -> u32 {
        self.states.get(job_id).map(|state| state.progress).unwrap_or(0)
    }

    pub fn target(&self, job_id: &str) -> u32 {
        find_job(&self.definitions, job_id)
            .map(|job| job.target)
            .unwrap_or(1)
    }

    pub fn progress_for(&self, job_id: &str) -> (u32, u32) {
        let current = self.progress(job_id);
        let target = if job_id == POWER_HOUR_JOB_ID {
            POWER_HOUR_SEQUENCE.len() as u32
        } else {
            self.target(job_id)
        };
        (current, target)
    }

    pub fn start_job(&mut self, job_id: &str) -> bool {
        if self.is_complete(job_id) || self.is_active(job_id) {
            return false;
        }
        let Some(state) = self.states.get_mut(job_id) else {
            return false;
        };
        state.active = true;
        state.progress = 0;
        true
    }

    pub fn increment_progress(&mut self, job_id: &str) -> JobActionResult {
        if !self.is_active(job_id) {
            return JobActionResult::NotActive;
        }
        let target = self.target(job_id);
        let Some(state) = self.states.get_mut(job_id) else {
            return JobActionResult::Ignored;
        };
        if state.progress >= target {
            return JobActionResult::AlreadyComplete;
        }
        state.progress += 1;
        if state.progress >= target {
            self.complete_job(job_id);
            JobActionResult::Completed
        } else {
            JobActionResult::Progressed
        }
    }

    pub fn complete_job(&mut self, job_id: &str) -> bool {
        let Some(state) = self.states.get_mut(job_id) else {
            return false;
        };
        if state.complete {
            return false;
        }
        state.active = false;
        state.complete = true;
        true
    }

    pub fn try_crane_interact(&mut self) -> JobActionResult {
        if self.is_complete(CRANE_JOB_ID) {
            return JobActionResult::AlreadyComplete;
        }
        if !self.is_active(CRANE_JOB_ID) {
            self.start_job(CRANE_JOB_ID);
        }
        self.increment_progress(CRANE_JOB_ID)
    }

    pub fn try_power_hour_interact(&mut self, breaker_index: u8) -> BreakerResult {
        if self.is_complete(POWER_HOUR_JOB_ID) {
            return BreakerResult::Completed;
        }
        if !self.is_active(POWER_HOUR_JOB_ID) {
            if !self.start_job(POWER_HOUR_JOB_ID) {
                return BreakerResult::Completed;
            }
            return BreakerResult::Flipped;
        }

        let step = self.progress(POWER_HOUR_JOB_ID) as usize;
        if step >= POWER_HOUR_SEQUENCE.len() {
            return BreakerResult::Completed;
        }

        if POWER_HOUR_SEQUENCE[step] == breaker_index {
            let Some(state) = self.states.get_mut(POWER_HOUR_JOB_ID) else {
                return BreakerResult::WrongBreaker;
            };
            state.progress += 1;
            if state.progress as usize >= POWER_HOUR_SEQUENCE.len() {
                self.complete_job(POWER_HOUR_JOB_ID);
                BreakerResult::Completed
            } else {
                BreakerResult::Flipped
            }
        } else {
            BreakerResult::WrongBreaker
        }
    }

    pub fn power_hour_step(&self) -> u32 {
        self.progress(POWER_HOUR_JOB_ID)
    }
}

/// Lightweight replication target for CI smoke tests.
#[derive(Component, Serialize, Deserialize, Clone, Default, Debug)]
pub struct SmokeJobFlags {
    pub crane_complete: bool,
    pub power_complete: bool,
}

impl SmokeJobFlags {
    pub fn sync_from(jobs: &JobSystem) -> Self {
        Self {
            crane_complete: jobs.is_complete(CRANE_JOB_ID),
            power_complete: jobs.is_complete(POWER_HOUR_JOB_ID),
        }
    }
}
/// Replicated snapshot for clients.
#[derive(Component, Serialize, Deserialize, Clone, Default, Debug)]
pub struct JobBoard {
    pub states: HashMap<String, JobRecord>,
}

impl JobBoard {
    pub fn sync_from(&mut self, jobs: &JobSystem) {
        self.states = jobs.states.clone();
    }
}

pub fn sync_job_boards(jobs: Option<Res<JobSystem>>, mut boards: Query<&mut JobBoard>) {
    let Some(jobs) = jobs else {
        return;
    };
    for mut board in &mut boards {
        board.sync_from(&jobs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::load_job_manifest;

    #[test]
    fn power_hour_sequence_matches_godot() {
        let _jobs = JobSystem::from_definitions(
            load_job_manifest("data/job_manifest.json").expect("manifest"),
        );
        assert_eq!(POWER_HOUR_SEQUENCE.len(), 12);
        assert_eq!(POWER_HOUR_SEQUENCE[0], 0);
        assert_eq!(POWER_HOUR_SEQUENCE[1], 5);
    }

    fn test_jobs() -> JobSystem {
        JobSystem::from_definitions(vec![JobDefinition {
            id: POWER_HOUR_JOB_ID.into(),
            name: "Power Hour".into(),
            zone: "Main Hub".into(),
            hint: String::new(),
            target: 12,
            satisfaction: 7.0,
        }])
    }

    #[test]
    fn power_hour_wrong_breaker_does_not_advance() {
        let mut jobs = test_jobs();
        jobs.start_job(POWER_HOUR_JOB_ID);
        assert_eq!(
            jobs.try_power_hour_interact(1),
            BreakerResult::WrongBreaker
        );
        assert_eq!(jobs.progress(POWER_HOUR_JOB_ID), 0);
    }

    #[test]
    fn power_hour_full_sequence_completes() {
        let mut jobs = test_jobs();
        jobs.start_job(POWER_HOUR_JOB_ID);
        for &breaker in &POWER_HOUR_SEQUENCE {
            assert_ne!(
                jobs.try_power_hour_interact(breaker),
                BreakerResult::WrongBreaker
            );
        }
        assert!(jobs.is_complete(POWER_HOUR_JOB_ID));
    }
}

pub fn apply_job_action(
    jobs: &mut JobSystem,
    boards: &mut Query<'_, '_, (&mut JobBoard, &mut SmokeJobFlags)>,
) {
    for (mut board, mut flags) in boards.iter_mut() {
        board.sync_from(jobs);
        *flags = SmokeJobFlags::sync_from(jobs);
    }
}
