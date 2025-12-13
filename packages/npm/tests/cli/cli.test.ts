import { describe, it, expect, beforeAll, afterEach } from 'vitest';
import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

describe('CLI Commands', () => {
  let tempDir: string;
  let cliPath: string;

  beforeAll(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'lumos-cli-test-'));
    cliPath = path.join(__dirname, '../../dist/cli.js');
  });

  afterEach(() => {
    // Clean up temp files
    const files = fs.readdirSync(tempDir);
    files.forEach(file => {
      fs.unlinkSync(path.join(tempDir, file));
    });
  });

  it('should show version', () => {
    const output = execSync(`node ${cliPath} --version`).toString().trim();
    expect(output).toMatch(/^\d+\.\d+\.\d+$/);
  });

  it('should show help', () => {
    const output = execSync(`node ${cliPath} --help`).toString();
    expect(output).toContain('LUMOS schema language CLI');
    expect(output).toContain('generate');
    expect(output).toContain('validate');
  });

  it('should generate code from schema', () => {
    const schemaPath = path.join(__dirname, '../fixtures/simple.lumos');
    const outputRust = path.join(tempDir, 'test.rs');

    const output = execSync(
      `node ${cliPath} generate ${schemaPath} --output-rust ${outputRust}`
    ).toString();

    expect(output).toContain('✅ Code generated successfully');
    expect(fs.existsSync(outputRust)).toBe(true);

    const content = fs.readFileSync(outputRust, 'utf-8');
    expect(content).toContain('pub struct TestAccount');
  });

  it('should validate valid schema', () => {
    const schemaPath = path.join(__dirname, '../fixtures/simple.lumos');

    const output = execSync(`node ${cliPath} validate ${schemaPath}`).toString();

    expect(output).toContain('✅ Schema is valid');
  });

  it('should fail on invalid schema', () => {
    const schemaPath = path.join(__dirname, '../fixtures/invalid.lumos');

    try {
      execSync(`node ${cliPath} validate ${schemaPath}`, { stdio: 'pipe' });
      // Should not reach here
      expect(true).toBe(false);
    } catch (error: unknown) {
      const err = error as { stderr?: Buffer; stdout?: Buffer };
      const output = err.stderr?.toString() || err.stdout?.toString() || '';
      expect(output).toContain('❌');
    }
  });

  it('should fail on non-existent file', () => {
    const schemaPath = path.join(__dirname, '../fixtures/nonexistent.lumos');

    try {
      execSync(`node ${cliPath} generate ${schemaPath}`, { stdio: 'pipe' });
      expect(true).toBe(false);
    } catch (error: unknown) {
      const err = error as { status: number };
      // Should throw error
      expect(err.status).not.toBe(0);
    }
  });
});
