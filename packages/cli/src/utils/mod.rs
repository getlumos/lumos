// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! CLI utility modules

pub mod backup;
pub mod type_helpers;
pub mod validation;

// Re-export commonly used functions for convenience
pub use backup::create_backup_if_exists;
pub use type_helpers::{
    format_type, infer_anchor_account_type, to_snake_case, type_info_to_rust_type,
};
pub use validation::validate_output_path;
