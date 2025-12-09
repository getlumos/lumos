// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! LUMOS CLI - Command-line interface for LUMOS schema code generator

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod utils;

use cli::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            schema,
            output,
            lang,
            target,
            watch,
            dry_run,
            backup,
            show_diff,
        } => {
            if watch {
                commands::watch::run(&schema, output.as_deref(), &lang, &target)
            } else {
                commands::generate::run(
                    &schema,
                    output.as_deref(),
                    &lang,
                    &target,
                    dry_run,
                    backup,
                    show_diff,
                )
            }
        }
        Commands::Validate { schema } => commands::validate::run(&schema),
        Commands::Init { name } => commands::init::run(name.as_deref()),
        Commands::Check { schema, output } => commands::check::run(&schema, output.as_deref()),
        Commands::CheckSize { schema, format } => commands::check::run_size(&schema, &format),
        Commands::Security { command } => match command {
            SecurityCommands::Analyze {
                schema,
                format,
                strict,
            } => commands::security::run_analyze(&schema, &format, strict),
        },
        Commands::Audit { command } => match command {
            AuditCommands::Generate {
                schema,
                output,
                format,
            } => commands::security::run_audit(&schema, output.as_deref(), &format),
        },
        Commands::Fuzz { command } => match command {
            FuzzCommands::Generate {
                schema,
                output,
                type_name,
            } => commands::fuzz::run_generate(&schema, output.as_deref(), type_name.as_deref()),
            FuzzCommands::Run {
                schema,
                type_name,
                jobs,
                max_time,
            } => commands::fuzz::run_fuzz(&schema, &type_name, jobs, max_time),
            FuzzCommands::Corpus {
                schema,
                output,
                type_name,
            } => commands::fuzz::run_corpus(&schema, output.as_deref(), type_name.as_deref()),
        },
        Commands::Diff {
            schema1,
            schema2,
            format,
        } => commands::diff::run(&schema1, &schema2, &format),
        Commands::Migrate {
            from_schema,
            to_schema,
            output,
            language,
            dry_run,
            force,
        } => commands::migrate::run(
            &from_schema,
            &to_schema,
            output.as_deref(),
            &language,
            dry_run,
            force,
        ),

        Commands::CheckCompat {
            from_schema,
            to_schema,
            format,
            verbose,
            strict,
        } => commands::check_compat::run(&from_schema, &to_schema, &format, verbose, strict),

        Commands::Anchor { command } => match command {
            AnchorCommands::Generate {
                schema,
                output,
                name,
                version,
                address,
                typescript,
                dry_run,
            } => commands::anchor::run_generate(
                &schema,
                output.as_deref(),
                name.as_deref(),
                &version,
                address.as_deref(),
                typescript,
                dry_run,
            ),
            AnchorCommands::Idl {
                schema,
                output,
                name,
                version,
                address,
                pretty,
            } => commands::anchor::run_idl(
                &schema,
                output.as_deref(),
                name.as_deref(),
                &version,
                address.as_deref(),
                pretty,
            ),
            AnchorCommands::Space {
                schema,
                format,
                account,
            } => commands::anchor::run_space(&schema, &format, account.as_deref()),
        },

        Commands::Metaplex { command } => match command {
            MetaplexCommands::Validate {
                schema,
                format,
                verbose,
            } => commands::metaplex::run_validate(&schema, &format, verbose),
            MetaplexCommands::Generate {
                schema,
                output,
                typescript,
                rust,
                dry_run,
            } => commands::metaplex::run_generate(
                &schema,
                output.as_deref(),
                typescript,
                rust,
                dry_run,
            ),
            MetaplexCommands::Types { format } => commands::metaplex::run_types(&format),
        },
    }
}
