# LUMOS Git Hooks

This directory contains git hooks for the LUMOS project to enforce quality and consistency.

## Available Hooks

### pre-commit

Validates all staged `.lumos` schema files before allowing a commit.

**What it does:**
- Runs `lumos validate` on each staged `.lumos` file
- Blocks commit if any validation errors are found
- Shows clear error messages for debugging

**Benefits:**
- Catches schema errors early (before CI)
- Prevents broken schemas from entering the codebase
- Saves time on failed CI builds
- Ensures all committed schemas are syntactically correct

## Installation

Run the installation script from the repository root:

```bash
bash .github/scripts/install-hooks.sh
```

This will copy the hooks to your local `.git/hooks/` directory.

## Manual Installation

You can also manually copy hooks:

```bash
cp .github/hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Requirements

- **lumos CLI**: The hooks require the LUMOS CLI to be installed
  ```bash
  cargo install lumos-cli
  ```

## Bypassing Hooks

In rare cases where you need to bypass validation (not recommended):

```bash
git commit --no-verify
```

## Uninstalling

To remove the hooks:

```bash
rm .git/hooks/pre-commit
```

## Troubleshooting

### "lumos CLI not found" error

Install the LUMOS CLI:
```bash
cargo install lumos-cli
```

### Permission denied

Make sure the hook is executable:
```bash
chmod +x .git/hooks/pre-commit
```

### Hook not running

Verify the hook is in the correct location:
```bash
ls -la .git/hooks/pre-commit
```

## Contributing

When adding new hooks:
1. Create the hook script in `.github/hooks/`
2. Make it executable: `chmod +x .github/hooks/hook-name`
3. Update the installation script: `.github/scripts/install-hooks.sh`
4. Document it in this README
5. Update `CONTRIBUTING.md` if relevant

---

**Need help?** Open an issue or reach out to @rz1989s
