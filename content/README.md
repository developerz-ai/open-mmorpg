# content/ — the data-driven game layer

**Core is compiled (`crates/`); content is data (here).** Factions, classes,
races, abilities, quests and zones are files, not code. Change one and the game
changes with **no `cargo build`**. If altering a faction needs a recompile, the
split is wrong.

This is how operators get the full thing to customize — add maps, retune rules,
ship a total-conversion datapack — while our compiled core stays strong. Every
bundle is described by [`manifest.json`](manifest.json), validated by the
`omm-content-schema` crate against the core content API version.

```
content/
├── manifest.json   # datapack root: id, version, api_version, factions…
├── factions/  classes/  races/  abilities/  quests/  zones/
└── scripts/        # WASM/Lua behavior, run sandboxed by omm-scripting
```

→ docs/architecture/05-ecs-and-scripting.md · docs/initial-idea/06-modding-and-extensibility.md
