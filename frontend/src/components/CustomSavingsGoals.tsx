import { useState, useEffect } from "react";
import { Target, Pencil, Check, X, Trash2, Plus } from "lucide-react";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { ProgressBar } from "./ui/ProgressBar";
import { Button } from "./ui/Button";
import { ReorderControls } from "./ui/ReorderControls";
import { SortableHandle, SortableItem, SortableList } from "./ui/SortableList";
import { useCurrency } from "../context/CurrencyContext";
import { api, CustomSavingsGoal } from "../api/client";
import { useSortableReorder } from "../hooks/useSortableReorder";

export function CustomSavingsGoals() {
  const { formatCurrency, getCurrencySymbol } = useCurrency();

  const [goals, setGoals] = useState<CustomSavingsGoal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isAddingNew, setIsAddingNew] = useState(false);
  const [newGoalName, setNewGoalName] = useState("");
  const [newGoalTarget, setNewGoalTarget] = useState("");
  const [editingGoalId, setEditingGoalId] = useState<number | null>(null);
  const [editCurrentAmount, setEditCurrentAmount] = useState("");
  const {
    orderedItems: orderedGoals,
    itemIds: goalIds,
    handleDragEnd: handleGoalDragEnd,
  } = useSortableReorder(goals, async (nextGoals) => {
    await api.savingsGoals.reorder(nextGoals.map((goal) => goal.id));
    setGoals(nextGoals);
  });

  useEffect(() => {
    api.savingsGoals
      .list()
      .then((data) => {
        setGoals(data);
      })
      .catch((e) => {
        console.error("Failed to load savings goals:", e);
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, []);

  const addNewGoal = async () => {
    const target = parseFloat(newGoalTarget);
    if (!newGoalName.trim() || isNaN(target) || target <= 0) return;

    try {
      const created = await api.savingsGoals.create({
        name: newGoalName.trim(),
        current_amount: 0,
        target_amount: target,
      });
      setGoals((prev) => [...prev, created]);
      setNewGoalName("");
      setNewGoalTarget("");
      setIsAddingNew(false);
    } catch (e) {
      console.error("Failed to create savings goal:", e);
    }
  };

  const startEditAmount = (goalId: number, currentAmount: number) => {
    setEditingGoalId(goalId);
    setEditCurrentAmount(currentAmount.toString());
  };

  const cancelEditAmount = () => {
    setEditingGoalId(null);
    setEditCurrentAmount("");
  };

  const saveEditAmount = async (goalId: number) => {
    const amount = parseFloat(editCurrentAmount);
    if (isNaN(amount) || amount < 0) return;

    try {
      const updated = await api.savingsGoals.update(goalId, { current_amount: amount });
      setGoals((prev) => prev.map((g) => (g.id === goalId ? updated : g)));
    } catch (e) {
      console.error("Failed to update savings goal:", e);
    }
    setEditingGoalId(null);
    setEditCurrentAmount("");
  };

  const deleteGoal = async (goalId: number) => {
    if (!confirm("Are you sure you want to delete this savings goal?")) return;

    try {
      await api.savingsGoals.delete(goalId);
      setGoals((prev) => prev.filter((g) => g.id !== goalId));
    } catch (e) {
      console.error("Failed to delete savings goal:", e);
    }
  };

  const moveGoal = async (index: number, direction: -1 | 1) => {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= orderedGoals.length) return;
    const next = [...orderedGoals];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];

    try {
      await api.savingsGoals.reorder(next.map((goal) => goal.id));
      setGoals(next);
    } catch (e) {
      console.error("Failed to reorder savings goals:", e);
    }
  };

  const cancelAddNew = () => {
    setIsAddingNew(false);
    setNewGoalName("");
    setNewGoalTarget("");
  };

  return (
    <Card className="!p-4">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Target size={16} className="text-sage-600" />
          <span className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
            Custom Savings Goals
          </span>
        </div>
        {!isAddingNew && (
          <button
            onClick={() => setIsAddingNew(true)}
            className="p-1.5 text-sage-600 hover:bg-sage-100 dark:hover:bg-sage-900 rounded transition-colors touch-manipulation"
            title="Add new goal"
          >
            <Plus size={16} />
          </button>
        )}
      </div>

      <div className="space-y-4">
        {isLoading && (
          <p className="text-xs text-charcoal-400 dark:text-charcoal-500 italic text-center py-4">
            Loading...
          </p>
        )}

        {!isLoading && goals.length === 0 && !isAddingNew && (
          <p className="text-xs text-charcoal-400 dark:text-charcoal-500 italic text-center py-4">
            No custom savings goals yet. Click + to add one!
          </p>
        )}

        <SortableList ids={goalIds} onDragEnd={handleGoalDragEnd}>
          {orderedGoals.map((goal, index) => {
          const percentage =
            goal.target_amount > 0
              ? (goal.current_amount / goal.target_amount) * 100
              : 0;
          const isComplete = goal.current_amount >= goal.target_amount;
          const remaining = goal.target_amount - goal.current_amount;

          return (
            <SortableItem
              key={goal.id}
              id={goal.id}
              className="border border-sand-300 dark:border-charcoal-700 rounded-lg p-3 space-y-2"
            >
              {({ attributes, listeners }) => (
                <div className="flex items-start justify-between gap-2">
                  {orderedGoals.length > 1 && (
                    <div className="flex items-center gap-0.5">
                      <SortableHandle attributes={attributes} listeners={listeners} />
                      <ReorderControls
                        index={index}
                        total={orderedGoals.length}
                        onMove={moveGoal}
                      />
                    </div>
                  )}
                  <div className="flex-1">
                    <h4 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200 mb-1">
                      {goal.name}
                    </h4>

                    {editingGoalId === goal.id ? (
                      <div className="flex items-center gap-1 mb-2">
                        <span className="text-xs text-charcoal-500 dark:text-charcoal-400">
                          {getCurrencySymbol()}
                        </span>
                        <Input
                          type="number"
                          value={editCurrentAmount}
                          onChange={(e) => setEditCurrentAmount(e.target.value)}
                          className="flex-1 !py-1 !text-sm"
                          autoFocus
                          placeholder="Current amount"
                        />
                        <button
                          onClick={() => saveEditAmount(goal.id)}
                          className="p-1 text-sage-600 hover:bg-sage-100 dark:hover:bg-sage-900 transition-colors"
                        >
                          <Check size={14} />
                        </button>
                        <button
                          onClick={cancelEditAmount}
                          className="p-1 text-charcoal-400 hover:bg-sand-100 dark:hover:bg-charcoal-800 transition-colors"
                        >
                          <X size={14} />
                        </button>
                      </div>
                    ) : (
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-sm font-medium text-sage-700 dark:text-sage-400">
                          {formatCurrency(goal.current_amount)}
                        </span>
                        <span className="text-xs text-charcoal-400 dark:text-charcoal-500">
                          / {formatCurrency(goal.target_amount)}
                        </span>
                        <button
                          onClick={() => startEditAmount(goal.id, goal.current_amount)}
                          className="p-1 text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-200 transition-colors ml-auto"
                          title="Edit current amount"
                        >
                          <Pencil size={12} />
                        </button>
                        <button
                          onClick={() => deleteGoal(goal.id)}
                          className="p-1 text-terracotta-500 hover:text-terracotta-600 dark:hover:text-terracotta-400 transition-colors"
                          title="Delete goal"
                        >
                          <Trash2 size={12} />
                        </button>
                      </div>
                    )}

                    <ProgressBar
                      value={goal.current_amount}
                      max={goal.target_amount}
                      showOverage={false}
                      invertColors={true}
                    />

                    <div className="flex items-center justify-between mt-2">
                      <span
                        className={`text-xs font-medium ${
                          isComplete
                            ? "text-sage-600 dark:text-sage-400"
                            : "text-charcoal-500 dark:text-charcoal-400"
                        }`}
                      >
                        {isComplete ? "✓ Complete!" : `${percentage.toFixed(1)}% complete`}
                      </span>
                      {!isComplete && (
                        <span className="text-xs text-charcoal-400 dark:text-charcoal-500">
                          {formatCurrency(remaining)} to go
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              )}
            </SortableItem>
          );
          })}
        </SortableList>

        {isAddingNew && (
          <div className="border-2 border-dashed border-sage-300 dark:border-sage-700 rounded-lg p-3 space-y-3">
            <div>
              <label className="block text-xs font-medium text-charcoal-600 dark:text-charcoal-300 mb-1">
                Goal Name
              </label>
              <Input
                type="text"
                value={newGoalName}
                onChange={(e) => setNewGoalName(e.target.value)}
                placeholder="e.g., Emergency Fund, Vacation, New Car"
                className="w-full"
                autoFocus
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-charcoal-600 dark:text-charcoal-300 mb-1">
                Target Amount
              </label>
              <Input
                type="number"
                value={newGoalTarget}
                onChange={(e) => setNewGoalTarget(e.target.value)}
                placeholder="0.00"
                className="w-full"
              />
            </div>

            <div className="flex gap-2">
              <Button
                onClick={addNewGoal}
                className="flex-1"
                disabled={
                  !newGoalName.trim() ||
                  !newGoalTarget ||
                  parseFloat(newGoalTarget) <= 0
                }
              >
                Add Goal
              </Button>
              <Button
                onClick={cancelAddNew}
                className="flex-1 bg-sand-200 text-charcoal-700 hover:bg-sand-300 dark:bg-charcoal-700 dark:text-sand-200 dark:hover:bg-charcoal-600"
              >
                Cancel
              </Button>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
