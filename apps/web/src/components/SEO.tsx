import { useIsRouting } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createEffect, createMemo, mergeProps, on } from 'solid-js';
import { t } from '../lib/i18n.ts';

export interface SEOProps {
  /** Page title (appended to site name). */
  title?: string;
  /** Meta description. */
  description?: string;
  /** Open Graph title. */
  ogTitle?: string;
  /** Open Graph description. */
  ogDescription?: string;
  /** Open Graph image URL. */
  ogImage?: string;
  /** Canonical URL. */
  canonical?: string;
  /** No index (for non-public pages). */
  noIndex?: boolean;
  /** Twitter card type. */
  twitterCard?: 'summary' | 'summary_large_image' | 'app' | 'player';
}

const SITE_NAME = 'Open MMORPG';
const DEFAULT_DESCRIPTION = t('seo.defaultDescription');

/**
 * SEO component — manages meta tags for search engines and social shares.
 * Call in each route with route-specific metadata.
 */
export function SEO(props: SEOProps): JSX.Element {
  const merged = mergeProps(
    {
      title: '',
      description: DEFAULT_DESCRIPTION,
      ogTitle: '',
      ogDescription: '',
      ogImage: '',
      canonical: '',
      noIndex: false,
      twitterCard: 'summary_large_image' as const,
    },
    props,
  );

  const isRouting = useIsRouting();

  const fullTitle = createMemo(() => {
    const base = merged.title ? `${merged.title} | ${SITE_NAME}` : SITE_NAME;
    return base;
  });

  // Update meta tags on mount and when props change (but not during routing)
  createEffect(
    on(
      () => ({
        title: merged.title,
        description: merged.description,
        ogTitle: merged.ogTitle,
        ogDescription: merged.ogDescription,
        ogImage: merged.ogImage,
        canonical: merged.canonical,
        noIndex: merged.noIndex,
        twitterCard: merged.twitterCard,
      }),
      (meta) => {
        if (isRouting()) return;

        document.title = fullTitle();

        setMeta('description', meta.description);
        setMeta('og:title', meta.ogTitle || fullTitle());
        setMeta('og:description', meta.ogDescription || meta.description);
        setMeta('og:type', 'website');
        setMeta('og:site_name', SITE_NAME);
        if (meta.ogImage) {
          setMeta('og:image', meta.ogImage);
        }
        if (meta.canonical) {
          setLink('canonical', meta.canonical);
        }
        if (meta.noIndex) {
          setMeta('robots', 'noindex, nofollow');
        }
        setMeta('twitter:card', meta.twitterCard);
        setMeta('twitter:title', meta.ogTitle || fullTitle());
        setMeta('twitter:description', meta.ogDescription || meta.description);
        if (meta.ogImage) {
          setMeta('twitter:image', meta.ogImage);
        }
      },
      { defer: true },
    ),
  );

  return null;
}

function setMeta(name: string, content: string): void {
  let meta = document.querySelector(`meta[name="${name}"]`) as HTMLMetaElement | null;
  if (!meta) {
    meta = document.createElement('meta');
    meta.name = name;
    document.head.appendChild(meta);
  }
  meta.content = content;
}

function setLink(rel: string, href: string): void {
  let link = document.querySelector(`link[rel="${rel}"]`) as HTMLLinkElement | null;
  if (!link) {
    link = document.createElement('link');
    link.rel = rel;
    document.head.appendChild(link);
  }
  link.href = href;
}
