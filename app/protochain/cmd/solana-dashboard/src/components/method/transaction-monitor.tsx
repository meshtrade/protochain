"use client";

import { useRef, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Square } from "lucide-react";

interface MonitorMessage {
  signature?: string;
  status?: number;
  slot?: string | bigint;
  errorMessage?: string;
  logs?: string[];
  computeUnitsConsumed?: string | bigint;
  currentCommitment?: number;
}

interface TransactionMonitorProps {
  messages: MonitorMessage[];
  streaming: boolean;
  onStop: () => void;
}

const STATUS_LABELS: Record<number, { label: string; variant: "default" | "secondary" | "destructive" | "outline" }> = {
  0: { label: "Unspecified", variant: "outline" },
  1: { label: "Received", variant: "secondary" },
  2: { label: "Processed", variant: "secondary" },
  3: { label: "Confirmed", variant: "default" },
  4: { label: "Finalized", variant: "default" },
  5: { label: "Failed", variant: "destructive" },
  6: { label: "Dropped", variant: "destructive" },
  7: { label: "Timeout", variant: "destructive" },
};

const COMMITMENT_LABELS: Record<number, string> = {
  0: "Unspecified",
  1: "Processed",
  2: "Confirmed",
  3: "Finalized",
};

function statusColor(status: number): string {
  switch (status) {
    case 1:
    case 2:
      return "bg-yellow-500";
    case 3:
      return "bg-blue-500";
    case 4:
      return "bg-green-500";
    case 5:
    case 6:
    case 7:
      return "bg-red-500";
    default:
      return "bg-muted";
  }
}

export function TransactionMonitor({
  messages,
  streaming,
  onStop,
}: TransactionMonitorProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  if (messages.length === 0 && !streaming) return null;

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">Transaction Monitor</CardTitle>
          <div className="flex items-center gap-2">
            {streaming && (
              <>
                <div className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                <span className="text-xs text-muted-foreground">Streaming</span>
                <Button variant="outline" size="sm" onClick={onStop}>
                  <Square className="h-3 w-3 mr-1" />
                  Stop
                </Button>
              </>
            )}
            {!streaming && messages.length > 0 && (
              <span className="text-xs text-muted-foreground">Complete</span>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[300px]" ref={scrollRef}>
          <div className="space-y-3">
            {messages.map((msg, i) => {
              const statusInfo = STATUS_LABELS[msg.status ?? 0] ?? STATUS_LABELS[0];
              return (
                <div
                  key={i}
                  className="rounded-md border p-3 space-y-2"
                >
                  <div className="flex items-center gap-2 flex-wrap">
                    <div className={`h-2 w-2 rounded-full ${statusColor(msg.status ?? 0)}`} />
                    <Badge variant={statusInfo.variant}>{statusInfo.label}</Badge>
                    {msg.slot && (
                      <span className="text-xs text-muted-foreground">
                        Slot: {String(msg.slot)}
                      </span>
                    )}
                    {msg.currentCommitment !== undefined && msg.currentCommitment > 0 && (
                      <Badge variant="outline" className="text-xs">
                        {COMMITMENT_LABELS[msg.currentCommitment] ?? "Unknown"}
                      </Badge>
                    )}
                    {msg.computeUnitsConsumed && String(msg.computeUnitsConsumed) !== "0" && (
                      <span className="text-xs text-muted-foreground">
                        CU: {String(msg.computeUnitsConsumed)}
                      </span>
                    )}
                  </div>
                  {msg.errorMessage && (
                    <p className="text-sm text-destructive font-mono break-all">
                      {msg.errorMessage}
                    </p>
                  )}
                  {msg.logs && msg.logs.length > 0 && (
                    <div className="rounded bg-muted p-2">
                      {msg.logs.map((log, j) => (
                        <p key={j} className="text-xs font-mono text-muted-foreground break-all">
                          {log}
                        </p>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
