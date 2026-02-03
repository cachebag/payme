import { useState, useEffect } from "react";
import { TrendingUp, Pencil, Check, X } from "lucide-react";
import { api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { useCurrency } from "../context/CurrencyContext";

interface BreakdownItem {
  id: string;
  label: string;
  amount: number;
}

const STORAGE_KEY = "retirementBreakdown";

interface RetirementSavingsCardProps {
  monthId?: number;
  refreshTrigger?: number;
}

export function RetirementSavingsCard({ refreshTrigger }: RetirementSavingsCardProps) {
  const [amount, setAmount] = useState<number>(0);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");
  const [breakdownItems, setBreakdownItems] = useState<BreakdownItem[]>(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch {
        return [];
      }
    }
    return [];
  });

  const { formatCurrency } = useCurrency();

  useEffect(() => {
    api.retirementSavings.get().then((res) => setAmount(res.retirement_savings));
  }, [refreshTrigger]);

  useEffect(() => {
    const handleBreakdownUpdate = (event: Event) => {
      if (event instanceof CustomEvent) {
        setBreakdownItems(event.detail);
      }
    };

    window.addEventListener("retirementBreakdownUpdated", handleBreakdownUpdate);
    return () => window.removeEventListener("retirementBreakdownUpdated", handleBreakdownUpdate);
  }, []);

  const startEdit = () => {
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
    await api.retirementSavings.update(value);
    setAmount(value);
    setIsEditing(false);
  };

  const breakdownTotal = breakdownItems.reduce((sum, item) => sum + item.amount, 0);
  const totalAmount = amount + breakdownTotal;

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
                {formatCurrency(totalAmount)}
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