#!/usr/bin/env node

/**
 * LUMOS CLI - Command-line interface for schema code generation
 */

import { Command } from 'commander';
import { generate, validate } from './index';
import * as path from 'path';
import * as fs from 'fs';

const program = new Command();

// Read version from package.json
const packageJson = JSON.parse(
  fs.readFileSync(path.join(__dirname, '../package.json'), 'utf-8')
);

program
  .name('lumos')
  .description('LUMOS schema language CLI - Generate type-safe Rust and TypeScript code for Solana')
  .version(packageJson.version);

// Generate command
program
  .command('generate <schema>')
  .description('Generate Rust and TypeScript code from schema')
  .option('--output-rust <path>', 'Output path for Rust code')
  .option('--output-typescript <path>', 'Output path for TypeScript code')
  .action(async (schemaPath: string, options: any) => {
    try {
      console.log(`🔧 Generating code from ${schemaPath}...`);

      await generate(schemaPath, {
        outputRust: options.outputRust,
        outputTypeScript: options.outputTypeScript,
      });

      console.log('✅ Code generated successfully');

      if (options.outputRust) {
        console.log(`   Rust: ${options.outputRust}`);
      }
      if (options.outputTypeScript) {
        console.log(`   TypeScript: ${options.outputTypeScript}`);
      }
    } catch (error) {
      console.error('❌ Generation failed:');
      console.error(error instanceof Error ? error.message : String(error));
      process.exit(1);
    }
  });

// Validate command
program
  .command('validate <schema>')
  .description('Validate schema syntax without generating code')
  .action(async (schemaPath: string) => {
    try {
      console.log(`🔍 Validating ${schemaPath}...`);

      const result = await validate(schemaPath);

      if (result.valid) {
        console.log('✅ Schema is valid');
      } else {
        console.error('❌ Validation failed:');
        console.error(result.error);
        process.exit(1);
      }
    } catch (error) {
      console.error('❌ Validation error:');
      console.error(error instanceof Error ? error.message : String(error));
      process.exit(1);
    }
  });

// Parse arguments
program.parse();
