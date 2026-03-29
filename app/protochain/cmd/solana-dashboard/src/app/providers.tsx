"use client";

import type { ReactNode } from "react";
import { ThemeProvider } from "@/providers/theme-provider";
import { ProtochainProvider } from "@/providers/protochain-provider";
import { TooltipProvider } from "@/components/ui/tooltip";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider>
      <TooltipProvider>
        <ProtochainProvider>{children}</ProtochainProvider>
      </TooltipProvider>
    </ThemeProvider>
  );
}
