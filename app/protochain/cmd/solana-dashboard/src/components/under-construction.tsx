"use client";

import { Construction } from "lucide-react";
import { Badge } from "@/components/ui/badge";

export function UnderConstruction({ methodName }: { methodName: string }) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 py-20">
      <Construction className="h-16 w-16 text-muted-foreground" />
      <h2 className="text-xl font-semibold">{methodName}</h2>
      <Badge variant="secondary" className="text-sm">
        Coming Soon
      </Badge>
      <p className="text-sm text-muted-foreground max-w-sm text-center">
        This method page is under construction and will be available in a future
        update.
      </p>
    </div>
  );
}
