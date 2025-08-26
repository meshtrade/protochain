package common

import (
	"context"
	"fmt"
	"time"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/trace"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

// GRPCClient defines the common interface that all generated gRPC service clients implement.
// This provides consistent resource management and connection lifecycle.
type GRPCClient interface {
	// Close closes the underlying gRPC connection and cleans up resources
	Close() error
	// Health returns the connection health status
	Health() HealthStatus
}

// HealthStatus represents the health state of a gRPC connection
type HealthStatus string

const (
	HealthStatusHealthy   HealthStatus = "HEALTHY"
	HealthStatusUnhealthy HealthStatus = "UNHEALTHY"
	HealthStatusUnknown   HealthStatus = "UNKNOWN"
)

// Executor provides the execution context for RPC calls with validation,
// tracing, timeout handling, and authentication.
type Executor struct {
	ServiceName string
	Tracer      trace.Tracer
	Timeout     time.Duration
	// Future: Add validation, authentication, etc.
}

// BaseGRPCClient provides common gRPC functionality for all generated service clients.
// It uses generics to maintain type safety while providing shared infrastructure.
type BaseGRPCClient[T any] struct {
	serviceName string
	conn        *grpc.ClientConn
	grpcClient  T
	executor    *Executor
}

// NewBaseGRPCClient creates a new BaseGRPCClient instance with the provided configuration.
// The clientFactory function creates the typed gRPC client from the connection.
func NewBaseGRPCClient[T any](
	serviceName string,
	clientFactory func(grpc.ClientConnInterface) T,
	opts ...ServiceOption,
) (*BaseGRPCClient[T], error) {
	// Apply default configuration
	config := &ServiceConfig{
		URL:     "localhost:9090",
		TLS:     false,
		Timeout: 30 * time.Second,
	}

	// Apply user options
	for _, opt := range opts {
		opt(config)
	}

	// Create gRPC connection
	conn, err := createConnection(config)
	if err != nil {
		return nil, fmt.Errorf("failed to create gRPC connection: %w", err)
	}

	// Create typed gRPC client
	grpcClient := clientFactory(conn)

	// Create tracer
	tracer := otel.Tracer(serviceName)

	// Create executor with configured settings
	executor := &Executor{
		ServiceName: serviceName,
		Tracer:      tracer,
		Timeout:     config.Timeout,
	}

	return &BaseGRPCClient[T]{
		serviceName: serviceName,
		conn:        conn,
		grpcClient:  grpcClient,
		executor:    executor,
	}, nil
}

// GrpcClient returns the typed gRPC client for making RPC calls
func (c *BaseGRPCClient[T]) GrpcClient() T {
	return c.grpcClient
}

// Executor returns the execution context for RPC calls
func (c *BaseGRPCClient[T]) Executor() *Executor {
	return c.executor
}

// Close closes the underlying gRPC connection
func (c *BaseGRPCClient[T]) Close() error {
	if c.conn != nil {
		return c.conn.Close()
	}
	return nil
}

// Health returns the current health status of the connection
func (c *BaseGRPCClient[T]) Health() HealthStatus {
	if c.conn == nil {
		return HealthStatusUnknown
	}

	state := c.conn.GetState()
	switch state.String() {
	case "READY":
		return HealthStatusHealthy
	case "IDLE", "CONNECTING":
		return HealthStatusHealthy // These are acceptable states
	default:
		return HealthStatusUnhealthy
	}
}

// Execute provides consistent execution of RPC calls with tracing, timeout handling,
// validation, and error handling. This ensures all RPC calls have the same behavior.
func Execute[Req, Resp any](
	executor *Executor,
	ctx context.Context,
	methodName string,
	request Req,
	rpcCall func(context.Context) (Resp, error),
) (Resp, error) {
	var zero Resp

	// Apply timeout if specified
	if executor.Timeout > 0 {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, executor.Timeout)
		defer cancel()
	}

	// Start distributed tracing span
	ctx, span := executor.Tracer.Start(ctx, executor.ServiceName+"."+methodName)
	defer span.End()

	// Future: Add request validation here
	// Future: Add authentication here

	// Execute the RPC call
	response, err := rpcCall(ctx)
	if err != nil {
		return zero, fmt.Errorf("%s failed: %w", methodName, err)
	}

	// Future: Add response validation here

	return response, nil
}

// createConnection creates a gRPC connection based on the configuration
func createConnection(config *ServiceConfig) (*grpc.ClientConn, error) {
	var dialOpts []grpc.DialOption

	// Configure transport credentials
	if config.TLS {
		dialOpts = append(dialOpts, grpc.WithTransportCredentials(credentials.NewClientTLSFromCert(nil, "")))
	} else {
		dialOpts = append(dialOpts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	// Add any custom interceptors
	if len(config.UnaryInterceptors) > 0 {
		dialOpts = append(dialOpts, grpc.WithChainUnaryInterceptor(config.UnaryInterceptors...))
	}

	// Add default call options
	dialOpts = append(dialOpts, grpc.WithDefaultCallOptions())

	// Create and return connection
	return grpc.NewClient(config.URL, dialOpts...)
}
