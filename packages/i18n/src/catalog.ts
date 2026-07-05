/**
 * Catalogs are authored nested (grouped by feature) and looked up by flat
 * dot-keys. One catalog per locale; operators add a locale by dropping a file.
 */

/** A nested, human-authored message tree. */
export interface Catalog {
  [key: string]: string | Catalog;
}

/** A flattened `dot.key -> message` map, ready for O(1) lookup. */
export type FlatCatalog = Record<string, string>;

/** Flatten a nested catalog into dot-keyed entries. */
export function flatten(catalog: Catalog, prefix = ''): FlatCatalog {
  const out: FlatCatalog = {};
  for (const [key, value] of Object.entries(catalog)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'string') {
      out[path] = value;
    } else {
      Object.assign(out, flatten(value, path));
    }
  }
  return out;
}
