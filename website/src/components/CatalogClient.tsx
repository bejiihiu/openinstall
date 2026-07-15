"use client";

import { useState } from "react";
import SearchBar from "./SearchBar";
import AppCard from "./AppCard";

interface Manifest {
  _id: string;
  name: string;
  publisher: string;
  description: string;
  version: string;
}

export default function CatalogClient({ manifests }: { manifests: Manifest[] }) {
  const [query, setQuery] = useState("");

  const filtered = manifests.filter((m) => {
    const q = query.toLowerCase();
    return (
      m.name.toLowerCase().includes(q) ||
      m.publisher.toLowerCase().includes(q)
    );
  });

  return (
    <>
      <SearchBar value={query} onChange={setQuery} />
      {filtered.length === 0 ? (
        <p className="text-center text-lg mt-8">No results found.</p>
      ) : (
        <div
          className="mt-6 grid gap-6"
          style={{ gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))" }}
        >
          {filtered.map((m) => (
            <AppCard
              key={m._id}
              id={m._id}
              name={m.name}
              publisher={m.publisher}
              description={m.description}
              version={m.version}
            />
          ))}
        </div>
      )}
    </>
  );
}
