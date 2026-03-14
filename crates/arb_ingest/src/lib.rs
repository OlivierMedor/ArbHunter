use std::fs;
use std::path::Path;
use tokio::sync::broadcast;

use arb_types::{FlashblockEvent, IngestEvent, PendingLogEvent};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum RawPayload {
    Flashblock(FlashblockEvent),
    PendingLog(PendingLogEvent),
}

pub struct IngestPipeline {
    tx: broadcast::Sender<IngestEvent>,
}

impl IngestPipeline {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IngestEvent> {
        self.tx.subscribe()
    }

    pub fn broadcast_event(&self, event: IngestEvent) -> Result<usize, broadcast::error::SendError<IngestEvent>> {
        self.tx.send(event)
    }

    pub fn handle_raw_payload(&self, payload: &str) {
        // Attempt to parse exactly using structure
        match serde_json::from_str::<RawPayload>(payload) {
            Ok(RawPayload::Flashblock(fb)) => {
                let _ = self.broadcast_event(IngestEvent::Flashblock(fb));
            }
            Ok(RawPayload::PendingLog(pl)) => {
                let _ = self.broadcast_event(IngestEvent::PendingLog(pl));
            }
            Err(e) => {
                // Ignore parsing errors for unknown messages, standard for raw provider stream filters
                let _ = e;
            }
        }
    }
}

pub struct ReplayHarness {
    fixture_path: String,
}

impl ReplayHarness {
    pub fn new(fixture_path: String) -> Self {
        Self { fixture_path }
    }

    pub async fn run_replay(&self, pipeline: &IngestPipeline) -> Result<(), String> {
        let path = Path::new(&self.fixture_path);
        if !path.exists() {
            return Err("Fixture file does not exist".to_string());
        }

        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        for line in content.lines() {
            if !line.trim().is_empty() {
                pipeline.handle_raw_payload(line);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_ingest_pipeline_broadcast() {
        let pipeline = IngestPipeline::new(10);
        let mut rx = pipeline.subscribe();

        // Valid Structured JSON
        pipeline.handle_raw_payload(r#"{"type":"flashblock","data":{"base_fee_per_gas":10,"block_number":1000,"transaction_count":5}}"#);
        
        if let Ok(IngestEvent::Flashblock(fb)) = rx.recv().await {
            assert_eq!(fb.block_number, 1000);
        } else {
            panic!("Expected Flashblock event");
        }
    }

    #[tokio::test]
    async fn test_replay_harness() {
        let path = "test_fixture.jsonl";
        let mut file = std::fs::File::create(path).unwrap();
        // Writing valid matching structs
        writeln!(file, r#"{{"type":"flashblock","data":{{"base_fee_per_gas":10,"block_number":1001,"transaction_count":5}}}}"#).unwrap();
        writeln!(file, r#"{{"type":"pending_log","data":{{"address":"0x0","topics":[],"data":"0x","transaction_hash":"0x0"}}}}"#).unwrap();

        let pipeline = IngestPipeline::new(10);
        let mut rx = pipeline.subscribe();
        
        let harness = ReplayHarness::new(path.to_string());
        assert!(harness.run_replay(&pipeline).await.is_ok());

        assert!(matches!(rx.recv().await.unwrap(), IngestEvent::Flashblock(_)));
        assert!(matches!(rx.recv().await.unwrap(), IngestEvent::PendingLog(_)));

        std::fs::remove_file(path).unwrap();
    }
}
