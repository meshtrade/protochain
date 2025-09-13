// Package main implements the template-some-executable application.
//
// This serves as a template app to be built out in future as required.
// App naming convention: folder path determines app name (template-some-executable).
package main

import (
	"fmt"
	"log"
	"os"
)

func main() {
	fmt.Println("🚀 Template Executable - some-executable")
	fmt.Println("📦 App name: template-some-executable (based on folder path names)")
	fmt.Println("📁 Location: ./app/template/cmd/some-executable")
	fmt.Println()

	log.Println("This is a template app to be built out in future as required")

	// Example of app structure awareness
	if len(os.Args) > 1 {
		fmt.Printf("Arguments: %v\n", os.Args[1:])
	}

	fmt.Println("✅ Template app running successfully")
}