"use client";

import { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import { useProtochain } from "@/providers/protochain-provider";
import { RequestForm } from "@/components/method/request-form";
import { ResponsePanel } from "@/components/method/response-panel";
import { FieldInput } from "@/components/method/field-input";
import { FieldSelect } from "@/components/method/field-select";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Plus, X } from "lucide-react";

// ---------------------------------------------------------------------------
// Extension-specific sub-forms
// ---------------------------------------------------------------------------

type ExtensionKind =
  | "metadata"
  | "mintCloseAuthority"
  | "transferFee"
  | "defaultAccountState"
  | "permanentDelegate"
  | "pausable";

const EXTENSION_OPTIONS: { label: string; value: ExtensionKind }[] = [
  { label: "Metadata", value: "metadata" },
  { label: "Mint Close Authority", value: "mintCloseAuthority" },
  { label: "Transfer Fee", value: "transferFee" },
  { label: "Default Account State", value: "defaultAccountState" },
  { label: "Permanent Delegate", value: "permanentDelegate" },
  { label: "Pausable", value: "pausable" },
];

// State shapes for each extension kind
interface MetadataState {
  metadataAddress: string;
  updateAuthorityPubKey: string;
  name: string;
  symbol: string;
  uri: string;
  additionalMetadata: { key: string; value: string }[];
}

interface MintCloseAuthorityState {
  closeAuthorityPubKey: string;
}

interface TransferFeeState {
  transferFeeConfigAuthorityPubKey: string;
  withdrawWithheldAuthorityPubKey: string;
  transferFeeBasisPoints: string;
  maximumFee: string;
}

interface DefaultAccountStateState {
  state: string; // enum value as string
}

interface PermanentDelegateState {
  delegatePubKey: string;
}

interface PausableState {
  authorityPubKey: string;
}

type ExtensionState =
  | { kind: "metadata"; data: MetadataState }
  | { kind: "mintCloseAuthority"; data: MintCloseAuthorityState }
  | { kind: "transferFee"; data: TransferFeeState }
  | { kind: "defaultAccountState"; data: DefaultAccountStateState }
  | { kind: "permanentDelegate"; data: PermanentDelegateState }
  | { kind: "pausable"; data: PausableState };

function makeDefaultExtension(kind: ExtensionKind): ExtensionState {
  switch (kind) {
    case "metadata":
      return {
        kind,
        data: {
          metadataAddress: "",
          updateAuthorityPubKey: "",
          name: "",
          symbol: "",
          uri: "",
          additionalMetadata: [],
        },
      };
    case "mintCloseAuthority":
      return { kind, data: { closeAuthorityPubKey: "" } };
    case "transferFee":
      return {
        kind,
        data: {
          transferFeeConfigAuthorityPubKey: "",
          withdrawWithheldAuthorityPubKey: "",
          transferFeeBasisPoints: "0",
          maximumFee: "0",
        },
      };
    case "defaultAccountState":
      return { kind, data: { state: "1" } };
    case "permanentDelegate":
      return { kind, data: { delegatePubKey: "" } };
    case "pausable":
      return { kind, data: { authorityPubKey: "" } };
  }
}

// ---------------------------------------------------------------------------
// Extension sub-form components
// ---------------------------------------------------------------------------

function MetadataForm({
  data,
  onChange,
}: {
  data: MetadataState;
  onChange: (d: MetadataState) => void;
}) {
  function addKV() {
    onChange({
      ...data,
      additionalMetadata: [...data.additionalMetadata, { key: "", value: "" }],
    });
  }
  function removeKV(idx: number) {
    onChange({
      ...data,
      additionalMetadata: data.additionalMetadata.filter((_, i) => i !== idx),
    });
  }
  function updateKV(idx: number, field: "key" | "value", val: string) {
    const updated = [...data.additionalMetadata];
    updated[idx] = { ...updated[idx], [field]: val };
    onChange({ ...data, additionalMetadata: updated });
  }

  return (
    <div className="space-y-3">
      <FieldInput
        label="Name"
        value={data.name}
        onChange={(v) => onChange({ ...data, name: v })}
        placeholder='e.g. "My Token"'
        required
      />
      <FieldInput
        label="Symbol"
        value={data.symbol}
        onChange={(v) => onChange({ ...data, symbol: v })}
        placeholder='e.g. "MYTKN"'
        required
      />
      <FieldInput
        label="URI"
        value={data.uri}
        onChange={(v) => onChange({ ...data, uri: v })}
        placeholder="https://... or ipfs://..."
        description="Off-chain metadata JSON URL"
      />
      <FieldInput
        label="Metadata Address"
        value={data.metadataAddress}
        onChange={(v) => onChange({ ...data, metadataAddress: v })}
        placeholder="Leave empty to default to mint address"
        description="Defaults to the mint public key if empty"
      />
      <FieldInput
        label="Update Authority"
        value={data.updateAuthorityPubKey}
        onChange={(v) => onChange({ ...data, updateAuthorityPubKey: v })}
        placeholder="Leave empty to default to mint authority"
        description="Defaults to the mint authority if empty"
      />

      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <p className="text-sm font-medium">Additional Metadata</p>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={addKV}
          >
            <Plus className="mr-1 h-3 w-3" />
            Add Field
          </Button>
        </div>
        {data.additionalMetadata.map((kv, idx) => (
          <div key={idx} className="flex items-center gap-2">
            <FieldInput
              label=""
              value={kv.key}
              onChange={(v) => updateKV(idx, "key", v)}
              placeholder="Key"
            />
            <FieldInput
              label=""
              value={kv.value}
              onChange={(v) => updateKV(idx, "value", v)}
              placeholder="Value"
            />
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="shrink-0"
              onClick={() => removeKV(idx)}
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        ))}
      </div>
    </div>
  );
}

function MintCloseAuthorityForm({
  data,
  onChange,
}: {
  data: MintCloseAuthorityState;
  onChange: (d: MintCloseAuthorityState) => void;
}) {
  return (
    <FieldInput
      label="Close Authority"
      value={data.closeAuthorityPubKey}
      onChange={(v) => onChange({ closeAuthorityPubKey: v })}
      placeholder="Leave empty to default to mint authority"
      description="Authority that may close the mint account"
    />
  );
}

function TransferFeeForm({
  data,
  onChange,
}: {
  data: TransferFeeState;
  onChange: (d: TransferFeeState) => void;
}) {
  return (
    <div className="space-y-3">
      <FieldInput
        label="Transfer Fee Config Authority"
        value={data.transferFeeConfigAuthorityPubKey}
        onChange={(v) =>
          onChange({ ...data, transferFeeConfigAuthorityPubKey: v })
        }
        placeholder="Leave empty for immutable fee"
        description="Authority that can modify the transfer fee"
      />
      <FieldInput
        label="Withdraw Withheld Authority"
        value={data.withdrawWithheldAuthorityPubKey}
        onChange={(v) =>
          onChange({ ...data, withdrawWithheldAuthorityPubKey: v })
        }
        placeholder="Leave empty for no withdraw authority"
        description="Authority that can withdraw withheld fees"
      />
      <FieldInput
        label="Transfer Fee (basis points)"
        value={data.transferFeeBasisPoints}
        onChange={(v) => onChange({ ...data, transferFeeBasisPoints: v })}
        placeholder="100 = 1%"
        description="Fee in basis points (100 = 1%, max 10000)"
      />
      <FieldInput
        label="Maximum Fee"
        value={data.maximumFee}
        onChange={(v) => onChange({ ...data, maximumFee: v })}
        placeholder="0 for no maximum"
        description="Maximum fee per transfer in base token units"
      />
    </div>
  );
}

function DefaultAccountStateForm({
  data,
  onChange,
}: {
  data: DefaultAccountStateState;
  onChange: (d: DefaultAccountStateState) => void;
}) {
  return (
    <FieldSelect
      label="Default Account State"
      value={data.state}
      onChange={(v) => onChange({ state: v })}
      options={[
        { label: "Initialized", value: "1" },
        { label: "Frozen", value: "2" },
      ]}
      description="Default state for new token accounts"
    />
  );
}

function PermanentDelegateForm({
  data,
  onChange,
}: {
  data: PermanentDelegateState;
  onChange: (d: PermanentDelegateState) => void;
}) {
  return (
    <FieldInput
      label="Delegate Public Key"
      value={data.delegatePubKey}
      onChange={(v) => onChange({ delegatePubKey: v })}
      placeholder="Base58-encoded public key"
      description="Irrevocable delegate authority over all token accounts"
      required
    />
  );
}

function PausableForm({
  data,
  onChange,
}: {
  data: PausableState;
  onChange: (d: PausableState) => void;
}) {
  return (
    <FieldInput
      label="Pause Authority"
      value={data.authorityPubKey}
      onChange={(v) => onChange({ authorityPubKey: v })}
      placeholder="Leave empty to default to mint authority"
      description="Authority that can pause and resume mint activity"
    />
  );
}

function ExtensionSubForm({
  ext,
  onChange,
}: {
  ext: ExtensionState;
  onChange: (e: ExtensionState) => void;
}) {
  switch (ext.kind) {
    case "metadata":
      return (
        <MetadataForm
          data={ext.data}
          onChange={(d) => onChange({ kind: "metadata", data: d })}
        />
      );
    case "mintCloseAuthority":
      return (
        <MintCloseAuthorityForm
          data={ext.data}
          onChange={(d) => onChange({ kind: "mintCloseAuthority", data: d })}
        />
      );
    case "transferFee":
      return (
        <TransferFeeForm
          data={ext.data}
          onChange={(d) => onChange({ kind: "transferFee", data: d })}
        />
      );
    case "defaultAccountState":
      return (
        <DefaultAccountStateForm
          data={ext.data}
          onChange={(d) => onChange({ kind: "defaultAccountState", data: d })}
        />
      );
    case "permanentDelegate":
      return (
        <PermanentDelegateForm
          data={ext.data}
          onChange={(d) => onChange({ kind: "permanentDelegate", data: d })}
        />
      );
    case "pausable":
      return (
        <PausableForm
          data={ext.data}
          onChange={(d) => onChange({ kind: "pausable", data: d })}
        />
      );
  }
}

// ---------------------------------------------------------------------------
// Serialise extension state → proto-compatible object
// ---------------------------------------------------------------------------

function extensionToProto(ext: ExtensionState): Record<string, unknown> {
  switch (ext.kind) {
    case "metadata": {
      const additionalMetadata: Record<string, string> = {};
      for (const kv of ext.data.additionalMetadata) {
        if (kv.key.trim()) additionalMetadata[kv.key] = kv.value;
      }
      return {
        extension: {
          case: "metadata",
          value: {
            metadataAddress: ext.data.metadataAddress,
            updateAuthorityPubKey: ext.data.updateAuthorityPubKey,
            name: ext.data.name,
            symbol: ext.data.symbol,
            uri: ext.data.uri,
            additionalMetadata,
          },
        },
      };
    }
    case "mintCloseAuthority":
      return {
        extension: {
          case: "mintCloseAuthority",
          value: {
            closeAuthorityPubKey: ext.data.closeAuthorityPubKey,
          },
        },
      };
    case "transferFee":
      return {
        extension: {
          case: "transferFee",
          value: {
            transferFeeConfigAuthorityPubKey:
              ext.data.transferFeeConfigAuthorityPubKey,
            withdrawWithheldAuthorityPubKey:
              ext.data.withdrawWithheldAuthorityPubKey,
            transferFeeBasisPoints: Number(ext.data.transferFeeBasisPoints) || 0,
            maximumFee: BigInt(ext.data.maximumFee || "0"),
          },
        },
      };
    case "defaultAccountState":
      return {
        extension: {
          case: "defaultAccountState",
          value: {
            state: Number(ext.data.state),
          },
        },
      };
    case "permanentDelegate":
      return {
        extension: {
          case: "permanentDelegate",
          value: {
            delegatePubKey: ext.data.delegatePubKey,
          },
        },
      };
    case "pausable":
      return {
        extension: {
          case: "pausable",
          value: {
            authorityPubKey: ext.data.authorityPubKey,
          },
        },
      };
  }
}

function extensionLabel(kind: ExtensionKind): string {
  return EXTENSION_OPTIONS.find((o) => o.value === kind)?.label ?? kind;
}

// ---------------------------------------------------------------------------
// Main form
// ---------------------------------------------------------------------------

function CreateToken2022MintForm() {
  const searchParams = useSearchParams();
  const { programTokenService } = useProtochain();

  const [payerPubKey, setPayerPubKey] = useState(
    searchParams.get("payer") ?? ""
  );
  const [mintPubKey, setMintPubKey] = useState(
    searchParams.get("mint") ?? ""
  );
  const [mintAuthorityPubKey, setMintAuthorityPubKey] = useState(
    searchParams.get("mintAuthority") ?? ""
  );
  const [freezeAuthorityPubKey, setFreezeAuthorityPubKey] = useState("");
  const [decimals, setDecimals] = useState("9");
  const [extensions, setExtensions] = useState<ExtensionState[]>([]);
  const [extensionToAdd, setExtensionToAdd] = useState<ExtensionKind>("metadata");

  const [response, setResponse] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  function addExtension() {
    setExtensions((prev) => [...prev, makeDefaultExtension(extensionToAdd)]);
  }

  function removeExtension(idx: number) {
    setExtensions((prev) => prev.filter((_, i) => i !== idx));
  }

  function updateExtension(idx: number, ext: ExtensionState) {
    setExtensions((prev) => prev.map((e, i) => (i === idx ? ext : e)));
  }

  async function handleSubmit() {
    setLoading(true);
    setError(null);
    try {
      const res = await programTokenService.createToken2022Mint({
        payerPubKey,
        mintPubKey,
        mintAuthorityPubKey,
        freezeAuthorityPubKey,
        decimals: Number(decimals) || 0,
        extensions: extensions.map(extensionToProto),
      } as Parameters<typeof programTokenService.createToken2022Mint>[0]);
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
          label="Payer Public Key"
          value={payerPubKey}
          onChange={setPayerPubKey}
          placeholder="Base58-encoded payer address (signer)"
          required
        />
        <FieldInput
          label="Mint Public Key"
          value={mintPubKey}
          onChange={setMintPubKey}
          placeholder="Base58-encoded mint account address (signer)"
          required
        />
        <FieldInput
          label="Mint Authority"
          value={mintAuthorityPubKey}
          onChange={setMintAuthorityPubKey}
          placeholder="Base58-encoded mint authority address"
          required
        />
        <FieldInput
          label="Freeze Authority"
          value={freezeAuthorityPubKey}
          onChange={setFreezeAuthorityPubKey}
          placeholder="Leave empty to disable freeze"
          description="Optional — leave empty to disable freeze functionality"
        />
        <FieldSelect
          label="Decimals"
          value={decimals}
          onChange={setDecimals}
          options={[
            { label: "0 (NFT)", value: "0" },
            { label: "6 (USDC-like)", value: "6" },
            { label: "9 (SOL-like)", value: "9" },
          ]}
          description="Number of decimal places for the token"
        />

        {/* Extensions section */}
        <Card className="border-dashed">
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Extensions</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {extensions.map((ext, idx) => (
              <Card key={idx}>
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <Badge variant="secondary">{extensionLabel(ext.kind)}</Badge>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6"
                      onClick={() => removeExtension(idx)}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  <ExtensionSubForm
                    ext={ext}
                    onChange={(e) => updateExtension(idx, e)}
                  />
                </CardContent>
              </Card>
            ))}

            <div className="flex items-end gap-2">
              <div className="flex-1">
                <FieldSelect
                  label="Extension Type"
                  value={extensionToAdd}
                  onChange={(v) => setExtensionToAdd(v as ExtensionKind)}
                  options={EXTENSION_OPTIONS}
                />
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={addExtension}
                className="mb-0.5"
              >
                <Plus className="mr-1 h-3 w-3" />
                Add
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              Extensions cannot be added after mint creation.
            </p>
          </CardContent>
        </Card>
      </RequestForm>

      <ResponsePanel data={response} error={error} loading={loading} />
    </>
  );
}

export default function CreateToken2022MintPage() {
  return (
    <div className="max-w-2xl space-y-4">
      <Suspense
        fallback={
          <div className="space-y-4">
            <Skeleton className="h-[400px] w-full" />
            <Skeleton className="h-[100px] w-full" />
          </div>
        }
      >
        <CreateToken2022MintForm />
      </Suspense>
    </div>
  );
}
