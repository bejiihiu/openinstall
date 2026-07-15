import CatalogClient from "@/components/CatalogClient";

async function getManifests() {
  const res = await fetch("/api/manifests", { cache: "no-store" });
  if (!res.ok) {
    throw new Error(`Failed to fetch manifests: ${res.status}`);
  }
  return res.json();
}

export default async function CatalogPage() {
  let manifests;
  try {
    manifests = await getManifests();
  } catch {
    return (
      <main className="max-w-[1200px] mx-auto px-6 py-12">
        <h1 className="text-[48px] font-black tracking-tight mb-8">Catalog</h1>
        <p className="text-red-600">Failed to load catalog. Please try again later.</p>
      </main>
    );
  }

  return (
    <main className="max-w-[1200px] mx-auto px-6 py-12">
      <h1 className="text-[48px] font-black tracking-tight mb-8">Catalog</h1>
      <CatalogClient manifests={manifests} />
    </main>
  );
}
