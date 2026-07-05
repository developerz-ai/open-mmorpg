/**
 * Replace `{name}` placeholders in a template with values from `params`.
 * A placeholder with no matching param is left verbatim so it is visible in the
 * UI (fail loud, never silently blank).
 */
export function interpolate(template: string, params?: Record<string, string | number>): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (whole, key: string) => {
    const value = params[key];
    return value === undefined ? whole : String(value);
  });
}
