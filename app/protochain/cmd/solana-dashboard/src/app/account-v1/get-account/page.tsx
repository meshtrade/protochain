"use client";

import { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { useProtochain } from "@/providers/protochain-provider";
import { RequestForm } from "@/components/method/request-form";
import { ResponsePanel } from "@/components/method/response-panel";
import { FieldInput } from "@/components/method/field-input";
import { FieldSelect } from "@/components/method/field-select";
import { CopyButton } from "@/components/method/copy-button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ArrowRight } from "lucide-react";

const COMMITMENT_OPTIONS = [
  { label: "Unspecified (default)", value: "0" },
  { label: "Processed", value: "1" },
  { label: "Confirmed", value: "2" },
  { label: "Finalized", value: "3" },
];

interface AccountData {
  address?: string;
  lamports?: string | bigint;
  owner?: string;
  executable?: boolean;
  data?: string;
  rentEpoch?: string | bigint;
}

function GetAccountForm() {
  const searchParams = useSearchParams();
  const { accountService } = useProtochain();

  const [address, setAddress] = useState(searchParams.get("address") ?? "");
  const [commitmentLevel, setCommitmentLevel] = useState("0");

  const [response, setResponse] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [account, setAccount] = useState<AccountData | null>(null);

  async function handleSubmit() {
    setLoading(true);
    setError(null);
    setAccount(null);
    try {
      const res = await accountService.getAccount({
        address,
        commitmentLevel: Number(commitmentLevel),
      } as Parameters<typeof accountService.getAccount>[0]);
      setResponse(res);
      const acct = (res as { account?: AccountData })?.account;
      if (acct) setAccount(acct);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  function formatLamports(lamports: string | bigint): string {
    const value = BigInt(lamports);
    const sol = Number(value) / 1_000_000_000;
    return `${sol.toLocaleString(undefined, { maximumFractionDigits: 9 })} SOL`;
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
        <FieldSelect
          label="Commitment Level"
          value={commitmentLevel}
          onChange={setCommitmentLevel}
          options={COMMITMENT_OPTIONS}
        />
      </RequestForm>

      <ResponsePanel data={response} error={error} loading={loading} />

      {account && (
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base">Account Details</CardTitle>
              {account.executable && <Badge>Executable</Badge>}
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid gap-3 text-sm">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Address</span>
                <div className="flex items-center gap-1">
                  <span className="font-mono truncate max-w-[300px]">
                    {account.address}
                  </span>
                  {account.address && <CopyButton value={account.address} />}
                </div>
              </div>
              {account.lamports != null && (
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Balance</span>
                  <span className="font-mono">
                    {formatLamports(account.lamports)}
                  </span>
                </div>
              )}
              {account.owner && (
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Owner</span>
                  <div className="flex items-center gap-1">
                    <span className="font-mono truncate max-w-[300px]">
                      {account.owner}
                    </span>
                    <CopyButton value={account.owner} />
                  </div>
                </div>
              )}
              {account.rentEpoch != null && (
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Rent Epoch</span>
                  <span className="font-mono">
                    {account.rentEpoch.toString()}
                  </span>
                </div>
              )}
            </div>
            {account.address && (
              <div className="flex flex-wrap gap-2 pt-2 border-t">
                <Link
                  href={`/account-v1/fund-native?address=${account.address}`}
                  className={buttonVariants({ variant: "outline", size: "sm" })}
                >
                  Fund this account
                  <ArrowRight className="ml-1 h-3 w-3" />
                </Link>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </>
  );
}

export default function GetAccountPage() {
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
        <GetAccountForm />
      </Suspense>
    </div>
  );
}
