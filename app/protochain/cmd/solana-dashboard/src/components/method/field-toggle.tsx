"use client";

import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

interface FieldToggleProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
}

export function FieldToggle({
  label,
  checked,
  onChange,
  description,
}: FieldToggleProps) {
  const id = label.toLowerCase().replace(/\s+/g, "-");

  return (
    <div className="flex items-center justify-between rounded-md border p-3">
      <div className="space-y-0.5">
        <Label htmlFor={id} className="text-sm">
          {label}
        </Label>
        {description && (
          <p className="text-xs text-muted-foreground">{description}</p>
        )}
      </div>
      <Switch id={id} checked={checked} onCheckedChange={onChange} />
    </div>
  );
}
