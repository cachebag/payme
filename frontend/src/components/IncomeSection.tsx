import { useState } from "react";
import { Plus, Trash2, Edit2, Check, X } from "lucide-react";
import { IncomeEntry, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Button } from "./ui/Button";
import { useCurrency } from "../context/CurrencyContext";

interface IncomeSectionProps {
  monthId: number;
  entries: IncomeEntry[];
  isReadOnly: boolean;
  onUpdate: () => void;
}

export function IncomeSection({ monthId, entries, isReadOnly, onUpdate }: IncomeSectionProps) {
  const { formatCurrency } = useCurrency();
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [label, setLabel] = useState("");
  const [amount, setAmount] = useState("");

  const handleAdd = async () => {
    if (!label || !amount) return;
    await api.income.create(monthId, { label, amount: parseFloat(amount) });
    setLabel("");
    setAmount("");
    setIsAdding(false);
    await onUpdate();
  };

  const handleUpdate = async (id: number) => {
    if (!label || !amount) return;
    await api.income.update(monthId, id, { label, amount: parseFloat(amount) });
    setEditingId(null);
    setLabel("");
    setAmount("");
    await onUpdate();
  };

  const handleDelete = async (id: number) => {
    await api.income.delete(monthId, id);
    await onUpdate();
  };

  const startEdit = (entry: IncomeEntry) => {
    setEditingId(entry.id);
    setLabel(entry.label);
    setAmount(entry.amount.toString());
  };

  const cancelEdit = () => {
    setEditingId(null);
    setLabel("");
    setAmount("");
    setIsAdding(false);
  };

  return (
    <Card>
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
          Income
        </h3>
        {!isReadOnly && !isAdding && (
          <button
            onClick={() => setIsAdding(true)}
            className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors"
          >
            <Plus size={16} />
          </button>
        )}
      </div>

      <div className="space-y-3">
        {entries.map((entry) => (
          <div key={entry.id}>
            {editingId === entry.id ? (
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
                  onClick={() => handleUpdate(entry.id)}
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
              <div className="flex items-center justify-between py-2 border-b border-sand-200 dark:border-charcoal-800">
                <span className="text-sm text-charcoal-700 dark:text-sand-300">
                  {entry.label}
                </span>
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-sage-600 dark:text-sage-400">
                    {formatCurrency(entry.amount)}
                  </span>
                  {!isReadOnly && (
                    <>
                      <button
                        onClick={() => startEdit(entry)}
                        className="p-1 opacity-0 group-hover:opacity-100 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-all"
                      >
                        <Edit2 size={14} />
                      </button>
                      <button
                        onClick={() => handleDelete(entry.id)}
                        className="p-1 text-terracotta-500 hover:bg-terracotta-100 dark:hover:bg-charcoal-800 transition-colors"
                      >
                        <Trash2 size={14} />
                      </button>
                    </>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}

        {isAdding && (
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
        )}

        {entries.length === 0 && !isAdding && (
          <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-4 text-center">
            No income entries
          </div>
        )}
      </div>
    </Card>
  );
}

