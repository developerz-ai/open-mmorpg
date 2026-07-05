# Data Layer — TanStack Query + Zod boundaries

> How server state reaches the UI: **TanStack Solid Query** hooks own all server state; **Zod validates every API response at the boundary** and the inferred type flows into the UI; **typed errors**, never bare throws. The web holds no game truth — it renders projections from [gateway](../../../../apps/gateway)/[worldsvc](../../../../apps/worldsvc) and submits intents. → [architecture/09 §SOLID](../../../architecture/09-operator-web.md).

## Server state = TanStack Query, only
All server state lives in TanStack Solid Query — **never** a hand-rolled fetch cache, `createResource` ad-hoc, or a global store shadowing the server. One hook per query, consumed by [thin components](../app-shell/README.md).

| Concern | Convention |
|---|---|
| **queryKey** | Hierarchical array: `['realm', 'status']`, `['armory', 'character', name]` |
| **refetchInterval** | Live data polls (realm status `30_000`ms); static reads don't |
| **Cache** | Query owns dedupe, staleness, background refetch — not the component |
| **States** | Component renders exactly three: `isPending` / `isError` / `data` |

Worked example — realm status:

```ts
// queries/useRealmStatus.ts
export function useRealmStatus() {
  return useQuery(() => ({
    queryKey: ['realm', 'status'],
    queryFn: fetchRealmStatus,      // fetch + Zod live in lib/realm.ts
    refetchInterval: 30_000,
  }));
}
```

## Zod at every boundary
Never trust a network shape. Every response is **parsed by a Zod schema at the boundary**; the schema *is* the type (`z.infer`), so the validated shape — not raw JSON — flows into the UI. The pattern, from [`lib/realm.ts`](../../../../apps/web/src/lib/realm.ts):

```ts
export const RealmStatusSchema = z.object({
  name: z.string(),
  online: z.boolean(),
  population: z.number().int().nonnegative(),
  capacity: z.number().int().positive(),
});
export type RealmStatus = z.infer<typeof RealmStatusSchema>;

export async function fetchRealmStatus(): Promise<RealmStatus> {
  const res = await fetch(`${config.gatewayUrl}/realm/status`);
  if (!res.ok) throw new Error(`realm status ${res.status}`);
  return RealmStatusSchema.parse(await res.json());   // validate, then trust
}
```

The schema **mirrors the gateway's Rust type** — the crates-side struct is source of truth; the Zod schema is the client's fail-loud restatement.

## Typed errors, not bare throws
Distinguish failure kinds — **network** (offline, 5xx), **validation** (Zod parse fail = contract drift), **auth** (401/403 → [account-auth](../account-auth/README.md)). Custom typed errors let components render the right message via [`t()`](../i18n/README.md); a bare `throw` collapses them into one useless state. A validation failure is a *contract bug* — surface it loudly, don't swallow it.

## The API client
Two backends, both over HTTP — the web never touches shards or the DB:

| Endpoint | Backend | Access |
|---|---|---|
| Realm status, auth | [`apps/gateway`](../../../../apps/gateway) | Public read + authed actions |
| Armory, auction house, world feed | [`apps/worldsvc`](../../../../apps/worldsvc) | Read-only cross-shard projections |

Public data → cacheable, unauthed. Account actions → session-bearing, authed. **Reads are projections, writes are intents** — the server stays authoritative ([game-server](../../game-server/README.md)); nothing here is game truth.

## Distilled from
| Reimagines | Keep | Fix |
|---|---|---|
| jQuery/`fetch`-and-hope, untyped JSON | Talking to the server | **Zod at the boundary** — validate, then the inferred type flows |
| Hand-rolled fetch caches / global stores | Caching, dedupe, refetch | **TanStack Query** owns all server state |
| `try/catch` swallowing every error the same | Resilience | **Typed errors** — network vs. validation vs. auth |
| Client trusting/holding game state | Fast reads | **Projections in, intents out** — server-authoritative |

## Rules
- **All server state via TanStack Query.** No hand-rolled caches, no shadow stores.
- **Zod-parse every response at the boundary.** Schema mirrors the Rust type; `z.infer` is the UI type.
- **Typed errors, not bare throws.** A Zod fail is contract drift — surface it.
- **queryKey is hierarchical; live data sets `refetchInterval`.** Component renders pending/error/data only.
- **Projections in, intents out.** The web holds no game truth ([game-server](../../game-server/README.md)).

## Links
[app-shell](../app-shell/README.md) · [design-system](../design-system/README.md) · [i18n](../i18n/README.md) · [account-auth](../account-auth/README.md) · [armory](../armory/README.md) · [auction-house](../auction-house/README.md) · [world-feed](../world-feed/README.md) · [testing-dx](../testing-dx/README.md) · [index](../README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [`apps/gateway`](../../../../apps/gateway) · [`apps/worldsvc`](../../../../apps/worldsvc) · [CLAUDE.md](../../../../CLAUDE.md)
