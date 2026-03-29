"use client";

import { ErrorBoundaryFallback } from "@/components/error-boundary";

export default function RootError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return <ErrorBoundaryFallback error={error} reset={reset} />;
}
