/**
 * Code generation API
 */

import { readFileSync, writeFileSync } from 'fs';
import { loadWasm } from '../wasm/loader';
import type { GenerateOptions, GeneratedCode } from '../types';

/**
 * Generate Rust and TypeScript code from a LUMOS schema
 *
 * @param schemaPath - Path to the .lumos schema file
 * @param options - Generation options
 * @returns Generated code object
 *
 * @example
 * ```typescript
 * import { generate } from '@getlumos/cli';
 *
 * await generate('schema.lumos', {
 *   outputRust: 'src/generated.rs',
 *   outputTypeScript: 'src/generated.ts'
 * });
 * ```
 */
export async function generate(
  schemaPath: string,
  options: GenerateOptions = {}
): Promise<GeneratedCode> {
  // Load WASM module
  const wasm = await loadWasm();

  // Read schema file
  const schemaContent = readFileSync(schemaPath, 'utf-8');

  // Generate code using WASM
  const result = wasm.generateCode(schemaContent);

  // Write output files if specified
  if (options.outputRust && result.rust) {
    writeFileSync(options.outputRust, result.rust, 'utf-8');
  }

  if (options.outputTypeScript && result.typescript) {
    writeFileSync(options.outputTypeScript, result.typescript, 'utf-8');
  }

  return {
    rust: result.rust,
    typescript: result.typescript,
  };
}
