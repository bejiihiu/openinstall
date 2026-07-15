"use client";

interface ButtonProps {
  variant?: "primary" | "secondary";
  disabled?: boolean;
  onClick?: () => void;
  children: React.ReactNode;
  type?: "button" | "submit";
  loading?: boolean;
}

export default function Button({
  variant = "primary",
  disabled = false,
  onClick,
  children,
  type = "button",
  loading = false,
}: ButtonProps) {
  const isPrimary = variant === "primary";

  return (
    <button
      type={type}
      disabled={disabled || loading}
      onClick={onClick}
      className="cursor-pointer font-semibold transition-opacity duration-200"
      style={{
        backgroundColor: isPrimary ? "#000000" : "#FFFFFF",
        color: isPrimary ? "#FFFFFF" : "#000000",
        border: "1px solid #000000",
        padding: "12px 24px",
        fontSize: "14px",
        opacity: disabled || loading ? 0.5 : 1,
        cursor: disabled || loading ? "not-allowed" : "pointer",
      }}
      onMouseEnter={(e) => {
        if (disabled) return;
        if (isPrimary) {
          e.currentTarget.style.opacity = "0.8";
        } else {
          e.currentTarget.style.backgroundColor = "#000000";
          e.currentTarget.style.color = "#FFFFFF";
        }
      }}
      onMouseLeave={(e) => {
        if (disabled) return;
        if (isPrimary) {
          e.currentTarget.style.opacity = "1";
        } else {
          e.currentTarget.style.backgroundColor = "#FFFFFF";
          e.currentTarget.style.color = "#000000";
        }
      }}
    >
      {loading ? (
        <span className="inline-block w-4 h-4 border-2 border-current border-t-transparent animate-spin" />
      ) : (
        children
      )}
    </button>
  );
}
