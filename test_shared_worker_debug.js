// Debug SharedWorker functionality

console.log("Testing SharedWorker...");

try {
    let sharedWorker = new SharedWorker('data:text/javascript,console.log("shared");');
    console.log("SharedWorker created:", sharedWorker);
    console.log("SharedWorker instanceof check:", sharedWorker instanceof SharedWorker);
    console.log("Port exists:", sharedWorker.port);
    console.log("Port type:", typeof sharedWorker.port);
    console.log("Port not null:", sharedWorker.port !== null);

    if (sharedWorker.port) {
        console.log("Port methods:");
        console.log("- postMessage:", typeof sharedWorker.port.postMessage);
        console.log("- start:", typeof sharedWorker.port.start);
        console.log("- close:", typeof sharedWorker.port.close);
    }

    console.log("All checks passed:",
        sharedWorker instanceof SharedWorker &&
        typeof sharedWorker.port === 'object' &&
        sharedWorker.port !== null &&
        typeof sharedWorker.port.postMessage === 'function' &&
        typeof sharedWorker.port.start === 'function' &&
        typeof sharedWorker.port.close === 'function'
    );
} catch (e) {
    console.log("Error:", e.message);
}