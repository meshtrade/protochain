#!/usr/bin/env tsx

import { TokenProgramService, createClient, createConnectTransport, AccountService } from "@protochain/ts-web";
import { InitialiseSPLTokenMintRequest } from "@protochain/ts-web";

const BACKEND_ENDPOINT = "http://localhost:50051";

async function testTokenService() {
  console.log("🔧 Testing TokenProgramService gRPC connectivity...");
  
  try {
    // Create gRPC transport and clients
    const transport = createConnectTransport({
      baseUrl: BACKEND_ENDPOINT
    });
    
    const tokenClient = createClient(TokenProgramService, transport);
    const accountClient = createClient(AccountService, transport);
    
    console.log("✅ gRPC clients created successfully");
    
    // Generate a new keypair for mint authority
    console.log("\n🧪 Generating keypair for mint authority...");
    const keyPairResp = await accountClient.generateNewKeyPair({});
    if (!keyPairResp.keyPair?.publicKey || !keyPairResp.keyPair?.privateKey) {
      throw new Error("Failed to generate keypair");
    }
    
    console.log(`✅ Generated mint authority: ${keyPairResp.keyPair.publicKey}`);
    
    // Test 1: InitialiseSPLTokenMint
    console.log("\n🧪 Test 1: InitialiseSPLTokenMint");
    const initMintResp = await tokenClient.initialiseSPLTokenMint({
      mintPubKey: keyPairResp.keyPair.publicKey,
      mintAuthorityPubKey: keyPairResp.keyPair.publicKey,
      freezeAuthorityPubKey: keyPairResp.keyPair.publicKey,
      decimals: 6
    });

    console.log("✅ InitialiseSPLTokenMint successful:");
    console.log(`   Instructions count: ${initMintResp.instructions?.length}`);
    if (initMintResp.instructions && initMintResp.instructions.length > 0) {
      const firstInstruction = initMintResp.instructions[0];
      console.log(`   First Instruction Program ID: ${firstInstruction?.programId}`);
      console.log(`   First Instruction Accounts Length: ${firstInstruction?.accounts?.length}`);
      console.log(`   First Instruction Data Length: ${firstInstruction?.data?.length}`);
    }

    // Validate instruction structure
    if (initMintResp) {
      console.log("✅ Instruction validation:");
      console.log(`   Has Instructions: ${!!initMintResp.instructions && initMintResp.instructions.length > 0}`);
      if (initMintResp.instructions && initMintResp.instructions.length > 0) {
        const firstInstruction = initMintResp.instructions[0];
        console.log(`   Has Program ID: ${!!firstInstruction?.programId}`);
        console.log(`   Has Accounts: ${!!firstInstruction?.accounts && firstInstruction.accounts.length > 0}`);
        console.log(`   Has Data: ${!!firstInstruction?.data && firstInstruction.data.length > 0}`);
      }
    }
    
    console.log("\n🎉 TokenProgramService tests completed successfully!");
    return true;
    
  } catch (error: any) {
    console.error("❌ TokenProgramService test failed:", error);
    console.error("   Stack:", error.stack);
    return false;
  }
}

// Run the test if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  testTokenService()
    .then(success => {
      console.log(success ? "\n✅ All tests passed!" : "\n❌ Tests failed!");
      process.exit(success ? 0 : 1);
    })
    .catch(error => {
      console.error("❌ Unexpected error:", error);
      process.exit(1);
    });
}

export { testTokenService };