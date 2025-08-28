package main

import (
	"fmt"

	"google.golang.org/protobuf/compiler/protogen"

	"github.com/BRBussy/protosol/tool/protoc-gen/cmd/protosolgo/pkg/generate"
)

func main() {
	protogen.Options{}.Run(func(p *protogen.Plugin) error {
		return Generate(p)
	})
}

func Generate(p *protogen.Plugin) error {
	for _, f := range p.Files {
		// confirm that file is not to be skipped
		if !f.Generate {
			continue
		}

		// if the file contains services then perform service related code generation
		if len(f.Services) != 0 {
			// confirm that file contains no more than 1 service
			if len(f.Services) > 1 {
				return fmt.Errorf("file '%s' contains more than 1 service", f.Desc.Path())
			}

			// get the 1 service in the file
			svc := f.Services[0]

			// generate the interface file
			if err := generate.ServiceInterface(p, f, svc); err != nil {
				return fmt.Errorf("error generating service interface: %w", err)
			}

			// generate the gRPC adaptor
			if err := generate.GRPCAdaptor(p, f, svc); err != nil {
				return fmt.Errorf("error generating gRPC adaptor: %w", err)
			}

			// generate the service client
			if err := generate.Service(p, f, svc); err != nil {
				return fmt.Errorf("error generating service: %w", err)
			}
		}
	}

	return nil
}
