/**
 * Configuration options for Protochain API clients using functional options pattern.
 *
 * **Server URL** (required):
 *    ```typescript
 *    const client = new AccountServiceWeb(
 *      WithServerUrl("http://localhost:50051")
 *    );
 *    ```
 *
 * **With logging enabled**:
 *    ```typescript
 *    const client = new AccountServiceWeb(
 *      WithServerUrl("http://localhost:50051"),
 *      WithLogging()
 *    );
 *    ```
 */

/**
 * Internal configuration class used to build client configuration.
 */
export class ClientConfig {
  /** API server URL (required - no default) */
  apiServerURL?: string;

  /** Whether to enable request/response logging */
  logging: boolean = false;

  /**
   * Validates the configuration.
   * @throws {Error} If the server URL is not provided
   */
  validate(): void {
    if (
      this.apiServerURL === undefined ||
      this.apiServerURL === null ||
      this.apiServerURL.trim() === ''
    ) {
      throw new Error(
        'Server URL is required. ' +
          'Please use WithServerUrl("http://localhost:50051") to configure the API endpoint.',
      );
    }
  }
}

/**
 * Client option function type for functional options pattern.
 * Each option function modifies the ClientConfig.
 */
export type ClientOption = (config: ClientConfig) => void;

/**
 * Configures the client with the API server URL.
 *
 * **Required**: This option must always be provided - there is no default URL.
 *
 * @param url - The API server URL (e.g., "http://localhost:50051")
 * @returns A client option function
 * @throws {Error} If the URL is empty
 *
 * @example
 * ```typescript
 * const client = new AccountServiceWeb(
 *   WithServerUrl("http://localhost:50051")
 * );
 * ```
 */
export function WithServerUrl(url: string): ClientOption {
  return (config: ClientConfig): void => {
    if (url === undefined || url === null || url.trim() === '') {
      throw new Error('Server URL cannot be empty');
    }
    config.apiServerURL = url;
  };
}

/**
 * Enables request/response logging via a logging interceptor.
 *
 * When enabled, all requests and responses will be logged to console.debug.
 * Errors will be logged to console.error.
 *
 * @returns A client option function
 *
 * @example
 * ```typescript
 * const client = new AccountServiceWeb(
 *   WithServerUrl("http://localhost:50051"),
 *   WithLogging()
 * );
 * ```
 */
export function WithLogging(): ClientOption {
  return (config: ClientConfig): void => {
    config.logging = true;
  };
}

/**
 * Builds client configuration from an array of option functions.
 *
 * @param opts - Variable number of option functions
 * @returns A validated ClientConfig instance
 * @throws {Error} If configuration is invalid (e.g., missing server URL)
 *
 * @example
 * ```typescript
 * const config = buildConfigFromOptions(
 *   WithServerUrl("http://localhost:50051"),
 *   WithLogging()
 * );
 * ```
 */
export function buildConfigFromOptions(...opts: ClientOption[]): ClientConfig {
  const config = new ClientConfig();

  // Apply each option
  for (const opt of opts) {
    opt(config);
  }

  // Validate the final configuration
  config.validate();

  return config;
}
