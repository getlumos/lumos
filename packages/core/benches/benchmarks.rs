// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 getlumos

//! Performance benchmarks for LUMOS core components
//!
//! Measures performance of:
//! - Parser (small, medium, large schemas)
//! - Transformer (AST → IR)
//! - Rust generator
//! - TypeScript generator
//! - End-to-end pipeline

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lumos_core::{generators, parser, transform};

// ===== Test Schemas =====

const SMALL_SCHEMA: &str = r#"
    #[solana]
    #[account]
    struct User {
        id: u64,
        name: String,
    }
"#;

const MEDIUM_SCHEMA: &str = r#"
    #[solana]
    #[account]
    struct PlayerAccount {
        wallet: PublicKey,
        level: u16,
        experience: u64,
        equipped_items: [PublicKey],
    }

    #[solana]
    struct MatchResult {
        player: PublicKey,
        opponent: Option<PublicKey>,
        score: u64,
        timestamp: i64,
    }

    #[solana]
    enum GameState {
        Active,
        Paused,
        Finished,
    }
"#;

const LARGE_SCHEMA: &str = r#"
    #[solana]
    #[account]
    struct DAOConfig {
        authority: PublicKey,
        voting_period: u64,
        quorum_percentage: u8,
        proposal_threshold: u64,
    }

    #[solana]
    #[account]
    struct Proposal {
        id: u64,
        proposer: PublicKey,
        description: String,
        start_time: i64,
        end_time: i64,
        yes_votes: u64,
        no_votes: u64,
        executed: bool,
    }

    #[solana]
    #[account]
    struct Vote {
        proposal_id: u64,
        voter: PublicKey,
        vote_type: VoteType,
        timestamp: i64,
    }

    #[solana]
    #[account]
    struct Member {
        wallet: PublicKey,
        voting_power: u64,
        joined_at: i64,
    }

    #[solana]
    enum VoteType {
        Yes,
        No,
        Abstain,
    }

    #[solana]
    enum ProposalStatus {
        Draft,
        Active,
        Succeeded,
        Failed,
        Executed,
    }

    #[solana]
    enum GameEvent {
        PlayerJoined(PublicKey),
        ScoreUpdated(PublicKey, u64),
        GameEnded(PublicKey, u64),
    }

    #[solana]
    enum GameInstruction {
        Initialize {
            authority: PublicKey,
            max_players: u32,
        },
        UpdateScore {
            player: PublicKey,
            new_score: u64,
        },
        Terminate,
    }
"#;

// ===== Parser Benchmarks =====

fn bench_parser_small(c: &mut Criterion) {
    c.bench_function("parser_small_schema", |b| {
        b.iter(|| parser::parse_lumos_file(black_box(SMALL_SCHEMA)))
    });
}

fn bench_parser_medium(c: &mut Criterion) {
    c.bench_function("parser_medium_schema", |b| {
        b.iter(|| parser::parse_lumos_file(black_box(MEDIUM_SCHEMA)))
    });
}

fn bench_parser_large(c: &mut Criterion) {
    c.bench_function("parser_large_schema", |b| {
        b.iter(|| parser::parse_lumos_file(black_box(LARGE_SCHEMA)))
    });
}

// ===== Transform Benchmarks =====

fn bench_transform_small(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(SMALL_SCHEMA).unwrap();
    c.bench_function("transform_small_schema", |b| {
        b.iter(|| transform::transform_to_ir(black_box(ast.clone())))
    });
}

fn bench_transform_medium(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(MEDIUM_SCHEMA).unwrap();
    c.bench_function("transform_medium_schema", |b| {
        b.iter(|| transform::transform_to_ir(black_box(ast.clone())))
    });
}

fn bench_transform_large(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(LARGE_SCHEMA).unwrap();
    c.bench_function("transform_large_schema", |b| {
        b.iter(|| transform::transform_to_ir(black_box(ast.clone())))
    });
}

// ===== Generator Benchmarks =====

fn bench_rust_generator_small(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(SMALL_SCHEMA).unwrap();
    let ir = transform::transform_to_ir(ast).unwrap();
    c.bench_function("rust_gen_small_schema", |b| {
        b.iter(|| generators::rust::generate_module(black_box(&ir)))
    });
}

fn bench_rust_generator_medium(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(MEDIUM_SCHEMA).unwrap();
    let ir = transform::transform_to_ir(ast).unwrap();
    c.bench_function("rust_gen_medium_schema", |b| {
        b.iter(|| generators::rust::generate_module(black_box(&ir)))
    });
}

fn bench_rust_generator_large(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(LARGE_SCHEMA).unwrap();
    let ir = transform::transform_to_ir(ast).unwrap();
    c.bench_function("rust_gen_large_schema", |b| {
        b.iter(|| generators::rust::generate_module(black_box(&ir)))
    });
}

fn bench_typescript_generator_small(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(SMALL_SCHEMA).unwrap();
    let ir = transform::transform_to_ir(ast).unwrap();
    c.bench_function("typescript_gen_small_schema", |b| {
        b.iter(|| generators::typescript::generate_module(black_box(&ir)))
    });
}

fn bench_typescript_generator_medium(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(MEDIUM_SCHEMA).unwrap();
    let ir = transform::transform_to_ir(ast).unwrap();
    c.bench_function("typescript_gen_medium_schema", |b| {
        b.iter(|| generators::typescript::generate_module(black_box(&ir)))
    });
}

fn bench_typescript_generator_large(c: &mut Criterion) {
    let ast = parser::parse_lumos_file(LARGE_SCHEMA).unwrap();
    let ir = transform::transform_to_ir(ast).unwrap();
    c.bench_function("typescript_gen_large_schema", |b| {
        b.iter(|| generators::typescript::generate_module(black_box(&ir)))
    });
}

// ===== End-to-End Pipeline Benchmarks =====

fn bench_e2e_pipeline_small(c: &mut Criterion) {
    c.bench_function("e2e_small_schema", |b| {
        b.iter(|| {
            let ast = parser::parse_lumos_file(black_box(SMALL_SCHEMA)).unwrap();
            let ir = transform::transform_to_ir(ast).unwrap();
            let _rust = generators::rust::generate_module(&ir);
            let _ts = generators::typescript::generate_module(&ir);
        })
    });
}

fn bench_e2e_pipeline_medium(c: &mut Criterion) {
    c.bench_function("e2e_medium_schema", |b| {
        b.iter(|| {
            let ast = parser::parse_lumos_file(black_box(MEDIUM_SCHEMA)).unwrap();
            let ir = transform::transform_to_ir(ast).unwrap();
            let _rust = generators::rust::generate_module(&ir);
            let _ts = generators::typescript::generate_module(&ir);
        })
    });
}

fn bench_e2e_pipeline_large(c: &mut Criterion) {
    c.bench_function("e2e_large_schema", |b| {
        b.iter(|| {
            let ast = parser::parse_lumos_file(black_box(LARGE_SCHEMA)).unwrap();
            let ir = transform::transform_to_ir(ast).unwrap();
            let _rust = generators::rust::generate_module(&ir);
            let _ts = generators::typescript::generate_module(&ir);
        })
    });
}

// ===== Benchmark Groups =====

criterion_group!(
    parser_benches,
    bench_parser_small,
    bench_parser_medium,
    bench_parser_large
);

criterion_group!(
    transform_benches,
    bench_transform_small,
    bench_transform_medium,
    bench_transform_large
);

criterion_group!(
    rust_gen_benches,
    bench_rust_generator_small,
    bench_rust_generator_medium,
    bench_rust_generator_large
);

criterion_group!(
    typescript_gen_benches,
    bench_typescript_generator_small,
    bench_typescript_generator_medium,
    bench_typescript_generator_large
);

criterion_group!(
    e2e_benches,
    bench_e2e_pipeline_small,
    bench_e2e_pipeline_medium,
    bench_e2e_pipeline_large
);

criterion_main!(
    parser_benches,
    transform_benches,
    rust_gen_benches,
    typescript_gen_benches,
    e2e_benches
);
