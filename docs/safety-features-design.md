# LUMOS Safety Features Design

**Feature:** Generate command safety flags
**Status:** Design phase
**Priority:** High (user safety)
**Target:** Phase 3.2+ enhancement

---

## Overview

Add safety mechanisms to `lumos generate` command to prevent accidental file overwrites and provide better visibility into changes.

**Three new flags:**
1. `--dry-run` - Preview what would be generated
2. `--backup` - Create backup before overwriting
3. `--show-diff` - Interactive mode with diff preview

---

## Feature 1: Dry-Run Mode

### Purpose

Allow users to preview generation output without writing files.

### CLI Interface

```bash
lumos generate schema.lumos --dry-run
# Or short form:
lumos generate schema.lumos -n
```

### Behavior

**No files written, only preview:**

```
🔍 Dry-run mode (no files will be written)

Analyzing schema.lumos...

Would generate:
  programs/game/src/types.rs
    Size: 420 lines (12.3 KB)
    Changes from existing:
      + Added: struct Guild (15 lines)
      + Added: enum GuildEvent (8 lines)
      ~ Modified: struct Player (1 field added)

  app/src/types.ts
    Size: 380 lines (10.8 KB)
    Changes from existing:
      + Added: interface Guild
      + Added: type GuildEvent
      ~ Modified: interface Player

Summary:
  Files to generate: 2
  Total lines: 800
  New types: 2
  Modified types: 1

No files written (dry-run mode).
Run without --dry-run to apply changes.
```

### Implementation

```rust
// packages/cli/src/commands/generate.rs

#[derive(Parser)]
pub struct GenerateArgs {
    /// Input LUMOS schema file
    pub schema: PathBuf,

    /// Rust output file
    #[arg(long)]
    pub rust: Option<PathBuf>,

    /// TypeScript output file
    #[arg(long)]
    pub typescript: Option<PathBuf>,

    /// Dry-run mode (preview only, don't write files)
    #[arg(long, short = 'n')]
    pub dry_run: bool,
}

pub fn execute(args: GenerateArgs) -> Result<()> {
    // Parse schema
    let schema_content = fs::read_to_string(&args.schema)?;
    let lumos_ast = lumos_core::parse(&schema_content)?;

    // Generate code
    let rust_code = lumos_core::generate_rust(&lumos_ast)?;
    let ts_code = lumos_core::generate_typescript(&lumos_ast)?;

    if args.dry_run {
        // Dry-run mode: show preview only
        println!("🔍 Dry-run mode (no files will be written)\n");

        if let Some(rust_path) = &args.rust {
            preview_changes(rust_path, &rust_code)?;
        }

        if let Some(typescript_path) = &args.typescript {
            preview_changes(typescript_path, &ts_code)?;
        }

        println!("\nNo files written (dry-run mode).");
        println!("Run without --dry-run to apply changes.");
        return Ok(());
    }

    // Normal mode: write files
    if let Some(rust_path) = &args.rust {
        fs::write(rust_path, rust_code)?;
        println!("✓ Generated {}", rust_path.display());
    }

    if let Some(typescript_path) = &args.typescript {
        fs::write(typescript_path, ts_code)?;
        println!("✓ Generated {}", typescript_path.display());
    }

    Ok(())
}

fn preview_changes(path: &Path, new_content: &str) -> Result<()> {
    println!("Would generate: {}", path.display());

    let new_lines = new_content.lines().count();
    let new_size = new_content.len();

    println!("  Size: {} lines ({} KB)", new_lines, new_size / 1024);

    if path.exists() {
        let old_content = fs::read_to_string(path)?;
        let changes = diff_summary(&old_content, new_content);

        if !changes.is_empty() {
            println!("  Changes from existing:");
            for change in changes {
                println!("    {}", change);
            }
        } else {
            println!("  No changes (identical to existing)");
        }
    } else {
        println!("  New file (doesn't exist yet)");
    }

    println!();
    Ok(())
}

fn diff_summary(old: &str, new: &str) -> Vec<String> {
    // Simple diff summary (can be enhanced)
    let mut summary = Vec::new();

    let old_lines = old.lines().count();
    let new_lines = new.lines().count();

    if new_lines > old_lines {
        summary.push(format!("+ {} lines added", new_lines - old_lines));
    } else if new_lines < old_lines {
        summary.push(format!("- {} lines removed", old_lines - new_lines));
    }

    // Could add more detailed analysis:
    // - New structs/enums
    // - Modified types
    // - Removed types

    summary
}
```

### Tests

```rust
#[test]
fn test_dry_run_no_files_written() {
    let temp = tempdir().unwrap();
    let schema = temp.path().join("schema.lumos");
    let output = temp.path().join("types.rs");

    fs::write(&schema, "struct Player { score: u64 }").unwrap();

    let args = GenerateArgs {
        schema,
        rust: Some(output.clone()),
        typescript: None,
        dry_run: true,
    };

    execute(args).unwrap();

    // File should NOT exist
    assert!(!output.exists());
}
```

---

## Feature 2: Backup Mode

### Purpose

Create automatic backup of existing files before overwriting.

### CLI Interface

```bash
lumos generate schema.lumos --backup
# Or short form:
lumos generate schema.lumos -b
```

### Behavior

**Creates backup before overwriting:**

```
Creating backups...
  programs/game/src/types.rs → programs/game/src/types.rs.backup
  app/src/types.ts → app/src/types.ts.backup

Generating...
  ✓ programs/game/src/types.rs
  ✓ app/src/types.ts

Backups saved. Restore with:
  mv programs/game/src/types.rs.backup programs/game/src/types.rs
  mv app/src/types.ts.backup app/src/types.ts
```

### Backup Naming Strategy

**Option 1: Simple `.backup` extension**
```
types.rs → types.rs.backup
```

**Option 2: Timestamp-based**
```
types.rs → types.rs.backup.2025-01-18-103045
```

**Option 3: Rotating backups**
```
types.rs.backup   (most recent)
types.rs.backup.1 (previous)
types.rs.backup.2 (older)
```

**Recommendation:** Start with Option 1 (simple), add Option 2 later if needed.

### Implementation

```rust
#[derive(Parser)]
pub struct GenerateArgs {
    // ... existing fields

    /// Create backup before overwriting existing files
    #[arg(long, short = 'b')]
    pub backup: bool,
}

pub fn execute(args: GenerateArgs) -> Result<()> {
    // ... parsing and generation code

    if args.backup {
        println!("Creating backups...");
    }

    if let Some(rust_path) = &args.rust {
        if args.backup && rust_path.exists() {
            create_backup(rust_path)?;
        }
        fs::write(rust_path, rust_code)?;
        println!("✓ Generated {}", rust_path.display());
    }

    if let Some(typescript_path) = &args.typescript {
        if args.backup && typescript_path.exists() {
            create_backup(typescript_path)?;
        }
        fs::write(typescript_path, ts_code)?;
        println!("✓ Generated {}", typescript_path.display());
    }

    if args.backup {
        println!("\nBackups saved. Restore with:");
        if let Some(rust_path) = &args.rust {
            let backup = backup_path(rust_path);
            println!("  mv {} {}", backup.display(), rust_path.display());
        }
        if let Some(typescript_path) = &args.typescript {
            let backup = backup_path(typescript_path);
            println!("  mv {} {}", backup.display(), typescript_path.display());
        }
    }

    Ok(())
}

fn create_backup(path: &Path) -> Result<()> {
    let backup = backup_path(path);
    fs::copy(path, &backup)?;
    println!("  {} → {}", path.display(), backup.display());
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    // Simple backup naming
    path.with_extension(
        format!("{}.backup", path.extension().unwrap_or_default().to_str().unwrap())
    )
}

// Alternative: Timestamp-based backup
fn backup_path_timestamped(path: &Path) -> PathBuf {
    use chrono::Local;
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");

    path.with_extension(
        format!(
            "{}.backup.{}",
            path.extension().unwrap_or_default().to_str().unwrap(),
            timestamp
        )
    )
}
```

### Tests

```rust
#[test]
fn test_backup_creates_backup_file() {
    let temp = tempdir().unwrap();
    let schema = temp.path().join("schema.lumos");
    let output = temp.path().join("types.rs");

    fs::write(&schema, "struct Player { score: u64 }").unwrap();
    fs::write(&output, "// old content").unwrap();

    let args = GenerateArgs {
        schema,
        rust: Some(output.clone()),
        typescript: None,
        dry_run: false,
        backup: true,
    };

    execute(args).unwrap();

    // Backup should exist
    let backup = output.with_extension("rs.backup");
    assert!(backup.exists());

    let backup_content = fs::read_to_string(&backup).unwrap();
    assert_eq!(backup_content, "// old content");

    // Original should be updated
    let new_content = fs::read_to_string(&output).unwrap();
    assert!(new_content.contains("struct Player"));
}
```

### .gitignore Recommendation

```gitignore
# Ignore backup files
*.backup
*.backup.*
```

---

## Feature 3: Show Diff (Interactive Mode)

### Purpose

Show diff between old and new content, ask for confirmation before writing.

### CLI Interface

```bash
lumos generate schema.lumos --show-diff
# Or short form:
lumos generate schema.lumos -d
```

### Behavior

**Interactive diff preview:**

```
Analyzing changes for: programs/game/src/types.rs

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DIFF: programs/game/src/types.rs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  #[account]
  pub struct Player {
      pub wallet: Pubkey,
      pub score: u64,
+     pub level: u16,
  }

+ #[account]
+ pub struct Guild {
+     pub leader: Pubkey,
+     pub name: String,
+ }

Summary:
  Lines added: 8
  Lines removed: 0
  Lines changed: 1

Apply changes to programs/game/src/types.rs? [y/N] █
```

**User interaction:**
- `y` or `Y` - Apply changes
- `n` or `N` - Skip this file
- `a` or `A` - Apply all (don't ask again)
- `q` or `Q` - Quit (cancel all)

### Implementation

```rust
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::io::{self, Write};

#[derive(Parser)]
pub struct GenerateArgs {
    // ... existing fields

    /// Show diff and ask for confirmation before writing
    #[arg(long, short = 'd')]
    pub show_diff: bool,
}

pub fn execute(args: GenerateArgs) -> Result<()> {
    // ... parsing and generation code

    let mut apply_all = false;

    if let Some(rust_path) = &args.rust {
        let should_write = if args.show_diff && rust_path.exists() {
            let old_content = fs::read_to_string(rust_path)?;
            show_diff_and_confirm(rust_path, &old_content, &rust_code, &mut apply_all)?
        } else {
            true
        };

        if should_write {
            fs::write(rust_path, rust_code)?;
            println!("✓ Generated {}", rust_path.display());
        } else {
            println!("⊘ Skipped {}", rust_path.display());
        }
    }

    // Similar for TypeScript...

    Ok(())
}

fn show_diff_and_confirm(
    path: &Path,
    old_content: &str,
    new_content: &str,
    apply_all: &mut bool,
) -> Result<bool> {
    if *apply_all {
        return Ok(true);
    }

    println!("\nAnalyzing changes for: {}", path.display());
    println!("{}", "━".repeat(60));
    println!("DIFF: {}", path.display());
    println!("{}", "━".repeat(60));
    println!();

    // Generate and display diff
    let diff = TextDiff::from_lines(old_content, new_content);

    let mut added = 0;
    let mut removed = 0;
    let mut changed = 0;

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => {
                removed += 1;
                format!("{}", "-".red())
            }
            ChangeTag::Insert => {
                added += 1;
                format!("{}", "+".green())
            }
            ChangeTag::Equal => {
                " ".to_string()
            }
        };

        print!("{} {}", sign, change);
    }

    println!();
    println!("Summary:");
    println!("  Lines added: {}", added.to_string().green());
    println!("  Lines removed: {}", removed.to_string().red());
    if added > 0 && removed > 0 {
        changed = added.min(removed);
        println!("  Lines changed: {}", changed.to_string().yellow());
    }
    println!();

    // Ask for confirmation
    loop {
        print!("Apply changes to {}? [y/N/a/q] ", path.display());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            "a" | "all" => {
                *apply_all = true;
                return Ok(true);
            }
            "q" | "quit" => {
                println!("Cancelled by user.");
                std::process::exit(0);
            }
            _ => {
                println!("Invalid input. Use: y (yes), n (no), a (all), q (quit)");
            }
        }
    }
}
```

### Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
similar = "2.4"        # For text diff
colored = "2.1"        # Already used
```

### Tests

```rust
#[test]
fn test_show_diff_no_changes() {
    // If old and new content are identical,
    // should show "no changes" and skip confirmation
}

#[test]
fn test_show_diff_calculates_stats() {
    let old = "line1\nline2\nline3";
    let new = "line1\nline2 modified\nline4";

    // Should detect:
    // - 1 line changed
    // - 1 line added
    // - 1 line removed
}
```

---

## Combining Flags

### Compatible Combinations

**Dry-run + Show-diff:**
```bash
lumos generate schema.lumos --dry-run --show-diff
```
**Behavior:** Show diff in dry-run mode (no confirmation needed)

**Backup + Show-diff:**
```bash
lumos generate schema.lumos --backup --show-diff
```
**Behavior:** Show diff, if user confirms, create backup then write

**All three:**
```bash
lumos generate schema.lumos --dry-run --backup --show-diff
```
**Behavior:** Dry-run takes precedence (no files written, no backups created)

### Flag Priority

```
--dry-run > --show-diff > --backup
```

If `--dry-run` is set:
- Don't write files
- Don't create backups
- Show preview only

---

## Configuration File Support

### .lumos.toml

Allow setting defaults:

```toml
[generate]
# Always create backups
backup = true

# Show diff by default
show_diff = true

# Dry-run (probably not a good default)
dry_run = false
```

**CLI flags override config file:**

```bash
# Even if config has backup=true, this disables it
lumos generate schema.lumos --no-backup
```

---

## User Experience

### Help Output

```bash
$ lumos generate --help

Generate code from LUMOS schema

USAGE:
    lumos generate [OPTIONS] <SCHEMA>

ARGS:
    <SCHEMA>    LUMOS schema file (.lumos)

OPTIONS:
    --rust <FILE>           Rust output file
    --typescript <FILE>     TypeScript output file

    -n, --dry-run           Preview changes without writing files
    -b, --backup            Create backup before overwriting
    -d, --show-diff         Show diff and ask for confirmation

    -h, --help              Print help information
    -V, --version           Print version information

EXAMPLES:
    # Preview changes
    lumos generate schema.lumos --dry-run

    # Create backup before generating
    lumos generate schema.lumos --backup

    # Interactive mode with diff
    lumos generate schema.lumos --show-diff

    # Combine flags
    lumos generate schema.lumos --backup --show-diff
```

---

## Implementation Roadmap

### Phase 1: Dry-Run (Easiest)
**Effort:** 2-3 hours
- Add `--dry-run` flag
- Implement preview function
- Basic diff summary
- Tests

### Phase 2: Backup (Easy)
**Effort:** 1-2 hours
- Add `--backup` flag
- Implement backup creation
- Tests

### Phase 3: Show-Diff (Moderate)
**Effort:** 4-6 hours
- Add `--show-diff` flag
- Integrate `similar` crate for diff
- Implement interactive confirmation
- Colorized output
- Tests

### Phase 4: Polish (Easy)
**Effort:** 2-3 hours
- Config file support
- Flag combination logic
- Documentation
- Integration tests

**Total Estimate:** 9-14 hours (1-2 days)

---

## Success Metrics

**After implementation:**

1. **Safety:**
   - Users can preview changes before applying
   - Automatic backups prevent data loss
   - Interactive mode reduces mistakes

2. **Usability:**
   - Clear, helpful output
   - Intuitive flag names
   - Good defaults

3. **Adoption:**
   - `--dry-run` used by beginners
   - `--backup` used in production
   - `--show-diff` used for code review

---

## Alternative Designs Considered

### Alternative 1: Always Show Diff

**Instead of flag, always show diff:**
```bash
lumos generate schema.lumos  # Always interactive
```

**Pros:**
- Safer by default
- Users see changes

**Cons:**
- Annoying in CI/CD
- Slower workflow

**Decision:** Use flag (opt-in)

### Alternative 2: Separate Backup Command

```bash
lumos backup programs/game/src/types.rs
lumos generate schema.lumos
```

**Pros:**
- More control
- Can backup manually

**Cons:**
- Extra step
- Easy to forget

**Decision:** Integrated flag (--backup)

### Alternative 3: Git Integration

```bash
lumos generate schema.lumos --git-commit
# Automatically commits generated files
```

**Pros:**
- Full Git integration
- Automatic versioning

**Cons:**
- Too opinionated
- Git should be manual

**Decision:** Don't integrate Git (users use Git manually)

---

## Future Enhancements

### Phase 5+ (Future)

**Rollback command:**
```bash
lumos rollback --restore-backup
# Restores most recent backup
```

**Interactive selection:**
```bash
lumos generate schema.lumos --interactive
# Choose which files to generate
```

**Partial generation:**
```bash
lumos generate schema.lumos --only Player,Guild
# Generate only specific types
```

**Watch mode with diff:**
```bash
lumos watch schema.lumos --show-diff
# Auto-regenerate on save, show diff each time
```

---

**Status:** Design complete, ready for implementation
**Priority:** High (improves user safety and confidence)
**Target:** Phase 3.2 enhancement (after VSCode extension)
