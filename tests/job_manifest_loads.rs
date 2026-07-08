use shiphappens::data::load_job_manifest;

#[test]
fn integration_loads_manifest_from_data_dir() {
    let jobs = load_job_manifest("data/job_manifest.json").expect("manifest loads");
    assert_eq!(jobs.len(), 10);
    assert!(jobs.iter().any(|job| job.id == "crane_of_regret"));
}
