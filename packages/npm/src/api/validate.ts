/**
 * Schema validation API
 */

import { readFileSync } from 'fs';
import { loadWasm } from '../wasm/loader';
import type { ValidationResult } from '../types';

/**
 * Validate a LUMOS schema without generating code
 *
 * @param schemaPath - Path to the .lumos schema file
 * @returns Validation result
 *
 * @example
 * ```typescript
 * import { validate } from '@getlumos/cli';
 *
 * const result = await validate('schema.lumos');
 * if (!result.valid) {
 *   console.error('Validation failed:', result.error);
 * }
 * ```
 */
export async function validate(schemaPath: string): Promise<ValidationResult> {
  try {
    // Load WASM module
    const wasm = await loadWasm();

    // Read schema file
    const schemaContent = readFileSync(schemaPath, 'utf-8');

    // Validate using WASM
    wasm.validateSchema(schemaContent);

    return { valid: true };
  } catch (error) {
    return {
      valid: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}
