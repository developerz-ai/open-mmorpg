import type { RouteSectionProps } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { t } from '../lib/i18n.ts';
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
