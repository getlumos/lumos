import { describe, it, expect, beforeAll, afterEach } from 'vitest';
import { generate } from '../../src/api/generate';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

describe('generate()', () => {
  let tempDir: string;

  beforeAll(() => {
    // Create temp directory for test outputs
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'lumos-test-'));
  });

  afterEach(() => {
    // Clean up temp files after each test
    const files = fs.readdirSync(tempDir);
    files.forEach(file => {
      fs.unlinkSync(path.join(tempDir, file));
    });
  });

  it('should generate Rust code from schema', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/simple.lumos');
    const outputRust = path.join(tempDir, 'generated.rs');

    const result = await generate(schemaPath, { outputRust });

    // Check that file was created
    expect(fs.existsSync(outputRust)).toBe(true);

    // Check that Rust code contains expected content
    expect(result.rust).toContain('pub struct TestAccount');
    expect(result.rust).toContain('pub wallet: Pubkey');
    expect(result.rust).toContain('pub amount: u64');
    expect(result.rust).toContain('#[account]');
  });

  it('should generate TypeScript code from schema', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/simple.lumos');
    const outputTypeScript = path.join(tempDir, 'generated.ts');

    const result = await generate(schemaPath, { outputTypeScript });

    // Check that file was created
    expect(fs.existsSync(outputTypeScript)).toBe(true);

    // Check that TypeScript code contains expected content
    expect(result.typescript).toContain('interface TestAccount');
    expect(result.typescript).toContain('wallet: PublicKey');
    expect(result.typescript).toContain('amount: number');
    expect(result.typescript).toContain('borsh.publicKey');
  });

  it('should generate both Rust and TypeScript', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/simple.lumos');
    const outputRust = path.join(tempDir, 'generated.rs');
    const outputTypeScript = path.join(tempDir, 'generated.ts');

    const result = await generate(schemaPath, {
      outputRust,
      outputTypeScript,
    });

    // Check that both files were created
    expect(fs.existsSync(outputRust)).toBe(true);
    expect(fs.existsSync(outputTypeScript)).toBe(true);

    // Check result object
    expect(result.rust).toBeTruthy();
    expect(result.typescript).toBeTruthy();
  });

  it('should return code without writing files if no output paths specified', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/simple.lumos');

    const result = await generate(schemaPath);

    // Check that code was generated
    expect(result.rust).toContain('pub struct TestAccount');
    expect(result.typescript).toContain('interface TestAccount');

    // Check that no files were created
    const files = fs.readdirSync(tempDir);
    expect(files.length).toBe(0);
  });

  it('should throw error for non-existent schema file', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/nonexistent.lumos');

    await expect(generate(schemaPath)).rejects.toThrow();
  });

  it('should throw error for invalid schema', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/invalid.lumos');

    await expect(generate(schemaPath)).rejects.toThrow();
  });
});
