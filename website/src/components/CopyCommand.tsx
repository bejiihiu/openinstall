"use client";

import { useState } from "react";

const COMMAND =
  "curl -sSf https://raw.githubusercontent.com/bejiihiu/openinstall/main/scripts/install.sh | sh";

export function CopyCommand() {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(COMMAND);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        border: "1px solid #000000",
        padding: "16px",
        borderRadius: 0,
        marginTop: "32px",
        maxWidth: "640px",
      }}
    >
      <code
        style={{
          fontFamily: "var(--font-geist-mono), monospace",
          fontSize: "14px",
          color: "#000000",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          flex: 1,
          marginRight: "16px",
        }}
      >
        {COMMAND}
      </code>
      <button
        onClick={handleCopy}
        style={{
          backgroundColor: copied ? "#000000" : "#FFFFFF",
          color: copied ? "#FFFFFF" : "#000000",
          border: "1px solid #000000",
          padding: "8px 16px",
          fontWeight: 600,
          cursor: "pointer",
          borderRadius: 0,
          transition: "all 200ms",
          whiteSpace: "nowrap",
          flexShrink: 0,
        }}
        onMouseEnter={(e) => {
          if (!copied) {
            e.currentTarget.style.backgroundColor = "#000000";
            e.currentTarget.style.color = "#FFFFFF";
          }
        }}
        onMouseLeave={(e) => {
          if (!copied) {
            e.currentTarget.style.backgroundColor = "#FFFFFF";
            e.currentTarget.style.color = "#000000";
          }
        }}
      >
        {copied ? "Copied" : "Copy"}
      </button>
    </div>
  );
}
