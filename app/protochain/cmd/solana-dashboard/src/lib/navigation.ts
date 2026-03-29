export interface NavLeaf {
  label: string;
  href: string;
}

export interface NavGroup {
  label: string;
  children: (NavGroup | NavLeaf)[];
}

export function isNavLeaf(node: NavGroup | NavLeaf): node is NavLeaf {
  return "href" in node;
}

export const navigationTree: NavGroup[] = [
  {
    label: "Account V1",
    children: [
      { label: "Get Account", href: "/account-v1/get-account" },
      {
        label: "Generate New Key Pair",
        href: "/account-v1/generate-new-key-pair",
      },
      { label: "Fund Native", href: "/account-v1/fund-native" },
      {
        label: "Get Token Account Balance",
        href: "/account-v1/get-token-account-balance",
      },
      {
        label: "Get Associated Token Address",
        href: "/account-v1/get-associated-token-address",
      },
    ],
  },
  {
    label: "Program",
    children: [
      {
        label: "System V1",
        children: [
          { label: "Create", href: "/program/system-v1/create" },
          { label: "Transfer", href: "/program/system-v1/transfer" },
          { label: "Allocate", href: "/program/system-v1/allocate" },
          { label: "Assign", href: "/program/system-v1/assign" },
          {
            label: "Create With Seed",
            href: "/program/system-v1/create-with-seed",
          },
          {
            label: "Allocate With Seed",
            href: "/program/system-v1/allocate-with-seed",
          },
          {
            label: "Assign With Seed",
            href: "/program/system-v1/assign-with-seed",
          },
          {
            label: "Transfer With Seed",
            href: "/program/system-v1/transfer-with-seed",
          },
          {
            label: "Initialize Nonce Account",
            href: "/program/system-v1/initialize-nonce-account",
          },
          {
            label: "Authorize Nonce Account",
            href: "/program/system-v1/authorize-nonce-account",
          },
          {
            label: "Withdraw Nonce Account",
            href: "/program/system-v1/withdraw-nonce-account",
          },
          {
            label: "Advance Nonce Account",
            href: "/program/system-v1/advance-nonce-account",
          },
          {
            label: "Upgrade Nonce Account",
            href: "/program/system-v1/upgrade-nonce-account",
          },
        ],
      },
      {
        label: "Token V1",
        children: [
          {
            label: "Create Token 2022 Mint",
            href: "/program/token-v1/create-token-2022-mint",
          },
          {
            label: "Create SPL Token Mint",
            href: "/program/token-v1/create-spl-token-mint",
          },
          { label: "Parse Mint", href: "/program/token-v1/parse-mint" },
          {
            label: "Create Token 2022 Holding Account",
            href: "/program/token-v1/create-token-2022-holding-account",
          },
          {
            label: "Create SPL Token Holding Account",
            href: "/program/token-v1/create-spl-token-holding-account",
          },
          { label: "Mint", href: "/program/token-v1/mint" },
        ],
      },
    ],
  },
  {
    label: "Transaction V1",
    children: [
      {
        label: "Compile Transaction",
        href: "/transaction-v1/compile-transaction",
      },
      {
        label: "Estimate Transaction",
        href: "/transaction-v1/estimate-transaction",
      },
      {
        label: "Simulate Transaction",
        href: "/transaction-v1/simulate-transaction",
      },
      {
        label: "Sign Transaction",
        href: "/transaction-v1/sign-transaction",
      },
      {
        label: "Check If Transaction Is Expired",
        href: "/transaction-v1/check-if-transaction-is-expired",
      },
      {
        label: "Submit Transaction",
        href: "/transaction-v1/submit-transaction",
      },
      {
        label: "Get Transaction",
        href: "/transaction-v1/get-transaction",
      },
      {
        label: "Monitor Transaction",
        href: "/transaction-v1/monitor-transaction",
      },
    ],
  },
  {
    label: "RPC Client V1",
    children: [
      {
        label: "Get Minimum Balance For Rent Exemption",
        href: "/rpc-client-v1/get-minimum-balance-for-rent-exemption",
      },
    ],
  },
];
