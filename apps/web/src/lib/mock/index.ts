/**
 * Mock backend entry point. Importing this registers every domain's mock routes
 * (side effect) and re-exports the router the API client calls. Each surface
 * contributes a `*.ts` sibling exporting its routes; they're wired here — one
 * place to see every mock.
 */

import { armoryRoutes } from './armory.ts';
import { auctionRoutes } from './auction.ts';
import { authRoutes } from './auth.ts';
import { registerRoutes } from './backend.ts';
import { feedRoutes } from './feed.ts';
import { realmRoutes } from './realm.ts';

registerRoutes(...realmRoutes, ...authRoutes, ...armoryRoutes, ...auctionRoutes, ...feedRoutes);

export { handleMock, MockNotFound } from './backend.ts';
