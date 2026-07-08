use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct JobDefinition {
    pub id: String,
    pub name: String,
    pub zone: String,
    pub hint: String,
    pub target: u32,
    pub satisfaction: f32,
}

#[derive(Debug, Deserialize)]
struct JobManifestFile {
    jobs: Vec<JobDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobManifestError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for JobManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "failed to read job manifest: {msg}"),
            Self::Parse(msg) => write!(f, "failed to parse job manifest: {msg}"),
        }
    }
}

impl std::error::Error for JobManifestError {}

pub fn load_job_manifest(path: impl AsRef<Path>) -> Result<Vec<JobDefinition>, JobManifestError> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|err| JobManifestError::Io(err.to_string()))?;
    let parsed: JobManifestFile = serde_json::from_str(&raw)
        .map_err(|err| JobManifestError::Parse(err.to_string()))?;
    Ok(parsed.jobs)
}

pub fn find_job<'a>(jobs: &'a [JobDefinition], id: &str) -> Option<&'a JobDefinition> {
    jobs.iter().find(|job| job.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_ten_jobs_from_manifest() {
        let jobs = load_job_manifest("data/job_manifest.json").expect("manifest loads");
        assert_eq!(jobs.len(), 10);
        let crane = find_job(&jobs, "crane_of_regret").expect("crane job present");
        assert_eq!(crane.name, "Crane of Regret");
        assert_eq!(crane.target, 3);
    }
}
