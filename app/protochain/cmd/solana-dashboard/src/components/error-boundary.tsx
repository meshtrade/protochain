"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { AlertCircle } from "lucide-react";

interface ErrorBoundaryProps {
  error: Error & { digest?: string };
  reset: () => void;
}

export function ErrorBoundaryFallback({ error, reset }: ErrorBoundaryProps) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 py-20">
      <Alert variant="destructive" className="max-w-lg">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription className="font-mono text-sm break-all">
          {error.message}
        </AlertDescription>
      </Alert>
      <Button onClick={reset} variant="outline">
        Try Again
      </Button>
    </div>
  );
}
