import { useState } from "react";
import { Plus, Trash2, Edit2, Check, X, Settings } from "lucide-react";
import { MonthlyBudgetWithCategory, BudgetCategory, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Button } from "./ui/Button";
import { ProgressBar } from "./ui/ProgressBar";
import { Modal } from "./ui/Modal";
import { ReorderControls } from "./ui/ReorderControls";
import { SortableHandle, SortableItem, SortableList } from "./ui/SortableList";
import { useCurrency } from "../context/CurrencyContext";
import { useSortableReorder } from "../hooks/useSortableReorder";

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
  const {
    orderedItems: orderedBudgets,
    itemIds: budgetIds,
    handleDragEnd: handleBudgetDragEnd,
  } = useSortableReorder(budgets, async (nextBudgets) => {
    await api.categories.reorder(nextBudgets.map((budget) => budget.category_id));
    await onUpdate();
  });
  const {
    orderedItems: orderedCategories,
    itemIds: categoryIds,
    handleDragEnd: handleCategoryDragEnd,
  } = useSortableReorder(categories, async (nextCategories) => {
    await api.categories.reorder(nextCategories.map((category) => category.id));
    await onUpdate();
  });

  const PRESET_COLORS = [
    "#71717a",
    "#ef4444",
    "#f97316",
    "#f59e0b",
    "#10b981",
    "#06b6d4",
    "#3b82f6",
    "#6366f1",
    "#8b5cf6",
    "#d946ef",
  ];

  const handleAddCategory = async () => {
    if (!label || !amount) return;
    await api.categories.create({
      label,
      default_amount: parseFloat(amount),
      color,
      month_id: monthId,
    });
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

  const handleDeleteCategory = async (categoryId: number, label: string) => {
    const confirmed = confirm(
      `Stop using "${label}" from this month on? Earlier months keep it, along with anything already spent.`
    );
    if (!confirmed) return;
    await api.categories.delete(monthId, categoryId);
    await onUpdate();
  };

  const handleUpdateBudget = async (budgetId: number) => {
    if (!amount) return;
    await api.budgets.update(monthId, budgetId, parseFloat(amount));
    setEditingBudgetId(null);
    setAmount("");
    await onUpdate();
  };

  const handleMoveBudget = async (index: number, direction: -1 | 1) => {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= orderedBudgets.length) return;
    const next = [...orderedBudgets];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    await api.categories.reorder(next.map((budget) => budget.category_id));
    await onUpdate();
  };

  const handleMoveCategory = async (index: number, direction: -1 | 1) => {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= orderedCategories.length) return;
    const next = [...orderedCategories];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    await api.categories.reorder(next.map((category) => category.id));
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
          <SortableList ids={budgetIds} onDragEnd={handleBudgetDragEnd}>
            {orderedBudgets.map((budget, index) => (
              <SortableItem key={budget.id} id={budget.id}>
                {({ attributes, listeners }) =>
                  editingBudgetId === budget.id && !isReadOnly ? (
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
                        <div className="flex min-w-0 items-center gap-2">
                          {!isReadOnly && orderedBudgets.length > 1 && (
                            <>
                              <SortableHandle attributes={attributes} listeners={listeners} />
                              <ReorderControls
                                index={index}
                                total={orderedBudgets.length}
                                onMove={handleMoveBudget}
                              />
                            </>
                          )}
                          <div
                            className="w-2 h-2 rounded-sm"
                            style={{ backgroundColor: budget.category_color }}
                          />
                          <span className="truncate text-sm text-charcoal-700 dark:text-sand-300">
                            {budget.category_label}
                          </span>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className="text-xs text-charcoal-500 dark:text-charcoal-400">
                            {formatCurrency(budget.spent_amount)} / {formatCurrency(budget.allocated_amount)}
                          </span>
                          {!isReadOnly && (
                            <>
                              <button
                                onClick={() => startEditBudget(budget)}
                                className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                              >
                                <Edit2 size={12} />
                              </button>
                              <button
                                onClick={() =>
                                  handleDeleteCategory(budget.category_id, budget.category_label)
                                }
                                title="Stop using this category"
                                className="p-1 text-terracotta-500 hover:bg-terracotta-100 dark:hover:bg-charcoal-800"
                              >
                                <Trash2 size={12} />
                              </button>
                            </>
                          )}
                        </div>
                      </div>
                      <ProgressBar value={budget.spent_amount} max={budget.allocated_amount} />
                    </div>
                  )
                }
              </SortableItem>
            ))}
          </SortableList>
          {budgets.length === 0 && (
            <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-4 text-center">
              No budget categories
            </div>
          )}
        </div>
      </Card>

      <Modal isOpen={isManaging} onClose={() => setIsManaging(false)} title="Manage Categories">
        <p className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-4">
          Categories define your budget types. Default amounts apply to new months. To stop
          using one, remove it from the budget list &mdash; the months before it keep their
          history.
        </p>
        <div className="space-y-3">
          <SortableList ids={categoryIds} onDragEnd={handleCategoryDragEnd}>
            {orderedCategories.map((cat, index) => (
              <SortableItem key={cat.id} id={cat.id}>
                {({ attributes, listeners }) =>
                  editingCategoryId === cat.id ? (
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
                            className={`w-6 h-6 rounded-sm border-2 transition-all ${
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
                    <div className="flex items-center justify-between gap-3 py-2 border-b border-sand-200 dark:border-charcoal-800">
                      <div className="flex min-w-0 items-center gap-2">
                        {orderedCategories.length > 1 && (
                          <>
                            <SortableHandle attributes={attributes} listeners={listeners} />
                            <ReorderControls
                              index={index}
                              total={orderedCategories.length}
                              onMove={handleMoveCategory}
                            />
                          </>
                        )}
                        <div
                          className="w-3 h-3 rounded-sm"
                          style={{ backgroundColor: cat.color }}
                        />
                        <span className="truncate text-sm">{cat.label}</span>
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
                      </div>
                    </div>
                  )
                }
              </SortableItem>
            ))}
          </SortableList>

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
                    className={`w-6 h-6 rounded-sm border-2 transition-all ${
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
