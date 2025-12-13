import { describe, it, expect } from 'vitest';
import { generate, validate } from '../../src';
import * as path from 'path';
import * as fs from 'fs';

describe('Integration Tests - Real Schemas', () => {
  it('should work with gaming example schema', async () => {
    // Use real gaming schema from examples
    const schemaPath = path.join(__dirname, '../../../examples/gaming/schema.lumos');

    // Skip if schema doesn't exist (optional dependency)
    if (!fs.existsSync(schemaPath)) {
      console.log('Skipping gaming schema test - file not found');
      return;
    }

    // Validate
    const validationResult = await validate(schemaPath);
    expect(validationResult.valid).toBe(true);

    // Generate
    const generateResult = await generate(schemaPath);
    expect(generateResult.rust).toContain('pub struct PlayerAccount');
    expect(generateResult.typescript).toContain('interface PlayerAccount');
  });

  it('should handle complex types', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/complex.lumos');

    // Create complex schema
    fs.writeFileSync(schemaPath, `
#[solana]
#[account]
struct ComplexAccount {
    id: PublicKey,
    items: [u64],
    optional_field: Option<String>,
}

#[solana]
enum Status {
    Active,
    Paused,
    Finished,
}
    `.trim());

    const result = await generate(schemaPath);

    // Check Rust
    expect(result.rust).toContain('pub struct ComplexAccount');
    expect(result.rust).toContain('pub items:');
    expect(result.rust).toContain('Vec<u64>');
    expect(result.rust).toContain('pub optional_field: Option<String>');
    expect(result.rust).toContain('pub enum Status');

    // Check TypeScript
    expect(result.typescript).toContain('interface ComplexAccount');
    expect(result.typescript).toContain('items: number[]');
    expect(result.typescript).toContain('optional_field?: string');
    expect(result.typescript).toContain('type Status');

    // Clean up
    fs.unlinkSync(schemaPath);
  });

  it('should handle multiple structs', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/multiple.lumos');

    fs.writeFileSync(schemaPath, `
#[solana]
#[account]
struct Account1 {
    value: u64,
}

#[solana]
#[account]
struct Account2 {
    name: String,
}
    `.trim());

    const result = await generate(schemaPath);

    expect(result.rust).toContain('pub struct Account1');
    expect(result.rust).toContain('pub struct Account2');
    expect(result.typescript).toContain('interface Account1');
    expect(result.typescript).toContain('interface Account2');

    fs.unlinkSync(schemaPath);
  });
});
