use arb_calibration::CalibrationAnalyzer;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let input_path = env::var("CANDIDATES_JSONL_PATH")
        .unwrap_or_else(|_| "historical_replay_full_day_candidates.jsonl".to_string());
    let output_path = env::var("CALIBRATION_REPORT_PATH")
        .unwrap_or_else(|_| "execution_calibration_report.json".to_string());

    println!("Starting Calibration Analysis of {}...", input_path);
    
    let analyzer = CalibrationAnalyzer::new(&input_path)?;
    let report = analyzer.analyze()?;
    
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&output_path, json)?;
    
    // Export stratified verification cases for arb_battery
    let cases_path = "fixtures/historical_cases_phase_18.json";
    let cases_json = serde_json::to_string_pretty(&report.sampled_verification_cases)?;
    std::fs::write(cases_path, cases_json)?;

    println!("Calibration Report generated at {}", output_path);
    println!("Stratified Verification Cases (40) exported to {}", cases_path);
    println!("Total candidates analyzed: {}", report.total_candidates);
    println!("0.05 ETH+ Count: {}", report.bucket_counts.get("0.05 ETH+").unwrap_or(&0));
    println!("Batchability Potential: {}", report.batching_potential_common_sense);

    Ok(())
}
