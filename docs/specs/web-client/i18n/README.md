# i18n — `t()` from day one, loud missing keys

> Every user-facing string goes through `t()` from the first commit. **Missing keys render loud** (`⟦key⟧`), never a silent English fallback. Catalogs authored nested, looked up by flat dot-key; one catalog per locale, namespaced by feature. Operators add a language by **dropping a catalog file** — no code change. → [architecture/09 §i18n](../../../architecture/09-operator-web.md).

## `t()` from day one
No hardcoded string reaches the DOM. Components call `t('realm.status.heading')`; the catalog is the single source of copy. Retrofitting i18n later is the failure mode — so it's day-one, enforced by [lint/review](../testing-dx/README.md). See [`lib/i18n.ts`](../../../../apps/web/src/lib/i18n.ts) and [`RealmStatusCard`](../../../../apps/web/src/components/RealmStatusCard.tsx) — the card holds zero literals.

## Missing keys render LOUD
A missing or misspelled key renders `⟦realm.status.heading⟧` — visible in the UI and in [screenshot diffs](../testing-dx/README.md). **Never** fall back to the key-as-text or to English silently; a gap you can't see is a gap that ships. Loud-fail is the whole point.

## Authoring vs. lookup
Catalogs are **authored nested** by feature (readable, mergeable), **looked up flat** by dot-key:

```ts
const en: Catalog = {
  realm: { status: { heading: 'Realm status', online: 'Online' } },
};
// component:
t('realm.status.heading')          // → "Realm status"
t('realm.status.population', { count, capacity })   // interpolated
```

- **One catalog per locale** (`en`, `de`, …), **namespaced by feature** (`realm.*`, `nav.*`, `armory.*`).
- **Interpolation** via named placeholders (`{count}`), not string concatenation.

## `packages/i18n` (SRP modules)
| Module | Job |
|---|---|
| `translator` | `createTranslator(catalog)` → `t()`; resolves dot-key, renders `⟦key⟧` on miss |
| `catalog` | The `Catalog` type + nested→flat shape |
| `locales` | Locale registry / active-locale selection |
| `interpolate` | `{name}` placeholder substitution |

## `Intl`, not i18n strings
Dates, numbers, currency → **`Intl.DateTimeFormat` / `NumberFormat`**, locale-aware, **never** hand-written format strings in the catalog. Catalogs hold *words*; `Intl` holds *formatting*. A gold price or a login timestamp formats via `Intl`, so a new locale gets correct dates for free.

## Operators add a locale = drop a file
A new language is a **new catalog file** for that locale — no component edits, no recompile. Mirrors the data-driven core: content (here, copy) is data, the app is compiled. Missing keys in a partial translation render loud, so operators see exactly what's left to translate.

## Distilled from
| Reimagines | Keep | Fix |
|---|---|---|
| i18n bolted on late, English hardcoded | Localizable UI | **Day-one `t()`** — no retrofit |
| Silent fallback to English on missing key | A shipping site | **Loud `⟦key⟧`** — gaps are visible, not hidden |
| Dates/money in translation strings | Localized formatting | **`Intl`** — formatting isn't copy |
| Adding a language = code change | Multi-locale | **Drop a catalog file** — no recompile |

## Rules
- **Every user-facing string via `t()`.** No bare literals in components.
- **Missing keys render `⟦key⟧`** — loud, never silent, never key-as-text-pretending-to-be-copy.
- **Author nested, look up flat dot-key.** One catalog per locale, namespaced by feature.
- **Dates/numbers/money via `Intl`**, not catalog strings.
- **New locale = new catalog file** — no code change, no recompile.

## Links
[app-shell](../app-shell/README.md) · [design-system](../design-system/README.md) · [data-layer](../data-layer/README.md) · [operator-brand](../operator-brand/README.md) · [testing-dx](../testing-dx/README.md) · [index](../README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [CLAUDE.md](../../../../CLAUDE.md)
