/**
 * Protochain gRPC Client Infrastructure
 * Provides centralized client management for all Protochain services
 */

import { createClient, Client } from "@connectrpc/connect";
import { createGrpcTransport } from "@connectrpc/connect-node";
import {
  AccountService,
  TransactionService,
  SystemProgramService,
  TokenProgramService,
  RPCClientService,
} from "@protochain/api";

// =============================================================================
// CONFIGURATION
// =============================================================================

interface GrpcConfig {
  baseUrl: string;
  timeout?: number;
  keepAlive?: boolean;
}

// Default configuration - can be overridden via environment variables
const defaultConfig: GrpcConfig = {
  baseUrl: process.env.PROTOSOL_GRPC_URL || "http://localhost:50051",
  timeout: 30000, // 30 seconds
  keepAlive: true,
};

// =============================================================================
// TRANSPORT FACTORY
// =============================================================================

let transportInstance: ReturnType<typeof createGrpcTransport> | null = null;

/**
 * Creates or returns existing gRPC transport instance
 * Implements singleton pattern to reuse connections
 */
function getGrpcTransport() {
  if (!transportInstance) {
    transportInstance = createGrpcTransport({
      baseUrl: defaultConfig.baseUrl,
      interceptors: [
        // Add request timeout
        (next) => (req) => {
          const timeoutSignal = AbortSignal.timeout(defaultConfig.timeout!);
          const combinedSignal = req.signal
            ? anySignal([req.signal, timeoutSignal])
            : timeoutSignal;
            
          return next({
            ...req,
            signal: combinedSignal,
          });
        },
        // Add error logging
        (next) => async (req) => {
          try {
            const response = await next(req);
            return response;
          } catch (error) {
            console.error(`gRPC Error for ${req.service.typeName}/${req.method.name}:`, {
              error: error instanceof Error ? error.message : String(error),
              service: req.service.typeName,
              method: req.method.name,
              url: defaultConfig.baseUrl,
            });
            throw error;
          }
        }
      ],
    });
  }
  return transportInstance;
}

// =============================================================================
// CLIENT TYPES
// =============================================================================

export interface GrpcClients {
  account: Client<typeof AccountService>;
  transaction: Client<typeof TransactionService>;
  rpcClient: Client<typeof RPCClientService>;
  program: {
    system: Client<typeof SystemProgramService>;
    token: Client<typeof TokenProgramService>;
  };
}

// =============================================================================
// CLIENT FACTORY
// =============================================================================

let clientsInstance: GrpcClients | null = null;

/**
 * Creates or returns existing gRPC client instances
 * All clients share the same transport connection
 */
export function getGrpcClients(): GrpcClients {
  if (!clientsInstance) {
    const transport = getGrpcTransport();
    
    clientsInstance = {
      account: createClient(AccountService, transport),
      transaction: createClient(TransactionService, transport),
      rpcClient: createClient(RPCClientService, transport),
      program: {
        system: createClient(SystemProgramService, transport),
        token: createClient(TokenProgramService, transport),
      },
    };
  }
  
  return clientsInstance;
}

// =============================================================================
// HEALTH CHECK UTILITIES
// =============================================================================

/**
 * Tests connectivity to the Protochain backend
 * Attempts a simple RPC call to verify the connection
 */
export async function testGrpcConnection(): Promise<{
  success: boolean;
  error?: string;
  url: string;
}> {
  try {
    const clients = getGrpcClients();
    
    // Try a simple call to test connectivity
    // Using GetMinimumBalanceForRentExemption as it's a lightweight call
    await clients.rpcClient.getMinimumBalanceForRentExemption({
      dataLength: BigInt(0),
    });
    
    return {
      success: true,
      url: defaultConfig.baseUrl,
    };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
      url: defaultConfig.baseUrl,
    };
  }
}

/**
 * Gets the current gRPC configuration
 */
export function getGrpcConfig(): GrpcConfig {
  return { ...defaultConfig };
}

/**
 * Updates the gRPC configuration and recreates clients
 * Useful for switching endpoints during development/testing
 */
export function updateGrpcConfig(newConfig: Partial<GrpcConfig>): void {
  Object.assign(defaultConfig, newConfig);
  
  // Clear existing instances to force recreation with new config
  transportInstance = null;
  clientsInstance = null;
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/**
 * Combines multiple AbortSignals into one
 * Used for request timeout implementation
 */
function anySignal(signals: AbortSignal[]): AbortSignal {
  const controller = new AbortController();
  
  for (const signal of signals) {
    if (signal.aborted) {
      controller.abort();
      break;
    }
    signal.addEventListener('abort', () => controller.abort(), { once: true });
  }
  
  return controller.signal;
}

// =============================================================================
// EXPORTS FOR SERVER-SIDE FUNCTIONS
// =============================================================================

// Export the main client getter for use in server actions
export { getGrpcClients as grpcClients };

// Export individual service getters for convenience
export const accountClient = () => getGrpcClients().account;
export const transactionClient = () => getGrpcClients().transaction;
export const rpcClient = () => getGrpcClients().rpcClient;
export const systemProgramClient = () => getGrpcClients().program.system;
export const tokenProgramClient = () => getGrpcClients().program.token;

// Export types for TypeScript consumers
export type { GrpcConfig };