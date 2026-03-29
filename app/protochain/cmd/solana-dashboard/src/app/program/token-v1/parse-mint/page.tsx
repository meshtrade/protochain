"use client";

import { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import { useProtochain } from "@/providers/protochain-provider";
import { RequestForm } from "@/components/method/request-form";
import { ResponsePanel } from "@/components/method/response-panel";
import { FieldInput } from "@/components/method/field-input";
import { Skeleton } from "@/components/ui/skeleton";

function ParseMintForm() {
  const searchParams = useSearchParams();
  const { programTokenService } = useProtochain();

  const [accountAddress, setAccountAddress] = useState(
    searchParams.get("address") ?? ""
  );
  const [response, setResponse] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit() {
    setLoading(true);
    setError(null);
    try {
      const res = await programTokenService.parseMint({
        accountAddress,
      } as Parameters<typeof programTokenService.parseMint>[0]);
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
          label="Account Address"
          value={accountAddress}
          onChange={setAccountAddress}
          placeholder="Base58-encoded mint account address"
          required
        />
      </RequestForm>

      <ResponsePanel data={response} error={error} loading={loading} />
    </>
  );
}

export default function ParseMintPage() {
  return (
    <div className="max-w-2xl space-y-4">
      <Suspense
        fallback={
          <div className="space-y-4">
            <Skeleton className="h-[150px] w-full" />
            <Skeleton className="h-[100px] w-full" />
          </div>
        }
      >
        <ParseMintForm />
      </Suspense>
    </div>
  );
}
