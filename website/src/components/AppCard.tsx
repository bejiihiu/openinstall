import Link from "next/link";

interface AppCardProps {
  id: string;
  name: string;
  publisher: string;
  description: string;
  version: string;
}

export default function AppCard({
  id,
  name,
  publisher,
  description,
  version,
}: AppCardProps) {
  return (
    <Link
      href={`/app/${id}`}
      className="block bg-white border border-black p-6 hover:bg-black hover:text-white transition-colors duration-200 group"
    >
      <h3 className="text-xl font-bold">{name}</h3>
      <p className="text-sm font-medium mt-2">{publisher}</p>
      <p className="text-sm font-normal mt-2 leading-relaxed">{description}</p>
      <span className="inline-block text-xs font-semibold border border-current px-2 py-0.5 mt-3">
        v{version}
      </span>
    </Link>
  );
}
