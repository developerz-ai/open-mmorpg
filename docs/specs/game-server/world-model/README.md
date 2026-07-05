# World Model

> One spatial index serves three jobs: **interest management** (who sees whom), **streaming** (what the client loads), and **shard boundaries** (who a shard owns). → [architecture/06](../../../architecture/06-world-and-assets.md)

## What it does
Partitions each zone by a **quadtree** over world XY. Entities live in leaf cells; the tree drives area-of-interest, tile streaming, pathfinding tiles, and where one shard's authority ends and the next begins ([sharding](../sharding/README.md)).

## Interest management (AoI)
- Each player subscribes to its cell + neighbours within a **view radius**; the replication system emits a per-player entity set each tick, fed to the [netcode](../netcode/README.md) delta encoder.
- Spatial hashing / quadtree lookup is **~30× faster** than all-pairs distance checks and makes bandwidth **O(nearby)**, not O(world) — the reason a shard can hold thousands online while each client sees a few hundred.
- **Relevance + priority:** near/combat entities update every tick; distant/idle ones throttle to fit the per-client byte budget (self > combat > near > far).

## Pathfinding
- **Recast/Detour navmesh**, baked **offline** from our world geometry — the same proven approach as WoW-emulator **MMaps** (which use Recast/Detour), but generated from our **open glTF/heightmap** source, not extracted from a proprietary client.
- Navmesh tiled to match the quadtree so a shard loads only its tiles.
- Line-of-sight / height / cover from collision geometry (the role WoW-emu **VMaps** play) derived from the same open source meshes — no client extraction step.

## Streaming
- Tiles = glTF meshes + heightmap chunks; client streams by camera position → **no loading screens** ([world-and-assets](../../../architecture/06-world-and-assets.md)).
- Server uses the identical partition for AoI and shard/zone boundaries — one mental model, client and server.

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| TrinityCore grid/cell (64×64 grids × 8×8 cells, lazy load/unload, ~90 yd visibility) | Coarse spatial grid with load-driven grid activation and radius-based visibility works and is cheap. But it's a fixed client-derived tiling, and AoI is a cell-visitor side effect, not a first-class system. | **Keep** coarse partition + lazy load/unload + radius AoI. **Replace** fixed client tiling with our quadtree over open-format geometry; make AoI a first-class ECS interest system. |
| TrinityCore MMaps (Recast/Detour) & VMaps | Offline-baked navmesh + collision from world geometry is the industry-standard, correct approach. | **Keep** Recast/Detour and the offline-bake model; **replace** client-extraction with a bake from our glTF/heightmap pipeline. |
| Modern WoW seamless zones (tunnel-masked streaming) | Continents are one continuous instance; sub-area culling + connecting tunnels hide streaming and server hops — loading screens only for true instances. | **Adopt** screen-free streamed borders as default; use portal/tunnel geometry to mask shard handoffs ([sharding](../sharding/README.md)). |
| Overwatch/Mirror spatial hashing | ~30× over distance checks; grid for uniform density, quadtree for clumpy. | **Adopt** quadtree (player density is clumpy — cities, world bosses). |

## Rules
- The quadtree is the **single** spatial authority — interest, streaming, and shard boundaries all read it. Don't fork a second index.
- AoI cell size ≈ view radius; cell boundaries are the natural shard-handoff seam ([sharding](../sharding/README.md)).
- Navmesh is baked, versioned, and shipped with content — never generated per-tick.

## Links
[netcode](../netcode/README.md) · [sharding](../sharding/README.md) · [tick-loop](../tick-loop/README.md) · Recast/Detour · [world-and-assets](../../../architecture/06-world-and-assets.md)
