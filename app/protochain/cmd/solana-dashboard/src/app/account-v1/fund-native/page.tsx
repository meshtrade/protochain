"use client";

import { Suspense, useState, useRef, useCallback } from "react";
import { useSearchParams } from "next/navigation";
import { useProtochain } from "@/providers/protochain-provider";
import { RequestForm } from "@/components/method/request-form";
import { ResponsePanel } from "@/components/method/response-panel";
import { FieldInput } from "@/components/method/field-input";
import { FieldSelect } from "@/components/method/field-select";
import { TransactionMonitor } from "@/components/method/transaction-monitor";
import { CopyButton } from "@/components/method/copy-button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";

const COMMITMENT_OPTIONS = [
  { label: "Unspecified (default)", value: "0" },
  { label: "Processed", value: "1" },
  { label: "Confirmed", value: "2" },
  { label: "Finalized", value: "3" },
];

interface MonitorMessage {
  signature?: string;
  status?: number;
  slot?: string | bigint;
  errorMessage?: string;
  logs?: string[];
  computeUnitsConsumed?: string | bigint;
  currentCommitment?: number;
}

function FundNativeForm() {
  const searchParams = useSearchParams();
  const { accountService, transactionService } = useProtochain();

  const [address, setAddress] = useState(searchParams.get("address") ?? "");
  const [amount, setAmount] = useState(
    searchParams.get("amount") ?? "1000000000"
  );
  const [commitmentLevel, setCommitmentLevel] = useState("2");

  const [response, setResponse] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [signature, setSignature] = useState<string | null>(null);

  const [monitorMessages, setMonitorMessages] = useState<MonitorMessage[]>([]);
  const [streaming, setStreaming] = useState(false);
  const abortRef = useRef<AbortController | null>(null);

  const startMonitoring = useCallback(
    async (sig: string, commitment: number) => {
      setMonitorMessages([]);
      setStreaming(true);

      const controller = new AbortController();
      abortRef.current = controller;

      try {
        const stream = transactionService.monitorTransaction({
          signature: sig,
          commitmentLevel: commitment || 2,
          includeLogs: true,
          timeoutSeconds: 60,
        } as Parameters<typeof transactionService.monitorTransaction>[0]);

        for await (const msg of stream) {
          if (controller.signal.aborted) break;
          setMonitorMessages((prev) => [...prev, msg as MonitorMessage]);
        }
      } catch (e) {
        if (!controller.signal.aborted) {
          setMonitorMessages((prev) => [
            ...prev,
            {
              status: 5,
              errorMessage: e instanceof Error ? e.message : String(e),
            },
          ]);
        }
      } finally {
        setStreaming(false);
        abortRef.current = null;
      }
    },
    [transactionService]
  );

  function handleStopMonitoring() {
    abortRef.current?.abort();
  }

  async function handleSubmit() {
    setLoading(true);
    setError(null);
    setSignature(null);
    setMonitorMessages([]);
    try {
      const res = await accountService.fundNative({
        address,
        amount,
        commitmentLevel: Number(commitmentLevel),
      } as Parameters<typeof accountService.fundNative>[0]);
      setResponse(res);
      const sig = (res as { signature?: string })?.signature;
      if (sig) {
        setSignature(sig);
        startMonitoring(sig, Number(commitmentLevel));
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <RequestForm onSubmit={handleSubmit} loading={loading}>
        <FieldInput
          label="Address"
          value={address}
          onChange={setAddress}
          placeholder="Base58-encoded Solana address"
          required
        />
        <FieldInput
          label="Amount"
          value={amount}
          onChange={setAmount}
          placeholder="Amount in lamports (1 SOL = 1000000000)"
          description="1 SOL = 1,000,000,000 lamports"
          required
        />
        <FieldSelect
          label="Commitment Level"
          value={commitmentLevel}
          onChange={setCommitmentLevel}
          options={COMMITMENT_OPTIONS}
        />
      </RequestForm>

      <ResponsePanel data={response} error={error} loading={loading} />

      {signature && (
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base">Transaction Signature</CardTitle>
              <Badge variant="outline">Airdrop</Badge>
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              <span className="font-mono text-sm break-all">{signature}</span>
              <CopyButton value={signature} />
            </div>
          </CardContent>
        </Card>
      )}

      <TransactionMonitor
        messages={monitorMessages}
        streaming={streaming}
        onStop={handleStopMonitoring}
      />
    </>
  );
}

export default function FundNativePage() {
  return (
    <div className="max-w-2xl space-y-4">
      <Suspense
        fallback={
          <div className="space-y-4">
            <Skeleton className="h-[200px] w-full" />
            <Skeleton className="h-[100px] w-full" />
          </div>
        }
      >
        <FundNativeForm />
      </Suspense>
    </div>
  );
}
