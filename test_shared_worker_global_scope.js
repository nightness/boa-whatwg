// Test script for SharedWorkerGlobalScope onconnect event handling
// This would be executed within a SharedWorkerGlobalScope context

console.log("Testing SharedWorkerGlobalScope onconnect...");

// Test that the global scope has the necessary properties
if (typeof name !== 'undefined') {
    console.log("✓ name property exists in SharedWorkerGlobalScope:", name);
} else {
    console.log("✗ name property missing in SharedWorkerGlobalScope");
}

if (typeof onconnect !== 'undefined') {
    console.log("✓ onconnect property exists in SharedWorkerGlobalScope");
} else {
    console.log("✗ onconnect property missing in SharedWorkerGlobalScope");
}

// Test onconnect event handler
onconnect = function(event) {
    console.log("✓ onconnect event fired!");
    console.log("Event type:", event.type);
    console.log("Event ports:", event.ports);

    if (event.ports && event.ports.length > 0) {
        const port = event.ports[0];
        console.log("✓ Received MessagePort in connect event");

        // Start the port
        port.start();

        // Send a message back to the connecting client
        port.postMessage("Hello from SharedWorker!");

        // Set up message handler
        port.onmessage = function(msgEvent) {
            console.log("Received message from client:", msgEvent.data);
            port.postMessage("Echo: " + msgEvent.data);
        };
    } else {
        console.log("✗ No ports in connect event");
    }
};

// Test internal connect dispatcher (if available)
if (typeof _dispatchConnect === 'function') {
    console.log("✓ _dispatchConnect function available");
} else {
    console.log("✗ _dispatchConnect function not available");
}

console.log("SharedWorkerGlobalScope onconnect test setup completed");