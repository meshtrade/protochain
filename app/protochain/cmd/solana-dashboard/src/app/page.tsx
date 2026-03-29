import Link from "next/link";

export default function Home() {
  return (
    <div className="max-w-2xl space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">
          Protochain Dashboard
        </h1>
        <p className="text-muted-foreground mt-1">
          A developer UI for exercising every RPC method exposed by the
          Protochain Solana gRPC API. Select a method from the sidebar to get
          started.
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        <QuickLink
          href="/account-v1/generate-new-key-pair"
          title="Generate Key Pair"
          description="Create a new Solana keypair"
        />
        <QuickLink
          href="/account-v1/fund-native"
          title="Fund Native"
          description="Airdrop SOL to an account"
        />
        <QuickLink
          href="/program/token-v1/parse-mint"
          title="Parse Mint"
          description="Inspect a token mint account"
        />
        <QuickLink
          href="/account-v1/get-account"
          title="Get Account"
          description="Fetch account data from Solana"
        />
      </div>
    </div>
  );
}

function QuickLink({
  href,
  title,
  description,
}: {
  href: string;
  title: string;
  description: string;
}) {
  return (
    <Link
      href={href}
      className="group rounded-lg border p-4 transition-colors hover:border-foreground/20 hover:bg-accent"
    >
      <h3 className="font-semibold group-hover:underline">{title}</h3>
      <p className="text-sm text-muted-foreground mt-1">{description}</p>
    </Link>
  );
}
