"use client";

import {
  createContext,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { AccountServiceWeb } from "@protochain/ts-web/solana/account/v1";
import { TransactionServiceWeb } from "@protochain/ts-web/solana/transaction/v1";
import { ProgramSystemServiceWeb } from "@protochain/ts-web/solana/program/system/v1";
import { ProgramTokenServiceWeb } from "@protochain/ts-web/solana/program/token/v1";
import { RpcClientServiceWeb } from "@protochain/ts-web/solana/rpc_client/v1";
import { WithServerUrl, WithLogging } from "@protochain/ts-web/config";

interface ProtochainContextValue {
  serverUrl: string;
  setServerUrl: (url: string) => void;
  accountService: AccountServiceWeb;
  transactionService: TransactionServiceWeb;
  programSystemService: ProgramSystemServiceWeb;
  programTokenService: ProgramTokenServiceWeb;
  rpcClientService: RpcClientServiceWeb;
}

const ProtochainContext = createContext<ProtochainContextValue | null>(null);

const DEFAULT_URL = "http://localhost:50051";

function getInitialServerUrl(): string {
  if (typeof window === "undefined") return DEFAULT_URL;
  const params = new URLSearchParams(window.location.search);
  const rpc = params.get("rpc");
  return rpc?.trim() || DEFAULT_URL;
}

export function ProtochainProvider({ children }: { children: ReactNode }) {
  const [serverUrl, setServerUrl] = useState(getInitialServerUrl);

  const clients = useMemo(() => {
    const opts = [WithServerUrl(serverUrl), WithLogging()];
    return {
      accountService: new AccountServiceWeb(...opts),
      transactionService: new TransactionServiceWeb(...opts),
      programSystemService: new ProgramSystemServiceWeb(...opts),
      programTokenService: new ProgramTokenServiceWeb(...opts),
      rpcClientService: new RpcClientServiceWeb(...opts),
    };
  }, [serverUrl]);

  return (
    <ProtochainContext.Provider
      value={{ serverUrl, setServerUrl, ...clients }}
    >
      {children}
    </ProtochainContext.Provider>
  );
}

export function useProtochain(): ProtochainContextValue {
  const ctx = useContext(ProtochainContext);
  if (!ctx) {
    throw new Error("useProtochain must be used within a ProtochainProvider");
  }
  return ctx;
}
