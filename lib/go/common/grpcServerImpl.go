package common

import (
	"context"
	"fmt"
	"net"
	"strings"

	"runtime/debug"

	"github.com/rs/zerolog/log"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/reflection"
	"google.golang.org/grpc/status"
)

var _ GRPCServer = &GRPCServerImpl{}

type GRPCServerImpl struct {
	*grpc.Server
	port int
}

type ServiceInterceptorCombo struct {
	Interceptors []grpc.UnaryServerInterceptor
	Services     []GRPCService
}

func NewGRPCServerImpl(
	port int,
	enableGRPCReflection bool,
	serviceInterceptorCombos []ServiceInterceptorCombo,
) (*GRPCServerImpl, error) {
	// Prepare list of default unary call interceptors (i.e. middleware).
	// These will be applied to every incoming gRPC call.
	interceptors := []grpc.UnaryServerInterceptor{
		// add logger into incoming request context so that we can do log.Ctx(ctx)...
		func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
			subLogger := log.Logger
			ctx = subLogger.WithContext(ctx)
			return handler(ctx, req)
		},

		// add a unary method interceptor so that the gRPC server can recover from panics
		func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (_ interface{}, err error) {
			panicked := true

			defer func() {
				if r := recover(); r != nil || panicked {
					// try and recover the error
					panicErr, ok := r.(error)
					if !ok {
						panicErr = fmt.Errorf("unknown error: %+v", r)
					}

					// log out error with all available information
					log.Ctx(ctx).
						Error().
						Err(panicErr).
						Str("stack", string(debug.Stack())).
						Msgf("panic occurred in gRPC method '%s'", info.FullMethod)

					// set returned error to opaque response
					err = status.Errorf(codes.Internal, "unexpected error in method %s", info.FullMethod)
				}
			}()

			resp, err := handler(ctx, req)
			panicked = false
			return resp, err
		},
	}

	// prepare a list of all service providers
	allServiceProviders := make([]GRPCService, 0)

	// prepare index of the names of all service providers for checking that service providers are not duplicated
	allServiceProvidersIdx := make(map[string]bool)

	// process each given service interceptor combos to only apply interceptors for associated services
	for _, serviceInterceptorCombo := range serviceInterceptorCombos {
		// prepare index of services associaated with this list of interceptors
		serviceProviderIdx := make(map[string]bool)

		// go through given services associated with this list of interceptors and:
		// - ensure that duplicate services are not given
		// - populate the index of associated services
		// - update the list of all service providers
		for _, serviceProvider := range serviceInterceptorCombo.Services {
			// confirm that service provider with this name not already added to "all service providers"
			if _, alreadyAdded := allServiceProvidersIdx[serviceProvider.ServiceProviderName()]; alreadyAdded {
				return nil, fmt.Errorf("service provider '%s' provided more than once", serviceProvider.ServiceProviderName())
			}
			allServiceProvidersIdx[serviceProvider.ServiceProviderName()] = true

			// add to list of all service providers
			allServiceProviders = append(allServiceProviders, serviceProvider)

			// update the index of service providers for this list of interceptors
			serviceProviderIdx[strings.ReplaceAll(serviceProvider.ServiceProviderName(), "-", ".")] = true
		}

		// wrap each given interceptor with another interceptor that by-passes or applies the given interceptor
		// depending on whether the particular service being invoked is on a service provider associated with
		// the interceptor as indexed above
		for _, interceptor := range serviceInterceptorCombo.Interceptors {
			interceptor := interceptor
			interceptors = append(
				interceptors,
				func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (_ interface{}, err error) {
					// extract service provider name from the FullMethod name on the given info
					fullMethodNameParts := strings.Split(info.FullMethod, "/")
					if len(fullMethodNameParts) != 3 {
						return nil, status.Errorf(codes.Internal, "invalid full method format '%s'", info.FullMethod)
					}
					serviceProviderName := fullMethodNameParts[1]

					// if the service being executed belongs to an associated service provider then apply the given interceptor
					if _, found := serviceProviderIdx[serviceProviderName]; found {
						return interceptor(ctx, req, info, handler)
					}

					// otherwise bypass it
					return handler(ctx, req)
				},
			)
		}
	}

	// construct server with the given interceptors
	server := grpc.NewServer(
		grpc.ChainUnaryInterceptor(interceptors...),
	)

	// enable grpc reflection if requested
	if enableGRPCReflection {
		reflection.Register(server)
	}

	// register all service providers with the server
	for _, serviceProvider := range allServiceProviders {
		serviceProvider.RegisterWithGRPCServer(server)
	}

	// construct and return
	return &GRPCServerImpl{
		Server: server,
		port:   port,
	}, nil
}

// StartServer implements GRPCServer.
func (g *GRPCServerImpl) StartServer() error {
	log.Debug().Msgf("starting gRPC server on port %d", g.port)

	// prepare connection on the given server port
	lis, err := net.Listen("tcp", fmt.Sprintf("[::]:%d", g.port))
	if err != nil {
		return fmt.Errorf("error listening on port %d: %v", g.port, err)
	}

	// start the grpc server listening on the port
	return g.Server.Serve(lis)
}

// StopServer implements GRPCServer.
func (g *GRPCServerImpl) StopServer() error {
	log.Debug().Msg("stopping gRPC server")

	g.Server.GracefulStop()

	return nil
}
