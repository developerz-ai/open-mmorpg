import type { RouteSectionProps } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { t } from '../lib/i18n.ts';
import { SEO } from './SEO.tsx';
import { SiteFooter } from './SiteFooter.tsx';
import { SiteHeader } from './SiteHeader.tsx';

/**
 * The shared chrome: skip link, brand header, page outlet, footer. Defined once
 * (one nav, one footer) — page content flows through `props.children`. Thin:
 * layout only, no data logic. → docs/specs/web-client/app-shell
 */
export function AppLayout(props: RouteSectionProps): JSX.Element {
  return (
    <div class="app">
      {/* Mounted once in the shell rather than per route. `SEO` existed but no
        * route ever rendered it, so every page shipped without a canonical link
        * and without og/twitter tags — for a site an operator brands and shares,
        * that is the whole social preview missing. Here it applies to all of
        * them, and a route can still render its own for a page-specific title. */}
      <SEO />
      <a class="skip-link" href="#main">
        {t('nav.skipToContent')}
      </a>
      <SiteHeader />
      <main id="main" class="app-main">
        {props.children}
      </main>
      <SiteFooter />
    </div>
  );
}
