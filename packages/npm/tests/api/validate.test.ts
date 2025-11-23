import { describe, it, expect } from 'vitest';
import { validate } from '../../src/api/validate';
import * as path from 'path';

describe('validate()', () => {
  it('should validate valid schema', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/simple.lumos');

    const result = await validate(schemaPath);

    expect(result.valid).toBe(true);
    expect(result.error).toBeUndefined();
  });

  it('should reject invalid schema', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/invalid.lumos');

    const result = await validate(schemaPath);

    expect(result.valid).toBe(false);
    expect(result.error).toBeTruthy();
    expect(result.error).toContain('error');
  });

  it('should handle non-existent file', async () => {
    const schemaPath = path.join(__dirname, '../fixtures/nonexistent.lumos');

    const result = await validate(schemaPath);

    expect(result.valid).toBe(false);
    expect(result.error).toBeTruthy();
  });
});
