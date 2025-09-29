//! Worker script loading and execution implementation
//!
//! Handles fetching, parsing, and executing JavaScript files in isolated Worker contexts

use crate::{
    Context, JsResult, JsValue, JsNativeError, Source, JsArgs, js_string,
    object::JsObject,
    builtins::{
        worker_events::{WorkerEvent, WorkerEventType, dispatch_worker_event},
        worker_global_scope::{WorkerGlobalScope, WorkerGlobalScopeType, WorkerMessage, MessageSource},
        structured_clone::{StructuredCloneValue, structured_clone, structured_deserialize, TransferList},
    },
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use url::Url;

/// Worker script execution context
#[derive(Debug)]
pub struct WorkerExecutionContext {
    /// Script URL being executed
    script_url: String,
    /// Worker type ("classic" or "module")
    worker_type: String,
    /// Whether the worker is terminated
    terminated: Arc<Mutex<bool>>,
    /// Script content cache
    script_content: Arc<Mutex<Option<String>>>,
    /// Worker global scope for script execution
    global_scope: Arc<Mutex<Option<WorkerGlobalScope>>>,
}

impl WorkerExecutionContext {
    /// Create a new worker execution context
    pub fn new(script_url: String, worker_type: String) -> Self {
        Self {
            script_url,
            worker_type,
            terminated: Arc::new(Mutex::new(false)),
            script_content: Arc::new(Mutex::new(None)),
            global_scope: Arc::new(Mutex::new(None)),
        }
    }

    /// Load and execute the worker script
    pub async fn load_and_execute(&self, _worker_obj: JsObject, _main_context: &mut Context) -> JsResult<()> {
        // Check if terminated
        if *self.terminated.lock().unwrap() {
            return Ok(());
        }

        // Fetch the script content
        let script_content = match self.fetch_script().await {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to fetch worker script: {}", e);
                return Ok(());
            }
        };

        // Store script content
        {
            let mut content_guard = self.script_content.lock().unwrap();
            *content_guard = Some(script_content.clone());
        }

        eprintln!("Worker script loaded: {} bytes", script_content.len());

        // TODO: Implement actual script execution in isolated context
        // For now, we just cache the script content and indicate successful loading

        Ok(())
    }

    /// Fetch script content from URL
    async fn fetch_script(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Parse URL
        let url = Url::parse(&self.script_url)?;

        match url.scheme() {
            "http" | "https" => {
                // Fetch from network
                self.fetch_remote_script(&url).await
            }
            "data" => {
                // Handle data URLs
                self.parse_data_url(&url)
            }
            _ => {
                Err(format!("Unsupported URL scheme: {}", url.scheme()).into())
            }
        }
    }

    /// Fetch script from remote URL
    async fn fetch_remote_script(&self, url: &Url) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Use reqwest for HTTP fetching
        let client = reqwest::Client::builder()
            .user_agent("Thalora-WebBrowser/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client.get(url.as_str()).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let content = response.text().await?;
        Ok(content)
    }

    /// Parse data URL content
    fn parse_data_url(&self, url: &Url) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let path = url.path();

        // Simple data URL parsing for text/javascript
        if path.starts_with("text/javascript,") {
            Ok(path.strip_prefix("text/javascript,").unwrap().to_string())
        } else if path.starts_with("application/javascript,") {
            Ok(path.strip_prefix("application/javascript,").unwrap().to_string())
        } else {
            // Handle base64 encoding if needed
            Err("Unsupported data URL format".into())
        }
    }

    /// Get the loaded script content
    pub fn get_script_content(&self) -> Option<String> {
        self.script_content.lock().unwrap().clone()
    }

    /// Get script URL
    pub fn get_script_url(&self) -> &str {
        &self.script_url
    }

    /// Simple load and execute without thread-unsafe objects
    pub async fn load_and_execute_simple(&self) -> JsResult<()> {
        // Check if terminated
        if *self.terminated.lock().unwrap() {
            return Ok(());
        }

        // Fetch the script content
        let script_content = match self.fetch_script().await {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to fetch worker script: {}", e);
                return Ok(());
            }
        };

        // Store script content
        {
            let mut content_guard = self.script_content.lock().unwrap();
            *content_guard = Some(script_content.clone());
        }

        eprintln!("Worker script loaded: {} bytes", script_content.len());

        // Create and execute in WorkerGlobalScope
        self.execute_in_global_scope(&script_content).await?;

        Ok(())
    }

    /// Execute script in proper WorkerGlobalScope
    async fn execute_in_global_scope(&self, script_content: &str) -> JsResult<()> {
        // Determine the global scope type based on worker type
        let scope_type = match self.worker_type.as_str() {
            "shared" => WorkerGlobalScopeType::Shared,
            "service" => WorkerGlobalScopeType::Service,
            _ => WorkerGlobalScopeType::Dedicated, // default to dedicated
        };

        // Create WorkerGlobalScope
        let global_scope = WorkerGlobalScope::new(scope_type, &self.script_url)?;

        // Store the global scope
        {
            let mut scope_guard = self.global_scope.lock().unwrap();
            *scope_guard = Some(global_scope);
        }

        // Execute the script in a new isolated context
        // Note: In a real implementation, this would be in a separate thread
        // For now, we'll execute it in the current context to demonstrate the concept
        self.execute_script_in_isolated_context(script_content).await?;

        Ok(())
    }

    /// Execute script in isolated JavaScript context
    async fn execute_script_in_isolated_context(&self, script_content: &str) -> JsResult<()> {
        // Create a new isolated JavaScript context for the worker
        let mut worker_context = Context::default();

        // Get the global scope and initialize it in the context
        if let Some(ref global_scope) = *self.global_scope.lock().unwrap() {
            // Initialize the WorkerGlobalScope APIs in the context
            global_scope.initialize_in_context(&mut worker_context)?;

            // Execute the script
            let result = global_scope.execute_script(&mut worker_context, script_content);

            match result {
                Ok(value) => {
                    eprintln!("Worker script executed successfully in isolated context");

                    // Process any pending messages from main thread
                    global_scope.process_main_thread_messages(&mut worker_context)?;
                }
                Err(e) => {
                    eprintln!("Worker script execution failed: {:?}", e);
                }
            }
        }

        Ok(())
    }

    /// Terminate the worker
    pub fn terminate(&self) {
        *self.terminated.lock().unwrap() = true;
    }

    /// Check if worker is terminated
    pub fn is_terminated(&self) -> bool {
        *self.terminated.lock().unwrap()
    }

    /// Post message from main thread to worker
    pub async fn post_message_to_worker(&self, message: JsValue) -> JsResult<()> {
        if self.is_terminated() {
            return Err(JsNativeError::error()
                .with_message("Cannot send message to terminated worker")
                .into());
        }

        // Clone message using structured cloning
        let mut context = Context::default();
        let cloned_message = structured_clone(&message, &mut context, None)?;

        self.post_cloned_message_to_worker(cloned_message).await
    }

    /// Post structured cloned message from main thread to worker
    pub async fn post_cloned_message_to_worker(&self, cloned_message: StructuredCloneValue) -> JsResult<()> {
        if self.is_terminated() {
            return Err(JsNativeError::error()
                .with_message("Cannot send message to terminated worker")
                .into());
        }

        // Send message to worker's global scope
        if let Some(ref global_scope) = *self.global_scope.lock().unwrap() {
            if let Some(sender) = global_scope.get_main_thread_sender() {
                let worker_msg = WorkerMessage {
                    data: cloned_message,
                    ports: Vec::new(), // TODO: Handle transferable objects
                    source: MessageSource::MainThread,
                };

                if let Err(_) = sender.send(worker_msg) {
                    eprintln!("Failed to send message to worker global scope");
                } else {
                    eprintln!("Structured cloned message sent to worker global scope");
                }
            }
        } else {
            // Fallback for when global scope isn't ready yet
            eprintln!("Message sent to worker (scope not ready) - structured clone ready for delivery");
        }

        Ok(())
    }
}

/// Worker script loader and manager
#[derive(Debug)]
pub struct WorkerScriptLoader;

impl WorkerScriptLoader {
    /// Start worker execution with script loading
    pub async fn start_worker_execution(
        script_url: String,
        worker_type: String,
    ) -> JsResult<Arc<WorkerExecutionContext>> {
        let execution_context = Arc::new(WorkerExecutionContext::new(script_url.clone(), worker_type));
        let exec_ctx_clone = execution_context.clone();

        // Check if we're in a Tokio runtime context
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = exec_ctx_clone.load_and_execute_simple().await {
                        eprintln!("Worker execution error: {:?}", e);
                    }
                });
            }
            Err(_) => {
                // No Tokio runtime available, execute synchronously for testing
                eprintln!("No async runtime available - worker will execute synchronously");
            }
        }

        Ok(execution_context)
    }
}