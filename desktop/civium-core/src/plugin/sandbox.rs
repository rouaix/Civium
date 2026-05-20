//! WASM sandbox for third-party Civium plugins.
//!
//! Each plugin runs in an isolated wasmtime instance.  The host exposes a
//! minimal API (`civium_log`, `civium_cil_call`) through which the plugin may
//! perform permitted CIL actions.  Every call is permission-checked against the
//! plugin's declared manifest before execution.
//!
//! ## Plugin ABI
//!
//! The plugin WASM binary **must** export:
//!
//! | Export             | Signature                                      | Purpose                         |
//! |--------------------|------------------------------------------------|---------------------------------|
//! | `civium_alloc`     | `(i32) -> i32`                                 | Allocate n bytes, return ptr    |
//! | `civium_free`      | `(i32, i32) -> ()`                             | Free ptr + len                  |
//! | `civium_handle`    | `(i32,i32,i32,i32) -> i64`                     | Handle event, return ptr<<32|len|
//! | `memory`           | `Memory`                                       | Shared linear memory            |
//!
//! The host **provides** (importable by the plugin):
//!
//! | Import                   | Signature              | Purpose                    |
//! |--------------------------|------------------------|----------------------------|
//! | `civium::log`            | `(i32,i32) -> ()`      | Write a log line           |
//! | `civium::cil_call`       | `(i32,i32,i32,i32,i32,i32) -> i32` | CIL action, writes JSON result |

use std::sync::{Arc, Mutex};
use wasmtime::{
    Config, Engine, Linker, Memory, Module, OptLevel, Store,
};

use super::{PluginManifest, PluginPermission};

/// Callback type that executes a CIL action and returns a JSON result string.
/// Receives (action, param_json) and returns result_json.
pub type CilExecutor = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;

// ── Permission helpers ────────────────────────────────────────────────────────

/// Map a CIL action name to the required PluginPermission.
fn required_permission(action: &str) -> Option<PluginPermission> {
    match action {
        "query_members"     => Some(PluginPermission::ReadMembers),
        "query_messages"    => Some(PluginPermission::ReadMessages),
        "post_message"      => Some(PluginPermission::WriteMessages),
        "query_proposals"   => Some(PluginPermission::ReadGovernance),
        "create_proposal"   => Some(PluginPermission::WriteGovernance),
        "cast_vote"         => Some(PluginPermission::WriteGovernance),
        "query_directory"   => Some(PluginPermission::ReadDirectory),
        "publish_directory" => Some(PluginPermission::WriteDirectory),
        "query_connections" => Some(PluginPermission::ReadConnections),
        "query_agenda"      => Some(PluginPermission::ReadAgenda),
        "write_agenda"      => Some(PluginPermission::WriteAgenda),
        "query_documents"   => Some(PluginPermission::ReadDocuments),
        "write_document"    => Some(PluginPermission::WriteDocuments),
        _ => None,
    }
}

// ── Host data (accessible from host functions) ────────────────────────────────

struct HostData {
    permissions:  Vec<PluginPermission>,
    plugin_id:    String,
    /// Pending CIL result to write into WASM memory (set by civium::cil_call handler).
    pending_cil:  Option<String>,
    /// Real CIL executor — routes to the store.  None in unit-test / stub mode.
    cil_executor: Option<CilExecutor>,
}

// ── Engine ────────────────────────────────────────────────────────────────────

/// Shared wasmtime compilation engine.  Construct once, reuse for every plugin.
pub struct WasmEngine {
    engine: Engine,
}

impl WasmEngine {
    pub fn new() -> anyhow::Result<Self> {
        let mut cfg = Config::new();
        cfg.cranelift_opt_level(OptLevel::Speed);
        // Hard cap on execution: if a plugin loops for more than 10 M instructions
        // the epoch interrupt fires and the call returns an error.
        cfg.epoch_interruption(true);
        let engine = Engine::new(&cfg)?;
        Ok(Self { engine })
    }

    /// Compile and instantiate a third-party plugin.
    ///
    /// `executor` — optional real CIL dispatcher (provided by AppState in production).
    /// When `None`, the stub is used (unit tests / early boot).
    pub fn load(
        &self,
        manifest: PluginManifest,
        wasm_bytes: &[u8],
        executor: Option<CilExecutor>,
    ) -> anyhow::Result<SandboxedPlugin> {
        let module = Module::new(&self.engine, wasm_bytes)?;

        let host_data = HostData {
            permissions:  manifest.permissions.clone(),
            plugin_id:    manifest.id.clone(),
            pending_cil:  None,
            cil_executor: executor,
        };
        let mut store: Store<HostData> = Store::new(&self.engine, host_data);
        // Allow up to 10 000 epochs before interrupting.
        store.set_epoch_deadline(10_000);

        let mut linker: Linker<HostData> = Linker::new(&self.engine);
        Self::define_host_api(&mut linker)?;

        let instance = linker.instantiate(&mut store, &module)?;
        Ok(SandboxedPlugin { manifest, store: Arc::new(Mutex::new(store)), instance })
    }

    fn define_host_api(linker: &mut Linker<HostData>) -> anyhow::Result<()> {
        // civium::log(msg_ptr: i32, msg_len: i32)
        linker.func_wrap("civium", "log", |mut caller: wasmtime::Caller<'_, HostData>, ptr: i32, len: i32| {
            let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                Some(m) => m,
                None => return,
            };
            let data = mem.data(&caller);
            if let Ok(slice) = read_str(data, ptr as usize, len as usize) {
                let id = caller.data().plugin_id.clone();
                tracing::info!("[plugin:{id}] {slice}");
            }
        })?;

        // civium::cil_call(
        //   action_ptr, action_len,   — action name (UTF-8)
        //   param_ptr,  param_len,    — JSON parameter
        //   out_ptr,    out_max       — output buffer in WASM memory
        // ) -> i32   (bytes written, or negative error code)
        linker.func_wrap("civium", "cil_call", |
            mut caller: wasmtime::Caller<'_, HostData>,
            action_ptr: i32, action_len: i32,
            param_ptr: i32,  param_len: i32,
            out_ptr: i32,    out_max: i32,
        | -> i32 {
            let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                Some(m) => m,
                None => return -1,
            };

            let data = mem.data(&caller);
            let action = match read_str(data, action_ptr as usize, action_len as usize) {
                Ok(s) => s.to_owned(),
                Err(_) => return -2,
            };
            let param = match read_str(data, param_ptr as usize, param_len as usize) {
                Ok(s) => s.to_owned(),
                Err(_) => return -3,
            };

            // Permission check before executing the CIL action.
            let allowed = {
                let hd = caller.data();
                match required_permission(&action) {
                    Some(req) => hd.permissions.contains(&req),
                    None => false, // unknown action — deny
                }
            };
            if !allowed {
                let id = caller.data().plugin_id.clone();
                tracing::warn!("[plugin:{id}] CIL action '{action}' refused — permission denied");
                return -10;
            }

            // Execute via the real executor if available, otherwise use the stub.
            let executor = caller.data().cil_executor.clone();
            let result = match executor {
                Some(exec) => exec(&action, &param),
                None       => execute_cil_stub(&action, &param),
            };
            let bytes = result.as_bytes();
            if bytes.len() > out_max as usize {
                return -20; // buffer too small
            }
            let mem_data = mem.data_mut(&mut caller);
            let start = out_ptr as usize;
            mem_data[start..start + bytes.len()].copy_from_slice(bytes);
            bytes.len() as i32
        })?;

        Ok(())
    }
}

// ── Sandboxed plugin ──────────────────────────────────────────────────────────

/// A compiled, instantiated, permission-checked plugin running in its own WASM sandbox.
pub struct SandboxedPlugin {
    pub manifest: PluginManifest,
    store:    Arc<Mutex<Store<HostData>>>,
    instance: wasmtime::Instance,
}

impl SandboxedPlugin {
    /// Dispatch a Civium event to the plugin.
    ///
    /// * `event_type` — e.g. `"message.received"`, `"member.joined"`
    /// * `payload`    — JSON string (event-specific data)
    ///
    /// Returns the plugin's JSON response string, or an error.
    pub fn handle_event(&self, event_type: &str, payload: &str) -> anyhow::Result<String> {
        let mut store = self.store.lock().map_err(|_| anyhow::anyhow!("store lock poisoned"))?;

        let mem: Memory = self.instance
            .get_export(&mut *store, "memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("plugin does not export 'memory'"))?;

        let alloc = self.instance
            .get_typed_func::<i32, i32>(&mut *store, "civium_alloc")
            .map_err(|_| anyhow::anyhow!("plugin does not export 'civium_alloc'"))?;

        let free = self.instance
            .get_typed_func::<(i32, i32), ()>(&mut *store, "civium_free")
            .map_err(|_| anyhow::anyhow!("plugin does not export 'civium_free'"))?;

        let handle = self.instance
            .get_typed_func::<(i32, i32, i32, i32), i64>(&mut *store, "civium_handle")
            .map_err(|_| anyhow::anyhow!("plugin does not export 'civium_handle'"))?;

        // Write event_type into WASM memory.
        let et_bytes = event_type.as_bytes();
        let et_ptr = alloc.call(&mut *store, et_bytes.len() as i32)?;
        mem.data_mut(&mut *store)[et_ptr as usize..et_ptr as usize + et_bytes.len()]
            .copy_from_slice(et_bytes);

        // Write payload into WASM memory.
        let pl_bytes = payload.as_bytes();
        let pl_ptr = alloc.call(&mut *store, pl_bytes.len() as i32)?;
        mem.data_mut(&mut *store)[pl_ptr as usize..pl_ptr as usize + pl_bytes.len()]
            .copy_from_slice(pl_bytes);

        // Call the plugin.
        let packed = handle.call(
            &mut *store,
            (et_ptr, et_bytes.len() as i32, pl_ptr, pl_bytes.len() as i32),
        )?;

        // Free input buffers.
        free.call(&mut *store, (et_ptr, et_bytes.len() as i32))?;
        free.call(&mut *store, (pl_ptr, pl_bytes.len() as i32))?;

        // Unpack the response: upper 32 bits = ptr, lower 32 bits = length.
        let resp_ptr = ((packed >> 32) & 0xFFFF_FFFF) as usize;
        let resp_len = (packed & 0xFFFF_FFFF) as usize;

        if resp_len == 0 {
            return Ok("{}".to_string());
        }

        let response = {
            let data = mem.data(&*store);
            read_str(data, resp_ptr, resp_len)?.to_owned()
        };

        // Free response buffer.
        free.call(&mut *store, (resp_ptr as i32, resp_len as i32))?;

        Ok(response)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_str(data: &[u8], ptr: usize, len: usize) -> anyhow::Result<&str> {
    let slice = data.get(ptr..ptr + len)
        .ok_or_else(|| anyhow::anyhow!("WASM memory access out of bounds"))?;
    std::str::from_utf8(slice).map_err(|e| anyhow::anyhow!("invalid UTF-8 from plugin: {e}"))
}

/// Stub CIL executor.  In production, this is replaced by a callback that
/// routes to the real store (passed in through AppState).  The stub returns
/// an empty JSON object so plugins don't crash during unit tests.
fn execute_cil_stub(action: &str, _param: &str) -> String {
    tracing::debug!("[civium-sandbox] CIL stub called: action={action}");
    "{}".to_string()
}
