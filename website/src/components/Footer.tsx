import Link from "next/link";

export default function Footer() {
  return (
    <footer className="bg-white border-t border-black" style={{ padding: "24px" }}>
      <div
        className="mx-auto flex items-center justify-between"
        style={{ maxWidth: "1200px" }}
      >
        <p className="text-sm">&copy; 2026 OpenInstall</p>
        <Link
          href="https://github.com/bejiihiu/openinstall"
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm underline hover:opacity-70 transition-opacity duration-200"
        >
          GitHub
        </Link>
      </div>
    </footer>
  );
}
