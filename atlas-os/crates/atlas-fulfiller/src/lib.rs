use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FulfillmentStatus {
    /// Escrow is settled, cargo is at port, waiting to offload.
    Pending,
    /// Cargo is being physically unloaded from the carrier.
    Offloading,
    /// Local authority has cleared the goods for final delivery.
    CustomsCleared,
    /// Delivery confirmed to the buyer's destination.
    Completed,
    /// Issue reported during fulfillment.
    Disputed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FulfillmentRecord {
    pub trade_id: String,
    pub carrier_handle: String, // e.g., "tanker-alpha.usdc"
    pub customs_handle: String, // e.g., "rotterdam-customs.usdc"
    pub status: FulfillmentStatus,
    pub verification_hash: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum FulfillerError {
    #[error("Invalid state transition: cannot move from {0:?} to {1:?}")]
    InvalidTransition(FulfillmentStatus, FulfillmentStatus),
    #[error("Unauthorized: approval must come from {0}")]
    UnauthorizedAuthority(String),
}

impl FulfillmentRecord {
    pub fn new(trade_id: &str, carrier: &str, customs: &str) -> Self {
        Self {
            trade_id: trade_id.to_string(),
            carrier_handle: carrier.to_string(),
            customs_handle: customs.to_string(),
            status: FulfillmentStatus::Pending,
            verification_hash: None,
        }
    }

    /// Transition to offloading stage.
    pub fn start_offloading(&mut self) -> Result<(), FulfillerError> {
        if self.status != FulfillmentStatus::Pending {
            return Err(FulfillerError::InvalidTransition(self.status.clone(), FulfillmentStatus::Offloading));
        }
        self.status = FulfillmentStatus::Offloading;
        Ok(())
    }

    /// Transition to customs cleared stage.
    /// In 2026, this would involve verifying a cryptographic signature from the customs authority.
    pub fn clear_customs(&mut self, authority_handle: &str) -> Result<(), FulfillerError> {
        if self.status != FulfillmentStatus::Offloading {
            return Err(FulfillerError::InvalidTransition(self.status.clone(), FulfillmentStatus::CustomsCleared));
        }
        if authority_handle != self.customs_handle {
            return Err(FulfillerError::UnauthorizedAuthority(self.customs_handle.clone()));
        }
        self.status = FulfillmentStatus::CustomsCleared;
        Ok(())
    }

    /// Finalize the delivery and generate a verification hash (Proof of Delivery).
    pub fn finalize_delivery(&mut self) -> Result<String, FulfillerError> {
        if self.status != FulfillmentStatus::CustomsCleared {
            return Err(FulfillerError::InvalidTransition(self.status.clone(), FulfillmentStatus::Completed));
        }
        
        // Simulating the generation of a final immutable proof of the completed trade cycle.
        let mock_hash = format!("proof_of_delivery_{}", self.trade_id);
        self.verification_hash = Some(mock_hash.clone());
        self.status = FulfillmentStatus::Completed;
        
        Ok(mock_hash)
    }

    /// Mark the fulfillment as disputed.
    pub fn dispute(&mut self) {
        self.status = FulfillmentStatus::Disputed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path_fulfillment() {
        let mut record = FulfillmentRecord::new("TRADE-1", "tanker-alpha.usdc", "rotterdam-customs.usdc");
        assert_eq!(record.status, FulfillmentStatus::Pending);

        // 1. Offload
        record.start_offloading().unwrap();
        assert_eq!(record.status, FulfillmentStatus::Offloading);

        // 2. Customs
        record.clear_customs("rotterdam-customs.usdc").unwrap();
        assert_eq!(record.status, FulfillmentStatus::CustomsCleared);

        // 3. Finalize
        let hash = record.finalize_delivery().unwrap();
        assert_eq!(record.status, FulfillmentStatus::Completed);
        assert!(record.verification_hash.is_some());
        assert_eq!(hash, "proof_of_delivery_TRADE-1");
    }

    #[test]
    fn test_invalid_transition() {
        let mut record = FulfillmentRecord::new("TRADE-1", "t", "c");
        // Cannot clear customs before offloading
        let res = record.clear_customs("c");
        assert!(res.is_err());
    }

    #[test]
    fn test_unauthorized_customs() {
        let mut record = FulfillmentRecord::new("TRADE-1", "t", "rotterdam-customs.usdc");
        record.start_offloading().unwrap();
        // Wrong authority
        let res = record.clear_customs("fake-customs.usdc");
        assert_eq!(res, Err(FulfillerError::UnauthorizedAuthority("rotterdam-customs.usdc".to_string())));
    }
}
