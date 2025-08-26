package generate

import "google.golang.org/protobuf/compiler/protogen"

const (
	// Go core packages
	ContextPkg = protogen.GoImportPath("context")
	FmtPkg     = protogen.GoImportPath("fmt")
	StringsPkg = protogen.GoImportPath("strings")

	// External packages
	TracingPkg = protogen.GoImportPath("go.opentelemetry.io/otel/trace")
	GRPCPkg    = protogen.GoImportPath("google.golang.org/grpc")

	// Protosol packages
	APIPkg = protogen.GoImportPath("github.com/BRBussy/protosol/lib/go/common")
)
