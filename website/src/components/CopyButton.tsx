"use client";

import { useState } from "react";

interface CopyButtonProps {
  text: string;
  label?: string;
}

export default function CopyButton({ text, label }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      className="bg-black text-white px-6 py-3 font-semibold cursor-pointer transition-opacity duration-200 hover:opacity-80"
      style={{ borderRadius: 0 }}
    >
      {copied ? "Copied!" : label ?? "Copy"}
    </button>
  );
}
