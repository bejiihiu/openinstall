interface ButtonProps {
  variant?: "primary" | "secondary";
  disabled?: boolean;
  onClick?: () => void;
  children: React.ReactNode;
  type?: "button" | "submit";
}

export default function Button({
  variant = "primary",
  disabled = false,
  onClick,
  children,
  type = "button",
}: ButtonProps) {
  const isPrimary = variant === "primary";

  return (
    <button
      type={type}
      disabled={disabled}
      onClick={onClick}
      className="cursor-pointer transition-opacity duration-200"
      style={{
        backgroundColor: isPrimary ? "#000000" : "#FFFFFF",
        color: isPrimary ? "#FFFFFF" : "#000000",
        border: isPrimary ? "1px solid #000000" : "1px solid #000000",
        padding: "12px 24px",
        fontWeight: 600,
        fontSize: "14px",
        opacity: disabled ? 0.5 : 1,
        cursor: disabled ? "not-allowed" : "pointer",
      }}
      onMouseEnter={(e) => {
        if (!disabled) {
          (e.currentTarget as HTMLButtonElement).style.opacity = "0.8";
        }
      }}
      onMouseLeave={(e) => {
        if (!disabled) {
          (e.currentTarget as HTMLButtonElement).style.opacity = "1";
        }
      }}
    >
      {children}
    </button>
  );
}
