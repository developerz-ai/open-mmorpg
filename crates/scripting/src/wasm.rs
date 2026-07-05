//! The WASM sandbox: compile untrusted module bytes and run an exported entry
//! function under a **fuel budget** with only the [capability API] linked.
//!
//! Guarantees enforced here:
//! - **Anti-hang:** every run is metered ([`Fuel`]); a script that loops forever
//!   is trapped with [`ScriptError::OutOfFuel`] instead of stalling the caller.
//! - **Closed surface:** the [`wasmtime::Linker`] only exposes the four
//!   [`HostCapabilities`] functions. A module importing anything else (a
//!   filesystem/network shim, WASI, …) fails to instantiate — proven by
//!   construction, since nothing else is linked.
//! - **Determinism:** fuel replaces wall-clock scheduling, NaN bits are
//!   canonicalized, and the nondeterministic wasm proposals (threads,
//!   relaxed-SIMD) are disabled. No clock or RNG is reachable from a script.
//!
//! [capability API]: crate::capability

use crate::capability::{EffectId, EntityId, HostCapabilities};
use crate::error::{ScriptError, ScriptResult};
use omm_protocol::ItemId;
use wasmtime::{Caller, Config, Engine, Linker, Module, Store, Trap};

/// The import module name every capability function lives under.
const CAP_MODULE: &str = "env";

/// A metered budget of abstract execution units for one script run. When it
/// reaches zero the script traps with [`ScriptError::OutOfFuel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fuel(u64);

impl Fuel {
    /// A generous default budget: enough for real ability/quest logic, small
    /// enough that a runaway loop is killed in well under a tick.
    pub const DEFAULT: Self = Self(1_000_000);

    /// A budget of `units` execution units.
    #[must_use]
    pub const fn new(units: u64) -> Self {
        Self(units)
    }

    /// The raw unit count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl Default for Fuel {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The result of a completed script run.
#[derive(Debug)]
#[non_exhaustive]
pub struct Outcome<H> {
    /// The value the entry function returned.
    pub value: i64,
    /// Fuel left unspent — useful for budgeting and telemetry.
    pub fuel_remaining: u64,
    /// The host back out of the sandbox, with every capability call it recorded.
    pub host: H,
}

/// A validated, compiled script ready to run. Compile once, run many times.
#[derive(Clone)]
pub struct CompiledScript {
    module: Module,
}

/// The sandbox host: owns a configured [`Engine`] and turns bytes into runs.
#[derive(Clone)]
pub struct WasmHost {
    engine: Engine,
}

impl WasmHost {
    /// Build a sandbox host with fuel metering and determinism enabled.
    ///
    /// # Errors
    /// [`ScriptError::Host`] if the runtime rejects the configuration.
    pub fn new() -> ScriptResult<Self> {
        let mut config = Config::new();
        config.consume_fuel(true);
        // Determinism: canonicalize NaN payloads so float results are identical
        // across hosts. The nondeterministic wasm proposals (threads/shared
        // memory, relaxed-SIMD) are compiled out entirely — their cargo features
        // are disabled, so a module using them fails to even validate.
        config.cranelift_nan_canonicalization(true);
        config.wasm_relaxed_simd(false);
        let engine = Engine::new(&config).map_err(|e| ScriptError::Host(e.to_string()))?;
        Ok(Self { engine })
    }

    /// Compile untrusted module bytes into a runnable script.
    ///
    /// # Errors
    /// [`ScriptError::CompileFailed`] if the bytes are not valid WebAssembly or
    /// use a disabled feature.
    pub fn compile(&self, wasm: &[u8]) -> ScriptResult<CompiledScript> {
        let module = Module::new(&self.engine, wasm)
            .map_err(|e| ScriptError::CompileFailed(e.to_string()))?;
        Ok(CompiledScript { module })
    }

    /// Run `entry(arg)` in the sandbox with `host` as its only capability
    /// surface and `fuel` as its hard execution budget.
    ///
    /// # Errors
    /// - [`ScriptError::Instantiation`] if the module imports anything outside
    ///   the capability API.
    /// - [`ScriptError::MissingExport`] / [`ScriptError::BadSignature`] if
    ///   `entry` is absent or not an `(i64) -> i64` function.
    /// - [`ScriptError::OutOfFuel`] if the run exhausts its budget.
    /// - [`ScriptError::Trap`] for any other runtime trap.
    pub fn run<H>(
        &self,
        script: &CompiledScript,
        host: H,
        fuel: Fuel,
        entry: &str,
        arg: i64,
    ) -> ScriptResult<Outcome<H>>
    where
        H: HostCapabilities + 'static,
    {
        let mut store = Store::new(&self.engine, host);
        store
            .set_fuel(fuel.get())
            .map_err(|e| ScriptError::Host(e.to_string()))?;

        let mut linker = Linker::new(&self.engine);
        link_capabilities(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &script.module)
            .map_err(|e| ScriptError::Instantiation(e.to_string()))?;
        let func = instance
            .get_func(&mut store, entry)
            .ok_or_else(|| ScriptError::MissingExport(entry.to_owned()))?;
        let typed = func
            .typed::<i64, i64>(&store)
            .map_err(|e| ScriptError::BadSignature(format!("{entry}: {e}")))?;

        let value = typed.call(&mut store, arg).map_err(map_call_error)?;
        let fuel_remaining = store.get_fuel().unwrap_or(0);
        Ok(Outcome {
            value,
            fuel_remaining,
            host: store.into_data(),
        })
    }
}

/// Link the capability API — and nothing else — into `linker`.
fn link_capabilities<H>(linker: &mut Linker<H>) -> ScriptResult<()>
where
    H: HostCapabilities + 'static,
{
    let host_err = |e: wasmtime::Error| ScriptError::Host(e.to_string());
    linker
        .func_wrap(
            CAP_MODULE,
            "spawn_entity",
            |mut caller: Caller<'_, H>, kind: i64, x: i64, y: i64| -> i64 {
                caller.data_mut().spawn_entity(kind, x, y).raw() as i64
            },
        )
        .map_err(host_err)?;
    linker
        .func_wrap(
            CAP_MODULE,
            "query_nearby",
            |caller: Caller<'_, H>, x: i64, y: i64, radius: i64| -> i64 {
                i64::from(caller.data().query_nearby(x, y, radius))
            },
        )
        .map_err(host_err)?;
    linker
        .func_wrap(
            CAP_MODULE,
            "apply_effect",
            |mut caller: Caller<'_, H>, target: i64, effect: i64, magnitude: i64| {
                caller.data_mut().apply_effect(
                    EntityId::new(target as u64),
                    EffectId::new(effect as u32),
                    magnitude,
                );
            },
        )
        .map_err(host_err)?;
    linker
        .func_wrap(
            CAP_MODULE,
            "grant_item",
            |mut caller: Caller<'_, H>, target: i64, item: i64, qty: i64| -> i64 {
                let qty = u32::try_from(qty).unwrap_or(0);
                match caller.data_mut().grant_item(
                    EntityId::new(target as u64),
                    ItemId::new(item as u64),
                    qty,
                ) {
                    Ok(()) => 0,
                    Err(_) => 1,
                }
            },
        )
        .map_err(host_err)?;
    Ok(())
}

/// Classify a trap from a script call into a typed error.
fn map_call_error(err: wasmtime::Error) -> ScriptError {
    if let Some(trap) = err.downcast_ref::<Trap>() {
        if *trap == Trap::OutOfFuel {
            return ScriptError::OutOfFuel;
        }
        return ScriptError::Trap(trap.to_string());
    }
    ScriptError::Trap(err.to_string())
}
