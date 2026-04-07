"use client";

import { useState } from "react";
import { Server } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarGroupContent,
  SidebarSeparator,
} from "@/components/ui/sidebar";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useProtochain } from "@/providers/protochain-provider";
import { navigationTree } from "@/lib/navigation";
import { NavTree } from "./nav-tree";

const URL_PRESETS = [
  { label: "Local (Docker Compose)", url: "http://localhost:50064" },
  { label: "Production", url: "https://protochain-solana-api.mesh.trade" },
  { label: "Test", url: "https://protochain-test.mesh.trade" },
] as const;

const CUSTOM_VALUE = "__custom__";

export function AppSidebar() {
  const { serverUrl, setServerUrl } = useProtochain();
  const [isCustom, setIsCustom] = useState(
    !URL_PRESETS.some((p) => p.url === serverUrl)
  );
  const [customUrl, setCustomUrl] = useState(
    isCustom ? serverUrl : ""
  );

  const selectValue = isCustom
    ? CUSTOM_VALUE
    : URL_PRESETS.find((p) => p.url === serverUrl)?.url ?? CUSTOM_VALUE;

  function handleSelectChange(value: string) {
    if (value === CUSTOM_VALUE) {
      setIsCustom(true);
      if (customUrl) setServerUrl(customUrl);
    } else {
      setIsCustom(false);
      setServerUrl(value);
    }
  }

  function handleCustomUrlBlur() {
    if (customUrl.trim()) {
      setServerUrl(customUrl.trim());
    }
  }

  function handleCustomUrlKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && customUrl.trim()) {
      setServerUrl(customUrl.trim());
    }
  }

  return (
    <Sidebar>
      <SidebarHeader className="p-4">
        <div className="flex items-center gap-2 mb-3">
          <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary text-primary-foreground">
            <Server className="h-4 w-4" />
          </div>
          <div>
            <p className="text-sm font-semibold">Protochain</p>
            <p className="text-xs text-muted-foreground">Dashboard</p>
          </div>
        </div>
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Server URL</Label>
          <Select value={selectValue} onValueChange={(v) => { if (v !== null) handleSelectChange(v); }}>
            <SelectTrigger className="h-8 text-xs">
              <SelectValue>
                {isCustom
                  ? "Custom URL"
                  : URL_PRESETS.find((p) => p.url === serverUrl)?.label ?? serverUrl}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {URL_PRESETS.map((preset) => (
                <SelectItem key={preset.url} value={preset.url} className="text-xs">
                  {preset.label}
                </SelectItem>
              ))}
              <SelectItem value={CUSTOM_VALUE} className="text-xs">
                Custom URL
              </SelectItem>
            </SelectContent>
          </Select>
          {isCustom && (
            <Input
              className="h-8 text-xs"
              placeholder="http://localhost:50064"
              value={customUrl}
              onChange={(e) => setCustomUrl(e.target.value)}
              onBlur={handleCustomUrlBlur}
              onKeyDown={handleCustomUrlKeyDown}
            />
          )}
        </div>
      </SidebarHeader>
      <SidebarSeparator />
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Services</SidebarGroupLabel>
          <SidebarGroupContent>
            <NavTree items={navigationTree} />
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
