import { useState } from "react";
import { Plus, Trash2, Edit2, Check, X } from "lucide-react";
import { IncomeEntry, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Button } from "./ui/Button";
import { ReorderControls } from "./ui/ReorderControls";
import { SortableHandle, SortableItem, SortableList } from "./ui/SortableList";
import { useCurrency } from "../context/CurrencyContext";
import { useSortableReorder } from "../hooks/useSortableReorder";

interface IncomeSectionProps {
  monthId: number;
  entries: IncomeEntry[];
  isReadOnly: boolean;
  onUpdate: () => void;
}

export function IncomeSection({ monthId, entries, isReadOnly, onUpdate }: IncomeSectionProps) {
  const { formatCurrency } = useCurrency();
  const today = new Date().toISOString().split("T")[0];
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [label, setLabel] = useState("");
  const [amount, setAmount] = useState("");
  const [paidOn, setPaidOn] = useState(today);
  const {
    orderedItems: orderedEntries,
    itemIds: entryIds,
    handleDragEnd: handleEntryDragEnd,
  } = useSortableReorder(entries, async (nextEntries) => {
    await api.income.reorder(monthId, nextEntries.map((entry) => entry.id));
    await onUpdate();
  });

  const handleAdd = async () => {
    if (!label || !amount) return;
    await api.income.create(monthId, {
      label,
      amount: parseFloat(amount),
      paid_on: paidOn || null,
    });
    setLabel("");
    setAmount("");
    setPaidOn(today);
    setIsAdding(false);
    await onUpdate();
  };

  const handleUpdate = async (id: number) => {
    if (!label || !amount) return;
    await api.income.update(monthId, id, {
      label,
      amount: parseFloat(amount),
      paid_on: paidOn || null,
    });
    setEditingId(null);
    setLabel("");
    setAmount("");
    setPaidOn(today);
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
    setPaidOn(entry.paid_on ?? "");
  };

  const cancelEdit = () => {
    setEditingId(null);
    setLabel("");
    setAmount("");
    setPaidOn(today);
    setIsAdding(false);
  };

  const handleMove = async (index: number, direction: -1 | 1) => {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= orderedEntries.length) return;
    const next = [...orderedEntries];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    await api.income.reorder(monthId, next.map((entry) => entry.id));
    await onUpdate();
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
        <SortableList ids={entryIds} onDragEnd={handleEntryDragEnd}>
          {orderedEntries.map((entry, index) => (
            <SortableItem key={entry.id} id={entry.id}>
              {({ attributes, listeners }) =>
                editingId === entry.id ? (
                  <div className="flex flex-wrap items-end gap-2">
                    <div className="min-w-0 flex-[1_1_10rem]">
                      <Input
                        placeholder="Label"
                        value={label}
                        onChange={(e) => setLabel(e.target.value)}
                      />
                    </div>
                    <div className="w-28">
                      <Input
                        type="number"
                        placeholder="Amount"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                      />
                    </div>
                    <div className="w-44">
                      <Input
                        type="date"
                        value={paidOn}
                        onChange={(e) => setPaidOn(e.target.value)}
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
                  <div className="flex items-center justify-between gap-3 py-2 border-b border-sand-200 dark:border-charcoal-800">
                    <div className="flex min-w-0 items-center gap-2">
                      {!isReadOnly && orderedEntries.length > 1 && (
                        <>
                          <SortableHandle attributes={attributes} listeners={listeners} />
                          <ReorderControls
                            index={index}
                            total={orderedEntries.length}
                            onMove={handleMove}
                          />
                        </>
                      )}
                      <div className="min-w-0">
                        <div className="truncate text-sm text-charcoal-700 dark:text-sand-300">
                          {entry.label}
                        </div>
                        {entry.paid_on && (
                          <div className="text-xs text-charcoal-400 dark:text-charcoal-500">
                            {entry.paid_on}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-sage-600 dark:text-sage-400">
                        {formatCurrency(entry.amount)}
                      </span>
                      {!isReadOnly && (
                        <>
                          <button
                            onClick={() => startEdit(entry)}
                            className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors"
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
                )
              }
            </SortableItem>
          ))}
        </SortableList>

        {isAdding && (
          <div className="flex flex-wrap items-end gap-2 pt-2">
            <div className="min-w-0 flex-[1_1_10rem]">
              <Input
                placeholder="Label"
                value={label}
                onChange={(e) => setLabel(e.target.value)}
              />
            </div>
            <div className="w-28">
              <Input
                type="number"
                placeholder="Amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>
            <div className="w-44">
              <Input
                type="date"
                value={paidOn}
                onChange={(e) => setPaidOn(e.target.value)}
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
