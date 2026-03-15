use arb_types::CandidateOpportunity;
use alloy_primitives::U256;

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub min_gross_profit: U256,
    pub min_gross_bps: u32,
    pub require_fresh: bool,
}

pub struct CandidateFilter {
    config: FilterConfig,
}

impl CandidateFilter {
    pub fn new(config: FilterConfig) -> Self {
        Self { config }
    }

    /// Promote candidates that pass the filtering criteria.
    pub fn filter_candidates(&self, candidates: Vec<CandidateOpportunity>) -> Vec<CandidateOpportunity> {
        candidates.into_iter()
            .filter(|c| self.should_promote(c))
            .collect()
    }

    fn should_promote(&self, candidate: &CandidateOpportunity) -> bool {
        if self.config.require_fresh && !candidate.is_fresh {
            return false;
        }

        if candidate.estimated_gross_profit < self.config.min_gross_profit {
            return false;
        }

        if candidate.estimated_gross_bps < self.config.min_gross_bps {
            return false;
        }

        true
    }
}
