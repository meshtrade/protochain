#!/bin/bash

# Convenience wrapper for linting scripts

case "$1" in
    "fix" | "--fix")
        ./scripts/lint/fix.sh
        ;;
    "check" | "--check" | "")
        ./scripts/lint/all.sh
        ;;
    "help" | "--help" | "-h")
        echo "Usage: ./lint.sh [command]"
        echo ""
        echo "Commands:"
        echo "  check (default)  Run all linting checks"
        echo "  fix              Auto-fix linting issues where possible"
        echo "  help             Show this help message"
        echo ""
        echo "Examples:"
        echo "  ./lint.sh        # Run all linting checks"
        echo "  ./lint.sh check  # Run all linting checks"
        echo "  ./lint.sh fix    # Auto-fix linting issues"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Use './lint.sh help' for usage information"
        exit 1
        ;;
esac