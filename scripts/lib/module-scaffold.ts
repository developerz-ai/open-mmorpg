/**
 * Pure scaffolding logic for `bin/new-module` — name validation and the file
 * templates a new compiled module needs. Kept pure (no IO) so the tricky parts
 * (name rules, the dependency line, the manifest) are unit-tested; the script
 * around it only does filesystem writes.
 *
 * See docs/architecture/10-modules.md for what these files mean.
 */

/** A validated module name plus the crate identifiers derived from it. */
export interface ModuleNames {
  /** The `modules/<name>/` directory and manifest name, e.g. `fast-travel`. */
  readonly name: string;
  /** The Cargo package name, e.g. `omm-module-fast-travel`. */
  readonly crate: string;
  /** The Rust lib name / extern ident, e.g. `omm_module_fast_travel`. */
  readonly ident: string;
  /** The Rust type name for the module struct, e.g. `FastTravel`. */
  readonly type: string;
}

/** Thrown when a module name breaks the naming rule. */
export class InvalidModuleName extends Error {
  constructor(readonly moduleName: string) {
    super(
      `invalid module name "${moduleName}": use lowercase kebab-case ` +
        '(letters, digits, single hyphens), e.g. "fast-travel"',
    );
    this.name = 'InvalidModuleName';
  }
}

/** Lowercase kebab-case: starts with a letter, no leading/trailing/double `-`. */
const NAME_RE = /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/;

/**
 * Validate a raw module name and derive every identifier from it. The one rule
 * (kebab-case) makes the crate name, lib ident, and type name mechanical — the
 * same convention `crates/modules/build.rs` relies on to discover the crate.
 */
export function deriveNames(raw: string): ModuleNames {
  const name = raw.trim();
  if (!NAME_RE.test(name)) throw new InvalidModuleName(raw);
  const ident = `omm_module_${name.replace(/-/g, '_')}`;
  const type = name
    .split('-')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join('');
  return { name, crate: `omm-module-${name}`, ident, type };
}

/** The `Cargo.toml` for a new module crate. */
export function cargoToml(n: ModuleNames): string {
  return `[package]
name = "${n.crate}"
description = "TODO: one-line description of the ${n.name} module."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true

[lib]
name = "${n.ident}"
path = "src/lib.rs"

[dependencies]
omm-module-api.workspace = true
tracing.workspace = true

[lints]
workspace = true
`;
}

/** The build-time `module.toml` manifest. */
export function moduleToml(n: ModuleNames): string {
  return `# Build-time manifest for the ${n.name} compiled module.
# See docs/architecture/10-modules.md.
[module]
name = "${n.name}"
version = "0.0.0"
# The omm-module-api hook-API version this module targets.
core-api-version = 1
description = "TODO: describe the ${n.name} module."
# The server hooks this module implements (documentation; code is the truth).
hooks = ["on_tick"]
`;
}

/** The starter `src/lib.rs`: a working module that hooks the tick event. */
export function libRs(n: ModuleNames): string {
  return `//! \`${n.name}\` — a compiled gameplay module.
//!
//! Implement the [\`ServerHooks\`] you need; every method is a default no-op, so
//! delete the ones you do not use. This crate is auto-discovered and linked in
//! by \`omm-modules\` — no core file is edited to add it.

use std::any::Any;

use omm_module_api::{declare_module, Module, ModuleManifest, ServerHooks, TickCtx};

/// The ${n.name} module. Hold any state here behind interior mutability — hooks
/// take \`&self\` because the host shares one instance across the tick thread.
#[derive(Debug, Default)]
pub struct ${n.type};

impl ServerHooks for ${n.type} {
    fn on_tick(&self, _ctx: &TickCtx) {
        // TODO: react to the tick. Fires ~30×/s — keep this cheap and non-blocking.
    }
}

impl Module for ${n.type} {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest::new("${n.name}", env!("CARGO_PKG_VERSION"))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Emit the \`module()\` entry point the generated \`omm-modules\` registry links to.
declare_module!(${n.type}::default());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_reports_its_manifest() {
        let m = module();
        assert_eq!(m.manifest().name, "${n.name}");
        assert!(m.manifest().is_compatible());
    }
}
`;
}

/** The dependency line appended to `crates/modules/Cargo.toml`. */
export function dependencyLine(n: ModuleNames): string {
  return `${n.crate} = { path = "../../modules/${n.name}" }`;
}

/** The marker in `crates/modules/Cargo.toml` the dep line is inserted after. */
export const DEPS_MARKER =
  '# --- modules (managed by bin/new-module — append one line per module) ---';

/**
 * Insert a module's dependency line into the aggregator's `Cargo.toml` text,
 * just after {@link DEPS_MARKER}. Idempotent: if the line is already present the
 * text is returned unchanged. Throws if the marker is missing.
 */
export function insertDependency(cargoToml: string, n: ModuleNames): string {
  const line = dependencyLine(n);
  if (cargoToml.includes(line)) return cargoToml;
  const idx = cargoToml.indexOf(DEPS_MARKER);
  if (idx === -1) {
    throw new Error(`marker not found in crates/modules/Cargo.toml: ${DEPS_MARKER}`);
  }
  const insertAt = cargoToml.indexOf('\n', idx);
  if (insertAt === -1)
    throw new Error('malformed crates/modules/Cargo.toml: no newline after marker');
  return `${cargoToml.slice(0, insertAt)}\n${line}${cargoToml.slice(insertAt)}`;
}
