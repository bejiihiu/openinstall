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
        className="w-full cursor-pointer border border-black bg-white px-6 py-3 text-sm font-semibold text-black transition-all duration-200 hover:bg-black hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
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
      className="w-full cursor-pointer border-none bg-black px-6 py-3 text-sm font-semibold text-white transition-opacity duration-200 hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50"
    >
      {children}
    </button>
  );
}
