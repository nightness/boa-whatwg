// Test script for SharedWorker onconnect events
// This would typically be executed in the main thread context

console.log("Testing SharedWorker with onconnect events...");

// Test 1: Create SharedWorker and check port property
try {
    const worker = new SharedWorker('https://example.com/shared-worker.js');

    if (worker.port) {
        console.log("✓ SharedWorker.port exists:", typeof worker.port);
    } else {
        console.log("✗ SharedWorker.port is missing");
    }

    // Test 2: Create multiple connections to same worker
    const worker2 = new SharedWorker('https://example.com/shared-worker.js');

    if (worker2.port) {
        console.log("✓ Second SharedWorker connection created");
    }

    // Test 3: Test with named workers (different instances)
    const namedWorker1 = new SharedWorker('https://example.com/shared-worker.js', 'worker1');
    const namedWorker2 = new SharedWorker('https://example.com/shared-worker.js', 'worker2');

    console.log("✓ Named SharedWorkers created");

    // Test 4: Test port readonly property
    const originalPort = worker.port;
    worker.port = null;

    if (worker.port === originalPort) {
        console.log("✓ SharedWorker.port is readonly");
    } else {
        console.log("✗ SharedWorker.port should be readonly");
    }

    console.log("SharedWorker onconnect test completed successfully");

} catch (error) {
    console.log("✗ Error in SharedWorker test:", error.message);
}