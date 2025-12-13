// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Metaplex Token Metadata Integration for LUMOS
//!
//! This module provides compatibility between LUMOS schemas and the Metaplex Token Metadata
//! standard, enabling seamless NFT development on Solana.
//!
//! # Features
//!
//! - **Standard Types**: Pre-defined Metaplex types (Metadata, Creator, Collection, etc.)
//! - **Validation**: Schema validation against Metaplex constraints
//! - **Code Generation**: Generate Metaplex-compatible Rust and TypeScript code
//!
//! # Example
//!
//! ```ignore
//! use lumos_core::metaplex::{MetaplexValidator, MetaplexGenerator};
//! use lumos_core::transform_to_ir;
//! use lumos_core::parser::parse_lumos_file;
//!
//! let source = r#"
//!     #[solana]
//!     #[metaplex(metadata)]
//!     struct NftMetadata {
//!         name: String,
//!         symbol: String,
//!         uri: String,
//!         seller_fee_basis_points: u16,
//!     }
//!
//!     #[solana]
//!     #[metaplex(creator)]
//!     struct Creator {
//!         address: PublicKey,
//!         verified: bool,
//!         share: u8,
//!     }
//! "#;
//!
//! let ast = parse_lumos_file(source).unwrap();
//! let ir = transform_to_ir(ast).unwrap();
//!
//! // Validate against Metaplex standards
//! let validator = MetaplexValidator::new();
//! let validation_result = validator.validate(&ir);
//!
//! // Generate Metaplex-compatible code
//! let generator = MetaplexGenerator::new();
//! let rust_code = generator.generate_rust(&ir);
//! let typescript_code = generator.generate_typescript(&ir);
//! ```
//!
//! # Metaplex Constraints
//!
//! The module enforces Metaplex Token Metadata constraints:
//!
//! | Field | Constraint |
//! |-------|------------|
//! | name | Max 32 characters |
//! | symbol | Max 10 characters |
//! | uri | Max 200 characters |
//! | seller_fee_basis_points | 0-10000 (100% = 10000) |
//! | creators | Max 5 creators |
//! | creator shares | Must sum to 100 |

mod generator;
mod types;
mod validator;

pub use generator::{MetaplexGenerator, MetaplexGeneratorConfig};
pub use types::{
    constraints, MetaplexAttribute, MetaplexType, ParsedMetaplexAttrs, TokenStandard, UseMethod,
};
pub use validator::{MetaplexValidator, Severity, ValidationError, ValidationResult};

/// Generate standard Metaplex type definitions as LUMOS schema string
pub use validator::generate_standard_types;
