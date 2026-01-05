import { useState, useEffect } from "react";
import { TrendingUp, Pencil, Check, X } from "lucide-react";
import { api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";

export function RetirementSavingsCard() {
  const [amount, setAmount] = useState<number>(0);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");

  useEffect(() => {
    api.retirementSavings.get().then((res) => setAmount(res.retirement_savings));
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
                ${amount.toFixed(2)}
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

