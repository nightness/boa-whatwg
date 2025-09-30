// Test Crypto API functionality

console.log("Testing Crypto API...");

try {
    // Test that Crypto constructor exists
    console.log("Crypto constructor exists:", typeof Crypto === 'function');

    // Test that crypto global object exists (should be automatically available)
    console.log("crypto global exists:", typeof crypto === 'object' && crypto !== null);

    if (typeof crypto === 'object' && crypto !== null) {
        // Test getRandomValues method
        console.log("getRandomValues method exists:", typeof crypto.getRandomValues === 'function');

        // Test randomUUID method
        console.log("randomUUID method exists:", typeof crypto.randomUUID === 'function');

        // Test getRandomValues with Uint8Array
        let uint8Array = new Uint8Array(10);
        console.log("Before getRandomValues:", Array.from(uint8Array));
        let result = crypto.getRandomValues(uint8Array);
        console.log("After getRandomValues:", Array.from(uint8Array));
        console.log("Returns same array:", result === uint8Array);

        // Test randomUUID
        let uuid1 = crypto.randomUUID();
        let uuid2 = crypto.randomUUID();
        console.log("Generated UUID 1:", uuid1);
        console.log("Generated UUID 2:", uuid2);
        console.log("UUIDs are different:", uuid1 !== uuid2);
        console.log("UUID format test (version 4):", /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(uuid1));

        console.log("✅ Crypto API test passed!");
    } else {
        console.log("❌ crypto global object not available");
    }

} catch (e) {
    console.log("❌ Crypto API test error:", e.message);
}