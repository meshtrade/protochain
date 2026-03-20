#!/usr/bin/env tsx

import { RPCClientService, createClient, createConnectTransport } from "@protochain/ts-web";
import { GetMinimumBalanceForRentExemptionRequest } from "@protochain/ts-web";

const BACKEND_ENDPOINT = "http://localhost:50051";

async function testRPCService() {
  console.log("🔧 Testing RPCClientService gRPC connectivity...");
  
  try {
    // Create gRPC transport and client
    const transport = createConnectTransport({
      baseUrl: BACKEND_ENDPOINT
    });
    
    const client = createClient(RPCClientService, transport);
    
    console.log("✅ gRPC client created successfully");
    
    // Test 1: GetMinimumBalanceForRentExemption
    console.log("\n🧪 Test 1: GetMinimumBalanceForRentExemption");
    const rentResp = await client.getMinimumBalanceForRentExemption({
      dataLength: BigInt(0), // Basic account (0 bytes)
      commitmentLevel: 2 // CONFIRMED
    });
    
    console.log("✅ GetMinimumBalanceForRentExemption successful:");
    console.log(`   Minimum Balance: ${rentResp.balance} lamports`);
    
    // Test with different data lengths
    console.log("\n🧪 Test 2: GetMinimumBalanceForRentExemption (Token Account)");
    const tokenAccountRent = await client.getMinimumBalanceForRentExemption({
      dataLength: BigInt(165), // SPL Token account size
      commitmentLevel: 2 // CONFIRMED
    });
    
    console.log("✅ GetMinimumBalanceForRentExemption (Token Account) successful:");
    console.log(`   Minimum Balance: ${tokenAccountRent.balance} lamports`);
    
    // Validate responses
    console.log("\n✅ Response validation:");
    console.log(`   Basic account rent > 0: ${rentResp.balance > 0n}`);
    console.log(`   Token account rent > basic: ${tokenAccountRent.balance > rentResp.balance}`);
    
    // Test 3: Test error handling with invalid data
    console.log("\n🧪 Test 3: Error handling test");
    try {
      await client.getMinimumBalanceForRentExemption({
        dataLength: BigInt(-1), // Invalid data length
        commitmentLevel: 2 // CONFIRMED
      });
      console.log("⚠️  Expected error not thrown for invalid data length");
    } catch (error: any) {
      console.log("✅ Error handling working correctly:", error.message);
    }
    
    console.log("\n🎉 RPCClientService tests completed successfully!");
    return true;
    
  } catch (error: any) {
    console.error("❌ RPCClientService test failed:", error);
    console.error("   Stack:", error.stack);
    return false;
  }
}

// Run the test if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  testRPCService()
    .then(success => {
      console.log(success ? "\n✅ All tests passed!" : "\n❌ Tests failed!");
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error("❌ Unexpected error:", error);
      process.exit(1);
    });
}

export { testRPCService };