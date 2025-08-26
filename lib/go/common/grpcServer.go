package common

import (
	"google.golang.org/grpc"
)

type GRPCServer interface {
	grpc.ServiceRegistrar
	StartServer() error
	StopServer() error
}

type GRPCService interface {
	ServiceProviderName() string
	RegisterWithGRPCServer(s grpc.ServiceRegistrar)
}
