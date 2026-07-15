"use client";

interface InputProps {
  type?: string;
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  placeholder?: string;
  name?: string;
  required?: boolean;
  disabled?: boolean;
}

export function Input({
  type = "text",
  value,
  onChange,
  placeholder,
  name,
  required = false,
  disabled = false,
}: InputProps) {
  return (
    <input
      type={type}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      name={name}
      required={required}
      disabled={disabled}
      className="w-full bg-white border border-black px-4 py-3 text-[16px] font-[inherit] outline-none focus:border-2 focus:px-[11px] focus:py-[11px] disabled:opacity-50"
    />
  );
}
