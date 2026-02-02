import { useState, useEffect } from "react";
import { TrendingUp, Pencil, Check, X } from "lucide-react";
import { api, MonthlySavings } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { useCurrency } from "../context/CurrencyContext";

interface RetirementSavingsCardProps {
  monthId: number;
  initialSavings?: MonthlySavings | null;
  isReadOnly?: boolean;
  refreshTrigger?: number;
}

export function RetirementSavingsCard({ monthId, initialSavings, isReadOnly, refreshTrigger }: RetirementSavingsCardProps) {
  const [amount, setAmount] = useState<number>(initialSavings?.retirement_savings ?? 0);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");

  const { formatCurrency } = useCurrency();

  useEffect(() => {
    if (initialSavings) {
      return;
    }
    api.monthlySavings.get(monthId).then((res) => setAmount(res.retirement_savings));
  }, [monthId, initialSavings, refreshTrigger]);

  const startEdit = () => {
    if (isReadOnly) return;
    setEditValue(amount.toString());
    setIsEditing(true);
  };

  const cancelEdit = () => {
    setIsEditing(false);
    setEditValue("");
  };

  const saveEdit = async () => {
    const value = parseFloat(editValue);
    if (isNaN(value)) return;
    await api.monthlySavings.update(monthId, { retirement_savings: value });
    setAmount(value);
    setIsEditing(false);
  };

  return (
    <Card>
      <div className="flex items-start justify-between">
        <div>
          <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">
            Retirement Savings
          </div>
          {isEditing ? (
            <div className="flex items-center gap-2">
              <Input
                type="number"
                value={editValue}
                onChange={(e) => setEditValue(e.target.value)}
                className="w-28 !py-1"
                autoFocus
              />
              <button
                onClick={saveEdit}
                className="p-1 text-sage-600 hover:bg-sage-100 dark:hover:bg-sage-900 transition-colors"
              >
                <Check size={16} />
              </button>
              <button
                onClick={cancelEdit}
                className="p-1 text-charcoal-400 hover:bg-sand-100 dark:hover:bg-charcoal-800 transition-colors"
              >
                <X size={16} />
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <span className="text-xl font-semibold text-sage-600 dark:text-sage-400">
                {formatCurrency(amount)}
              </span>
              <button
                onClick={startEdit}
                className="p-1 text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-200 transition-colors"
              >
                <Pencil size={14} />
              </button>
            </div>
          )}
        </div>
        <TrendingUp size={20} className="text-sage-600 dark:text-sage-400" />
      </div>
    </Card>
  );
}