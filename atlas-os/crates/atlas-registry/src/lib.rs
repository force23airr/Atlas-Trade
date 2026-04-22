use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EntityType {
    Government,
    Corporation,
    Vessel,
    Individual,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainRecord {
    pub owner: String,
    pub target_address: String,
    pub entity_type: EntityType,
}

#[derive(Error, Debug, PartialEq)]
pub enum RegistryError {
    #[error("Invalid domain name: {0}")]
    InvalidDomainName(String),
    #[error("Domain already exists: {0}")]
    DomainAlreadyExists(String),
    #[error("Domain not found: {0}")]
    DomainNotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

pub struct AtlasRegistry {
    records: HashMap<String, DomainRecord>,
}

impl AtlasRegistry {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Validates the domain name according to the .usdc standard.
    fn validate_domain(domain: &str) -> Result<(), RegistryError> {
        if !domain.ends_with(".usdc") {
            return Err(RegistryError::InvalidDomainName("Must end with .usdc".to_string()));
        }

        let prefix = &domain[..domain.len() - 5];
        if prefix.len() < 3 || prefix.len() > 63 {
            return Err(RegistryError::InvalidDomainName("Length must be between 3 and 63 characters".to_string()));
        }

        if prefix.starts_with('-') || prefix.ends_with('-') {
            return Err(RegistryError::InvalidDomainName("Cannot start or end with a hyphen".to_string()));
        }

        if !prefix.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(RegistryError::InvalidDomainName("Must contain only lowercase alphanumeric characters and hyphens".to_string()));
        }

        Ok(())
    }

    /// Registers a new .usdc domain.
    pub fn register(
        &mut self,
        domain: &str,
        owner: &str,
        target: &str,
        entity: EntityType,
    ) -> Result<(), RegistryError> {
        Self::validate_domain(domain)?;

        if self.records.contains_key(domain) {
            return Err(RegistryError::DomainAlreadyExists(domain.to_string()));
        }

        let record = DomainRecord {
            owner: owner.to_string(),
            target_address: target.to_string(),
            entity_type: entity,
        };

        self.records.insert(domain.to_string(), record);
        Ok(())
    }

    /// Resolves a domain to its target address.
    pub fn resolve(&self, domain: &str) -> Option<String> {
        self.records.get(domain).map(|r| r.target_address.clone())
    }

    /// Updates the target address for a domain. Only the owner can call this.
    pub fn update_target(
        &mut self,
        domain: &str,
        caller: &str,
        new_target: &str,
    ) -> Result<(), RegistryError> {
        let record = self.records.get_mut(domain)
            .ok_or_else(|| RegistryError::DomainNotFound(domain.to_string()))?;

        if record.owner != caller {
            return Err(RegistryError::Unauthorized("Caller is not the owner".to_string()));
        }

        record.target_address = new_target.to_string();
        Ok(())
    }

    /// Transfers ownership of a domain. Only the owner can call this.
    pub fn transfer(
        &mut self,
        domain: &str,
        caller: &str,
        new_owner: &str,
    ) -> Result<(), RegistryError> {
        let record = self.records.get_mut(domain)
            .ok_or_else(|| RegistryError::DomainNotFound(domain.to_string()))?;

        if record.owner != caller {
            return Err(RegistryError::Unauthorized("Caller is not the owner".to_string()));
        }

        record.owner = new_owner.to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_registration() {
        let mut registry = AtlasRegistry::new();
        let domain = "atlas-corp.usdc";
        let res = registry.register(domain, "owner1", "addr1", EntityType::Corporation);
        assert!(res.is_ok());
        assert_eq!(registry.resolve(domain), Some("addr1".to_string()));
    }

    #[test]
    fn test_invalid_names() {
        let mut registry = AtlasRegistry::new();
        
        // No .usdc suffix
        assert!(registry.register("invalid", "o", "a", EntityType::Individual).is_err());
        // Too short
        assert!(registry.register("ab.usdc", "o", "a", EntityType::Individual).is_err());
        // Uppercase
        assert!(registry.register("Atlas.usdc", "o", "a", EntityType::Individual).is_err());
        // Starts with hyphen
        assert!(registry.register("-atlas.usdc", "o", "a", EntityType::Individual).is_err());
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = AtlasRegistry::new();
        registry.register("atlas.usdc", "o1", "a1", EntityType::Individual).unwrap();
        let res = registry.register("atlas.usdc", "o2", "a2", EntityType::Individual);
        assert_eq!(res, Err(RegistryError::DomainAlreadyExists("atlas.usdc".to_string())));
    }

    #[test]
    fn test_update_target() {
        let mut registry = AtlasRegistry::new();
        let domain = "atlas.usdc";
        registry.register(domain, "owner1", "addr1", EntityType::Individual).unwrap();
        
        // Unauthorized
        assert!(registry.update_target(domain, "hacker", "addr2").is_err());
        
        // Success
        registry.update_target(domain, "owner1", "addr2").unwrap();
        assert_eq!(registry.resolve(domain), Some("addr2".to_string()));
    }

    #[test]
    fn test_transfer() {
        let mut registry = AtlasRegistry::new();
        let domain = "atlas.usdc";
        registry.register(domain, "owner1", "addr1", EntityType::Individual).unwrap();
        
        // Transfer
        registry.transfer(domain, "owner1", "owner2").unwrap();
        
        // Old owner cannot update
        assert!(registry.update_target(domain, "owner1", "addr2").is_err());
        
        // New owner can update
        registry.update_target(domain, "owner2", "addr2").unwrap();
        assert_eq!(registry.resolve(domain), Some("addr2".to_string()));
    }
}
