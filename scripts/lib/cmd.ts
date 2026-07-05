import { repoRoot } from './paths.ts';

/** Result of a captured subprocess run. */
export interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

/** Thrown when a command that must succeed exits non-zero. */
export class CmdError extends Error {
  constructor(
    readonly cmd: string[],
    readonly result: RunResult,
  ) {
    super(`command failed (${result.code}): ${cmd.join(' ')}`);
    this.name = 'CmdError';
  }
}

interface RunOpts {
  cwd?: string;
  env?: Record<string, string>;
}

/** Run a command, streaming its output to this process. Returns the exit code. */
export function runInherit(cmd: string[], opts: RunOpts = {}): number {
  const proc = Bun.spawnSync(cmd, {
    cwd: opts.cwd ?? repoRoot,
    env: { ...process.env, ...opts.env },
    stdout: 'inherit',
    stderr: 'inherit',
  });
  return proc.exitCode;
}

/** Run a command, capturing stdout/stderr. Never throws on non-zero. */
export function run(cmd: string[], opts: RunOpts = {}): RunResult {
  const proc = Bun.spawnSync(cmd, {
    cwd: opts.cwd ?? repoRoot,
    env: { ...process.env, ...opts.env },
    stdout: 'pipe',
    stderr: 'pipe',
  });
  return {
    code: proc.exitCode,
    stdout: proc.stdout.toString(),
    stderr: proc.stderr.toString(),
  };
}

/** Run a command that must succeed, or throw [`CmdError`]. */
export function runOrThrow(cmd: string[], opts: RunOpts = {}): RunResult {
  const result = run(cmd, opts);
  if (result.code !== 0) throw new CmdError(cmd, result);
  return result;
}

/** Whether an executable is on PATH. */
export function have(bin: string): boolean {
  return run(['bash', '-lc', `command -v ${bin}`]).code === 0;
}
