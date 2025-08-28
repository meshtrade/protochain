#!/usr/bin/env node

const WebSocket = require('ws');

console.log('🔍 Testing Solana WebSocket connectivity...');

const ws = new WebSocket('ws://localhost:8900');

ws.on('open', function open() {
    console.log('✅ WebSocket connected successfully!');
    
    // Test a simple getHealth subscription
    const subscribeRequest = {
        jsonrpc: "2.0",
        id: 1,
        method: "getHealth"
    };
    
    console.log('📤 Sending test request:', JSON.stringify(subscribeRequest));
    ws.send(JSON.stringify(subscribeRequest));
    
    setTimeout(() => {
        console.log('✅ WebSocket test completed successfully');
        ws.close();
    }, 2000);
});

ws.on('message', function message(data) {
    console.log('📨 Received:', data.toString());
});

ws.on('error', function error(err) {
    console.error('❌ WebSocket error:', err.message);
});

ws.on('close', function close(code, reason) {
    console.log('🔌 WebSocket connection closed:', code, reason.toString());
    process.exit(0);
});

// Timeout after 5 seconds
setTimeout(() => {
    console.error('⏰ WebSocket connection timeout');
    ws.close();
    process.exit(1);
}, 5000);