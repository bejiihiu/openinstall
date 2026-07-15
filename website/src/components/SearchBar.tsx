interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export default function SearchBar({
  value,
  onChange,
  placeholder = "Search applications...",
}: SearchBarProps) {
  return (
    <input
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      className="w-full border border-black px-4 py-3 text-base bg-white text-black outline-none focus:border-2 focus:px-[15px] focus:py-[11px] transition-all duration-200"
    />
  );
}
