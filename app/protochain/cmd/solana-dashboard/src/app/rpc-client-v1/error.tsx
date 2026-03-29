"use client";

import { ErrorBoundaryFallback } from "@/components/error-boundary";

export default function ServiceError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return <ErrorBoundaryFallback error={error} reset={reset} />;
}
