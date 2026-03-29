"use client";

import type { ReactNode, FormEvent } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Loader2, Play } from "lucide-react";

interface RequestFormProps {
  onSubmit: () => void;
  loading: boolean;
  children: ReactNode;
  title?: string;
}

export function RequestForm({
  onSubmit,
  loading,
  children,
  title = "Request",
}: RequestFormProps) {
  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    onSubmit();
  }

  return (
    <Card>
      <CardHeader className="pb-4">
        <CardTitle className="text-base">{title}</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {children}
          <Button type="submit" disabled={loading} className="w-full">
            {loading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Play className="mr-2 h-4 w-4" />
            )}
            {loading ? "Sending..." : "Go"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
