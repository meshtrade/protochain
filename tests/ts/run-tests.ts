#!/usr/bin/env tsx

import { testAccountService } from "./test-account.js";
import { testTokenService } from "./test-token.js";
import { testSystemService } from "./test-system.js";
import { testRPCService } from "./test-rpc.js";
import { testTransactionService } from "./test-transaction.js";

const BACKEND_ENDPOINT = "http://localhost:50051";

interface TestResult {
  name: string;
  success: boolean;
  duration: number;
  error?: Error;
}

async function checkBackendHealth(): Promise<boolean> {
  console.log("🔍 Checking backend health...");
  try {
    // Try a basic HTTP request to see if the server is reachable
    const response = await fetch(`${BACKEND_ENDPOINT}/`, {
      method: "GET",
    }).catch(() => null);
    
    if (response) {
      console.log(`✅ Backend is reachable at ${BACKEND_ENDPOINT}`);
      return true;
    } else {
      console.log(`❌ Backend is not reachable at ${BACKEND_ENDPOINT}`);
      console.log("   Make sure the Protochain backend is running:");
      console.log("   1. Start Solana validator: ./scripts/tests/start-validator.sh");
      console.log("   2. Start Protochain backend: cargo run --package protochain-solana-api");
      return false;
    }
  } catch (error) {
    console.log(`❌ Health check failed: ${error}`);
    return false;
  }
}

async function runTest(name: string, testFunction: () => Promise<boolean>): Promise<TestResult> {
  console.log(`\n${"=".repeat(60)}`);
  console.log(`🚀 Running ${name}...`);
  console.log(`${"=".repeat(60)}`);
  
  const startTime = Date.now();
  
  try {
    const success = await testFunction();
    const duration = Date.now() - startTime;
    return { name, success, duration };
  } catch (error: any) {
    const duration = Date.now() - startTime;
    return { name, success: false, duration, error };
  }
}

async function main() {
  console.log("🔧 Protochain gRPC Client Validation Test Suite");
  console.log("=".repeat(60));
  
  // Check backend health first
  // Note: Health check temporarily disabled - gRPC server doesn't support plain HTTP requests
  // All individual service tests are passing successfully
  // const healthOk = await checkBackendHealth();
  // if (!healthOk) {
  //   console.log("\n❌ Backend health check failed. Exiting.");
  //   process.exit(1);
  // }
  console.log("✅ Skipping HTTP health check (backend is gRPC-only)");
  console.log("   Running service tests directly...");
  
  // Define test suite
  const tests = [
    { name: "AccountService", testFunction: testAccountService },
    { name: "RPCClientService", testFunction: testRPCService },
    { name: "SystemProgramService", testFunction: testSystemService },
    { name: "TokenProgramService", testFunction: testTokenService },
    { name: "TransactionService", testFunction: testTransactionService },
  ];
  
  // Run all tests
  const results: TestResult[] = [];
  const startTime = Date.now();
  
  for (const test of tests) {
    const result = await runTest(test.name, test.testFunction);
    results.push(result);
    
    // Short pause between tests
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  const totalTime = Date.now() - startTime;
  
  // Print summary
  console.log("\n" + "=".repeat(60));
  console.log("📊 TEST RESULTS SUMMARY");
  console.log("=".repeat(60));
  
  let passed = 0;
  let failed = 0;
  
  results.forEach((result, index) => {
    const status = result.success ? "✅ PASS" : "❌ FAIL";
    const duration = `${result.duration}ms`;
    console.log(`${index + 1}. ${result.name.padEnd(25)} ${status.padEnd(8)} ${duration.padStart(8)}`);
    
    if (result.success) {
      passed++;
    } else {
      failed++;
      if (result.error) {
        console.log(`   Error: ${result.error.message}`);
      }
    }
  });
  
  console.log("-".repeat(60));
  console.log(`Total Tests: ${results.length}`);
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  console.log(`Total Time: ${totalTime}ms`);
  console.log("-".repeat(60));
  
  if (failed === 0) {
    console.log("🎉 All tests passed! gRPC connectivity is working correctly.");
    process.exit(0);
  } else {
    console.log(`❌ ${failed} test(s) failed. Check the error messages above.`);
    process.exit(1);
  }
}

// Handle unhandled errors
process.on('unhandledRejection', (error) => {
  console.error('❌ Unhandled promise rejection:', error);
  process.exit(1);
});

process.on('uncaughtException', (error) => {
  console.error('❌ Uncaught exception:', error);
  process.exit(1);
});

// Run main function
main().catch((error) => {
  console.error('❌ Main function failed:', error);
  process.exit(1);
});