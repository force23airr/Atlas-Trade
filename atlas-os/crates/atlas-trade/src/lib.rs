use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum EscrowStatus {
    AwaitingArrival,
    FundsLocked,
    Settled,
    Disputed,
}

pub struct TradeEscrow {
    pub trade_id: String,
    pub amount: f64,
    pub vessel_handle: String,    // e.g., "tanker-alpha.usdc"
    pub target_coordinates: (f32, f32), // GPS Lat/Long
    pub status: EscrowStatus,
}

impl TradeEscrow {
    pub fn new(id: &str, amount: f64, vessel: &str, lat: f32, lon: f32) -> Self {
        Self {
            trade_id: id.to_string(),
            amount,
            vessel_handle: vessel.to_string(),
            target_coordinates: (lat, lon),
            status: EscrowStatus::FundsLocked,
        }
    }

    /// The "Magic" Trigger: Recharges the world's trade velocity.
    /// This would be called by an Oracle when GPS coordinates match.
    pub fn release_on_arrival(&mut self, current_lat: f32, current_lon: f32) -> Result<String, String> {
        let (target_lat, target_lon) = self.target_coordinates;
        
        // Simple 2026 GPS geofence logic (approximate within a few meters)
        if (current_lat - target_lat).abs() < 0.01 && (current_lon - target_lon).abs() < 0.01 {
            self.status = EscrowStatus::Settled;
            Ok(format!("SUCCESS: ${} released to {}", self.amount, self.vessel_handle))
        } else {
            Err("VESSEL_NOT_IN_PORT: Funds remain locked in escrow.".to_string())
        }
    }
}

