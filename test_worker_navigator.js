// Test script for WorkerNavigator in worker context
// This script should be executed within a worker context

console.log("Testing WorkerNavigator...");

// Test navigator object exists
if (typeof navigator !== 'undefined') {
    console.log("✓ navigator object exists");

    // Test userAgent property
    if (typeof navigator.userAgent === 'string') {
        console.log("✓ navigator.userAgent:", navigator.userAgent);
    } else {
        console.log("✗ navigator.userAgent is not a string");
    }

    // Test platform property
    if (typeof navigator.platform === 'string') {
        console.log("✓ navigator.platform:", navigator.platform);
    } else {
        console.log("✗ navigator.platform is not a string");
    }

    // Test language property
    if (typeof navigator.language === 'string') {
        console.log("✓ navigator.language:", navigator.language);
    } else {
        console.log("✗ navigator.language is not a string");
    }

    // Test languages property
    if (Array.isArray(navigator.languages)) {
        console.log("✓ navigator.languages:", navigator.languages);
    } else {
        console.log("✗ navigator.languages is not an array");
    }

    // Test onLine property
    if (typeof navigator.onLine === 'boolean') {
        console.log("✓ navigator.onLine:", navigator.onLine);
    } else {
        console.log("✗ navigator.onLine is not a boolean");
    }

    // Test hardwareConcurrency property
    if (typeof navigator.hardwareConcurrency === 'number') {
        console.log("✓ navigator.hardwareConcurrency:", navigator.hardwareConcurrency);
    } else {
        console.log("✗ navigator.hardwareConcurrency is not a number");
    }

} else {
    console.log("✗ navigator object does not exist");
}

console.log("WorkerNavigator test completed");