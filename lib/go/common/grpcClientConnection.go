package common

import (
	"fmt"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

// NewGRPCClientConnection constructs a new gRPC client connection configured with the given parameters.
func NewGRPCClientConnection(
	url string,
	port int,
	tls bool,
	unaryClientInterceptors []grpc.UnaryClientInterceptor,
) (*grpc.ClientConn, error) {
	// prepare dial options
	dialOpts := make([]grpc.DialOption, 0)

	// set transport credentials
	if tls {
		dialOpts = append(dialOpts, grpc.WithTransportCredentials(credentials.NewClientTLSFromCert(nil, "")))
	} else {
		dialOpts = append(dialOpts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	// add given unary client interceptors
	dialOpts = append(dialOpts, grpc.WithChainUnaryInterceptor(unaryClientInterceptors...))

	// construct and return gRPC client connection
	return grpc.NewClient(
		fmt.Sprintf("%s:%d", url, port),
		dialOpts...,
	)
}
