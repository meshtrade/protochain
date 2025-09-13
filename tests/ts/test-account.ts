#!/usr/bin/env tsx

import { AccountService, createClient, createGrpcTransport } from "@protochain/api";
import { GenerateNewKeyPairRequest, GetAccountRequest } from "@protochain/api";

const BACKEND_ENDPOINT = "http://localhost:50051";

async function testAccountService() {
  console.log("🔧 Testing AccountService gRPC connectivity...");
  
  try {
    // Create gRPC transport and client
    const transport = createGrpcTransport({
      baseUrl: BACKEND_ENDPOINT
    });
    
    const client = createClient(AccountService, transport);
    
    console.log("✅ gRPC client created successfully");
    
    // Test 1: GenerateNewKeyPair
    console.log("\n🧪 Test 1: GenerateNewKeyPair");
    const keyPairResp = await client.generateNewKeyPair({});
    console.log("✅ GenerateNewKeyPair successful:");
    console.log(`   PublicKey: ${keyPairResp.keyPair?.publicKey}`);
    console.log(`   PrivateKey: ${keyPairResp.keyPair?.privateKey?.substring(0, 10)}...`);
    
    // Test 2: GetAccount (for a known account or newly generated one)
    console.log("\n🧪 Test 2: GetAccount");
    if (keyPairResp.keyPair?.publicKey) {
      try {
        const accountResp = await client.getAccount({
          address: keyPairResp.keyPair.publicKey,
          commitmentLevel: 3 // FINALIZED
        });
        console.log("✅ GetAccount successful:");
        console.log(`   Owner: ${accountResp.owner}`);
        console.log(`   Lamports: ${accountResp.lamports}`);
      } catch (error: any) {
        // Expected for new accounts that don't exist yet
        console.log("ℹ️  GetAccount returned error (expected for new account):", error.message);
      }
    }
    
    console.log("\n🎉 AccountService tests completed successfully!");
    return true;
    
  } catch (error: any) {
    console.error("❌ AccountService test failed:", error);
    console.error("   Stack:", error.stack);
    return false;
  }
}

// Run the test if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  testAccountService()
    .then(success => {
      console.log(success ? "\n✅ All tests passed!" : "\n❌ Tests failed!");
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error("❌ Unexpected error:", error);
      process.exit(1);
    });
}

export { testAccountService };