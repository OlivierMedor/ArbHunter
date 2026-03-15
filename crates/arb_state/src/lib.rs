use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::{debug, info};

use arb_metrics::MetricsRegistry;
use arb_types::{
    EventStamp, PoolFreshness, PoolId, PoolKind, PoolStateSnapshot, PoolUpdate,
    ReserveSnapshot, CLSnapshot,
};

/// Staleness threshold: pools not updated within this window are marked stale.
const STALE_THRESHOLD_MS: u64 = 30_000; // 30 seconds

// ============================================================
// Pool Adapter Stubs
// ============================================================

/// Adapter interface for reserve-based pools (Uniswap V2 style).
/// Phase 3: structure only — no quoting or route search.
pub struct ReservePoolAdapter;

impl ReservePoolAdapter {
    /// Apply a reserve update, returning an updated snapshot.
    /// In future phases this will also invalidate cached quotes.
    pub fn apply(snapshot: &mut PoolStateSnapshot, reserves: ReserveSnapshot, stamp: EventStamp) {
        snapshot.reserves = Some(reserves);
        snapshot.freshness = PoolFreshness {
            last_stamp: stamp,
            age_ms: 0,
            is_stale: false,
        };
    }
}

/// Adapter stub for concentrated-liquidity pools (Uniswap V3 style).
/// Phase 3: structure only — tick data model deferred to Phase 4+.
pub struct CLPoolAdapter;

impl CLPoolAdapter {
    /// Apply a concentrated-liquidity update, returning an updated snapshot.
    pub fn apply(snapshot: &mut PoolStateSnapshot, cl_snapshot: CLSnapshot, stamp: EventStamp) {
        snapshot.cl_snapshot = Some(cl_snapshot);
        snapshot.freshness = PoolFreshness {
            last_stamp: stamp,
            age_ms: 0,
            is_stale: false,
        };
    }
}

// ============================================================
// Pool Store
// ============================================================

/// In-memory canonical store for all pool states.
#[derive(Default)]
pub struct PoolStore {
    pools: HashMap<PoolId, PoolStateSnapshot>,
    /// Wall-clock instant of the last applied update per pool.
    last_seen: HashMap<PoolId, Instant>,
}

impl PoolStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a `PoolUpdate`, inserting or mutating the pool entry.
    /// Returns true if this was a newer update (not stale).
    pub fn apply_update(&mut self, update: PoolUpdate) -> bool {
        let pool_id = update.pool_id.clone();

        if let Some(existing) = self.pools.get(&pool_id) {
            // Reject out-of-order / duplicate updates
            if update.stamp <= existing.freshness.last_stamp {
                return false;
            }
        }

        let freshness = PoolFreshness {
            last_stamp: update.stamp,
            age_ms: 0,
            is_stale: false,
        };

        let snapshot = PoolStateSnapshot {
            pool_id: pool_id.clone(),
            kind: update.kind,
            token0: update.token0,
            token1: update.token1,
            reserves: update.reserves.clone(),
            cl_snapshot: update.cl_snapshot.clone(),
            freshness,
        };

        match update.kind {
            PoolKind::ReserveBased => {
                let entry = self.pools.entry(pool_id.clone()).or_insert(snapshot.clone());
                if let Some(reserves) = update.reserves {
                    ReservePoolAdapter::apply(entry, reserves, update.stamp);
                }
            }
            PoolKind::ConcentratedLiquidity => {
                let entry = self.pools.entry(pool_id.clone()).or_insert(snapshot.clone());
                if let Some(cl_snapshot) = update.cl_snapshot {
                    CLPoolAdapter::apply(entry, cl_snapshot, update.stamp);
                }
            }
            PoolKind::Unknown => {
                self.pools.insert(pool_id.clone(), snapshot);
            }
        };

        self.last_seen.insert(pool_id, Instant::now());
        true
    }

    /// Mark pools whose last-seen wall time exceeds `STALE_THRESHOLD_MS`.
    /// Returns how many pools were newly marked stale.
    pub fn refresh_freshness(&mut self) -> usize {
        let threshold = Duration::from_millis(STALE_THRESHOLD_MS);
        let mut newly_stale = 0usize;

        for (pool_id, snapshot) in self.pools.iter_mut() {
            if let Some(seen) = self.last_seen.get(pool_id) {
                let age_ms = seen.elapsed().as_millis() as u64;
                snapshot.freshness.age_ms = age_ms;
                let was_stale = snapshot.freshness.is_stale;
                snapshot.freshness.is_stale = seen.elapsed() > threshold;
                if snapshot.freshness.is_stale && !was_stale {
                    newly_stale += 1;
                }
            }
        }
        newly_stale
    }

    /// Return a cloned snapshot for a given pool, if it exists.
    pub fn get(&self, pool_id: &PoolId) -> Option<PoolStateSnapshot> {
        self.pools.get(pool_id).cloned()
    }

    /// Number of pools currently tracked.
    pub fn len(&self) -> usize {
        self.pools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pools.is_empty()
    }
}

// ============================================================
// State Engine
// ============================================================

/// Thread-safe in-memory state engine.
/// Wraps `PoolStore` behind an RwLock and increments Prometheus metrics on each update.
pub struct StateEngine {
    store: Arc<RwLock<PoolStore>>,
    metrics: Arc<MetricsRegistry>,
}

impl StateEngine {
    pub fn new(metrics: Arc<MetricsRegistry>) -> Self {
        Self {
            store: Arc::new(RwLock::new(PoolStore::new())),
            metrics,
        }
    }

    /// Apply a pool update.
    pub async fn apply(&self, update: PoolUpdate) {
        let mut store = self.store.write().await;
        let accepted = store.apply_update(update);
        if accepted {
            self.metrics.inc_state_updates();
            self.metrics.set_pools_tracked(store.len() as i64);
            debug!("State update applied. Pools tracked: {}", store.len());
        } else {
            self.metrics.inc_stale_pool_events();
        }
    }

    /// Tick: refresh staleness and update metrics.
    pub async fn tick_freshness(&self) {
        let mut store = self.store.write().await;
        let stale_count = store.refresh_freshness();
        if stale_count > 0 {
            info!("{} pool(s) newly marked stale", stale_count);
        }
        self.metrics.set_pools_tracked(store.len() as i64);
    }

    /// Read-only snapshot access.
    pub async fn get_pool(&self, pool_id: &PoolId) -> Option<PoolStateSnapshot> {
        self.store.read().await.get(pool_id)
    }

    /// Current pool count.
    pub async fn pool_count(&self) -> usize {
        self.store.read().await.len()
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use arb_types::{EventStamp, PoolId, PoolKind, PoolUpdate, ReserveSnapshot, TokenAddress};
    use std::sync::Arc;

    fn make_update(pool: &str, block: u64, r0: u128, r1: u128) -> PoolUpdate {
        PoolUpdate {
            pool_id: PoolId(pool.to_string()),
            kind: PoolKind::ReserveBased,
            token0: TokenAddress("0xAAA".to_string()),
            token1: TokenAddress("0xBBB".to_string()),
            reserves: Some(ReserveSnapshot { reserve0: r0, reserve1: r1 }),
            cl_snapshot: None,
            stamp: EventStamp { block_number: block, log_index: 0 },
        }
    }

    fn make_cl_update(pool: &str, block: u64, sqrt_p: u128, liq: u128, tick: i32) -> PoolUpdate {
        use alloy_primitives::{U128, U256};
        PoolUpdate {
            pool_id: PoolId(pool.to_string()),
            kind: PoolKind::ConcentratedLiquidity,
            token0: TokenAddress("0xAAA".to_string()),
            token1: TokenAddress("0xBBB".to_string()),
            reserves: None,
            cl_snapshot: Some(CLSnapshot {
                sqrt_price_x96: U256::from(sqrt_p),
                liquidity: U128::from(liq),
                tick,
            }),
            stamp: EventStamp { block_number: block, log_index: 0 },
        }
    }

    #[test]
    fn test_apply_fresh_update() {
        let mut store = PoolStore::new();
        let update = make_update("pool_a", 100, 1000, 2000);
        assert!(store.apply_update(update));
        assert_eq!(store.len(), 1);
        let snap = store.get(&PoolId("pool_a".to_string())).unwrap();
        assert_eq!(snap.reserves.unwrap().reserve0, 1000);
    }

    #[test]
    fn test_apply_cl_update() {
        let mut store = PoolStore::new();
        let update = make_cl_update("pool_v3", 100, 123456789, 1000000, 500);
        assert!(store.apply_update(update));
        assert_eq!(store.len(), 1);
        let snap = store.get(&PoolId("pool_v3".to_string())).unwrap();
        assert_eq!(snap.cl_snapshot.unwrap().tick, 500);
    }

    #[test]
    fn test_stale_update_rejected() {
        let mut store = PoolStore::new();
        store.apply_update(make_update("pool_a", 100, 1000, 2000));
        // block 99 is older — should be rejected
        let older = make_update("pool_a", 99, 9999, 9999);
        assert!(!store.apply_update(older));
        // Original state preserved
        let snap = store.get(&PoolId("pool_a".to_string())).unwrap();
        assert_eq!(snap.reserves.unwrap().reserve0, 1000);
    }

    #[test]
    fn test_multiple_pools() {
        let mut store = PoolStore::new();
        store.apply_update(make_update("pool_a", 1, 100, 200));
        store.apply_update(make_update("pool_b", 1, 300, 400));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_freshness_tracking() {
        let mut store = PoolStore::new();
        store.apply_update(make_update("pool_a", 1, 100, 200));
        // Immediately after apply, age is near 0 — should not be stale
        let stale = store.refresh_freshness();
        assert_eq!(stale, 0);
        let snap = store.get(&PoolId("pool_a".to_string())).unwrap();
        assert!(!snap.freshness.is_stale);
    }

    #[test]
    fn test_unknown_pool_kind() {
        let mut store = PoolStore::new();
        let update = PoolUpdate {
            pool_id: PoolId("pool_x".to_string()),
            kind: PoolKind::Unknown,
            token0: TokenAddress("0xAAA".to_string()),
            token1: TokenAddress("0xBBB".to_string()),
            reserves: None,
            cl_snapshot: None,
            stamp: EventStamp { block_number: 1, log_index: 0 },
        };
        assert!(store.apply_update(update));
        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn test_state_engine_apply() {
        let metrics = Arc::new(MetricsRegistry::new());
        let engine = StateEngine::new(metrics);
        engine.apply(make_update("pool_a", 50, 500, 1000)).await;
        assert_eq!(engine.pool_count().await, 1);
        let snap = engine.get_pool(&PoolId("pool_a".to_string())).await;
        assert!(snap.is_some());
    }

    #[tokio::test]
    async fn test_state_engine_rejects_stale() {
        let metrics = Arc::new(MetricsRegistry::new());
        let engine = StateEngine::new(metrics);
        engine.apply(make_update("pool_a", 100, 1000, 2000)).await;
        engine.apply(make_update("pool_a", 90, 9999, 9999)).await;
        let snap = engine.get_pool(&PoolId("pool_a".to_string())).await.unwrap();
        assert_eq!(snap.reserves.unwrap().reserve0, 1000);
    }

    #[tokio::test]
    async fn test_replay_fixture_sequence() {
        let metrics = Arc::new(MetricsRegistry::new());
        let engine = StateEngine::new(metrics);

        // Simulate a sequence of events as if replayed from a fixture file
        let updates = vec![
            make_update("pool_0x1234", 10, 1_000_000, 2_000_000),
            make_update("pool_0x5678", 10, 500_000, 1_000_000),
            make_update("pool_0x1234", 11, 1_100_000, 1_900_000), // newer block
            make_update("pool_0x1234", 9, 999, 999),               // older — rejected
        ];

        for u in updates {
            engine.apply(u).await;
        }

        assert_eq!(engine.pool_count().await, 2);

        let snap1 = engine.get_pool(&PoolId("pool_0x1234".to_string())).await.unwrap();
        assert_eq!(snap1.reserves.unwrap().reserve0, 1_100_000); // block 11 wins

        let snap2 = engine.get_pool(&PoolId("pool_0x5678".to_string())).await.unwrap();
        assert_eq!(snap2.reserves.unwrap().reserve0, 500_000);
    }
}
