/**
 * Type definitions for LUMOS npm package
 */

/**
 * Result of code generation
 */
export interface GeneratedCode {
  /** Generated Rust code */
  rust: string;
  /** Generated TypeScript code */
  typescript: string;
}

/**
 * Options for code generation
 */
export interface GenerateOptions {
  /** Output path for Rust code */
  outputRust?: string;
  /** Output path for TypeScript code */
  outputTypeScript?: string;
}

/**
 * Validation result
 */
export interface ValidationResult {
  /** Whether the schema is valid */
  valid: boolean;
  /** Error message if invalid */
  error?: string;
}
