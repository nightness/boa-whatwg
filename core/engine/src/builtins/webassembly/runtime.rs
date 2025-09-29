//! WebAssembly runtime management using wasmtime
//!
//! This module provides the runtime infrastructure for executing WebAssembly
//! modules, managing engines, stores, and compiled modules.

use crate::{Context, JsResult, JsNativeError, JsData};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use wasmtime::*;

/// Global WebAssembly runtime manager
///
/// This provides a singleton runtime that manages the wasmtime Engine,
/// compiled modules, instances, and stores for the entire Boa context.
#[derive(Clone, Trace, Finalize, JsData)]
pub struct WebAssemblyRuntime {
    #[unsafe_ignore_trace]
    engine: Arc<Engine>,
    #[unsafe_ignore_trace]
    modules: Arc<Mutex<HashMap<String, Module>>>,
    #[unsafe_ignore_trace]
    instances: Arc<Mutex<HashMap<String, Instance>>>,
    #[unsafe_ignore_trace]
    stores: Arc<Mutex<HashMap<String, Store<()>>>>,
    #[unsafe_ignore_trace]
    memories: Arc<Mutex<HashMap<String, Memory>>>,
    #[unsafe_ignore_trace]
    tables: Arc<Mutex<HashMap<String, Table>>>,
    #[unsafe_ignore_trace]
    globals: Arc<Mutex<HashMap<String, Global>>>,
}

static RUNTIME: OnceLock<WebAssemblyRuntime> = OnceLock::new();

impl std::fmt::Debug for WebAssemblyRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebAssemblyRuntime")
            .field("modules", &self.modules)
            .field("instances", &self.instances)
            .field("stores", &self.stores)
            .field("memories", &self.memories)
            .field("tables", &self.tables)
            .field("globals", &self.globals)
            .finish()
    }
}

impl WebAssemblyRuntime {
    /// Create a new WebAssembly runtime with optimized configuration
    fn new() -> Self {
        // Configure wasmtime engine with optimal settings for web compatibility
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.wasm_bulk_memory(true);
        config.wasm_reference_types(true);
        config.wasm_simd(true);
        config.wasm_multi_memory(true);
        config.wasm_memory64(true); // Enable 64-bit memory support for WebAssembly 3.0
        config.wasm_threads(true);
        config.wasm_tail_call(true);
        config.wasm_function_references(true);
        config.wasm_gc(true); // Enable garbage collection support for WebAssembly 3.0
        config.wasm_multi_value(true);
        config.cranelift_opt_level(OptLevel::Speed);

        let engine = Arc::new(Engine::new(&config).expect("Failed to create WebAssembly engine"));

        Self {
            engine,
            modules: Arc::new(Mutex::new(HashMap::new())),
            instances: Arc::new(Mutex::new(HashMap::new())),
            stores: Arc::new(Mutex::new(HashMap::new())),
            memories: Arc::new(Mutex::new(HashMap::new())),
            tables: Arc::new(Mutex::new(HashMap::new())),
            globals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create the global WebAssembly runtime
    pub fn get_or_create(_context: &mut Context) -> JsResult<&'static WebAssemblyRuntime> {
        Ok(RUNTIME.get_or_init(Self::new))
    }

    /// Get the wasmtime engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Compile WebAssembly bytes into a module
    pub fn compile_module(&self, bytes: &[u8]) -> Result<String, wasmtime::Error> {
        let module = Module::new(&*self.engine, bytes)?;
        let module_id = self.generate_module_id();

        self.modules.lock().unwrap().insert(module_id.clone(), module);
        Ok(module_id)
    }

    /// Get a compiled module by ID
    pub fn get_module(&self, module_id: &str) -> Option<Module> {
        self.modules.lock().unwrap().get(module_id).cloned()
    }

    /// Create a new store for WebAssembly execution
    pub fn create_store(&self) -> String {
        let store = Store::new(&*self.engine, ());
        let store_id = self.generate_store_id();

        self.stores.lock().unwrap().insert(store_id.clone(), store);
        store_id
    }

    /// Get a store by ID (mutable access)
    pub fn with_store_mut<F, R>(&self, store_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut Store<()>) -> R,
    {
        self.stores.lock().unwrap()
            .get_mut(store_id)
            .map(f)
    }

    /// Instantiate a module with imports
    pub fn instantiate_module(
        &self,
        module_id: &str,
        store_id: String,
        imports: std::collections::HashMap<String, std::collections::HashMap<String, wasmtime::Extern>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let module = self.get_module(module_id)
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Module not found")) as Box<dyn std::error::Error>)?;

        let instance_id = self.generate_instance_id();

        // Convert the nested HashMap imports to a flat Vec<Extern> for wasmtime
        let mut import_vec = Vec::new();

        // Iterate through module imports to maintain correct order
        for import in module.imports() {
            let module_name = import.module();
            let import_name = import.name();

            if let Some(module_imports) = imports.get(module_name) {
                if let Some(extern_val) = module_imports.get(import_name) {
                    import_vec.push(extern_val.clone());
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Import {}.{} not found", module_name, import_name)
                    )));
                }
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Import module {} not found", module_name)
                )));
            }
        }

        self.with_store_mut(&store_id, |store| {
            let instance = Instance::new(store, &module, &import_vec)?;
            self.instances.lock().unwrap().insert(instance_id.clone(), instance);
            Ok(instance_id)
        })
        .unwrap_or_else(|| Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Store not found")) as Box<dyn std::error::Error>))
    }

    /// Get an instance by ID
    pub fn get_instance(&self, instance_id: &str) -> Option<Instance> {
        self.instances.lock().unwrap().get(instance_id).cloned()
    }

    /// Create a WebAssembly memory
    pub fn create_memory(&self, memory_type: MemoryType) -> Result<String, wasmtime::Error> {
        let store_id = self.create_store();
        let memory_id = self.generate_memory_id();

        self.with_store_mut(&store_id, |store| {
            let memory = Memory::new(store, memory_type)?;
            self.memories.lock().unwrap().insert(memory_id.clone(), memory);
            Ok(memory_id)
        })
        .unwrap_or_else(|| Err(wasmtime::Error::msg("Failed to create store")))
    }

    /// Get a memory by ID
    pub fn get_memory(&self, memory_id: &str) -> Option<Memory> {
        self.memories.lock().unwrap().get(memory_id).cloned()
    }

    /// Create a WebAssembly table
    pub fn create_table(&self, table_type: TableType, init: wasmtime::Ref) -> Result<String, wasmtime::Error> {
        let store_id = self.create_store();
        let table_id = self.generate_table_id();

        self.with_store_mut(&store_id, |store| {
            let table = Table::new(store, table_type, init)?;
            self.tables.lock().unwrap().insert(table_id.clone(), table);
            Ok(table_id)
        })
        .unwrap_or_else(|| Err(wasmtime::Error::msg("Failed to create store")))
    }

    /// Get a table by ID
    pub fn get_table(&self, table_id: &str) -> Option<Table> {
        self.tables.lock().unwrap().get(table_id).cloned()
    }

    /// Create a WebAssembly global
    pub fn create_global(&self, global_type: GlobalType, init: wasmtime::Val) -> Result<String, wasmtime::Error> {
        let store_id = self.create_store();
        let global_id = self.generate_global_id();

        self.with_store_mut(&store_id, |store| {
            let global = Global::new(store, global_type, init)?;
            self.globals.lock().unwrap().insert(global_id.clone(), global);
            Ok(global_id)
        })
        .unwrap_or_else(|| Err(wasmtime::Error::msg("Failed to create store")))
    }

    /// Get a global by ID
    pub fn get_global(&self, global_id: &str) -> Option<Global> {
        self.globals.lock().unwrap().get(global_id).cloned()
    }

    /// Generate a unique module ID
    fn generate_module_id(&self) -> String {
        format!("module_{}", self.generate_unique_id())
    }

    /// Generate a unique instance ID
    fn generate_instance_id(&self) -> String {
        format!("instance_{}", self.generate_unique_id())
    }

    /// Generate a unique store ID
    fn generate_store_id(&self) -> String {
        format!("store_{}", self.generate_unique_id())
    }

    /// Generate a unique memory ID
    fn generate_memory_id(&self) -> String {
        format!("memory_{}", self.generate_unique_id())
    }

    /// Generate a unique table ID
    fn generate_table_id(&self) -> String {
        format!("table_{}", self.generate_unique_id())
    }

    /// Generate a unique global ID
    fn generate_global_id(&self) -> String {
        format!("global_{}", self.generate_unique_id())
    }

    /// Generate a unique ID using random number
    fn generate_unique_id(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        hasher.finish()
    }

    /// Clean up resources (called when context is dropped)
    pub fn cleanup(&self) {
        self.instances.lock().unwrap().clear();
        self.modules.lock().unwrap().clear();
        self.stores.lock().unwrap().clear();
        self.memories.lock().unwrap().clear();
        self.tables.lock().unwrap().clear();
        self.globals.lock().unwrap().clear();
    }
}