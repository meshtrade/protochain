#!/usr/bin/env tsx

import { SystemProgramService, createClient, createConnectTransport, AccountService } from "@protochain/ts-web";
import { CreateRequest, TransferRequest } from "@protochain/ts-web";

const BACKEND_ENDPOINT = "http://localhost:50051";

async function testSystemService() {
  console.log("🔧 Testing SystemProgramService gRPC connectivity...");
  
  try {
    // Create gRPC transport and clients
    const transport = createConnectTransport({
      baseUrl: BACKEND_ENDPOINT,
    });
    
    const systemClient = createClient(SystemProgramService, transport);
    const accountClient = createClient(AccountService, transport);
    
    console.log("✅ gRPC clients created successfully");
    
    // Generate keypairs for testing
    console.log("\n🧪 Generating keypairs for testing...");
    const fromKeyPair = await accountClient.generateNewKeyPair({});
    const toKeyPair = await accountClient.generateNewKeyPair({});
    
    if (!fromKeyPair.keyPair?.publicKey || !toKeyPair.keyPair?.publicKey) {
      throw new Error("Failed to generate keypairs");
    }
    
    console.log(`✅ From account: ${fromKeyPair.keyPair.publicKey}`);
    console.log(`✅ To account: ${toKeyPair.keyPair.publicKey}`);
    
    // Test 1: Create (Create account instruction)
    console.log("\n🧪 Test 1: Create Account Instruction");
    const createResp = await systemClient.create({
      payer: fromKeyPair.keyPair.publicKey,
      newAccount: toKeyPair.keyPair.publicKey,
      lamports: BigInt(1000000), // 0.001 SOL
      space: BigInt(0), // Basic account
      owner: "11111111111111111111111111111111" // System program
    });
    
    console.log("✅ Create instruction successful:");
    console.log(`   Instruction Program ID: ${createResp.instruction?.programId}`);
    console.log(`   Accounts Length: ${createResp.instruction?.accounts?.length}`);
    console.log(`   Data Length: ${createResp.instruction?.data?.length}`);
    
    // Test 2: Transfer
    console.log("\n🧪 Test 2: Transfer Instruction");
    const transferResp = await systemClient.transfer({
      from: fromKeyPair.keyPair.publicKey,
      to: toKeyPair.keyPair.publicKey,
      lamports: BigInt(500000) // 0.0005 SOL
    });
    
    console.log("✅ Transfer instruction successful:");
    console.log(`   Instruction Program ID: ${transferResp.instruction?.programId}`);
    console.log(`   Accounts Length: ${transferResp.instruction?.accounts?.length}`);
    console.log(`   Data Length: ${transferResp.instruction?.data?.length}`);
    
    // Validate instruction structures
    console.log("\n✅ Instruction validation:");
    if (createResp) {
      console.log(`   Create - Program ID: ${!!createResp.instruction?.programId}`);
      console.log(`   Create - Accounts: ${createResp.instruction?.accounts?.length || 0}`);
      console.log(`   Create - Data: ${createResp.instruction?.data?.length || 0} bytes`);
    }

    if (transferResp) {
      console.log(`   Transfer - Program ID: ${!!transferResp.instruction?.programId}`);
      console.log(`   Transfer - Accounts: ${transferResp.instruction?.accounts?.length || 0}`);
      console.log(`   Transfer - Data: ${transferResp.instruction?.data?.length || 0} bytes`);
    }
    
    console.log("\n🎉 SystemProgramService tests completed successfully!");
    return true;
    
  } catch (error: any) {
    console.error("❌ SystemProgramService test failed:", error);
    console.error("   Stack:", error.stack);
    return false;
  }
}

// Run the test if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  testSystemService()
    .then(success => {
      console.log(success ? "\n✅ All tests passed!" : "\n❌ Tests failed!");
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error("❌ Unexpected error:", error);
      process.exit(1);
    });
}

export { testSystemService };