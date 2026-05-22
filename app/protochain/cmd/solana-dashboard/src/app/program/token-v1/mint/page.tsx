"use client";

import { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import { useProtochain } from "@/providers/protochain-provider";
import { RequestForm } from "@/components/method/request-form";
import { ResponsePanel } from "@/components/method/response-panel";
import { FieldInput } from "@/components/method/field-input";
import { Skeleton } from "@/components/ui/skeleton";

function MintForm() {
  const searchParams = useSearchParams();
  const { programTokenService } = useProtochain();

  const [mintPubKey, setMintPubKey] = useState(
    searchParams.get("mint") ?? ""
  );
  const [destinationOwnerPubKey, setDestinationOwnerPubKey] = useState(
    searchParams.get("owner") ?? ""
  );
  const [amount, setAmount] = useState(searchParams.get("amount") ?? "");

  const [response, setResponse] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit() {
    setLoading(true);
    setError(null);
    try {
      const res = await programTokenService.mint({
        mintPubKey,
        destinationOwnerPubKey,
        amount,
      } as Parameters<typeof programTokenService.mint>[0]);
      setResponse(res);
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
          label="Mint Public Key"
          value={mintPubKey}
          onChange={setMintPubKey}
          placeholder="Base58-encoded mint account address"
          description="The token mint to mint from"
          required
        />
        <FieldInput
          label="Destination Owner Public Key"
          value={destinationOwnerPubKey}
          onChange={setDestinationOwnerPubKey}
          placeholder="Base58-encoded wallet address (not the ATA)"
          description="The system account (wallet) that owns the destination token account. The ATA is derived automatically."
          required
        />
        <FieldInput
          label="Amount"
          value={amount}
          onChange={setAmount}
          placeholder='e.g. "1.5" for 1.5 tokens'
          description='Human-readable token amount in whole-token units (e.g. "1.0", "0.5", "1000"). Converted to base units using the mint&apos;s decimals.'
          required
        />
      </RequestForm>

      <ResponsePanel data={response} error={error} loading={loading} />
    </>
  );
}

export default function MintPage() {
  return (
    <div className="max-w-2xl space-y-4">
      <Suspense
        fallback={
          <div className="space-y-4">
            <Skeleton className="h-[250px] w-full" />
            <Skeleton className="h-[100px] w-full" />
          </div>
        }
      >
        <MintForm />
      </Suspense>
    </div>
  );
}
