#!/usr/bin/env node

// Simple WebSocket test for Solana local validator
console.log('🔍 Testing Solana WebSocket connectivity to ws://localhost:8900...');

// Try a simple telnet-like connection test first
const net = require('net');
const client = new net.Socket();

client.connect(8900, '127.0.0.1', function() {
    console.log('✅ TCP connection to port 8900 successful!');
    client.destroy();
    
    // Now test actual WebSocket
    testWebSocket();
});

client.on('error', function(err) {
    console.error('❌ TCP connection failed:', err.message);
    console.log('❌ WebSocket port 8900 is not accessible');
    process.exit(1);
});

function testWebSocket() {
    try {
        const WebSocket = require('ws');
        
        console.log('📡 Attempting WebSocket connection...');
        const ws = new WebSocket('ws://localhost:8900');
        
        let connected = false;
        
        ws.on('open', function open() {
            connected = true;
            console.log('🎉 WebSocket connected successfully!');
            
            // Test signature subscription request
            const testRequest = {
                jsonrpc: "2.0",
                id: 1,
                method: "signatureSubscribe",
                params: [
                    "5VRTkvGLdcBhgUTyJhZWYKyHAmdZhiU2qb2PmEZbRTfYSRZKQ8fkV7x3w2r8vHqXLqVWnpC3jFcq9xE8qQ5UaJ1u",
                    { "commitment": "confirmed" }
                ]
            };
            
            console.log('📤 Sending signature subscription test...');
            ws.send(JSON.stringify(testRequest));
            
            setTimeout(() => {
                console.log('✅ WebSocket test completed - connection is working!');
                ws.close();
            }, 2000);
        });
        
        ws.on('message', function message(data) {
            console.log('📨 Received response:', data.toString());
        });
        
        ws.on('error', function error(err) {
            console.error('❌ WebSocket error:', err.message);
            console.log('💡 This suggests WebSocket endpoint may not be properly configured');
        });
        
        ws.on('close', function close(code, reason) {
            console.log('🔌 WebSocket closed:', code, reason.toString());
            if (connected) {
                console.log('✅ WebSocket functionality confirmed working');
                process.exit(0);
            } else {
                console.log('❌ WebSocket connection never established');
                process.exit(1);
            }
        });
        
        // Timeout
        setTimeout(() => {
            if (!connected) {
                console.error('⏰ WebSocket connection timeout');
                ws.close();
                process.exit(1);
            }
        }, 5000);
        
    } catch (err) {
        console.error('❌ Error creating WebSocket:', err.message);
        console.log('💡 Install ws module with: npm install ws');
        process.exit(1);
    }
}