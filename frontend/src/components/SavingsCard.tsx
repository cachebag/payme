import { useState, useEffect } from "react";
import { Vault, Pencil, Check, X, Info } from "lucide-react";
import { api, MonthlySavings } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { ProgressBar } from "./ui/ProgressBar";
import { Modal } from "./ui/Modal";
import { Button } from "./ui/Button";
import { useCurrency } from "../context/CurrencyContext";

interface SavingsCardProps {
  monthId: number;
  initialSavings?: MonthlySavings | null;
  isReadOnly?: boolean;
  onSavingsChange?: (savings: number) => void;
  refreshTrigger?: number;
}

export function SavingsCard({ monthId, initialSavings, isReadOnly, onSavingsChange, refreshTrigger }: SavingsCardProps) {
  const [savings, setSavings] = useState<number>(initialSavings?.savings ?? 0);
  const [savingsGoal, setSavingsGoal] = useState<number>(initialSavings?.savings_goal ?? 0);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");
  const [editGoalValue, setEditGoalValue] = useState("");
  const [showInfoModal, setShowInfoModal] = useState(false);
  const [isEditingGoal, setIsEditingGoal] = useState(false);

  const { formatCurrency } = useCurrency();

  useEffect(() => {
    // If we have initial savings data from the month summary, use it
    if (initialSavings) {
      setSavings(initialSavings.savings);
      setSavingsGoal(initialSavings.savings_goal);
      onSavingsChange?.(initialSavings.savings);
    } else {
      // Otherwise fetch month-specific savings
      api.monthlySavings.get(monthId).then((res) => {
        setSavings(res.savings);
        setSavingsGoal(res.savings_goal);
        onSavingsChange?.(res.savings);
      });
    }
  }, [monthId, initialSavings, onSavingsChange, refreshTrigger]);

  const startEdit = () => {
    if (isReadOnly) return;
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
    await api.monthlySavings.update(monthId, { savings: value });
    setSavings(value);
    onSavingsChange?.(value);
    setIsEditing(false);
  };

  const startEditGoal = () => {
    if (isReadOnly) return;
    setEditGoalValue(savingsGoal.toString());
    setIsEditingGoal(true);
  };

  const cancelEditGoal = () => {
    setIsEditingGoal(false);
    setEditGoalValue("");
  };

  const saveEditGoal = async () => {
    const value = parseFloat(editGoalValue);
    if (isNaN(value) || value < 0) return;
    await api.monthlySavings.update(monthId, { savings_goal: value });
    setSavingsGoal(value);
    setIsEditingGoal(false);
  };

  const target = savingsGoal > 0 ? savingsGoal : 0;
  const percentage = target > 0 ? (savings / target) * 100 : 0;
  const difference = savings - target;
  const isAhead = difference >= 0;

  return (
    <>
    <Card className="!p-4">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-1.5">
          <span className="text-xs text-charcoal-500 dark:text-charcoal-400">
            Savings
          </span>
          <button
            onClick={() => setShowInfoModal(true)}
            className="p-0.5 hover:bg-sand-200 dark:hover:bg-charcoal-700 rounded transition-colors touch-manipulation"
            title="How this works"
          >
            <Info size={12} className="text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-300" />
          </button>
        </div>
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
            {formatCurrency(savings)}
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
        {isEditingGoal ? (
          <div className="flex items-center gap-1">
            <span className="text-xs text-charcoal-500 dark:text-charcoal-400">Goal:</span>
            <Input
              type="number"
              value={editGoalValue}
              onChange={(e) => setEditGoalValue(e.target.value)}
              className="flex-1 !py-0.5 !text-xs"
              autoFocus
            />
            <button
              onClick={saveEditGoal}
              className="p-0.5 text-sage-600 hover:bg-sage-100 dark:hover:bg-sage-900 transition-colors"
            >
              <Check size={12} />
            </button>
            <button
              onClick={cancelEditGoal}
              className="p-0.5 text-charcoal-400 hover:bg-sand-100 dark:hover:bg-charcoal-800 transition-colors"
            >
              <X size={12} />
            </button>
          </div>
        ) : (
          <div className="flex items-center justify-between text-xs">
            <span className="text-charcoal-500 dark:text-charcoal-400">
              Goal: {formatCurrency(target)}
            </span>
            <button
              onClick={startEditGoal}
              className="p-0.5 text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-200 transition-colors"
            >
              <Pencil size={10} />
            </button>
          </div>
        )}
        
        <ProgressBar value={savings} max={target} />
        
        <div className="flex items-center justify-between text-xs">
          <span className={`font-medium ${isAhead ? 'text-sage-600 dark:text-sage-400' : 'text-terracotta-600 dark:text-terracotta-400'}`}>
            {isAhead ? '✓' : '⚠️'} {Math.abs(percentage - 100).toFixed(1)}% {isAhead ? 'ahead' : 'behind'}
          </span>
          <span className="text-charcoal-500 dark:text-charcoal-400">
            {isAhead ? '+' : ''}{formatCurrency(difference, { showSymbol: false })}
          </span>
        </div>
        
        <p className="text-xs text-charcoal-400 dark:text-charcoal-500 italic">
          {savingsGoal > 0 ? 'based on your goal' : 'no goal set'}
        </p>
      </div>
    </Card>

    <Modal isOpen={showInfoModal} onClose={() => setShowInfoModal(false)} title="How Savings Tracking Works">
      <div className="space-y-4">
        <div>
          <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200 mb-2">
            Monthly Savings Snapshot
          </h3>
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300">
            Each month tracks its own savings balance and goal. When you create a new month, it starts with your current savings values, 
            but changes won't affect other months.
          </p>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200 mb-2">
            Current Savings
          </h3>
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300">
            Your savings balance for this specific month. Update it to track your progress over time.
          </p>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200 mb-2">
            Savings Goal
          </h3>
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300 mb-2">
            Set a target amount for this month. You can adjust goals month-by-month as your targets change.
          </p>
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300">
            The progress bar will only show when you have a goal set (greater than 0).
          </p>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200 mb-2">
            Progress Tracking
          </h3>
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300">
            The progress bar shows how close you are to your goal. Green means you're ahead, red means you're behind.
          </p>
        </div>

        <div className="pt-4 border-t border-sand-300 dark:border-charcoal-700">
          <Button onClick={() => setShowInfoModal(false)} className="w-full sm:w-auto">
            Got it
          </Button>
        </div>
      </div>
    </Modal>
    </>
  );
}
