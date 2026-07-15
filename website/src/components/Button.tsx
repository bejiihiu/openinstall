"use client";

interface ButtonProps {
  variant?: "primary" | "secondary";
  disabled?: boolean;
  onClick?: () => void;
  children: React.ReactNode;
  type?: "button" | "submit";
}

export function Button({
  variant = "primary",
  disabled = false,
  onClick,
  children,
  type = "button",
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled}
      onClick={onClick}
      className="cursor-pointer font-semibold transition-opacity duration-200"
      style={{
        borderRadius: 0,
        padding: "12px 24px",
        opacity: disabled ? 0.5 : 1,
        ...(variant === "primary"
          ? {
              backgroundColor: "#000000",
              color: "#FFFFFF",
              border: "none",
            }
          : {
              backgroundColor: "#FFFFFF",
              color: "#000000",
              border: "1px solid #000000",
            }),
      }}
      onMouseEnter={(e) => {
        if (variant === "primary") {
          e.currentTarget.style.opacity = "0.8";
        } else {
          e.currentTarget.style.backgroundColor = "#000000";
          e.currentTarget.style.color = "#FFFFFF";
        }
      }}
      onMouseLeave={(e) => {
        if (variant === "primary") {
          e.currentTarget.style.opacity = disabled ? "0.5" : "1";
        } else {
          e.currentTarget.style.backgroundColor = "#FFFFFF";
          e.currentTarget.style.color = "#000000";
        }
      }}
    >
      {children}
    </button>
  );
}
