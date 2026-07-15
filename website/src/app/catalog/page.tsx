import CatalogClient from "@/components/CatalogClient";

const mockManifests = [
  { _id: "1", name: "Cursor", publisher: "Anysphere", description: "AI Code Editor", version: "1.5.0" },
  { _id: "2", name: "OBS Studio", publisher: "OBS Project", description: "Video recording and streaming", version: "30.0.0" },
  { _id: "3", name: "VS Code", publisher: "Microsoft", description: "Code editor", version: "1.85.0" },
  { _id: "4", name: "Firefox", publisher: "Mozilla", description: "Web browser", version: "121.0" },
  { _id: "5", name: "GIMP", publisher: "GNU", description: "Image editor", version: "2.10.36" },
  { _id: "6", name: "Blender", publisher: "Blender Foundation", description: "3D creation suite", version: "4.0.2" },
];

async function getManifests() {
  const apiUrl = process.env.NEXT_PUBLIC_API_URL;
  if (!apiUrl) return mockManifests;
  try {
    const res = await fetch(`${apiUrl}/api/manifests`, { cache: "no-store" });
    if (!res.ok) return mockManifests;
    return res.json();
  } catch {
    return mockManifests;
  }
}

export default async function CatalogPage() {
  const manifests = await getManifests();

  return (
    <main className="max-w-[1200px] mx-auto px-6 py-12">
      <h1 className="text-[48px] font-black tracking-tight mb-8">Catalog</h1>
      <CatalogClient manifests={manifests} />
    </main>
  );
}
