#!/usr/bin/env python3

import asyncio
import websockets
import json
import sys

async def test_websocket():
    try:
        print("Testing WebSocket connection to ws://localhost:8900...")
        
        async with websockets.connect("ws://localhost:8900") as websocket:
            print("✅ WebSocket connection opened successfully!")
            
            # Test a simple subscription request
            subscribe_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "signatureSubscribe",
                "params": [
                    "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYlCJjBRCN8FHXvVSs8h7oprNJfj6gJV26pEgJZNMAUh2tCgKHU9Sy"
                ]
            }
            
            print("Sending subscription request...")
            await websocket.send(json.dumps(subscribe_request))
            
            # Wait for response with timeout
            try:
                response = await asyncio.wait_for(websocket.recv(), timeout=3.0)
                print(f"📨 Received: {response}")
            except asyncio.TimeoutError:
                print("⏰ No response received within 3 seconds")
            
    except Exception as e:
        print(f"❌ WebSocket error: {e}")
        return False
    
    return True

if __name__ == "__main__":
    result = asyncio.run(test_websocket())
    sys.exit(0 if result else 1)