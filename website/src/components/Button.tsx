"use client";

import React from "react";

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
  const baseStyles =
    "px-6 py-3 font-semibold cursor-pointer transition-all duration-200";

  const variantStyles =
    variant === "primary"
      ? "bg-black text-white border-none hover:opacity-80"
      : "bg-white text-black border border-black hover:bg-black hover:text-white";

  const disabledStyles = disabled ? "opacity-50 cursor-not-allowed" : "";

  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={`${baseStyles} ${variantStyles} ${disabledStyles}`}
    >
      {children}
    </button>
  );
}