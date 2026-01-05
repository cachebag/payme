import { useState, useEffect } from "react";
import { Vault, Pencil, Check, X } from "lucide-react";
import { api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";

interface SavingsCardProps {
  onSavingsChange?: (savings: number) => void;
}

export function SavingsCard({ onSavingsChange }: SavingsCardProps) {
  const [savings, setSavings] = useState<number>(0);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");

  useEffect(() => {
    api.savings.get().then((res) => {
      setSavings(res.savings);
      onSavingsChange?.(res.savings);
    });
  }, [onSavingsChange]);

  const startEdit = () => {
    setEditValue(savings.toString());
    setIsEditing(true);
  };

  const cancelEdit = () => {
    setIsEditing(false);
    setEditValue("");
  };

  const saveEdit = async () => {
    const value = parseFloat(editValue);
    if (isNaN(value)) return;
    await api.savings.update(value);
    setSavings(value);
    onSavingsChange?.(value);
    setIsEditing(false);
  };

  return (
    <Card className="!p-3">
      <div className="flex items-center justify-between mb-1">
        <span className="text-xs text-charcoal-500 dark:text-charcoal-400">
          Savings
        </span>
        <Vault size={14} className="text-sage-600" />
      </div>
      {isEditing ? (
        <div className="flex items-center gap-1">
          <Input
            type="number"
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            className="flex-1 !py-1 !text-sm"
            autoFocus
          />
          <button
            onClick={saveEdit}
            className="p-1 text-sage-600 hover:bg-sage-100 dark:hover:bg-sage-900 transition-colors"
          >
            <Check size={14} />
          </button>
          <button
            onClick={cancelEdit}
            className="p-1 text-charcoal-400 hover:bg-sand-100 dark:hover:bg-charcoal-800 transition-colors"
          >
            <X size={14} />
          </button>
        </div>
      ) : (
        <div className="flex items-center justify-between">
          <span className="text-sm font-semibold text-sage-700 dark:text-sage-400">
            ${savings.toFixed(2)}
          </span>
          <button
            onClick={startEdit}
            className="p-1 text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-200 transition-colors"
          >
            <Pencil size={12} />
          </button>
        </div>
      )}
    </Card>
  );
}

