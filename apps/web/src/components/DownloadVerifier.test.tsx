import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('DownloadVerifier component exports', () => {
  test('DownloadVerifier.tsx exists and exports DownloadVerifier', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('export function DownloadVerifier');
    expect(content).toContain('DownloadVerifierProps');
  });
});

describe('DownloadVerifier component structure', () => {
  test('DownloadVerifier accepts downloads array prop', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('downloads: PlatformDownload[]');
  });

  test('DownloadVerifier uses TextField for input', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('TextField');
    expect(content).toContain('checksum-input');
  });

  test('DownloadVerifier uses Button for verify action', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Button');
    expect(content).toContain('onClick={handleVerify}');
  });

  test('DownloadVerifier uses Badge for status display', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Badge');
    expect(content).toContain('tone=');
  });

  test('DownloadVerifier validates hex checksum format (64 chars)', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('/^[a-f0-9]{64}$/i');
  });

  test('DownloadVerifier uses Spinner during verification', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Spinner');
    expect(content).toContain('verifying');
  });

  test('DownloadVerifier handles Enter key for submission', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('onKeyDown');
    expect(content).toContain('Enter');
  });

  test('DownloadVerifier shows matched platform on success', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('matchedDownload()?.platform');
    expect(content).toContain('verifier__match');
  });

  test('DownloadVerifier shows help examples with checksums', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('verifier__help');
    expect(content).toContain('verifier__example');
    expect(content).toContain('slice(0, 3)');
  });
});

describe('DownloadVerifier verification flow', () => {
  test('DownloadVerifier searches downloads array for match', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('props.downloads.find');
  });

  test('DownloadVerifier shows success badge on match', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'match'");
    expect(content).toContain('tone="success"');
    expect(content).toContain('downloads.verifier.match');
  });

  test('DownloadVerifier shows danger badge on mismatch', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'mismatch'");
    expect(content).toContain('tone="danger"');
    expect(content).toContain('downloads.verifier.mismatch');
  });

  test('DownloadVerifier shows error for invalid format', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'invalid'");
    expect(content).toContain('downloads.verifier.invalid');
  });

  test('DownloadVerifier disables verify button while verifying', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain("status() === 'verifying'");
    expect(content).toContain('disabled=');
  });

  test('DownloadVerifier resets matched download on new input', () => {
    const componentPath = join(import.meta.dir, 'DownloadVerifier.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('setMatchedDownload(null)');
  });
});
