import { A } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import { config } from '../config.ts';
import { isEnabled } from '../lib/features.ts';
import { t } from '../lib/i18n.ts';

/**
 * Brand header + primary nav. The realm name/logo resolve once here from
 * operator config and flow down; no page hard-codes a realm name. Nav entries
 * are gated by feature flags — a disabled surface isn't offered.
 */
export function SiteHeader(): JSX.Element {
  return (
    <header class="site-header">
      <A class="brand" href="/" aria-label={config.brand.realmName}>
        <Show
          when={config.brand.logoUrl}
          fallback={<span class="brand__name">{config.brand.realmName}</span>}
        >
          {(url) => <img class="brand__logo" src={url()} alt={config.brand.realmName} />}
        </Show>
      </A>
      <nav class="site-nav" aria-label={t('nav.home')}>
        <A href="/" end>
          {t('nav.home')}
        </A>
        <Show when={isEnabled('armoryPublic')}>
          <A href="/armory">{t('nav.armory')}</A>
        </Show>
        <Show when={isEnabled('auctionHouse')}>
          <A href="/auction">{t('nav.auction')}</A>
        </Show>
        <Show when={isEnabled('worldFeed')}>
          <A href="/feed">{t('nav.feed')}</A>
        </Show>
      </nav>
    </header>
  );
}
