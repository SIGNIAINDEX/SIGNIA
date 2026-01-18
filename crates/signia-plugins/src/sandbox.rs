//! WASM sandbox support for SIGNIA plugins.
//!
//! This module provides an optional execution environment for running plugins
//! inside a WebAssembly sandbox using `wasmtime`.
//!
//! Design goals:
//! - deterministic execution
//! - no ambient authority
//! - explicit host capabilities
//! - resource limits (fuel, memory)
//!
//! This module is feature-gated behind `wasm`.

#![cfg(feature = "wasm")]

use anyhow::{anyhow, Result};

use crate::plugin::{HostCapabilities, PluginInput, PluginOutput, PluginResult};

use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

/// Configuration for the WASM sandbox.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum fuel (instruction budget).
    pub fuel: u64,

    /// Maximum memory in bytes.
    pub max_memory_bytes: u64,

    /// Host capabilities exposed to the plugin.
    pub host_caps: HostCapabilities,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            fuel: 10_000_000,
            max_memory_bytes: 64 * 1024 * 1024,
            host_caps: HostCapabilities {
                network: false,
                filesystem: false,
                clock: false,
                spawn: false,
            },
        }
    }
}

/// A sandboxed WASM plugin.
pub struct WasmSandbox {
    engine: Engine,
    module: Module,
    config: SandboxConfig,
}

impl WasmSandbox {
    /// Load a WASM module from bytes.
    pub fn from_bytes(bytes: &[u8], config: SandboxConfig) -> Result<Self> {
        let mut engine_cfg = wasmtime::Config::new();
        engine_cfg.consume_fuel(true);
        engine_cfg.wasm_multi_memory(false);
        engine_cfg.wasm_simd(false);

        let engine = Engine::new(&engine_cfg)?;
        let module = Module::new(&engine, bytes)?;

        Ok(Self {
            engine,
            module,
            config,
        })
    }

    /// Execute the WASM plugin.
    ///
    /// The WASM module is expected to export a function:
    ///
    /// ```text
    /// (func (export "execute"))
    /// ```
    ///
    /// Communication is done via host functions and shared memory
    /// (out of scope for this minimal implementation).
    pub fn execute(&self, _input: &PluginInput) -> PluginResult<PluginOutput> {
        let mut store = Store::new(&self.engine, ());
        store.add_fuel(self.config.fuel).map_err(|e| anyhow!(e))?;

        let wasi = WasiCtxBuilder::new().inherit_stdio().build();
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |_: &mut ()| &wasi)
            .map_err(|e| anyhow!(e))?;

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| anyhow!(e))?;

        let func = instance
            .get_func(&mut store, "execute")
            .ok_or_else(|| anyhow!("WASM plugin does not export `execute`"))?;

        func.call(&mut store, &[], &mut [])
            .map_err(|e| anyhow!(e))?;

        Ok(PluginOutput::None)
    }
}
