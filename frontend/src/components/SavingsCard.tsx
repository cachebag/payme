import { useState, useEffect } from "react";
import { Vault, Pencil, Check, X, HelpCircle } from "lucide-react";
import { api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { ProgressBar } from "./ui/ProgressBar";

interface SavingsCardProps {
  remaining: number;
  onAnalyzeClick?: () => void;
}

export function SavingsCard({ remaining, onAnalyzeClick }: SavingsCardProps) {
  const [savings, setSavings] = useState<number>(0);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");

  useEffect(() => {
    api.savings.get().then((res) => {
      setSavings(res.savings);
    });
  }, []);

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
    setIsEditing(false);
  };

  const target = savings + remaining;
  const percentage = target > 0 ? (savings / target) * 100 : 0;
  const difference = savings - target;
  const isAhead = difference >= 0;

  return (
    <Card className="!p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs text-charcoal-500 dark:text-charcoal-400">
          Savings
        </span>
        <Vault size={16} className="text-sage-600" />
      </div>
      
      {isEditing ? (
        <div className="flex items-center gap-1 mb-3">
          <Input
            type="number"
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            className="flex-1 !py-1 !text-base"
            autoFocus
          />
          <button
            onClick={saveEdit}
            className="p-1.5 text-sage-600 hover:bg-sage-100 dark:hover:bg-sage-900 transition-colors touch-manipulation"
          >
            <Check size={16} />
          </button>
          <button
            onClick={cancelEdit}
            className="p-1.5 text-charcoal-400 hover:bg-sand-100 dark:hover:bg-charcoal-800 transition-colors touch-manipulation"
          >
            <X size={16} />
          </button>
        </div>
      ) : (
        <div className="flex items-center justify-between mb-3">
          <span className="text-lg sm:text-xl font-semibold text-sage-700 dark:text-sage-400">
            ${savings.toFixed(2)}
          </span>
          <button
            onClick={startEdit}
            className="p-1.5 text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-200 transition-colors touch-manipulation"
          >
            <Pencil size={14} />
          </button>
        </div>
      )}

      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-charcoal-500 dark:text-charcoal-400">
            Target: ${target.toFixed(2)}
          </span>
          {onAnalyzeClick && (
            <button
              onClick={onAnalyzeClick}
              className="p-0.5 hover:bg-sand-200 dark:hover:bg-charcoal-700 rounded transition-colors touch-manipulation"
              title="Why this amount?"
            >
              <HelpCircle size={14} className="text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-300" />
            </button>
          )}
        </div>
        
        <ProgressBar value={savings} max={target} />
        
        <div className="flex items-center justify-between text-xs">
          <span className={`font-medium ${isAhead ? 'text-sage-600 dark:text-sage-400' : 'text-terracotta-600 dark:text-terracotta-400'}`}>
            {isAhead ? '✓' : '⚠️'} {Math.abs(percentage - 100).toFixed(1)}% {isAhead ? 'ahead' : 'behind'}
          </span>
          <span className="text-charcoal-500 dark:text-charcoal-400">
            {isAhead ? '+' : ''}{difference.toFixed(2)}
          </span>
        </div>
        
        <p className="text-xs text-charcoal-400 dark:text-charcoal-500 italic">
          based on remaining budget
        </p>
      </div>
    </Card>
  );
}
