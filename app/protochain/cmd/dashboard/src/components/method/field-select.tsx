"use client";

import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface FieldSelectOption {
  label: string;
  value: string;
}

interface FieldSelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: FieldSelectOption[];
  description?: string;
  placeholder?: string;
}

export function FieldSelect({
  label,
  value,
  onChange,
  options,
  description,
  placeholder = "Select...",
}: FieldSelectProps) {
  const selectedLabel = options.find((o) => o.value === value)?.label;

  return (
    <div className="space-y-1.5">
      <Label className="text-sm">{label}</Label>
      <Select
        value={value}
        onValueChange={(v) => {
          if (v !== null) onChange(v);
        }}
      >
        <SelectTrigger className="text-sm">
          <SelectValue placeholder={placeholder}>
            {selectedLabel ?? placeholder}
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          {options.map((opt) => (
            <SelectItem key={opt.value} value={opt.value} className="text-sm">
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
    </div>
  );
}
