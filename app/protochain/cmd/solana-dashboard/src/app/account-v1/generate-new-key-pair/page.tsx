"use client";

import { useState } from "react";
import Link from "next/link";
import { useProtochain } from "@/providers/protochain-provider";
import { RequestForm } from "@/components/method/request-form";
import { ResponsePanel } from "@/components/method/response-panel";
import { FieldInput } from "@/components/method/field-input";
import { CopyButton } from "@/components/method/copy-button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { buttonVariants } from "@/components/ui/button";
import { ArrowRight } from "lucide-react";

export default function GenerateNewKeyPairPage() {
  const { accountService } = useProtochain();
  const [response, setResponse] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [seed, setSeed] = useState("");
  const [generatedAddress, setGeneratedAddress] = useState<string | null>(null);

  async function handleSubmit() {
    setLoading(true);
    setError(null);
    setGeneratedAddress(null);
    try {
      const res = await accountService.generateNewKeyPair({
        seed: seed || undefined,
      } as Parameters<typeof accountService.generateNewKeyPair>[0]);
      setResponse(res);
      const pubKey = (res as { keyPair?: { publicKey?: string } })?.keyPair
        ?.publicKey;
      if (pubKey) setGeneratedAddress(pubKey);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="max-w-2xl space-y-4">
      <RequestForm onSubmit={handleSubmit} loading={loading}>
        <FieldInput
          label="Seed"
          value={seed}
          onChange={setSeed}
          placeholder="Optional hex-encoded seed for deterministic generation"
          description="Leave empty for a random keypair"
        />
      </RequestForm>

      <ResponsePanel data={response} error={error} loading={loading} />

      {generatedAddress && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Quick Actions</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-2 items-center">
            <div className="flex items-center gap-1 text-sm text-muted-foreground">
              <span className="font-mono truncate max-w-[200px]">
                {generatedAddress}
              </span>
              <CopyButton value={generatedAddress} />
            </div>
            <Link
              href={`/account-v1/fund-native?address=${generatedAddress}`}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              Fund this account
              <ArrowRight className="ml-1 h-3 w-3" />
            </Link>
            <Link
              href={`/account-v1/get-account?address=${generatedAddress}`}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              View account
              <ArrowRight className="ml-1 h-3 w-3" />
            </Link>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
