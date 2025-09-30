// Functional tests for Boa APIs
console.log('=== Testing Boa API Functionality ===');

// Test Storage API functionality
try {
    // Test localStorage
    window.localStorage.setItem('test', 'value');
    const stored = window.localStorage.getItem('test');
    console.log('localStorage test:', stored === 'value' ? '✅ PASS' : '❌ FAIL');

    // Test sessionStorage
    window.sessionStorage.setItem('session', 'data');
    const sessionData = window.sessionStorage.getItem('session');
    console.log('sessionStorage test:', sessionData === 'data' ? '✅ PASS' : '❌ FAIL');

    // Test storage length
    console.log('localStorage length:', window.localStorage.length);

    // Clean up
    window.localStorage.clear();
    window.sessionStorage.clear();
} catch (e) {
    console.log('❌ Storage functionality error:', e.message);
}

// Test Events API functionality
try {
    // Test Event constructor
    const event = new Event('test', { bubbles: true, cancelable: true });
    console.log('Event constructor:', event.type === 'test' ? '✅ PASS' : '❌ FAIL');
    console.log('Event bubbles:', event.bubbles === true ? '✅ PASS' : '❌ FAIL');

    // Test CustomEvent
    const customEvent = new CustomEvent('custom', { detail: { data: 'test' } });
    console.log('CustomEvent constructor:', customEvent.type === 'custom' ? '✅ PASS' : '❌ FAIL');
} catch (e) {
    console.log('❌ Events functionality error:', e.message);
}

// Test WebSocket constructor
try {
    // This should throw an error for invalid URL, but constructor should work
    try {
        new WebSocket('invalid-url');
        console.log('WebSocket constructor: ❌ FAIL - should have thrown error');
    } catch (wsError) {
        console.log('WebSocket constructor validation: ✅ PASS - correctly rejected invalid URL');
    }

    // Test valid WebSocket creation (will fail to connect but constructor should work)
    const ws = new WebSocket('ws://localhost:8080');
    console.log('WebSocket creation:', ws instanceof WebSocket ? '✅ PASS' : '❌ FAIL');
    console.log('WebSocket readyState:', ws.readyState === WebSocket.CONNECTING ? '✅ PASS' : '❌ FAIL');
} catch (e) {
    console.log('❌ WebSocket functionality error:', e.message);
}

// Test Response API functionality
try {
    // Test Response constructor
    const response = new Response('test body', { status: 201, statusText: 'Created' });
    console.log('Response constructor:', response.status === 201 ? '✅ PASS' : '❌ FAIL');
    console.log('Response ok property:', response.ok === true ? '✅ PASS' : '❌ FAIL');

    // Test text() method returns Promise
    const textPromise = response.text();
    console.log('Response.text() returns Promise:', textPromise instanceof Promise ? '✅ PASS' : '❌ FAIL');
} catch (e) {
    console.log('❌ Response functionality error:', e.message);
}

// Test Headers functionality
try {
    const headers = new Headers();
    headers.set('Content-Type', 'application/json');
    console.log('Headers constructor and set:', headers.has('Content-Type') ? '✅ PASS' : '❌ FAIL');
} catch (e) {
    console.log('❌ Headers functionality error:', e.message);
}

// Test WebSocketStream constructor
try {
    const wsStream = new WebSocketStream('ws://localhost:8080');
    console.log('WebSocketStream creation:', wsStream instanceof WebSocketStream ? '✅ PASS' : '❌ FAIL');
    console.log('WebSocketStream url property:', wsStream.url === 'ws://localhost:8080' ? '✅ PASS' : '❌ FAIL');
    console.log('WebSocketStream readyState:', wsStream.readyState === 0 ? '✅ PASS' : '❌ FAIL'); // CONNECTING = 0
} catch (e) {
    console.log('❌ WebSocketStream functionality error:', e.message);
}

console.log('=== Functional Testing Complete ===');