import { ChevronDown, ChevronUp } from "lucide-react";

interface ReorderControlsProps {
  index: number;
  total: number;
  onMove: (index: number, direction: -1 | 1) => void;
  className?: string;
}

export function ReorderControls({
  index,
  total,
  onMove,
  className = "",
}: ReorderControlsProps) {
  return (
    <div className={`flex items-center gap-0.5 ${className}`}>
      <button
        type="button"
        onClick={() => onMove(index, -1)}
        disabled={index === 0}
        className="rounded p-1 text-charcoal-500 transition-colors hover:bg-sand-200 hover:text-charcoal-800 disabled:pointer-events-none disabled:opacity-25 dark:text-charcoal-400 dark:hover:bg-charcoal-800 dark:hover:text-sand-200"
        title="Move up"
      >
        <ChevronUp size={14} />
      </button>
      <button
        type="button"
        onClick={() => onMove(index, 1)}
        disabled={index >= total - 1}
        className="rounded p-1 text-charcoal-500 transition-colors hover:bg-sand-200 hover:text-charcoal-800 disabled:pointer-events-none disabled:opacity-25 dark:text-charcoal-400 dark:hover:bg-charcoal-800 dark:hover:text-sand-200"
        title="Move down"
      >
        <ChevronDown size={14} />
      </button>
    </div>
  );
}
