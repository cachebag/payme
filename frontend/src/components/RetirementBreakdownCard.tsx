import { useState, useEffect, useId } from "react";
import { Trash2, Edit2, Check, X } from "lucide-react";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Button } from "./ui/Button";
import { useCurrency } from "../context/CurrencyContext";
import { useUIPreferences } from "../context/UIPreferencesContext";

interface BreakdownItem {
  id: string;
  label: string;
  amount: number;
}

const STORAGE_KEY = "retirementBreakdown";

export function RetirementBreakdownCard() {
  const { formatCurrency } = useCurrency();
  const { retirementBreakdownEnabled } = useUIPreferences();
  const [breakdownItems, setBreakdownItems] = useState<BreakdownItem[]>([]);
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [label, setLabel] = useState("");
  const [amount, setAmount] = useState("");

  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        setBreakdownItems(JSON.parse(stored));
      } catch {
        setBreakdownItems([]);
      }
    }
  }, []);

  const saveItems = (items: BreakdownItem[]) => {
    setBreakdownItems(items);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
    window.dispatchEvent(new CustomEvent('retirementBreakdownUpdated', { detail: items }));
  };

  const idPrefix = useId();
  let idCounter = 0;

  const handleAdd = () => {
    if (!label || !amount) return;
    const newItem: BreakdownItem = {
      id: `${idPrefix}-${++idCounter}`,
      label,
      amount: parseFloat(amount),
    };
    saveItems([...breakdownItems, newItem]);
    resetForm();
  };

  const handleUpdate = (id: string) => {
    if (!label || !amount) return;
    const updated = breakdownItems.map((item) =>
      item.id === id ? { ...item, label, amount: parseFloat(amount) } : item
    );
    saveItems(updated);
    resetForm();
  };

  const handleDelete = (id: string) => {
    saveItems(breakdownItems.filter((item) => item.id !== id));
  };

  const startEdit = (item: BreakdownItem) => {
    setEditingId(item.id);
    setLabel(item.label);
    setAmount(item.amount.toString());
  };

  const resetForm = () => {
    setEditingId(null);
    setLabel("");
    setAmount("");
    setIsAdding(false);
  };

  if (!retirementBreakdownEnabled && breakdownItems.length === 0) {
    return null;
  }

  return (
    <Card>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
            Retirement Savings Breakdown
          </h3>
        </div>
        {!isAdding && retirementBreakdownEnabled && (
          <button
            onClick={() => {
              setIsAdding(true);
            }}
            className="p-2 md:p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 active:bg-sand-300 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
          >
            <span className="text-lg">+</span>
          </button>
        )}
      </div>

      {isAdding && (
        <div className="mb-4 p-4 bg-sand-100 dark:bg-charcoal-800">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <Input
              placeholder="Account/Source"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
            />
            <Input
              type="number"
              placeholder="Amount"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
            />
          </div>
          <div className="flex gap-2 mt-3">
            <Button size="sm" onClick={handleAdd}>
              <Check size={16} className="mr-1" />
              Add
            </Button>
            <Button size="sm" variant="ghost" onClick={resetForm}>
              <X size={16} className="mr-1" />
              Cancel
            </Button>
          </div>
        </div>
      )}

      <div className="overflow-x-auto -mx-4 px-4">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-sand-300 dark:border-charcoal-700">
              <th className="text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm">
                Account/Source
              </th>
              <th className="text-right py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm">
                Amount
              </th>
              {retirementBreakdownEnabled && <th className="w-16 md:w-20"></th>}
            </tr>
          </thead>
          <tbody>
            {breakdownItems.map((item) => {
              return (
                <tr
                  key={item.id}
                  className="border-b border-sand-200 dark:border-charcoal-800 hover:bg-sand-100 dark:hover:bg-charcoal-900/50 active:bg-sand-200 dark:active:bg-charcoal-900 transition-colors"
                >
                  {editingId === item.id ? (
                    <>
                      <td className="py-2">
                        <Input
                          placeholder="Label"
                          value={label}
                          onChange={(e) => setLabel(e.target.value)}
                          className="text-xs"
                        />
                      </td>
                      <td className="py-2">
                        <Input
                          type="number"
                          placeholder="Amount"
                          value={amount}
                          onChange={(e) => setAmount(e.target.value)}
                          className="text-xs text-right"
                        />
                      </td>
                      <td className="py-2">
                        <div className="flex gap-0.5 md:gap-1 justify-end">
                          <button
                            onClick={() => handleUpdate(item.id)}
                            className="p-2 md:p-1 text-sage-600 hover:bg-sage-100 dark:hover:bg-charcoal-800 active:bg-sage-200 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
                          >
                            <Check size={14} />
                          </button>
                          <button
                            onClick={resetForm}
                            className="p-2 md:p-1 text-charcoal-500 hover:bg-sand-200 dark:hover:bg-charcoal-800 active:bg-sand-300 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
                          >
                            <X size={14} />
                          </button>
                        </div>
                      </td>
                    </>
                  ) : (
                    <>
                      <td className="py-2 px-1 text-charcoal-800 dark:text-sand-200 text-xs md:text-sm font-medium">
                        {item.label}
                      </td>
                      <td className="py-2 px-1 text-right font-medium text-xs md:text-sm whitespace-nowrap text-sage-600 dark:text-sage-400">
                        {formatCurrency(item.amount)}
                      </td>
                      {retirementBreakdownEnabled && (
                        <td className="py-2 px-1">
                          <div className="flex gap-0.5 md:gap-1 justify-end">
                            <button
                              onClick={() => startEdit(item)}
                              className="p-2 md:p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 active:bg-sand-300 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
                            >
                              <Edit2 size={14} />
                            </button>
                            <button
                              onClick={() => handleDelete(item.id)}
                              className="p-2 md:p-1 text-terracotta-500 hover:bg-terracotta-100 dark:hover:bg-charcoal-800 active:bg-terracotta-200 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
                            >
                              <Trash2 size={14} />
                            </button>
                          </div>
                        </td>
                      )}
                    </>
                  )}
                </tr>
              );
            })}
          </tbody>
        </table>

        {breakdownItems.length === 0 && (
          <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-8 text-center">
            No breakdown items. Add items to track what makes up your retirement savings.
          </div>
        )}
      </div>
    </Card>
  );
}
