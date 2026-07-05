//! End-to-end sandbox tests exercised through the public crate API only.
//!
//! Kept out of `src/wasm.rs` so the module stays under the 300-LOC limit, and
//! useful as executable documentation of the sandbox contract.

use omm_protocol::ItemId;
use omm_scripting::{EffectId, EntityId, Fuel, HostCall, RecordingHost, ScriptError, WasmHost};

// Test-only helper: the fixtures are literals, so a parse failure is a bug in
// the test itself and should panic loudly.
#[allow(clippy::expect_used)]
fn wasm(text: &str) -> Vec<u8> {
    wat::parse_str(text).expect("valid wat fixture")
}

#[test]
fn valid_module_runs_and_returns() {
    let host = WasmHost::new().expect("host");
    let script = host
        .compile(&wasm(
            "(module (func (export \"run\") (param i64) (result i64) \
             (i64.add (local.get 0) (i64.const 10))))",
        ))
        .expect("compile");
    let out = host
        .run(&script, RecordingHost::new(0), Fuel::DEFAULT, "run", 5)
        .expect("run");
    assert_eq!(out.value, 15);
    assert!(out.fuel_remaining < Fuel::DEFAULT.get());
}

#[test]
fn infinite_loop_is_fuel_killed() {
    let host = WasmHost::new().expect("host");
    let script = host
        .compile(&wasm(
            "(module (func (export \"run\") (param i64) (result i64) \
             (loop $l (br $l)) (i64.const 0)))",
        ))
        .expect("compile");
    // Completes fast: the budget is spent, then it traps — it does not hang.
    let err = host
        .run(&script, RecordingHost::new(0), Fuel::new(100_000), "run", 0)
        .expect_err("must be fuel-killed");
    assert!(matches!(err, ScriptError::OutOfFuel));
}

#[test]
fn capability_calls_are_dispatched_to_the_host() {
    let host = WasmHost::new().expect("host");
    let script = host
        .compile(&wasm(
            "(module \
             (import \"env\" \"spawn_entity\" (func $spawn (param i64 i64 i64) (result i64))) \
             (import \"env\" \"apply_effect\" (func $effect (param i64 i64 i64))) \
             (import \"env\" \"grant_item\" (func $grant (param i64 i64 i64) (result i64))) \
             (func (export \"run\") (param i64) (result i64) (local $e i64) \
             (local.set $e (call $spawn (i64.const 7) (i64.const 1) (i64.const 2))) \
             (call $effect (local.get $e) (i64.const 5) (i64.const 12)) \
             (drop (call $grant (local.get $e) (i64.const 100) (i64.const 3))) \
             (local.get 0)))",
        ))
        .expect("compile");
    let out = host
        .run(&script, RecordingHost::new(0), Fuel::DEFAULT, "run", 99)
        .expect("run");
    assert_eq!(out.value, 99);
    let entity = EntityId::new(1);
    assert_eq!(
        out.host.calls(),
        &[
            HostCall::Spawn {
                kind: 7,
                x: 1,
                y: 2,
                entity,
            },
            HostCall::Effect {
                target: entity,
                effect: EffectId::new(5),
                magnitude: 12,
            },
            HostCall::Grant {
                target: entity,
                item: ItemId::new(100),
                qty: 3,
            },
        ]
    );
}

#[test]
fn missing_export_is_typed() {
    let host = WasmHost::new().expect("host");
    let script = host
        .compile(&wasm(
            "(module (func (export \"other\") (param i64) (result i64) (local.get 0)))",
        ))
        .expect("compile");
    let err = host
        .run(&script, RecordingHost::new(0), Fuel::DEFAULT, "run", 0)
        .expect_err("no such export");
    assert!(matches!(err, ScriptError::MissingExport(_)));
}

#[test]
fn wrong_signature_is_typed() {
    let host = WasmHost::new().expect("host");
    let script = host
        .compile(&wasm(
            "(module (func (export \"run\") (result i64) (i64.const 1)))",
        ))
        .expect("compile");
    let err = host
        .run(&script, RecordingHost::new(0), Fuel::DEFAULT, "run", 0)
        .expect_err("wrong signature");
    assert!(matches!(err, ScriptError::BadSignature(_)));
}

#[test]
fn import_outside_the_capability_api_is_rejected() {
    // The sandbox surface is closed by construction: an unknown import cannot be
    // linked, so the module never instantiates.
    let host = WasmHost::new().expect("host");
    let script = host
        .compile(&wasm(
            "(module \
             (import \"env\" \"read_file\" (func (param i64) (result i64))) \
             (func (export \"run\") (param i64) (result i64) (local.get 0)))",
        ))
        .expect("compile");
    let err = host
        .run(&script, RecordingHost::new(0), Fuel::DEFAULT, "run", 0)
        .expect_err("unknown import rejected");
    assert!(matches!(err, ScriptError::Instantiation(_)));
}
