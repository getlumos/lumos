/**
 * @getlumos/cli - LUMOS schema language CLI for JavaScript/TypeScript
 *
 * Generate type-safe Rust and TypeScript code for Solana from `.lumos` schemas.
 *
 * @packageDocumentation
 */

// Export API functions
export { generate } from './api/generate';
export { validate } from './api/validate';

// Export types
export type { GenerateOptions, GeneratedCode, ValidationResult } from './types';
