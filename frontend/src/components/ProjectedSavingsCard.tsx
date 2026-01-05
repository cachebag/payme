import { TrendingUp } from "lucide-react";
import { Card } from "./ui/Card";

interface ProjectedSavingsCardProps {
  savings: number;
  remaining: number;
}

export function ProjectedSavingsCard({ savings, remaining }: ProjectedSavingsCardProps) {
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
          ${projected.toFixed(2)}
        </span>
      </div>
    </Card>
  );
}

