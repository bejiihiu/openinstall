import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  weight: ["300", "400", "500", "600", "700", "900"],
});

export function generateMetadata(): Metadata {
  return {
    title: "OpenInstall — Cross-Distro Linux App Installer",
    description: "Install Linux apps across any distro with a single command.",
  };
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={inter.variable}>
      <body className="min-h-screen antialiased" style={{ paddingTop: "64px" }}>
        <Navbar />
        <main className="mx-auto max-w-[var(--max-width)] px-4">{children}</main>
        <Footer />
      </body>
    </html>
  );
}
