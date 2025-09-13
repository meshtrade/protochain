package generate

import "google.golang.org/protobuf/compiler/protogen"

const (
	// Go core packages
	ContextPkg = protogen.GoImportPath("context")
	FmtPkg     = protogen.GoImportPath("fmt")
	StringsPkg = protogen.GoImportPath("strings")
	IOPkg      = protogen.GoImportPath("io")

	// External packages
	TracingPkg = protogen.GoImportPath("go.opentelemetry.io/otel/trace")
	GRPCPkg    = protogen.GoImportPath("google.golang.org/grpc")

	// Protochain packages
	APIPkg = protogen.GoImportPath("github.com/BRBussy/protochain/lib/go/common")
)
