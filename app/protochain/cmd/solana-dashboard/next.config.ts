import type { NextConfig } from "next";
import path from "path";

const nextConfig: NextConfig = {
  output: "standalone",
  // Trace from the monorepo root so workspace dependencies (e.g. @protochain/ts-web) are included
  outputFileTracingRoot: path.join(__dirname, "../../../../"),
};

export default nextConfig;
