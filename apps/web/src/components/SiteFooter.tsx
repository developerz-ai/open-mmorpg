import type { JSX } from 'solid-js';
import { config } from '../config.ts';
import { t } from '../lib/i18n.ts';

/** Static site footer. The year is fixed at module load for deterministic diffs. */
const YEAR = new Date().getFullYear();

export function SiteFooter(): JSX.Element {
  return (
    <footer class="site-footer text-fg-muted">
      <p>{t('footer.tagline')}</p>
      <p>{t('footer.rights', { year: YEAR, name: config.brand.realmName })}</p>
    </footer>
  );
}
