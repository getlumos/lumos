// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Anchor-specific attribute parsing and instruction context generation
//!
//! This module handles:
//! - Parsing `#[anchor(...)]` attributes with constraints, has_one, init, mut, signer
//! - Parsing `#[instruction]` structs for Accounts context generation
//! - Generating `#[derive(Accounts)]` Rust code

use crate::ir::{StructDefinition, TypeInfo};
use std::collections::HashMap;

/// Anchor account attribute types
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorAccountAttr {
    /// `#[account(init)]` - Initialize a new account
    Init,

    /// `#[account(init_if_needed)]` - Initialize if doesn't exist
    InitIfNeeded,

    /// `#[account(mut)]` - Account is mutable
    Mut,

    /// `#[account(signer)]` - Account must be a signer (deprecated, use Signer type)
    Signer,

    /// `#[account(close = target)]` - Close account and send lamports to target
    Close(String),

    /// `#[account(constraint = "expression")]` - Custom constraint
    Constraint(String),

    /// `#[account(has_one = field)]` - Validate field matches
    HasOne(String),

    /// `#[account(seeds = [...])]` - PDA seeds
    Seeds(Vec<SeedComponent>),

    /// `#[account(bump)]` - PDA bump seed
    Bump,

    /// `#[account(bump = expr)]` - Explicit bump value
    BumpExpr(String),

    /// `#[account(payer = account)]` - Payer for init
    Payer(String),

    /// `#[account(space = size)]` - Account space
    Space(String),

    /// `#[account(owner = program)]` - Account owner constraint
    Owner(String),

    /// `#[account(address = pubkey)]` - Exact address constraint
    Address(String),

    /// `#[account(zero)]` - Account must be zeroed
    Zero,

    /// `#[account(rent_exempt = skip)]` - Skip rent exemption check
    RentExemptSkip,

    /// `#[account(realloc = size)]` - Realloc to new size
    Realloc(String),

    /// `#[account(realloc::payer = payer)]` - Payer for realloc
    ReallocPayer(String),

    /// `#[account(realloc::zero = bool)]` - Zero realloc memory
    ReallocZero(bool),
}

/// PDA seed component
#[derive(Debug, Clone, PartialEq)]
pub enum SeedComponent {
    /// Literal bytes: `b"prefix"`
    Literal(String),

    /// Account field reference: `authority.key()`
    AccountKey(String),

    /// Instruction argument: `args.id`
    Arg(String),

    /// Raw bytes expression
    Bytes(String),
}

/// Parsed anchor field attributes
#[derive(Debug, Clone, Default)]
pub struct AnchorFieldAttrs {
    /// Account attributes
    pub attrs: Vec<AnchorAccountAttr>,

    /// Account type (Account, Signer, Program, etc.)
    pub account_type: Option<AnchorAccountType>,
}

/// Anchor account wrapper types
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorAccountType {
    /// `Account<'info, T>` - Program account
    Account(String),

    /// `Signer<'info>` - Transaction signer
    Signer,

    /// `Program<'info, T>` - Program reference
    Program(String),

    /// `SystemAccount<'info>` - System account
    SystemAccount,

    /// `UncheckedAccount<'info>` - Unchecked (unsafe)
    UncheckedAccount,

    /// `AccountInfo<'info>` - Raw account info
    AccountInfo,

    /// `Box<Account<'info, T>>` - Boxed account (for large accounts)
    BoxedAccount(String),

    /// `Sysvar<'info, T>` - Sysvar
    Sysvar(String),
}

/// Instruction context definition
#[derive(Debug, Clone)]
pub struct InstructionContext {
    /// Instruction name
    pub name: String,

    /// Account fields
    pub accounts: Vec<InstructionAccount>,

    /// Instruction arguments (passed to handler)
    pub args: Vec<InstructionArg>,
}

/// Account in instruction context
#[derive(Debug, Clone)]
pub struct InstructionAccount {
    /// Account name
    pub name: String,

    /// Account type
    pub account_type: AnchorAccountType,

    /// Account attributes
    pub attrs: Vec<AnchorAccountAttr>,

    /// Whether optional (wrapped in Option)
    pub optional: bool,

    /// Documentation
    pub docs: Vec<String>,
}

/// Instruction argument
#[derive(Debug, Clone)]
pub struct InstructionArg {
    /// Argument name
    pub name: String,

    /// Argument type
    pub ty: TypeInfo,
}

/// Parse anchor attributes from a string
///
/// Handles formats like:
/// - `#[anchor(init, payer = authority, space = 8 + 32)]`
/// - `#[anchor(mut, has_one = owner, constraint = "amount > 0")]`
/// - `#[anchor(seeds = [b"vault", authority.key().as_ref()], bump)]`
pub fn parse_anchor_attrs(attr_str: &str) -> Vec<AnchorAccountAttr> {
    let mut attrs = Vec::new();

    // Remove #[anchor(...)] wrapper if present
    let content = if attr_str.starts_with("#[anchor(") && attr_str.ends_with(")]") {
        &attr_str[9..attr_str.len() - 2]
    } else if attr_str.starts_with("anchor(") && attr_str.ends_with(")") {
        &attr_str[7..attr_str.len() - 1]
    } else {
        attr_str
    };

    // Parse comma-separated attributes
    for part in split_attrs(content) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some(attr) = parse_single_attr(part) {
            attrs.push(attr);
        }
    }

    attrs
}

/// Split attributes respecting nested brackets and quotes
fn split_attrs(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for c in s.chars() {
        if escape_next {
            current.push(c);
            escape_next = false;
            continue;
        }

        match c {
            '\\' => {
                current.push(c);
                escape_next = true;
            }
            '"' => {
                current.push(c);
                in_string = !in_string;
            }
            '[' | '(' if !in_string => {
                current.push(c);
                depth += 1;
            }
            ']' | ')' if !in_string => {
                current.push(c);
                depth -= 1;
            }
            ',' if depth == 0 && !in_string => {
                parts.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

/// Parse a single attribute
fn parse_single_attr(s: &str) -> Option<AnchorAccountAttr> {
    let s = s.trim();

    // Simple flags
    match s {
        "init" => return Some(AnchorAccountAttr::Init),
        "init_if_needed" => return Some(AnchorAccountAttr::InitIfNeeded),
        "mut" => return Some(AnchorAccountAttr::Mut),
        "signer" => return Some(AnchorAccountAttr::Signer),
        "bump" => return Some(AnchorAccountAttr::Bump),
        "zero" => return Some(AnchorAccountAttr::Zero),
        "rent_exempt = skip" => return Some(AnchorAccountAttr::RentExemptSkip),
        _ => {}
    }

    // Key-value pairs
    if let Some((key, value)) = s.split_once('=') {
        let key = key.trim();
        let value = value.trim().trim_matches('"');

        match key {
            "constraint" => return Some(AnchorAccountAttr::Constraint(value.to_string())),
            "has_one" => return Some(AnchorAccountAttr::HasOne(value.to_string())),
            "close" => return Some(AnchorAccountAttr::Close(value.to_string())),
            "payer" => return Some(AnchorAccountAttr::Payer(value.to_string())),
            "space" => return Some(AnchorAccountAttr::Space(value.to_string())),
            "owner" => return Some(AnchorAccountAttr::Owner(value.to_string())),
            "address" => return Some(AnchorAccountAttr::Address(value.to_string())),
            "bump" => return Some(AnchorAccountAttr::BumpExpr(value.to_string())),
            "realloc" => return Some(AnchorAccountAttr::Realloc(value.to_string())),
            "realloc::payer" => return Some(AnchorAccountAttr::ReallocPayer(value.to_string())),
            "realloc::zero" => {
                return Some(AnchorAccountAttr::ReallocZero(value == "true"));
            }
            _ => {}
        }
    }

    // Seeds array
    if s.starts_with("seeds") {
        if let Some(seeds_str) = s.strip_prefix("seeds").and_then(|s| {
            let s = s.trim();
            s.strip_prefix('=').map(|stripped| stripped.trim())
        }) {
            let seeds = parse_seeds(seeds_str);
            return Some(AnchorAccountAttr::Seeds(seeds));
        }
    }

    None
}

/// Parse seeds array
fn parse_seeds(s: &str) -> Vec<SeedComponent> {
    let mut seeds = Vec::new();

    // Remove outer brackets
    let s = s.trim();
    let inner = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len() - 1]
    } else {
        s
    };

    for part in split_attrs(inner) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if part.starts_with("b\"") || part.starts_with("b'") {
            // Literal bytes
            let literal = part[2..part.len() - 1].to_string();
            seeds.push(SeedComponent::Literal(literal));
        } else if part.contains(".key()") {
            // Account key reference
            let account = part.split(".key()").next().unwrap_or("").to_string();
            seeds.push(SeedComponent::AccountKey(account));
        } else if part.starts_with("args.") || part.starts_with("ctx.args.") {
            // Instruction argument
            let arg = part
                .strip_prefix("args.")
                .or_else(|| part.strip_prefix("ctx.args."))
                .unwrap_or(part)
                .to_string();
            seeds.push(SeedComponent::Arg(arg));
        } else {
            // Raw bytes expression
            seeds.push(SeedComponent::Bytes(part.to_string()));
        }
    }

    seeds
}

/// Generate Rust code for instruction accounts context
pub fn generate_accounts_context(ctx: &InstructionContext) -> String {
    let mut output = String::new();

    // Derive macro
    output.push_str("#[derive(Accounts)]\n");

    // Add instruction args if any seeds reference args
    let has_arg_seeds = ctx.accounts.iter().any(|acc| {
        acc.attrs.iter().any(|attr| {
            if let AnchorAccountAttr::Seeds(seeds) = attr {
                seeds.iter().any(|s| matches!(s, SeedComponent::Arg(_)))
            } else {
                false
            }
        })
    });

    if has_arg_seeds || !ctx.args.is_empty() {
        output.push_str("#[instruction(");
        let args: Vec<String> = ctx
            .args
            .iter()
            .map(|arg| format!("{}: {}", arg.name, type_info_to_rust(&arg.ty)))
            .collect();
        output.push_str(&args.join(", "));
        output.push_str(")]\n");
    }

    // Struct definition
    output.push_str(&format!("pub struct {}<'info> {{\n", ctx.name));

    for account in &ctx.accounts {
        // Documentation
        for doc in &account.docs {
            output.push_str(&format!("    /// {}\n", doc));
        }

        // Account attribute
        let attr_str = generate_account_attr(&account.attrs);
        if !attr_str.is_empty() {
            output.push_str(&format!("    #[account({})]\n", attr_str));
        }

        // Field definition
        let type_str = generate_account_type(&account.account_type);
        if account.optional {
            output.push_str(&format!(
                "    pub {}: Option<{}>,\n",
                account.name, type_str
            ));
        } else {
            output.push_str(&format!("    pub {}: {},\n", account.name, type_str));
        }
    }

    output.push_str("}\n");

    output
}

/// Generate account attribute string
fn generate_account_attr(attrs: &[AnchorAccountAttr]) -> String {
    let mut parts = Vec::new();

    for attr in attrs {
        match attr {
            AnchorAccountAttr::Init => parts.push("init".to_string()),
            AnchorAccountAttr::InitIfNeeded => parts.push("init_if_needed".to_string()),
            AnchorAccountAttr::Mut => parts.push("mut".to_string()),
            AnchorAccountAttr::Signer => parts.push("signer".to_string()),
            AnchorAccountAttr::Close(target) => parts.push(format!("close = {}", target)),
            AnchorAccountAttr::Constraint(expr) => parts.push(format!("constraint = \"{}\"", expr)),
            AnchorAccountAttr::HasOne(field) => parts.push(format!("has_one = {}", field)),
            AnchorAccountAttr::Seeds(seeds) => {
                let seed_strs: Vec<String> = seeds
                    .iter()
                    .map(|s| match s {
                        SeedComponent::Literal(lit) => format!("b\"{}\"", lit),
                        SeedComponent::AccountKey(acc) => format!("{}.key().as_ref()", acc),
                        SeedComponent::Arg(arg) => format!("{}.as_ref()", arg),
                        SeedComponent::Bytes(expr) => expr.clone(),
                    })
                    .collect();
                parts.push(format!("seeds = [{}]", seed_strs.join(", ")));
            }
            AnchorAccountAttr::Bump => parts.push("bump".to_string()),
            AnchorAccountAttr::BumpExpr(expr) => parts.push(format!("bump = {}", expr)),
            AnchorAccountAttr::Payer(payer) => parts.push(format!("payer = {}", payer)),
            AnchorAccountAttr::Space(space) => parts.push(format!("space = {}", space)),
            AnchorAccountAttr::Owner(owner) => parts.push(format!("owner = {}", owner)),
            AnchorAccountAttr::Address(addr) => parts.push(format!("address = {}", addr)),
            AnchorAccountAttr::Zero => parts.push("zero".to_string()),
            AnchorAccountAttr::RentExemptSkip => parts.push("rent_exempt = skip".to_string()),
            AnchorAccountAttr::Realloc(size) => parts.push(format!("realloc = {}", size)),
            AnchorAccountAttr::ReallocPayer(payer) => {
                parts.push(format!("realloc::payer = {}", payer))
            }
            AnchorAccountAttr::ReallocZero(zero) => parts.push(format!("realloc::zero = {}", zero)),
        }
    }

    parts.join(", ")
}

/// Generate account type string
fn generate_account_type(ty: &AnchorAccountType) -> String {
    match ty {
        AnchorAccountType::Account(inner) => format!("Account<'info, {}>", inner),
        AnchorAccountType::Signer => "Signer<'info>".to_string(),
        AnchorAccountType::Program(prog) => format!("Program<'info, {}>", prog),
        AnchorAccountType::SystemAccount => "SystemAccount<'info>".to_string(),
        AnchorAccountType::UncheckedAccount => "UncheckedAccount<'info>".to_string(),
        AnchorAccountType::AccountInfo => "AccountInfo<'info>".to_string(),
        AnchorAccountType::BoxedAccount(inner) => format!("Box<Account<'info, {}>>", inner),
        AnchorAccountType::Sysvar(inner) => format!("Sysvar<'info, {}>", inner),
    }
}

/// Convert TypeInfo to Rust type string
fn type_info_to_rust(ty: &TypeInfo) -> String {
    match ty {
        TypeInfo::Primitive(name) => match name.as_str() {
            "PublicKey" | "Pubkey" => "Pubkey".to_string(),
            _ => name.clone(),
        },
        TypeInfo::Generic(name) => name.clone(),
        TypeInfo::UserDefined(name) => name.clone(),
        TypeInfo::Array(inner) => format!("Vec<{}>", type_info_to_rust(inner)),
        TypeInfo::FixedArray { element, size } => {
            format!("[{}; {}]", type_info_to_rust(element), size)
        }
        TypeInfo::Option(inner) => format!("Option<{}>", type_info_to_rust(inner)),
    }
}

/// Parse instruction context from LUMOS struct definition
///
/// Example LUMOS:
/// ```ignore
/// #[instruction]
/// struct Initialize {
///     #[anchor(init, payer = authority, space = 8 + MyAccount::LEN)]
///     my_account: MyAccount,
///
///     #[anchor(mut)]
///     authority: Signer,
///
///     system_program: Program<System>,
/// }
/// ```
pub fn parse_instruction_context(
    struct_def: &StructDefinition,
    account_attrs: &HashMap<String, Vec<String>>,
) -> Option<InstructionContext> {
    // Check if struct has #[instruction] attribute
    let is_instruction = struct_def
        .metadata
        .attributes
        .iter()
        .any(|a| a == "instruction");

    if !is_instruction {
        return None;
    }

    let mut accounts = Vec::new();

    for field in &struct_def.fields {
        let field_attrs = account_attrs.get(&field.name).cloned().unwrap_or_default();

        // Parse anchor attributes for this field
        let mut anchor_attrs = Vec::new();
        for attr in &field_attrs {
            anchor_attrs.extend(parse_anchor_attrs(attr));
        }

        // Determine account type from field type
        let account_type = infer_account_type(&field.type_info);

        accounts.push(InstructionAccount {
            name: field.name.clone(),
            account_type,
            attrs: anchor_attrs,
            optional: field.optional,
            docs: Vec::new(),
        });
    }

    Some(InstructionContext {
        name: struct_def.name.clone(),
        accounts,
        args: Vec::new(),
    })
}

/// Infer Anchor account type from LUMOS type
fn infer_account_type(ty: &TypeInfo) -> AnchorAccountType {
    match ty {
        TypeInfo::Primitive(name) if name == "Signer" => AnchorAccountType::Signer,
        TypeInfo::UserDefined(name) => {
            // Check for special types
            match name.as_str() {
                "Signer" => AnchorAccountType::Signer,
                "SystemAccount" => AnchorAccountType::SystemAccount,
                "UncheckedAccount" => AnchorAccountType::UncheckedAccount,
                "AccountInfo" => AnchorAccountType::AccountInfo,
                _ if name.starts_with("Program<") => {
                    let inner = name
                        .strip_prefix("Program<")
                        .and_then(|s| s.strip_suffix('>'))
                        .unwrap_or("System");
                    AnchorAccountType::Program(inner.to_string())
                }
                _ if name.starts_with("Sysvar<") => {
                    let inner = name
                        .strip_prefix("Sysvar<")
                        .and_then(|s| s.strip_suffix('>'))
                        .unwrap_or("Rent");
                    AnchorAccountType::Sysvar(inner.to_string())
                }
                _ => AnchorAccountType::Account(name.clone()),
            }
        }
        _ => AnchorAccountType::AccountInfo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_attrs() {
        let attrs = parse_anchor_attrs("init, mut, payer = authority");
        assert_eq!(attrs.len(), 3);
        assert!(attrs.contains(&AnchorAccountAttr::Init));
        assert!(attrs.contains(&AnchorAccountAttr::Mut));
        assert!(attrs.contains(&AnchorAccountAttr::Payer("authority".to_string())));
    }

    #[test]
    fn test_parse_constraint() {
        let attrs = parse_anchor_attrs("constraint = \"amount > 0\"");
        assert_eq!(attrs.len(), 1);
        assert!(matches!(
            &attrs[0],
            AnchorAccountAttr::Constraint(c) if c == "amount > 0"
        ));
    }

    #[test]
    fn test_parse_has_one() {
        let attrs = parse_anchor_attrs("mut, has_one = owner");
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains(&AnchorAccountAttr::Mut));
        assert!(attrs.contains(&AnchorAccountAttr::HasOne("owner".to_string())));
    }

    #[test]
    fn test_parse_seeds() {
        let attrs = parse_anchor_attrs("seeds = [b\"vault\", authority.key().as_ref()], bump");
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains(&AnchorAccountAttr::Bump));

        if let Some(AnchorAccountAttr::Seeds(seeds)) = attrs.first() {
            assert_eq!(seeds.len(), 2);
            assert!(matches!(&seeds[0], SeedComponent::Literal(s) if s == "vault"));
            assert!(matches!(&seeds[1], SeedComponent::AccountKey(s) if s == "authority"));
        } else {
            panic!("Expected Seeds attribute");
        }
    }

    #[test]
    fn test_generate_account_attr() {
        let attrs = vec![
            AnchorAccountAttr::Init,
            AnchorAccountAttr::Payer("authority".to_string()),
            AnchorAccountAttr::Space("8 + 32".to_string()),
        ];
        let result = generate_account_attr(&attrs);
        assert!(result.contains("init"));
        assert!(result.contains("payer = authority"));
        assert!(result.contains("space = 8 + 32"));
    }

    #[test]
    fn test_generate_accounts_context() {
        let ctx = InstructionContext {
            name: "Initialize".to_string(),
            accounts: vec![
                InstructionAccount {
                    name: "my_account".to_string(),
                    account_type: AnchorAccountType::Account("MyAccount".to_string()),
                    attrs: vec![
                        AnchorAccountAttr::Init,
                        AnchorAccountAttr::Payer("authority".to_string()),
                        AnchorAccountAttr::Space("8 + MyAccount::LEN".to_string()),
                    ],
                    optional: false,
                    docs: vec![],
                },
                InstructionAccount {
                    name: "authority".to_string(),
                    account_type: AnchorAccountType::Signer,
                    attrs: vec![AnchorAccountAttr::Mut],
                    optional: false,
                    docs: vec![],
                },
                InstructionAccount {
                    name: "system_program".to_string(),
                    account_type: AnchorAccountType::Program("System".to_string()),
                    attrs: vec![],
                    optional: false,
                    docs: vec![],
                },
            ],
            args: vec![],
        };

        let result = generate_accounts_context(&ctx);
        assert!(result.contains("#[derive(Accounts)]"));
        assert!(result.contains("pub struct Initialize<'info>"));
        assert!(result.contains("pub my_account: Account<'info, MyAccount>"));
        assert!(result.contains("pub authority: Signer<'info>"));
        assert!(result.contains("pub system_program: Program<'info, System>"));
        assert!(result.contains("#[account(init, payer = authority"));
    }

    #[test]
    fn test_generate_account_type() {
        assert_eq!(
            generate_account_type(&AnchorAccountType::Account("MyAccount".to_string())),
            "Account<'info, MyAccount>"
        );
        assert_eq!(
            generate_account_type(&AnchorAccountType::Signer),
            "Signer<'info>"
        );
        assert_eq!(
            generate_account_type(&AnchorAccountType::Program("System".to_string())),
            "Program<'info, System>"
        );
    }

    #[test]
    fn test_parse_full_anchor_attr() {
        let attrs = parse_anchor_attrs("#[anchor(init, payer = user, space = 8 + 100)]");
        assert_eq!(attrs.len(), 3);
        assert!(attrs.contains(&AnchorAccountAttr::Init));
        assert!(attrs.contains(&AnchorAccountAttr::Payer("user".to_string())));
        assert!(attrs.contains(&AnchorAccountAttr::Space("8 + 100".to_string())));
    }
}
