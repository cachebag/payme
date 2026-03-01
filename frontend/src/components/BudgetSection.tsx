import { useState } from "react";
import { Plus, Trash2, Edit2, Check, X, Settings } from "lucide-react";
import { MonthlyBudgetWithCategory, BudgetCategory, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Button } from "./ui/Button";
import { ProgressBar } from "./ui/ProgressBar";
import { Modal } from "./ui/Modal";
import { useCurrency } from "../context/CurrencyContext";

interface BudgetSectionProps {
  monthId: number;
  budgets: MonthlyBudgetWithCategory[];
  categories: BudgetCategory[];
  isReadOnly: boolean;
  onUpdate: () => void;
}

export function BudgetSection({
  monthId,
  budgets,
  categories,
  isReadOnly,
  onUpdate,
}: BudgetSectionProps) {
  const { formatCurrency } = useCurrency();
  const [isManaging, setIsManaging] = useState(false);
  const [isAddingCategory, setIsAddingCategory] = useState(false);
  const [editingCategoryId, setEditingCategoryId] = useState<number | null>(null);
  const [editingBudgetId, setEditingBudgetId] = useState<number | null>(null);
  const [label, setLabel] = useState("");
  const [amount, setAmount] = useState("");
  const [color, setColor] = useState("#71717a");

  const PRESET_COLORS = [
    "#71717a", // Zinc
    "#ef4444", // Red
    "#f97316", // Orange
    "#f59e0b", // Amber
    "#10b981", // Emerald
    "#06b6d4", // Cyan
    "#3b82f6", // Blue
    "#6366f1", // Indigo
    "#8b5cf6", // Violet
    "#d946ef", // Fuchsia
  ];

  const handleAddCategory = async () => {
    if (!label || !amount) return;
    await api.categories.create({ label, default_amount: parseFloat(amount), color });
    setLabel("");
    setAmount("");
    setColor("#71717a");
    setIsAddingCategory(false);
    await onUpdate();
  };

  const handleUpdateCategory = async (id: number) => {
    if (!label || !amount) return;
    await api.categories.update(id, { label, default_amount: parseFloat(amount), color });
    setEditingCategoryId(null);
    setLabel("");
    setAmount("");
    setColor("#71717a");
    await onUpdate();
  };

  const handleDeleteCategory = async (id: number) => {
    await api.categories.delete(id);
    await onUpdate();
  };

  const handleUpdateBudget = async (budgetId: number) => {
    if (!amount) return;
    await api.budgets.update(monthId, budgetId, parseFloat(amount));
    setEditingBudgetId(null);
    setAmount("");
    await onUpdate();
  };

  const startEditCategory = (cat: BudgetCategory) => {
    setEditingCategoryId(cat.id);
    setLabel(cat.label);
    setAmount(cat.default_amount.toString());
    setColor(cat.color);
  };

  const startEditBudget = (budget: MonthlyBudgetWithCategory) => {
    setEditingBudgetId(budget.id);
    setAmount(budget.allocated_amount.toString());
  };

  const cancelEdit = () => {
    setEditingCategoryId(null);
    setEditingBudgetId(null);
    setLabel("");
    setAmount("");
    setColor("#71717a");
    setIsAddingCategory(false);
  };

  return (
    <>
      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
            Budget
          </h3>
          <button
            onClick={() => setIsManaging(true)}
            className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors"
          >
            <Settings size={16} />
          </button>
        </div>

        <div className="space-y-4">
          {budgets.map((budget) => (
            <div key={budget.id}>
              {editingBudgetId === budget.id && !isReadOnly ? (
                <div className="flex items-end gap-2">
                  <div className="flex-1">
                    <div className="text-sm mb-1">{budget.category_label}</div>
                  </div>
                  <div className="w-24">
                    <Input
                      type="number"
                      placeholder="Budget"
                      value={amount}
                      onChange={(e) => setAmount(e.target.value)}
                    />
                  </div>
                  <button
                    onClick={() => handleUpdateBudget(budget.id)}
                    className="p-2 text-sage-600 hover:bg-sage-100 dark:hover:bg-charcoal-800"
                  >
                    <Check size={16} />
                  </button>
                  <button
                    onClick={cancelEdit}
                    className="p-2 text-charcoal-500 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                  >
                    <X size={16} />
                  </button>
                </div>
              ) : (
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-2">
                      <div
                        className="w-2 h-2 rounded-full"
                        style={{ backgroundColor: budget.category_color }}
                      />
                      <span className="text-sm text-charcoal-700 dark:text-sand-300">
                        {budget.category_label}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-charcoal-500 dark:text-charcoal-400">
                        {formatCurrency(budget.spent_amount)} / {formatCurrency(budget.allocated_amount)}
                      </span>
                      {!isReadOnly && (
                        <button
                          onClick={() => startEditBudget(budget)}
                          className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                        >
                          <Edit2 size={12} />
                        </button>
                      )}
                    </div>
                  </div>
                  <ProgressBar value={budget.spent_amount} max={budget.allocated_amount} />
                </div>
              )}
            </div>
          ))}
          {budgets.length === 0 && (
            <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-4 text-center">
              No budget categories
            </div>
          )}
        </div>
      </Card>

      <Modal isOpen={isManaging} onClose={() => setIsManaging(false)} title="Manage Categories">
        <p className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-4">
          Categories define your budget types. Default amounts apply to new months.
        </p>
        <div className="space-y-3">
          {categories.map((cat) => (
            <div key={cat.id}>
              {editingCategoryId === cat.id ? (
                <div className="space-y-3 p-3 bg-sand-100 dark:bg-charcoal-900/50 rounded-lg">
                  <div className="flex items-end gap-2">
                    <div className="flex-1">
                      <Input
                        placeholder="Label"
                        value={label}
                        onChange={(e) => setLabel(e.target.value)}
                      />
                    </div>
                    <div className="w-24">
                      <Input
                        type="number"
                        placeholder="Default"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                      />
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-1.5">
                    {PRESET_COLORS.map((c) => (
                      <button
                        key={c}
                        onClick={() => setColor(c)}
                        className={`w-6 h-6 rounded-full border-2 transition-all ${
                          color === c ? "border-charcoal-800 dark:border-sand-200 scale-110" : "border-transparent hover:scale-105"
                        }`}
                        style={{ backgroundColor: c }}
                      />
                    ))}
                  </div>
                  <div className="flex justify-end gap-2">
                    <button
                      onClick={() => handleUpdateCategory(cat.id)}
                      className="p-2 text-sage-600 hover:bg-sage-100 dark:hover:bg-charcoal-800"
                    >
                      <Check size={16} />
                    </button>
                    <button
                      onClick={cancelEdit}
                      className="p-2 text-charcoal-500 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                    >
                      <X size={16} />
                    </button>
                  </div>
                </div>
              ) : (
                <div className="flex items-center justify-between py-2 border-b border-sand-200 dark:border-charcoal-800">
                  <div className="flex items-center gap-2">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: cat.color }}
                    />
                    <span className="text-sm">{cat.label}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-charcoal-500">
                      {formatCurrency(cat.default_amount)}
                    </span>
                    <button
                      onClick={() => startEditCategory(cat)}
                      className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                    >
                      <Edit2 size={14} />
                    </button>
                    <button
                      onClick={() => handleDeleteCategory(cat.id)}
                      className="p-1 text-terracotta-500 hover:bg-terracotta-100 dark:hover:bg-charcoal-800"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}

          {isAddingCategory ? (
            <div className="space-y-3 p-3 bg-sand-100 dark:bg-charcoal-900/50 rounded-lg pt-2">
              <div className="flex items-end gap-2">
                <div className="flex-1">
                  <Input
                    placeholder="Category name"
                    value={label}
                    onChange={(e) => setLabel(e.target.value)}
                  />
                </div>
                <div className="w-24">
                  <Input
                    type="number"
                    placeholder="Default"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                  />
                </div>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {PRESET_COLORS.map((c) => (
                  <button
                    key={c}
                    onClick={() => setColor(c)}
                    className={`w-6 h-6 rounded-full border-2 transition-all ${
                      color === c ? "border-charcoal-800 dark:border-sand-200 scale-110" : "border-transparent hover:scale-105"
                    }`}
                    style={{ backgroundColor: c }}
                  />
                ))}
              </div>
              <div className="flex justify-end gap-2">
                <Button size="sm" onClick={handleAddCategory}>
                  <Check size={16} className="mr-1" /> Add
                </Button>
                <Button size="sm" variant="ghost" onClick={cancelEdit}>
                  <X size={16} className="mr-1" /> Cancel
                </Button>
              </div>
            </div>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setIsAddingCategory(true)}
              className="w-full mt-2"
            >
              <Plus size={16} className="mr-2" />
              Add Category
            </Button>
          )}
        </div>
      </Modal>
    </>
  );
}

