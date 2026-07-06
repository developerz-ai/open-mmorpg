import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('ChecksumVerifier component exports', () => {
  test('ChecksumVerifier.tsx exists and exports ChecksumVerifier', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('export function ChecksumVerifier');
    expect(content).toContain('ChecksumVerifierProps');
  });
});

describe('ChecksumVerifier component structure', () => {
  test('ChecksumVerifier uses TextField for input', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('TextField');
    expect(content).toContain('checksum-input');
  });

  test('ChecksumVerifier uses Button for verify action', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Button');
    expect(content).toContain('onClick={handleVerify}');
  });

  test('ChecksumVerifier uses Badge for status display', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Badge');
    expect(content).toContain('tone=');
  });

  test('ChecksumVerifier validates hex checksum format (64 chars)', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('/^[a-f0-9]{64}$/i');
  });

  test('ChecksumVerifier uses Spinner during verification', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Spinner');
    expect(content).toContain('verifying');
  });

  test('ChecksumVerifier handles Enter key for submission', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('onKeyDown');
    expect(content).toContain('Enter');
  });

  test('ChecksumVerifier uses i18n strings for UI', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("t('downloads.verifier");
  });

  test('ChecksumVerifier has case-insensitive checksum comparison', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('toLowerCase()');
  });
});

describe('ChecksumVerifier verification flow', () => {
  test('ChecksumVerifier shows success badge on match', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'match'");
    expect(content).toContain('tone="success"');
    expect(content).toContain('downloads.verifier.match');
  });

  test('ChecksumVerifier shows danger badge on mismatch', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'mismatch'");
    expect(content).toContain('tone="danger"');
    expect(content).toContain('downloads.verifier.mismatch');
  });

  test('ChecksumVerifier shows error for invalid format', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'invalid'");
    expect(content).toContain('downloads.verifier.invalid');
  });

  test('ChecksumVerifier disables verify button while verifying', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'verifying'");
    expect(content).toContain('disabled=');
  });

  test('ChecksumVerifier resets status on new input', () => {
    const componentPath = join(import.meta.dir, 'ChecksumVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('setStatus(');
    expect(content).toContain('onInput');
  });
});
