"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";

const navLinks = [
  { href: "/catalog", label: "Catalog" },
  { href: "/publish", label: "Publish" },
  { href: "/dashboard", label: "Dashboard" },
  { href: "/auth/login", label: "Login" },
];

export default function Navbar() {
  const pathname = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <nav
      className="fixed top-0 left-0 w-full bg-white border-b border-black z-50"
      style={{ height: "64px" }}
    >
      <div
        className="mx-auto flex items-center justify-between h-full"
        style={{ maxWidth: "1200px", padding: "0 24px" }}
      >
        <Link href="/" className="font-bold text-lg" style={{ fontWeight: 700 }}>
          OpenInstall
        </Link>

        <div className="hidden md:flex items-center gap-0">
          {navLinks.map((link) => {
            const isActive = pathname === link.href;
            return (
              <Link
                key={link.href}
                href={link.href}
                className="px-4 py-2 text-sm transition-colors duration-200"
                style={{
                  fontWeight: isActive ? 600 : 400,
                  backgroundColor: isActive ? "#000000" : "transparent",
                  color: isActive ? "#FFFFFF" : "#000000",
                  border: "1px solid #000000",
                  borderRight: "none",
                }}
              >
                {link.label}
              </Link>
            );
          })}
          {navLinks.length > 0 && (
            <div
              style={{
                width: "1px",
                height: "100%",
                backgroundColor: "#000000",
                alignSelf: "stretch",
              }}
            />
          )}
        </div>

        <button
          className="md:hidden p-2 border border-black"
          onClick={() => setMenuOpen(!menuOpen)}
          aria-label="Toggle menu"
          style={{ backgroundColor: menuOpen ? "#000000" : "#FFFFFF" }}
        >
          <div className="flex flex-col gap-1">
            <span
              className="block w-5 h-0.5"
              style={{
                backgroundColor: menuOpen ? "#FFFFFF" : "#000000",
                transform: menuOpen ? "rotate(45deg) translateY(3px)" : "none",
              }}
            />
            <span
              className="block w-5 h-0.5"
              style={{
                backgroundColor: menuOpen ? "#FFFFFF" : "#000000",
                opacity: menuOpen ? 0 : 1,
              }}
            />
            <span
              className="block w-5 h-0.5"
              style={{
                backgroundColor: menuOpen ? "#FFFFFF" : "#000000",
                transform: menuOpen ? "rotate(-45deg) translateY(-3px)" : "none",
              }}
            />
          </div>
        </button>
      </div>

      {menuOpen && (
        <div className="md:hidden bg-white border-t border-black">
          {navLinks.map((link) => {
            const isActive = pathname === link.href;
            return (
              <Link
                key={link.href}
                href={link.href}
                className="block px-6 py-3 border-b border-black text-sm"
                style={{
                  fontWeight: isActive ? 600 : 400,
                  backgroundColor: isActive ? "#000000" : "#FFFFFF",
                  color: isActive ? "#FFFFFF" : "#000000",
                }}
                onClick={() => setMenuOpen(false)}
              >
                {link.label}
              </Link>
            );
          })}
        </div>
      )}
    </nav>
  );
}
