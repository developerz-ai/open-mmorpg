/** A class value: a string, or falsy (dropped). Keeps class lists readable. */
export type ClassValue = string | false | null | undefined;

/** Join truthy class values into one space-separated string. Pure. */
export function cx(...values: ClassValue[]): string {
  return values.filter((v): v is string => typeof v === 'string' && v.length > 0).join(' ');
}
