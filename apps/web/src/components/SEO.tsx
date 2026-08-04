import { useIsRouting, useLocation } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createEffect, createMemo, mergeProps, on } from 'solid-js';
import { config } from '../config.ts';
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
  const location = useLocation();

  const fullTitle = createMemo(() => {
    const base = merged.title ? `${merged.title} | ${SITE_NAME}` : SITE_NAME;
    return base;
  });

  // Update meta tags on mount and when props change (but not during routing)
  createEffect(
    on(
      () => ({
        // Pathname is tracked so the canonical (and title) follow navigation:
        // this component is mounted once in the shell, not per route.
        path: location.pathname,
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
        const image = meta.ogImage || defaultOgImage();
        if (image) {
          setMeta('og:image', image);
        }
        // Default to the URL actually being viewed. Requiring every route to
        // pass this is why not one of them had a canonical link — a default the
        // caller can still override is the version that ships.
        setLink('canonical', meta.canonical || canonicalForCurrentUrl());
        if (meta.noIndex) {
          setMeta('robots', 'noindex, nofollow');
        }
        setMeta('twitter:card', meta.twitterCard);
        setMeta('twitter:title', meta.ogTitle || fullTitle());
        setMeta('twitter:description', meta.ogDescription || meta.description);
        const twImage = meta.ogImage || defaultOgImage();
        if (twImage) {
          setMeta('twitter:image', twImage);
        }
      },
    ),
  );

  return null;
}

/** The operator's own logo, made absolute — every realm brands its own share card. */
function defaultOgImage(): string {
  const url = config.brand.logoUrl;
  if (!url) return '';
  return url.startsWith('http') ? url : new URL(url, window.location.origin).href;
}

/** Current URL without query or hash — the canonical form for these routes. */
function canonicalForCurrentUrl(): string {
  const { origin, pathname } = window.location;
  return `${origin}${pathname}`;
}

/**
 * Open Graph is addressed by `property`, not `name` — a crawler reading
 * `<meta name="og:title">` sees nothing. Everything else (description, robots,
 * twitter:*) is a `name`. Writing all of them as `name` meant the og tags were
 * emitted and ignored.
 */
function setMeta(key: string, content: string): void {
  const attr = key.startsWith('og:') ? 'property' : 'name';
  let meta = document.querySelector(`meta[${attr}="${key}"]`) as HTMLMetaElement | null;
  if (!meta) {
    meta = document.createElement('meta');
    meta.setAttribute(attr, key);
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
