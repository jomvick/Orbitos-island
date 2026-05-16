interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBar({ value, onChange }: SearchBarProps) {
  return (
    <div className="relative">
      <span className="absolute left-3 top-1/2 -translate-y-1/2 text-white/20 text-xs">
        \u2315
      </span>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Search sessions, agents, repos..."
        className="w-full bg-white/[0.04] border border-white/[0.06]
          rounded-lg pl-7 pr-3 py-1.5
          text-xs text-white/60 placeholder-white/20
          outline-none focus:border-white/[0.12] focus:bg-white/[0.06]
          transition-colors"
      />
      {value && (
        <button
          onClick={() => onChange("")}
          className="absolute right-2 top-1/2 -translate-y-1/2
            text-white/20 hover:text-white/40 text-xs transition-colors"
        >
          \u2715
        </button>
      )}
    </div>
  );
}
