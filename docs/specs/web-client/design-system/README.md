# Design System — dark-only tokens, `packages/ui`

> One **dark palette**, semantic tokens **by role**, defined once in `:root`; a shared [`packages/ui`](#packagesui) component library where each component does one job. No light theme, no toggle. Operators re-brand **within** the dark palette by overriding token values — never by editing components. → [architecture/09 §Dark theme](../../../architecture/09-operator-web.md).

## Dark theme ONLY
No light theme, no toggle, no `@media (prefers-color-scheme)`, no `data-theme`. One palette lives in `:root`. This is a **product decision** — it also makes screenshots deterministic, which makes the site [AI-testable](../testing-dx/README.md).

## Tokens by role, not value
Tokens name a **role** (`--color-bg`, `--color-accent`), never a value (`--zinc-900`). Stored as **space-separated RGB channels** so Tailwind's `/ alpha` syntax works: `rgb(var(--color-bg) / 0.8)`. Components reference `bg-bg text-fg` — **never** raw hex.

```css
:root {
  --color-bg:          18  18  20;
  --color-bg-soft:     28  28  32;
  --color-surface:     34  34  39;
  --color-fg:         228 226 222;
  --color-fg-strong:  248 247 245;
  --color-fg-muted:   150 146 140;
  --color-line:        54  54  60;
  --color-accent:      96 170 240;
  --color-accent-strong:130 190 248;
}
```

| Token | Role |
|---|---|
| `--color-bg` / `--color-bg-soft` | Page background / recessed panels |
| `--color-surface` | Cards, raised elements |
| `--color-fg` / `--color-fg-strong` / `--color-fg-muted` | Body / headings / secondary text |
| `--color-line` | Borders, dividers |
| `--color-accent` / `--color-accent-strong` | CTAs, links, focus — and the **operator's brand hook** |

## Operator re-brand — within the dark palette
An operator overrides token **values** (accent, surfaces) via [operator config](../operator-brand/README.md) → different realm, same dark, accessible system. They **cannot** ship a light theme or a per-page hex — the tokens are the only knob. Component code never changes to re-brand.

## `packages/ui`
Shared, presentational components — one component, one job, ≤300 LOC ([SRP](../../../../CLAUDE.md)). Buttons, cards, badges, tables, form fields. No data fetching (that's the [data-layer](../data-layer/README.md)), no strings (that's [`t()`](../i18n/README.md)) — a `ui` component takes props and renders tokens. Both the [SSR and SPA halves](../app-shell/README.md) import the same library.

## Accessibility
- **Contrast** — the dark tokens meet WCAG AA for body text; operator accent overrides must too (checked in [testing](../testing-dx/README.md)).
- **Focus** — visible `--color-accent` focus ring on every interactive element; never `outline: none` without a replacement.
- **Keyboard** — full tab-order, no mouse-only affordances; menus/dialogs trap and restore focus.

## Motion & transitions
Polish is motion done tastefully. Same rule as color: **tokenize once, reference by role** — never hand-pick a `0.23s` in a component. CSS3 transitions/animations only; **animate `transform` and `opacity`** (compositor-cheap), never `width`/`top`/`box-shadow` on the hot path.

```css
:root {
  --dur-instant: 90ms;   /* press feedback, tab switch          */
  --dur-fast:   150ms;   /* hover, focus ring, small state      */
  --dur-base:   220ms;   /* default enter/leave, dropdowns      */
  --dur-slow:   360ms;   /* page/route transition, modal        */
  --ease-out:      cubic-bezier(0.16, 1, 0.3, 1);   /* enter — decelerate */
  --ease-in-out:   cubic-bezier(0.65, 0, 0.35, 1);  /* move/reflow        */
}
```

| Interaction | Token | Property |
|---|---|---|
| Hover / focus ring | `--dur-fast` `--ease-out` | `opacity`, `--color-accent` |
| Button/press | `--dur-instant` | `transform: scale` |
| Dropdown / tooltip / toast enter-leave | `--dur-base` `--ease-out` | `opacity` + `transform: translateY` |
| Modal / route transition | `--dur-slow` `--ease-in-out` | `opacity` + `transform` |
| List reorder | `--dur-base` `--ease-in-out` | `transform` (FLIP) |

- **SolidJS-native** — enter/leave via Solid's `<Transition>` / `<TransitionGroup>` and CSS classes; no JS animation libraries for standard cases (keeps `packages/ui` lean, SRP).
- **`prefers-reduced-motion: reduce` is honored globally** — transitions collapse to near-zero, essential fades kept. Non-negotiable a11y.
- Motion is **feedback, not decoration** — it clarifies state change and spatial relationship; nothing loops or distracts. Durations stay ≤ `--dur-slow`.
- Deterministic: [screenshot tests](../testing-dx/README.md) disable animation (reduced-motion) so diffs stay stable — motion never makes the site flaky.

## Distilled from
| Reimagines | Keep | Fix |
|---|---|---|
| Ad-hoc per-page CSS, hex sprinkled everywhere | A cohesive look | One token set by role; **no raw hex** in components |
| Light+dark toggle churn (double the CSS, flaky diffs) | A polished dark UI | **Dark only** — half the surface, deterministic screenshots |
| Brand = fork-and-restyle components | Operator brandability | Brand = **token override**, components untouched |
| Jarring/instant UI or heavy JS animation libs | Snappy, alive feel | **Tokenized CSS3 motion**, transform/opacity only, reduced-motion honored |

## Rules
- **Dark only.** No toggle, no media query, no `data-theme`. Palette in `:root`.
- **Tokens by role**, space-separated RGB channels; components use `bg-bg text-fg`, **never hex**.
- **Operators re-brand via token values**, not component edits — within the dark palette.
- **`packages/ui` = presentational + SRP.** No fetch, no strings, ≤300 LOC per component.
- **A11y is not optional** — AA contrast, visible focus, full keyboard, **`prefers-reduced-motion` honored**.
- **Motion is tokenized** — `--dur-*`/`--ease-*` only, animate `transform`/`opacity`, CSS3 + Solid `<Transition>`, no JS anim libs for standard cases.

## Links
[app-shell](../app-shell/README.md) · [i18n](../i18n/README.md) · [data-layer](../data-layer/README.md) · [operator-brand](../operator-brand/README.md) · [testing-dx](../testing-dx/README.md) · [index](../README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [CLAUDE.md](../../../../CLAUDE.md)
