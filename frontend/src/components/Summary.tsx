import { TrendingDown, Wallet, CreditCard, PiggyBank } from "lucide-react";
import { Card } from "./ui/Card";
import { ReactNode } from "react";

interface SummaryProps {
  totalIncome: number;
  totalFixed: number;
  totalSpent: number;
  remaining: number;
  extraCard?: ReactNode;
}

export function Summary({ totalIncome, totalFixed, totalSpent, remaining, extraCard }: SummaryProps) {
  const isPositive = remaining >= 0;

  const items = [
    {
      label: "Income",
      value: totalIncome,
      icon: Wallet,
      color: "text-sage-600 dark:text-sage-400",
    },
    {
      label: "Fixed",
      value: totalFixed,
      icon: CreditCard,
      color: "text-charcoal-600 dark:text-charcoal-400",
    },
  ];

  const itemsAfter = [
    {
      label: "Spent",
      value: totalSpent,
      icon: TrendingDown,
      color: "text-terracotta-600 dark:text-terracotta-400",
    },
    {
      label: "Remaining",
      value: remaining,
      icon: isPositive ? PiggyBank : TrendingDown,
      color: isPositive
        ? "text-sage-600 dark:text-sage-400"
        : "text-terracotta-600 dark:text-terracotta-400",
    },
  ];

  const renderCard = (item: typeof items[0]) => (
    <Card key={item.label}>
      <div className="flex items-start justify-between">
        <div>
          <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">
            {item.label}
          </div>
          <div className={`text-lg sm:text-xl font-semibold ${item.color}`}>
            ${Math.abs(item.value).toFixed(2)}
            {item.label === "Remaining" && item.value < 0 && (
              <span className="text-xs ml-1">deficit</span>
            )}
          </div>
        </div>
        <item.icon size={18} className={`${item.color} sm:w-5 sm:h-5`} />
      </div>
    </Card>
  );

  return (
    <div className={`grid grid-cols-2 gap-3 sm:gap-4 ${extraCard ? "lg:grid-cols-5" : "lg:grid-cols-4"}`}>
      {items.map(renderCard)}
      {extraCard}
      {itemsAfter.map(renderCard)}
    </div>
  );
}
