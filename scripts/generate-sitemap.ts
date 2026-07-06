#!/usr/bin/env bun
/**
 * Generate sitemap.xml for the operator web portal.
 * Run this after deployment to update search engines.
 */

const BASE_URL = process.env.PUBLIC_URL || 'https://open-mmorpg.org';
const OUTPUT_PATH = 'public/sitemap.xml';

// Static routes (all publicly accessible)
const routes = [
  { loc: '/', changefreq: 'daily', priority: '1.0' },
  { loc: '/downloads', changefreq: 'weekly', priority: '0.8' },
  { loc: '/armory', changefreq: 'daily', priority: '0.9' },
  { loc: '/auction', changefreq: 'hourly', priority: '0.9' },
  { loc: '/feed', changefreq: 'hourly', priority: '0.9' },
];

// Generate XML sitemap
function generateSitemap(): string {
  const urls = routes
    .map(
      (route) => `  <url>
    <loc>${BASE_URL}${route.loc}</loc>
    <changefreq>${route.changefreq}</changefreq>
    <priority>${route.priority}</priority>
  </url>`,
    )
    .join('\n');

  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls}
</urlset>
`;
}

// Write sitemap to public directory
const sitemap = generateSitemap();
await Bun.write(OUTPUT_PATH, sitemap);

console.log(`✓ Sitemap generated: ${OUTPUT_PATH}`);
console.log(`  Base URL: ${BASE_URL}`);
console.log(`  URLs: ${routes.length}`);
