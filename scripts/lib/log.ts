/** Leveled, colored logging for DX scripts. One place, so output stays uniform. */

const COLORS = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  dim: '\x1b[2m',
} as const;

/** Wrap text in an ANSI color (pure — exported for tests). */
export function paint(color: keyof typeof COLORS, text: string): string {
  return `${COLORS[color]}${text}${COLORS.reset}`;
}

export const log = {
  step: (msg: string) => console.log(`${paint('blue', '▸')} ${msg}`),
  ok: (msg: string) => console.log(`${paint('green', '✔')} ${msg}`),
  warn: (msg: string) => console.warn(`${paint('yellow', '!')} ${msg}`),
  fail: (msg: string) => console.error(`${paint('red', '✘')} ${msg}`),
  dim: (msg: string) => console.log(paint('dim', msg)),
};
