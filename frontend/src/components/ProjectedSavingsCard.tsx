import { TrendingUp, HelpCircle } from "lucide-react";
import { Card } from "./ui/Card";
import { useCurrency } from "../context/CurrencyContext";

interface ProjectedSavingsCardProps {
  savings: number;
  remaining: number;
  onAnalyzeClick?: () => void;
}

export function ProjectedSavingsCard({ savings, remaining, onAnalyzeClick }: ProjectedSavingsCardProps) {
  const { formatCurrency } = useCurrency();
  const projected = savings + remaining;

  return (
    <Card className="!p-3">
      <div className="flex items-center justify-between mb-1">
        <span className="text-xs text-charcoal-500 dark:text-charcoal-400">
          Projected
        </span>
        <TrendingUp size={14} className="text-sage-600" />
      </div>
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold text-sage-700 dark:text-sage-400">
          {formatCurrency(projected)}
        </span>
        {onAnalyzeClick && (
          <button
            onClick={onAnalyzeClick}
            className="p-0.5 hover:bg-sand-200 dark:hover:bg-charcoal-700 rounded transition-colors"
            title="Why this amount?"
          >
            <HelpCircle size={14} className="text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-300" />
          </button>
        )}
      </div>
    </Card>
  );
}

