/**
 * Connect-ES interceptors for the Protochain API client.
 *
 * Provides interceptor utilities for use with @connectrpc/connect clients,
 * including request/response logging for debugging and development.
 */

/* eslint-disable no-console -- Logging interceptor requires console output by design */

import { Interceptor } from '@connectrpc/connect';

/**
 * Creates a logging interceptor that logs all requests and responses.
 * Useful for debugging and development.
 *
 * @returns An interceptor function that logs request/response details
 *
 * @example
 * ```typescript
 * const loggingInterceptor = createLoggingInterceptor();
 *
 * const transport = createConnectTransport({
 *   baseUrl: 'http://localhost:50051',
 *   interceptors: [loggingInterceptor]
 * });
 * ```
 */
export function createLoggingInterceptor(): Interceptor {
  return (next) => async (req) => {
    // Convert headers to plain object for logging
    const headers: Record<string, string> = {};
    req.header.forEach((value, key) => {
      headers[key] = value;
    });

    // Log the request
    console.debug(`[Connect] ${req.method.name} request:`, {
      service: req.service.typeName,
      method: req.method.name,
      headers,
    });

    try {
      // Call the next interceptor and get the response
      const response = await next(req);

      // Log successful response
      console.debug(`[Connect] ${req.method.name} response:`, {
        service: req.service.typeName,
        method: req.method.name,
        status: 'success',
      });

      return response;
    } catch (error) {
      // Log error
      console.error(`[Connect] ${req.method.name} error:`, {
        service: req.service.typeName,
        method: req.method.name,
        error: error,
      });

      // Re-throw the error
      throw error;
    }
  };
}
