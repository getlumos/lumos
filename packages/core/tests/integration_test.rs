// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Integration tests for LUMOS parser with real example schemas

use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_parse_gaming_schema() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from packages/core
    path.pop(); // Go up from packages
    path.push("examples/gaming/schema.lumos");

    let content = fs::read_to_string(&path).expect("Failed to read gaming schema file");

    // Parse to AST
    let ast = parse_lumos_file(&content).expect("Failed to parse gaming schema");

    // Should have 4 items: PlayerAccount, GameItem, Leaderboard, MatchResult
    assert_eq!(ast.items.len(), 4);

    // Check struct names
    if let lumos_core::ast::Item::Struct(s) = &ast.items[0] {
        assert_eq!(s.name, "PlayerAccount");
    }
    if let lumos_core::ast::Item::Struct(s) = &ast.items[1] {
        assert_eq!(s.name, "GameItem");
    }
    if let lumos_core::ast::Item::Struct(s) = &ast.items[2] {
        assert_eq!(s.name, "Leaderboard");
    }
    if let lumos_core::ast::Item::Struct(s) = &ast.items[3] {
        assert_eq!(s.name, "MatchResult");
    }

    // Transform to IR
    let ir = transform_to_ir(ast).expect("Failed to transform to IR");

    assert_eq!(ir.len(), 4);

    // Verify PlayerAccount has @solana and @account attributes
    assert!(ir[0].metadata().unwrap().solana);
    assert!(ir[0]
        .metadata()
        .unwrap()
        .attributes
        .contains(&"account".to_string()));
}

#[test]
fn test_parse_nft_marketplace_schema() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples/nft-marketplace/schema.lumos");

    let content = fs::read_to_string(&path).expect("Failed to read NFT marketplace schema file");

    let ast = parse_lumos_file(&content).expect("Failed to parse NFT marketplace schema");

    // Should have 4 items: Marketplace, NftListing, NftMetadata, PurchaseReceipt
    assert_eq!(ast.items.len(), 4);

    let ir = transform_to_ir(ast).expect("Failed to transform to IR");

    assert_eq!(ir.len(), 4);
}

#[test]
fn test_parse_defi_staking_schema() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples/defi-staking/schema.lumos");

    let content = fs::read_to_string(&path).expect("Failed to read DeFi staking schema file");

    let ast = parse_lumos_file(&content).expect("Failed to parse DeFi staking schema");

    assert!(!ast.items.is_empty());

    let ir = transform_to_ir(ast).expect("Failed to transform to IR");

    assert!(!ir.is_empty());
}

#[test]
fn test_parse_token_vesting_schema() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples/token-vesting/schema.lumos");

    let content = fs::read_to_string(&path).expect("Failed to read token vesting schema file");

    let ast = parse_lumos_file(&content).expect("Failed to parse token vesting schema");

    assert!(!ast.items.is_empty());

    let ir = transform_to_ir(ast).expect("Failed to transform to IR");

    assert!(!ir.is_empty());
}

#[test]
fn test_parse_dao_governance_schema() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples/dao-governance/schema.lumos");

    let content = fs::read_to_string(&path).expect("Failed to read DAO governance schema file");

    let ast = parse_lumos_file(&content).expect("Failed to parse DAO governance schema");

    assert!(!ast.items.is_empty());

    let ir = transform_to_ir(ast).expect("Failed to transform to IR");

    assert!(!ir.is_empty());
}
