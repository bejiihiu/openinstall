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
  if (variant === "secondary") {
    return (
      <button
        type={type}
        disabled={disabled}
        onClick={onClick}
        className="w-full bg-white text-black border border-black px-6 py-3 font-semibold cursor-pointer transition-colors duration-200 hover:bg-black hover:text-white disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {children}
      </button>
    );
  }

  return (
    <button
      type={type}
      disabled={disabled}
      onClick={onClick}
      className="w-full bg-black text-white border-none px-6 py-4 font-semibold cursor-pointer transition-opacity duration-200 hover:opacity-80 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {children}
    </button>
  );
}
