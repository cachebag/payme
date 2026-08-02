import { useState } from "react";
import { Plus, Trash2, Edit2, Check, X, Settings } from "lucide-react";
import { MonthlyFixedExpense, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Button } from "./ui/Button";
import { Modal } from "./ui/Modal";
import { ReorderControls } from "./ui/ReorderControls";
import { SortableHandle, SortableItem, SortableList } from "./ui/SortableList";
import { useCurrency } from "../context/CurrencyContext";
import { useSortableReorder } from "../hooks/useSortableReorder";

interface FixedExpensesProps {
  monthId: number;
  expenses: MonthlyFixedExpense[];
  isReadOnly?: boolean;
  onUpdate: () => void;
}

export function FixedExpenses({ monthId, expenses, isReadOnly, onUpdate }: FixedExpensesProps) {
  const { formatCurrency } = useCurrency();
  const [isManaging, setIsManaging] = useState(false);
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [label, setLabel] = useState("");
  const [amount, setAmount] = useState("");
  const {
    orderedItems: orderedExpenses,
    itemIds: expenseIds,
    handleDragEnd: handleExpenseDragEnd,
  } = useSortableReorder(expenses, async (nextExpenses) => {
    await api.monthlyFixedExpenses.reorder(monthId, nextExpenses.map((expense) => expense.id));
    await onUpdate();
  });

  const handleAdd = async () => {
    if (!label || !amount) return;
    await api.monthlyFixedExpenses.create(monthId, { label, amount: parseFloat(amount) });
    setLabel("");
    setAmount("");
    setIsAdding(false);
    await onUpdate();
  };

  const handleUpdate = async (id: number) => {
    if (!label || !amount) return;
    await api.monthlyFixedExpenses.update(monthId, id, { label, amount: parseFloat(amount) });
    setEditingId(null);
    setLabel("");
    setAmount("");
    await onUpdate();
  };

  const handleDelete = async (id: number) => {
    await api.monthlyFixedExpenses.delete(monthId, id);
    await onUpdate();
  };

  const handleMove = async (index: number, direction: -1 | 1) => {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= orderedExpenses.length) return;
    const next = [...orderedExpenses];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    await api.monthlyFixedExpenses.reorder(monthId, next.map((expense) => expense.id));
    await onUpdate();
  };

  const startEdit = (expense: MonthlyFixedExpense) => {
    if (isReadOnly) return;
    setEditingId(expense.id);
    setLabel(expense.label);
    setAmount(expense.amount.toString());
  };

  const cancelEdit = () => {
    setEditingId(null);
    setLabel("");
    setAmount("");
    setIsAdding(false);
  };

  const total = expenses.reduce((sum, e) => sum + e.amount, 0);

  return (
    <>
      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
            Fixed Expenses
          </h3>
          {!isReadOnly && (
            <button
              onClick={() => setIsManaging(true)}
              className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors"
            >
              <Settings size={16} />
            </button>
          )}
        </div>

        <div className="space-y-2">
          <SortableList ids={expenseIds} onDragEnd={handleExpenseDragEnd}>
            {orderedExpenses.map((expense, index) => (
              <SortableItem
                key={expense.id}
                id={expense.id}
                className="flex items-center justify-between py-2 border-b border-sand-200 dark:border-charcoal-800"
              >
                {({ attributes, listeners }) => (
                  <>
                    <div className="flex min-w-0 items-center gap-2">
                      {!isReadOnly && orderedExpenses.length > 1 && (
                        <>
                          <SortableHandle attributes={attributes} listeners={listeners} />
                          <ReorderControls
                            index={index}
                            total={orderedExpenses.length}
                            onMove={handleMove}
                          />
                        </>
                      )}
                      <span className="truncate text-sm text-charcoal-700 dark:text-sand-300">
                        {expense.label}
                      </span>
                    </div>
                    <span className="text-sm text-charcoal-600 dark:text-charcoal-400">
                      {formatCurrency(expense.amount)}
                    </span>
                  </>
                )}
              </SortableItem>
            ))}
          </SortableList>
          {expenses.length === 0 && (
            <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-4 text-center">
              No fixed expenses
            </div>
          )}
        </div>

        {expenses.length > 0 && (
          <div className="mt-4 pt-3 border-t border-sand-300 dark:border-charcoal-700 flex justify-between">
            <span className="text-sm font-medium text-charcoal-600 dark:text-sand-300">
              Total
            </span>
            <span className="text-sm font-semibold text-charcoal-800 dark:text-sand-100">
              {formatCurrency(total)}
            </span>
          </div>
        )}
      </Card>

      <Modal isOpen={isManaging} onClose={() => setIsManaging(false)} title="Manage Fixed Expenses">
        <div className="space-y-3">
          <SortableList ids={expenseIds} onDragEnd={handleExpenseDragEnd}>
            {orderedExpenses.map((expense, index) => (
              <SortableItem key={expense.id} id={expense.id}>
                {({ attributes, listeners }) =>
                  editingId === expense.id ? (
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
                          placeholder="Amount"
                          value={amount}
                          onChange={(e) => setAmount(e.target.value)}
                        />
                      </div>
                      <button
                        onClick={() => handleUpdate(expense.id)}
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
                    <div className="flex items-center justify-between gap-3 py-2 border-b border-sand-200 dark:border-charcoal-800">
                      <div className="flex min-w-0 items-center gap-2">
                        {orderedExpenses.length > 1 && (
                          <>
                            <SortableHandle attributes={attributes} listeners={listeners} />
                            <ReorderControls
                              index={index}
                              total={orderedExpenses.length}
                              onMove={handleMove}
                            />
                          </>
                        )}
                        <span className="truncate text-sm">{expense.label}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-sm">{formatCurrency(expense.amount)}</span>
                        <button
                          onClick={() => startEdit(expense)}
                          className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                        >
                          <Edit2 size={14} />
                        </button>
                        <button
                          onClick={() => handleDelete(expense.id)}
                          className="p-1 text-terracotta-500 hover:bg-terracotta-100 dark:hover:bg-charcoal-800"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>
                  )
                }
              </SortableItem>
            ))}
          </SortableList>

          {isAdding ? (
            <div className="flex items-end gap-2 pt-2">
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
                  placeholder="Amount"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                />
              </div>
              <Button size="sm" onClick={handleAdd}>
                <Check size={16} />
              </Button>
              <Button size="sm" variant="ghost" onClick={cancelEdit}>
                <X size={16} />
              </Button>
            </div>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setIsAdding(true)}
              className="w-full mt-2"
            >
              <Plus size={16} className="mr-2" />
              Add Expense
            </Button>
          )}
        </div>
      </Modal>
    </>
  );
}
