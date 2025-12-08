// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Metaplex Token Metadata standard types
//!
//! This module defines the standard Metaplex types that can be used in LUMOS schemas
//! to ensure compatibility with the Metaplex ecosystem.

use serde::{Deserialize, Serialize};

/// Metaplex Token Metadata constraints
pub mod constraints {
    /// Maximum length for NFT name (32 characters)
    pub const MAX_NAME_LENGTH: usize = 32;

    /// Maximum length for NFT symbol (10 characters)
    pub const MAX_SYMBOL_LENGTH: usize = 10;

    /// Maximum length for NFT URI (200 characters)
    pub const MAX_URI_LENGTH: usize = 200;

    /// Maximum number of creators (5)
    pub const MAX_CREATORS: usize = 5;

    /// Maximum seller fee basis points (100% = 10000)
    pub const MAX_SELLER_FEE_BASIS_POINTS: u16 = 10000;

    /// Creator shares must sum to this value (100%)
    pub const CREATOR_SHARES_TOTAL: u8 = 100;
}

/// Token Standard enum for Metaplex
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenStandard {
    /// Standard NFT (1/1)
    NonFungible,

    /// Fungible asset with metadata
    FungibleAsset,

    /// Standard fungible token
    Fungible,

    /// Edition of a Master NFT
    NonFungibleEdition,

    /// Programmable NFT with rules
    ProgrammableNonFungible,

    /// Programmable edition
    ProgrammableNonFungibleEdition,
}

impl TokenStandard {
    /// Get the discriminant value for Borsh serialization
    pub fn discriminant(&self) -> u8 {
        match self {
            TokenStandard::NonFungible => 0,
            TokenStandard::FungibleAsset => 1,
            TokenStandard::Fungible => 2,
            TokenStandard::NonFungibleEdition => 3,
            TokenStandard::ProgrammableNonFungible => 4,
            TokenStandard::ProgrammableNonFungibleEdition => 5,
        }
    }

    /// Create from discriminant
    pub fn from_discriminant(value: u8) -> Option<Self> {
        match value {
            0 => Some(TokenStandard::NonFungible),
            1 => Some(TokenStandard::FungibleAsset),
            2 => Some(TokenStandard::Fungible),
            3 => Some(TokenStandard::NonFungibleEdition),
            4 => Some(TokenStandard::ProgrammableNonFungible),
            5 => Some(TokenStandard::ProgrammableNonFungibleEdition),
            _ => None,
        }
    }
}

/// Use method for NFTs (burning, multiple uses, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseMethod {
    /// Single use - burns after use
    Burn,

    /// Multiple uses with a limit
    Multiple,

    /// Single use but doesn't burn
    Single,
}

impl UseMethod {
    /// Get the discriminant value
    pub fn discriminant(&self) -> u8 {
        match self {
            UseMethod::Burn => 0,
            UseMethod::Multiple => 1,
            UseMethod::Single => 2,
        }
    }
}

/// Metaplex standard type definitions for LUMOS schemas
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaplexType {
    /// Token Metadata data structure
    TokenMetadata,

    /// Creator with address, verified status, and share
    Creator,

    /// Collection reference
    Collection,

    /// Uses configuration
    Uses,

    /// Master Edition v2
    MasterEditionV2,

    /// Edition account
    Edition,

    /// Edition marker for large collections
    EditionMarker,
}

impl MetaplexType {
    /// Get the struct name for this type
    pub fn struct_name(&self) -> &'static str {
        match self {
            MetaplexType::TokenMetadata => "Metadata",
            MetaplexType::Creator => "Creator",
            MetaplexType::Collection => "Collection",
            MetaplexType::Uses => "Uses",
            MetaplexType::MasterEditionV2 => "MasterEditionV2",
            MetaplexType::Edition => "Edition",
            MetaplexType::EditionMarker => "EditionMarker",
        }
    }

    /// Get all standard Metaplex types
    pub fn all() -> Vec<MetaplexType> {
        vec![
            MetaplexType::TokenMetadata,
            MetaplexType::Creator,
            MetaplexType::Collection,
            MetaplexType::Uses,
            MetaplexType::MasterEditionV2,
            MetaplexType::Edition,
            MetaplexType::EditionMarker,
        ]
    }
}

/// Metaplex attribute types for #[metaplex(...)] attribute
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaplexAttribute {
    /// Mark struct as Metaplex-compatible metadata
    Metadata,

    /// Mark struct as a Creator type
    Creator,

    /// Mark struct as a Collection type
    Collection,

    /// Mark struct as Uses type
    Uses,

    /// Mark struct as Master Edition
    MasterEdition,

    /// Mark struct as Edition
    Edition,

    /// Custom validation rule
    Validate(String),

    /// NFT name field (max 32 chars)
    Name,

    /// NFT symbol field (max 10 chars)
    Symbol,

    /// NFT URI field (max 200 chars)
    Uri,

    /// Seller fee basis points (0-10000)
    SellerFeeBasisPoints,

    /// Creators array
    Creators,

    /// Creator share (must sum to 100)
    Share,
}

impl MetaplexAttribute {
    /// Parse from string (not to be confused with std::str::FromStr)
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "metadata" => Some(MetaplexAttribute::Metadata),
            "creator" => Some(MetaplexAttribute::Creator),
            "collection" => Some(MetaplexAttribute::Collection),
            "uses" => Some(MetaplexAttribute::Uses),
            "master_edition" => Some(MetaplexAttribute::MasterEdition),
            "edition" => Some(MetaplexAttribute::Edition),
            "name" => Some(MetaplexAttribute::Name),
            "symbol" => Some(MetaplexAttribute::Symbol),
            "uri" => Some(MetaplexAttribute::Uri),
            "seller_fee_basis_points" => Some(MetaplexAttribute::SellerFeeBasisPoints),
            "creators" => Some(MetaplexAttribute::Creators),
            "share" => Some(MetaplexAttribute::Share),
            s if s.starts_with("validate") => {
                let rule = s.strip_prefix("validate")?.trim();
                let rule = rule.strip_prefix('=')?.trim();
                let rule = rule.trim_matches('"');
                Some(MetaplexAttribute::Validate(rule.to_string()))
            }
            _ => None,
        }
    }
}

/// Parsed metaplex attributes for a struct or field
#[derive(Debug, Clone, Default)]
pub struct ParsedMetaplexAttrs {
    /// Struct-level attributes
    pub struct_type: Option<MetaplexAttribute>,

    /// Field-level attributes
    pub field_attrs: Vec<MetaplexAttribute>,

    /// Custom validation rules
    pub validations: Vec<String>,
}

impl ParsedMetaplexAttrs {
    /// Check if this is a metadata struct
    pub fn is_metadata(&self) -> bool {
        matches!(self.struct_type, Some(MetaplexAttribute::Metadata))
    }

    /// Check if this is a creator struct
    pub fn is_creator(&self) -> bool {
        matches!(self.struct_type, Some(MetaplexAttribute::Creator))
    }

    /// Check if this is a collection struct
    pub fn is_collection(&self) -> bool {
        matches!(self.struct_type, Some(MetaplexAttribute::Collection))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_standard_discriminant() {
        assert_eq!(TokenStandard::NonFungible.discriminant(), 0);
        assert_eq!(TokenStandard::FungibleAsset.discriminant(), 1);
        assert_eq!(TokenStandard::Fungible.discriminant(), 2);
        assert_eq!(TokenStandard::NonFungibleEdition.discriminant(), 3);
        assert_eq!(TokenStandard::ProgrammableNonFungible.discriminant(), 4);
    }

    #[test]
    fn test_token_standard_from_discriminant() {
        assert_eq!(
            TokenStandard::from_discriminant(0),
            Some(TokenStandard::NonFungible)
        );
        assert_eq!(
            TokenStandard::from_discriminant(4),
            Some(TokenStandard::ProgrammableNonFungible)
        );
        assert_eq!(TokenStandard::from_discriminant(99), None);
    }

    #[test]
    fn test_metaplex_attribute_parsing() {
        assert_eq!(
            MetaplexAttribute::parse_from_str("metadata"),
            Some(MetaplexAttribute::Metadata)
        );
        assert_eq!(
            MetaplexAttribute::parse_from_str("creator"),
            Some(MetaplexAttribute::Creator)
        );
        assert_eq!(
            MetaplexAttribute::parse_from_str("name"),
            Some(MetaplexAttribute::Name)
        );
        assert_eq!(
            MetaplexAttribute::parse_from_str("seller_fee_basis_points"),
            Some(MetaplexAttribute::SellerFeeBasisPoints)
        );
    }

    #[test]
    fn test_metaplex_attribute_validate() {
        let attr = MetaplexAttribute::parse_from_str("validate = \"len <= 32\"");
        assert!(matches!(attr, Some(MetaplexAttribute::Validate(_))));
        if let Some(MetaplexAttribute::Validate(rule)) = attr {
            assert_eq!(rule, "len <= 32");
        }
    }

    #[test]
    fn test_metaplex_type_struct_names() {
        assert_eq!(MetaplexType::TokenMetadata.struct_name(), "Metadata");
        assert_eq!(MetaplexType::Creator.struct_name(), "Creator");
        assert_eq!(MetaplexType::Collection.struct_name(), "Collection");
        assert_eq!(MetaplexType::MasterEditionV2.struct_name(), "MasterEditionV2");
    }

    #[test]
    fn test_constraints() {
        assert_eq!(constraints::MAX_NAME_LENGTH, 32);
        assert_eq!(constraints::MAX_SYMBOL_LENGTH, 10);
        assert_eq!(constraints::MAX_URI_LENGTH, 200);
        assert_eq!(constraints::MAX_CREATORS, 5);
        assert_eq!(constraints::MAX_SELLER_FEE_BASIS_POINTS, 10000);
        assert_eq!(constraints::CREATOR_SHARES_TOTAL, 100);
    }
}
