import { Button } from '@omm/ui';
import { A } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { t } from '../lib/i18n.ts';

/** 404 page. Static copy via `t()`; a route back home. */
export function NotFound(): JSX.Element {
  return (
    <section class="stack notfound">
      <h1 class="text-fg-strong">{t('notFound.heading')}</h1>
      <p class="text-fg-muted">{t('notFound.body')}</p>
      <A href="/">
        <Button variant="ghost">{t('notFound.home')}</Button>
      </A>
    </section>
  );
}
