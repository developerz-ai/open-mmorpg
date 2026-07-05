#!/usr/bin/env bun
/**
 * Content validation script.
 *
 * Validates that:
 * 1. content/manifest.json exists and parses as valid JSON
 * 2. All JSON files in content/ parse successfully
 * 3. The Rust content-schema library validates the manifest
 */

import { readFileSync, existsSync } from "fs";
import { execSync } from "child_process";
import { exit } from "process";

interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

function validateManifestJson(): ValidationResult {
  const result: ValidationResult = { valid: true, errors: [], warnings: [] };

  // Check manifest exists
  if (!existsSync("content/manifest.json")) {
    result.valid = false;
    result.errors.push("content/manifest.json does not exist");
    return result;
  }

  // Try to parse manifest
  try {
    const manifestContent = readFileSync("content/manifest.json", "utf8");
    JSON.parse(manifestContent);
  } catch (e) {
    result.valid = false;
    result.errors.push(`Failed to parse content/manifest.json: ${e}`);
  }

  return result;
}

function validateAllJsonFiles(): ValidationResult {
  const result: ValidationResult = { valid: true, errors: [], warnings: [] };

  try {
    const files = execSync("find content -name '*.json'", { encoding: "utf8" })
      .split("\n")
      .filter((f) => f.length > 0);

    for (const file of files) {
      try {
        const content = readFileSync(file, "utf8");
        JSON.parse(content);
      } catch (e) {
        result.valid = false;
        result.errors.push(`Failed to parse ${file}: ${e}`);
      }
    }
  } catch (e) {
    result.valid = false;
    result.errors.push(`Failed to find JSON files: ${e}`);
  }

  return result;
}

function validateViaRustSchema(): ValidationResult {
  const result: ValidationResult = { valid: true, errors: [], warnings: [] };

  // Check if cargo is available
  try {
    execSync("which cargo", { stdio: "pipe" });
  } catch {
    result.warnings.push("cargo not found - skipping Rust tests");
    return result;
  }

  try {
    execSync("cargo test --package content-schema --lib", {
      stdio: "pipe",
    });
  } catch (e) {
    result.valid = false;
    result.errors.push(`Rust content-schema tests failed: ${e}`);
  }

  return result;
}

function main() {
  console.log("🔍 Validating content...");

  const manifestResult = validateManifestJson();
  const jsonResult = validateAllJsonFiles();
  const rustResult = validateViaRustSchema();

  const allErrors = [
    ...manifestResult.errors,
    ...jsonResult.errors,
    ...rustResult.errors,
  ];

  const allWarnings = [
    ...manifestResult.warnings,
    ...jsonResult.warnings,
    ...rustResult.warnings,
  ];

  if (allErrors.length > 0) {
    console.error("❌ Content validation FAILED:");
    for (const error of allErrors) {
      console.error(`  - ${error}`);
    }
    exit(1);
  }

  if (allWarnings.length > 0) {
    console.warn("⚠️  Content validation warnings:");
    for (const warning of allWarnings) {
      console.warn(`  - ${warning}`);
    }
  }

  console.log("✅ Content validation PASSED");
  exit(0);
}

main();
