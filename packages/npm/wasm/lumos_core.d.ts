/* tslint:disable */
/* eslint-disable */
/**
 * Generate Rust and TypeScript code from a LUMOS schema
 *
 * # Arguments
 *
 * * `source` - The .lumos schema source code
 *
 * # Returns
 *
 * A `GeneratedCode` struct containing both Rust and TypeScript outputs,
 * or a JavaScript Error if parsing/generation fails
 *
 * # Example (JavaScript)
 *
 * ```js
 * import { generateCode } from 'lumos-wasm';
 *
 * const schema = `
 * #[solana]
 * #[account]
 * struct PlayerAccount {
 *     wallet: PublicKey,
 *     level: u16,
 * }
 * `;
 *
 * try {
 *     const result = generateCode(schema);
 *     console.log('Rust:', result.rust);
 *     console.log('TypeScript:', result.typescript);
 * } catch (error) {
 *     console.error('Generation failed:', error.message);
 * }
 * ```
 */
export function generateCode(source: string): GeneratedCode;
/**
 * Validate a LUMOS schema without generating code
 *
 * Useful for providing real-time feedback in the editor without
 * the overhead of full code generation.
 *
 * # Arguments
 *
 * * `source` - The .lumos schema source code
 *
 * # Returns
 *
 * `Ok(())` if the schema is valid, or a JavaScript Error with the validation message
 */
export function validateSchema(source: string): void;
/**
 * Result of code generation containing both Rust and TypeScript outputs
 */
export class GeneratedCode {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Generated Rust code
   */
  rust: string;
  /**
   * Generated TypeScript code
   */
  typescript: string;
}
