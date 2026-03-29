"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { AlertCircle } from "lucide-react";
import { CopyButton } from "./copy-button";

interface ResponsePanelProps {
  data: unknown;
  error: string | null;
  loading: boolean;
}

function isCopiableValue(value: unknown): value is string {
  if (typeof value !== "string") return false;
  if (value.length < 20) return false;
  // Base58 addresses, signatures, hashes
  if (/^[1-9A-HJ-NP-Za-km-z]{32,128}$/.test(value)) return true;
  // Hex strings
  if (/^[0-9a-fA-F]{32,}$/.test(value)) return true;
  return false;
}

function isLamportField(key: string): boolean {
  return /lamport|fee|balance|amount/i.test(key);
}

function formatLamports(value: string | number | bigint): string {
  const num = typeof value === "bigint" ? value : BigInt(String(value));
  const sol = Number(num) / 1_000_000_000;
  return `${String(num)} lamports (${sol.toFixed(9)} SOL)`;
}

function JsonValue({
  keyName,
  value,
  depth,
}: {
  keyName?: string;
  value: unknown;
  depth: number;
}) {
  if (value === null || value === undefined) {
    return <span className="text-muted-foreground">null</span>;
  }

  if (typeof value === "boolean") {
    return (
      <Badge variant={value ? "default" : "secondary"}>
        {String(value)}
      </Badge>
    );
  }

  if (typeof value === "number" || typeof value === "bigint") {
    const str = String(value);
    if (keyName && isLamportField(keyName)) {
      return (
        <span className="font-mono text-sm text-blue-600 dark:text-blue-400">
          {formatLamports(value)}
        </span>
      );
    }
    return <span className="font-mono text-sm text-blue-600 dark:text-blue-400">{str}</span>;
  }

  if (typeof value === "string") {
    if (keyName && isLamportField(keyName) && /^\d+$/.test(value)) {
      return (
        <div className="flex items-center gap-1">
          <span className="font-mono text-sm text-blue-600 dark:text-blue-400 break-all">
            {formatLamports(value)}
          </span>
          <CopyButton value={value} />
        </div>
      );
    }
    return (
      <div className="flex items-center gap-1">
        <span className="font-mono text-sm break-all">{value}</span>
        {isCopiableValue(value) && <CopyButton value={value} />}
      </div>
    );
  }

  if (Array.isArray(value)) {
    if (value.length === 0) {
      return <span className="text-muted-foreground text-sm">[]</span>;
    }
    return (
      <div className="ml-4 border-l border-border pl-4 space-y-1">
        {value.map((item, i) => (
          <div key={i} className="flex items-start gap-2">
            <span className="text-muted-foreground text-xs mt-1 shrink-0">[{i}]</span>
            <JsonValue value={item} depth={depth + 1} />
          </div>
        ))}
      </div>
    );
  }

  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) {
      return <span className="text-muted-foreground text-sm">{"{}"}</span>;
    }
    return (
      <div className={depth > 0 ? "ml-4 border-l border-border pl-4 space-y-2" : "space-y-2"}>
        {entries.map(([k, v]) => (
          <div key={k}>
            <span className="text-xs font-medium text-muted-foreground">{k}</span>
            <div className="mt-0.5">
              <JsonValue keyName={k} value={v} depth={depth + 1} />
            </div>
          </div>
        ))}
      </div>
    );
  }

  return <span className="font-mono text-sm">{String(value)}</span>;
}

function serializeResponse(data: unknown): unknown {
  if (data === null || data === undefined) return data;
  if (typeof data === "bigint") return String(data);
  if (typeof data !== "object") return data;
  if (Array.isArray(data)) return data.map(serializeResponse);
  const result: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(data as Record<string, unknown>)) {
    // Skip $typeName and $unknown internal protobuf fields
    if (k === "$typeName" || k === "$unknown") continue;
    result[k] = serializeResponse(v);
  }
  return result;
}

export function ResponsePanel({ data, error, loading }: ResponsePanelProps) {
  if (loading) {
    return (
      <Card>
        <CardHeader className="pb-4">
          <CardTitle className="text-base">Response</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-3/4" />
          <Skeleton className="h-4 w-1/2" />
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="text-base">Response</CardTitle>
            <Badge variant="destructive">Error</Badge>
          </div>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription className="font-mono text-sm break-all">
              {error}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  if (data === null || data === undefined) {
    return (
      <Card>
        <CardHeader className="pb-4">
          <CardTitle className="text-base">Response</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            Response will appear here
          </p>
        </CardContent>
      </Card>
    );
  }

  const serialized = serializeResponse(data);

  return (
    <Card>
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">Response</CardTitle>
          <Badge variant="default" className="bg-green-600">Success</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <JsonValue value={serialized} depth={0} />
      </CardContent>
    </Card>
  );
}
