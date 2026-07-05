import { type Catalog, createTranslator } from '@omm/i18n';

/**
 * The English catalog, authored nested by feature. Every user-facing string in
 * the app lives here; components call `t('...')`, never a bare literal. A missing
 * key renders as ⟦key⟧ so gaps are impossible to miss.
 */
const en: Catalog = {
  realm: {
    title: '{name}',
    tagline: 'A realm running on the Open-MMORPG engine.',
    status: {
      heading: 'Realm status',
      online: 'Online',
      offline: 'Offline',
      population: '{count} / {capacity} adventurers online',
      loading: 'Checking realm…',
      error: 'Realm status unavailable',
    },
  },
  nav: { play: 'Play', armory: 'Armory', register: 'Create account' },
};

export const t = createTranslator(en);
