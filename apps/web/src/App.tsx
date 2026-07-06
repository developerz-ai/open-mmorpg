import { Route, Router } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { lazy, Show } from 'solid-js';
import { AppLayout } from './components/AppLayout.tsx';
import { isEnabled } from './lib/features.ts';
import { Home } from './routes/Home.tsx';
import { NotFound } from './routes/NotFound.tsx';

// Surface routes are lazy chunks — the armory bundle never ships to a visitor
// reading the landing page. → docs/specs/web-client/app-shell (code-splitting)
const Register = lazy(() => import('./routes/Register.tsx'));
const Login = lazy(() => import('./routes/Login.tsx'));
const Account = lazy(() => import('./routes/Account.tsx'));
const Downloads = lazy(() => import('./routes/Downloads.tsx'));
const Armory = lazy(() => import('./routes/Armory.tsx'));
const CharacterPage = lazy(() => import('./routes/CharacterPage.tsx'));
const GuildPage = lazy(() => import('./routes/GuildPage.tsx'));
const AuctionHouse = lazy(() => import('./routes/AuctionHouse.tsx'));
const WorldFeed = lazy(() => import('./routes/WorldFeed.tsx'));

/**
 * The SPA router. Shared chrome lives in `AppLayout`; each route is thin
 * (parse → hook → render). Optional surfaces are gated by operator feature
 * flags — a disabled surface has no route, mirroring the off endpoint.
 */
export function App(): JSX.Element {
  return (
    <Router root={AppLayout}>
      <Route path="/" component={Home} />
      <Route path="/login" component={Login} />
      <Show when={isEnabled('registrationOpen')}>
        <Route path="/register" component={Register} />
      </Show>
      <Route path="/downloads" component={Downloads} />
      <Route path="/account" component={Account} />
      <Show when={isEnabled('armoryPublic')}>
        <Route path="/armory" component={Armory} />
        <Route path="/armory/character/:name" component={CharacterPage} />
        <Route path="/armory/guild/:name" component={GuildPage} />
      </Show>
      <Show when={isEnabled('auctionHouse')}>
        <Route path="/auction" component={AuctionHouse} />
      </Show>
      <Show when={isEnabled('worldFeed')}>
        <Route path="/feed" component={WorldFeed} />
      </Show>
      <Route path="*" component={NotFound} />
    </Router>
  );
}
