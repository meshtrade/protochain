# Method Page Status Tracker

The agent should read the proto files in `lib/proto/protochain/solana/` for the full source of truth on request/response types when building each page.

All method pages that are not yet implemented MUST show an "Under Construction" splash page (a simple centered message with a construction icon, the method name, and a "Coming Soon" label). This is the default state for every route. Pages are only replaced with a functional form+response UI when they are marked as complete below.

| Service | Method | Route | Status |
|---|---|---|---|
| Account V1 | GenerateNewKeyPair | `/account-v1/generate-new-key-pair` | Complete |
| Account V1 | GetAccount | `/account-v1/get-account` | Complete |
| Account V1 | FundNative | `/account-v1/fund-native` | Complete |
| Program > Token V1 | ParseMint | `/program/token-v1/parse-mint` | Complete |
| Program > Token V1 | CreateToken2022Mint | `/program/token-v1/create-token-2022-mint` | Complete |
| Program > Token V1 | Mint | `/program/token-v1/mint` | Complete |
