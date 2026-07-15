import CopyButton from "@/components/CopyButton";
import { notFound } from "next/navigation";

interface Manifest {
  _id: string;
  name: string;
  publisher: string;
  description: string;
  version: string;
  homepage?: string;
  license?: string;
  packages: Record<string, string>;
  sha256?: string;
  signature?: string;
}

const PLATFORM_LABELS: Record<string, string> = {
  ubuntu: "Ubuntu / Debian",
  arch: "Arch Linux",
  fedora: "Fedora / RHEL",
  opensuse: "openSUSE",
  flatpak: "Flatpak",
  appimage: "AppImage",
  fallback: "AppImage (fallback)",
};

async function getManifest(id: string): Promise<Manifest> {
  const res = await fetch(`/api/manifests/${id}`, {
    next: { revalidate: 60 },
  });
  if (!res.ok) {
    notFound();
  }
  return res.json();
}

export default async function AppDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const manifest = await getManifest(id);
  const manifestUrl = `/api/manifests/${manifest._id}`;
  const installCmd = `installer install ${manifestUrl}`;
  const platforms = Object.keys(manifest.packages);

  return (
    <main className="max-w-[1200px] mx-auto px-6 pt-24 pb-32">
      <h1
        className="font-black tracking-tight"
        style={{
          fontSize: "clamp(2rem, 5vw, 6rem)",
          letterSpacing: "-0.05em",
          fontWeight: 900,
          lineHeight: 1,
        }}
      >
        {manifest.name}
      </h1>

      <p className="text-lg font-medium mt-2">{manifest.publisher}</p>

      <div className="mt-4 inline-block border border-black px-3 py-1 text-sm font-semibold">
        v{manifest.version}
      </div>

      <p
        className="text-lg leading-relaxed mt-6 max-w-[600px]"
        style={{ lineHeight: 1.6 }}
      >
        {manifest.description}
      </p>

      {manifest.homepage && (
        <a
          href={manifest.homepage}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-block mt-4 text-base font-semibold border-b border-black"
          style={{ color: "black" }}
        >
          {manifest.homepage}
        </a>
      )}

      <section className="mt-12">
        <h2 className="text-2xl font-bold mb-4">Install</h2>
        <div
          className="border border-black p-4 flex items-center justify-between gap-4"
          style={{ fontFamily: "var(--font-geist-mono), monospace" }}
        >
          <code className="text-sm whitespace-pre-wrap break-all">
            {installCmd}
          </code>
          <CopyButton text={installCmd} label="Copy" />
        </div>
      </section>

      <section className="mt-12">
        <h2 className="text-2xl font-bold mb-4">Supported platforms</h2>
        <div className="flex flex-wrap gap-3">
          {platforms.map((platform) => (
            <div
              key={platform}
              className="border border-black px-4 py-2 font-medium"
            >
              {PLATFORM_LABELS[platform] ?? platform}
            </div>
          ))}
        </div>
      </section>

      {manifest.signature && (
        <section className="mt-12">
          <div
            className="inline-flex items-center border-2 border-black px-4 py-2 font-semibold"
            style={{
              borderColor: "black",
              borderWidth: "2px",
            }}
          >
            Verified
          </div>
        </section>
      )}
    </main>
  );
}
