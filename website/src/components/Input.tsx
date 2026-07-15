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

export default function Input({
  type = "text",
  value,
  onChange,
  placeholder,
  name,
  required = false,
  disabled = false,
  minLength,
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
      minLength={minLength}
      className="w-full outline-none transition-all duration-200"
      style={{
        backgroundColor: "#FFFFFF",
        border: "1px solid #000000",
        padding: "12px 16px",
        fontSize: "16px",
        opacity: disabled ? 0.5 : 1,
      }}
      onFocus={(e) => {
        (e.currentTarget as HTMLInputElement).style.borderWidth = "2px";
        (e.currentTarget as HTMLInputElement).style.padding = "11px 15px";
      }}
      onBlur={(e) => {
        (e.currentTarget as HTMLInputElement).style.borderWidth = "1px";
        (e.currentTarget as HTMLInputElement).style.padding = "12px 16px";
      }}
    />
  );
}
