"use client";

import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";

interface FieldInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  description?: string;
  required?: boolean;
  multiline?: boolean;
}

export function FieldInput({
  label,
  value,
  onChange,
  placeholder,
  description,
  required,
  multiline,
}: FieldInputProps) {
  const id = label.toLowerCase().replace(/\s+/g, "-");

  return (
    <div className="space-y-1.5">
      <Label htmlFor={id} className="text-sm">
        {label}
        {required && <span className="text-destructive ml-1">*</span>}
      </Label>
      {multiline ? (
        <Textarea
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          className="font-mono text-sm min-h-[80px]"
        />
      ) : (
        <Input
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          className="font-mono text-sm"
        />
      )}
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
    </div>
  );
}
