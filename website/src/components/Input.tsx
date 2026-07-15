"use client";

import { useState } from "react";

interface InputProps {
  type?: string;
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  placeholder?: string;
  name?: string;
  required?: boolean;
  disabled?: boolean;
  minLength?: number;
}

export function Input({
  type = "text",
  value,
  onChange,
  placeholder,
  name,
  required = false,
  disabled = false,
  minLength,
}: InputProps) {
  const [focused, setFocused] = useState(false);

  return (
    <input
      type={type}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      name={name}
      required={required}
      disabled={disabled}
      minLength={minLength}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      className={`w-full border bg-white px-4 py-3 text-base outline-none ${
        focused ? "border-2 px-[11px] py-[11px]" : "border"
      } border-black text-black disabled:cursor-not-allowed disabled:opacity-50`}
      style={{ borderRadius: 0 }}
    />
  );
}
