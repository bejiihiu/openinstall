"use client";

import { useState } from "react";

interface InstallButtonProps {
  manifestUrl: string;
  appName: string;
}

export default function InstallButton({ manifestUrl, appName }: InstallButtonProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    const command = `installer install ${manifestUrl}`;
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      className="cursor-pointer transition-opacity duration-200"
      style={{
        backgroundColor: "#000000",
        color: "#FFFFFF",
        border: "1px solid #000000",
        padding: "12px 24px",
        fontWeight: 600,
        fontSize: "14px",
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLButtonElement).style.opacity = "0.8";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLButtonElement).style.opacity = "1";
      }}
    >
      {copied ? "Copied!" : `Install ${appName}`}
    </button>
  );
}
