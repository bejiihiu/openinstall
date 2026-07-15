import Link from "next/link";

interface AppCardProps {
  id: string;
  name: string;
  publisher: string;
  description: string;
  version: string;
}

export default function AppCard({ id, name, publisher, description, version }: AppCardProps) {
  return (
    <Link href={`/app/${id}`} className="block">
      <article
        className="bg-white border border-black p-6 transition-opacity duration-200 hover:opacity-80"
        style={{ padding: "24px" }}
      >
        <div className="flex items-start justify-between mb-2">
          <h3 className="text-lg" style={{ fontWeight: 700 }}>
            {name}
          </h3>
          <span
            className="text-xs border border-black px-2 py-0.5 ml-2 shrink-0"
            style={{ fontWeight: 500 }}
          >
            v{version}
          </span>
        </div>
        <p className="text-xs mb-2" style={{ opacity: 0.6 }}>
          {publisher}
        </p>
        <p className="text-sm" style={{ opacity: 0.8 }}>
          {description}
        </p>
      </article>
    </Link>
  );
}
