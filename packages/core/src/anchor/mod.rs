// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Anchor Framework Integration for LUMOS
//!
//! This module provides deep integration between LUMOS schemas and the Anchor framework,
//! enabling seamless Solana program development.
//!
//! # Features
//!
//! - **IDL Generation**: Generate Anchor IDL JSON from LUMOS schemas
//! - **Account Space Calculation**: Auto-calculate account sizes with discriminator
//! - **Type Mapping**: Convert LUMOS types to Anchor IDL types
//!
//! # Example
//!
//! ```ignore
//! use lumos_core::anchor::{IdlGenerator, IdlGeneratorConfig};
//! use lumos_core::transform_to_ir;
//! use lumos_core::parser::parse_lumos_file;
//!
//! let source = r#"
//!     #[solana]
//!     #[account]
//!     struct PlayerAccount {
//!         authority: PublicKey,
//!         level: u16,
//!         experience: u64,
//!     }
//! "#;
//!
//! let ast = parse_lumos_file(source).unwrap();
//! let ir = transform_to_ir(ast).unwrap();
//!
//! let config = IdlGeneratorConfig {
//!     program_name: "my_game".to_string(),
//!     version: "0.1.0".to_string(),
//! };
//!
//! let generator = IdlGenerator::new(config);
//! let idl = generator.generate(&ir);
//! let json = serde_json::to_string_pretty(&idl).unwrap();
//! ```

mod attributes;
mod idl;

pub use attributes::{
    generate_accounts_context, parse_anchor_attrs, parse_instruction_context, AnchorAccountAttr,
    AnchorAccountType, AnchorFieldAttrs, InstructionAccount, InstructionArg, InstructionContext,
    SeedComponent,
};
pub use idl::{
    Idl, IdlAccount, IdlAccountItem, IdlEnumVariant, IdlField, IdlGenerator, IdlGeneratorConfig,
    IdlInstruction, IdlType, IdlTypeDef, IdlTypeDefTy,
};
