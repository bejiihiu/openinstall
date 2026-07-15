"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Button from "@/components/Button";
import Input from "@/components/Input";

interface PackageUrls {
  ubuntu: string;
  arch: string;
  fedora: string;
  opensuse: string;
  flatpak: string;
  appimage: string;
}

export default function PublishPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState<{ id: string } | null>(null);

  const [name, setName] = useState("");
  const [publisher, setPublisher] = useState("");
  const [version, setVersion] = useState("");
  const [description, setDescription] = useState("");
  const [homepage, setHomepage] = useState("");
  const [license, setLicense] = useState("");
  const [sha256, setSha256] = useState("");
  const [apiKey, setApiKey] = useState("");

  const [packages, setPackages] = useState<PackageUrls>({
    ubuntu: "",
    arch: "",
    fedora: "",
    opensuse: "",
    flatpak: "",
    appimage: "",
  });

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) {
      router.push("/auth/login");
    } else {
      setLoading(false);
    }
  }, [router]);

  function updatePackage(key: keyof PackageUrls, value: string) {
    setPackages((prev) => ({ ...prev, [key]: value }));
  }

  function hasAtLeastOnePackage(): boolean {
    return Object.values(packages).some((v) => v.trim() !== "");
  }

  function buildPackagesPayload(): Record<string, string> {
    const result: Record<string, string> = {};
    if (packages.ubuntu) result["ubuntu"] = packages.ubuntu;
    if (packages.arch) result["arch"] = packages.arch;
    if (packages.fedora) result["fedora"] = packages.fedora;
    if (packages.opensuse) result["opensuse"] = packages.opensuse;
    if (packages.flatpak) result["flatpak"] = packages.flatpak;
    if (packages.appimage) result["appimage"] = packages.appimage;
    return result;
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");

    if (!hasAtLeastOnePackage()) {
      setError("Provide at least one package URL.");
      return;
    }

    setSubmitting(true);

    try {
      const res = await fetch("/api/manifests", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          name,
          publisher,
          version,
          description,
          homepage: homepage || undefined,
          license: license || undefined,
          sha256: sha256 || undefined,
          packages: buildPackagesPayload(),
        }),
      });

      if (!res.ok) {
        const data = await res.json().catch(() => null);
        throw new Error(data?.error || `Request failed (${res.status})`);
      }

      const data = await res.json();
      setSuccess({ id: data.id || data._id });
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Something went wrong.");
    } finally {
      setSubmitting(false);
    }
  }

  if (loading) return null;

  if (success) {
    return (
      <main className="pt-[80px] px-6 pb-16">
        <div className="max-w-[600px] mx-auto">
          <h1 className="text-[48px] font-black leading-none tracking-tight mb-8">
            Published
          </h1>
          <p className="text-[18px] leading-relaxed mb-8">
            Your app has been submitted.
          </p>
          <a
            href={`/app/${success.id}`}
            className="inline-block border border-black px-6 py-3 font-semibold hover:bg-black hover:text-white transition-colors duration-200"
          >
            View your app →
          </a>
        </div>
      </main>
    );
  }

  const packageFields: { key: keyof PackageUrls; label: string }[] = [
    { key: "ubuntu", label: "Ubuntu / Debian (.deb)" },
    { key: "arch", label: "Arch Linux (.pkg.tar.zst)" },
    { key: "fedora", label: "Fedora / RHEL (.rpm)" },
    { key: "opensuse", label: "openSUSE (.rpm)" },
    { key: "flatpak", label: "Flatpak" },
    { key: "appimage", label: "AppImage (fallback)" },
  ];

  return (
    <main className="pt-[80px] px-6 pb-16">
      <div className="max-w-[600px] mx-auto">
        <h1 className="text-[48px] font-black leading-none tracking-tight mb-8">
          Publish an app
        </h1>
        <p className="text-[18px] leading-relaxed mb-12">
          Submit your application to the OpenInstall catalog.
        </p>

        <form onSubmit={handleSubmit}>
          {/* Required fields */}
          <div className="mb-6">
            <label className="block text-[14px] font-semibold mb-2">
              App name *
            </label>
            <Input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
          </div>

          <div className="mb-6">
            <label className="block text-[14px] font-semibold mb-2">
              Publisher *
            </label>
            <Input
              type="text"
              value={publisher}
              onChange={(e) => setPublisher(e.target.value)}
              required
            />
          </div>

          <div className="mb-6">
            <label className="block text-[14px] font-semibold mb-2">
              Version *
            </label>
            <Input
              type="text"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
              required
            />
          </div>

          <div className="mb-6">
            <label className="block text-[14px] font-semibold mb-2">
              Description *
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              required
              rows={3}
              className="w-full bg-white border border-black px-4 py-3 text-[16px] font-[inherit] resize-y min-h-[120px] outline-none focus:border-2 focus:px-[11px] focus:py-[11px]"
            />
          </div>

          {/* Optional fields */}
          <div className="mb-6">
            <label className="block text-[14px] font-semibold mb-2">
              Homepage URL
            </label>
            <Input
              type="url"
              value={homepage}
              onChange={(e) => setHomepage(e.target.value)}
            />
          </div>

          <div className="mb-6">
            <label className="block text-[14px] font-semibold mb-2">
              License
            </label>
            <Input
              type="text"
              value={license}
              onChange={(e) => setLicense(e.target.value)}
              placeholder="e.g. MIT, GPL-2.0"
            />
          </div>

          <div className="mb-6">
            <label className="block text-[14px] font-semibold mb-2">
              SHA256 hash
            </label>
            <Input
              type="text"
              value={sha256}
              onChange={(e) => setSha256(e.target.value)}
            />
          </div>

          {/* Package URLs */}
          <div className="border border-black p-6 mt-8">
            <p className="text-[14px] font-semibold mb-6">
              Package URLs * (at least one required)
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {packageFields.map(({ key, label }) => (
                <div key={key}>
                  <label className="block text-[14px] font-semibold mb-2">
                    {label}
                  </label>
                  <Input
                    type="url"
                    value={packages[key]}
                    onChange={(e) => updatePackage(key, e.target.value)}
                  />
                </div>
              ))}
            </div>
          </div>

          {/* API Key */}
          <div className="mt-8 mb-8">
            <label className="block text-[14px] font-semibold mb-2">
              API Key *
            </label>
            <Input
              type="text"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              required
            />
          </div>

          {/* Error */}
          {error && (
            <div className="border border-black p-4 mb-6 text-[14px]">
              {error}
            </div>
          )}

          {/* Submit */}
          <Button type="submit" disabled={submitting}>
            {submitting ? "Publishing…" : "Publish"}
          </Button>
        </form>
      </div>
    </main>
  );
}
