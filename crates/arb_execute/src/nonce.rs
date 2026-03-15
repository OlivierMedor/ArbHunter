use std::sync::atomic::{AtomicU64, Ordering};

/// A minimal, deterministic nonce manager.
/// For Phase 9, it focuses on sequential execution for a single address.
pub struct NonceManager {
    next_nonce: AtomicU64,
}

impl NonceManager {
    /// Creates a new NonceManager starting from the given nonce.
    pub fn new(start_nonce: u64) -> Self {
        Self {
            next_nonce: AtomicU64::new(start_nonce),
        }
    }

    /// Returns the next available nonce and increments the internal counter.
    pub fn next(&self) -> u64 {
        self.next_nonce.fetch_add(1, Ordering::SeqCst)
    }

    /// Peek at the current next nonce without incrementing.
    pub fn peek(&self) -> u64 {
        self.next_nonce.load(Ordering::SeqCst)
    }

    /// Resets the nonce to a specific value (e.g. after a provider sync).
    pub fn reset(&self, new_nonce: u64) {
        self.next_nonce.store(new_nonce, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_manager_sequence() {
        let nm = NonceManager::new(10);
        assert_eq!(nm.peek(), 10);
        assert_eq!(nm.next(), 10);
        assert_eq!(nm.next(), 11);
        assert_eq!(nm.peek(), 12);
    }

    #[test]
    fn test_nonce_manager_reset() {
        let nm = NonceManager::new(10);
        nm.next();
        nm.reset(20);
        assert_eq!(nm.next(), 20);
    }
}
