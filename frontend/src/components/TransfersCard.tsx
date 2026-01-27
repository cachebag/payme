import { useState } from "react";
import { Plus, Trash2, Edit2, Check, X } from "lucide-react";
import { ItemWithCategory, BudgetCategory, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Select } from "./ui/Select";
import { Button } from "./ui/Button";
import { useCurrency } from "../context/CurrencyContext";
import { useUIPreferences } from "../context/UIPreferencesContext";

interface TransfersCardProps {
  monthId: number;
  items: ItemWithCategory[];
  categories: BudgetCategory[];
  isReadOnly: boolean;
  onUpdate: () => void;
}

export function TransfersCard({
  monthId,
  items,
  categories,
  isReadOnly,
  onUpdate,
}: TransfersCardProps) {
  const { formatCurrency } = useCurrency();
  const { transfersEnabled } = useUIPreferences();
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [description, setDescription] = useState("");
  const [amount, setAmount] = useState("");
  const [spentOn, setSpentOn] = useState(new Date().toISOString().split("T")[0]);
  const [savingsDestination, setSavingsDestination] = useState("savings");

  const transferItems = items.filter(
    (item) =>
      item.savings_destination === "savings" ||
      item.savings_destination === "retirement_savings"
  );

  const handleAdd = async () => {
    if (!description || !amount) return;
    const catId = categories.length > 0 ? categories[0].id : 1;
    await api.items.create(monthId, {
      description,
      amount: parseFloat(amount),
      category_id: catId,
      spent_on: spentOn,
      savings_destination: savingsDestination,
    });
    resetForm();
    await onUpdate();
  };

  const handleUpdate = async (id: number) => {
    if (!description || !amount) return;
    const catId = categories.length > 0 ? categories[0].id : 1;
    await api.items.update(monthId, id, {
      description,
      amount: parseFloat(amount),
      category_id: catId,
      spent_on: spentOn,
      savings_destination: savingsDestination,
    });
    resetForm();
    await onUpdate();
  };

  const handleDelete = async (id: number) => {
    await api.items.delete(monthId, id);
    await onUpdate();
  };

  const startEdit = (item: ItemWithCategory) => {
    setEditingId(item.id);
    setDescription(item.description);
    setAmount(item.amount.toString());
    setSpentOn(item.spent_on);
    setSavingsDestination(item.savings_destination);
  };

  const resetForm = () => {
    setEditingId(null);
    setDescription("");
    setAmount("");
    setSpentOn(new Date().toISOString().split("T")[0]);
    setSavingsDestination("savings");
    setIsAdding(false);
  };

  return (
    <Card className="col-span-full">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
          Transferred Items
        </h3>
        {!isReadOnly && !isAdding && transfersEnabled && (
          <button
            onClick={() => {
              setIsAdding(true);
            }}
            className="p-2 md:p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 active:bg-sand-300 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
          >
            <Plus size={16} />
          </button>
        )}
      </div>

      {isAdding && (
        <div className="mb-4 p-4 bg-sand-100 dark:bg-charcoal-800">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <Input
              placeholder="Description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
            <Input
              type="number"
              placeholder="Amount"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
            />
            <Input
              type="date"
              value={spentOn}
              onChange={(e) => setSpentOn(e.target.value)}
            />
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mt-3 mb-3">
            <div>
              <label className="text-sm text-charcoal-700 dark:text-sand-300 mb-1 block">
                Where should this money go?
              </label>
              <Select
                options={[
                  { value: "savings", label: "Savings" },
                  { value: "retirement_savings", label: "Retirement Savings" },
                ]}
                value={savingsDestination}
                onChange={(e) => setSavingsDestination(e.target.value)}
              />
            </div>
          </div>
          <div className="flex gap-2">
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
                Date
              </th>
              <th className="text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm">
                Description
              </th>
              <th className="text-center py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm">
                Destination
              </th>
              <th className="text-right py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm">
                Amount
              </th>
              {!isReadOnly && <th className="w-16 md:w-20"></th>}
            </tr>
          </thead>
          <tbody>
            {transferItems.map((item) => (
              <tr
                key={item.id}
                className="border-b border-sand-200 dark:border-charcoal-800 hover:bg-sand-100 dark:hover:bg-charcoal-900/50 active:bg-sand-200 dark:active:bg-charcoal-900 transition-colors"
              >
                {editingId === item.id ? (
                  <>
                    <td className="py-2">
                      <Input
                        type="date"
                        value={spentOn}
                        onChange={(e) => setSpentOn(e.target.value)}
                        className="text-xs"
                      />
                    </td>
                    <td className="py-2">
                      <Input
                        placeholder="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        className="text-xs"
                      />
                    </td>
                    <td className="py-2">
                      <Select
                        options={[
                          { value: "savings", label: "Savings" },
                          { value: "retirement_savings", label: "Retirement" },
                        ]}
                        value={savingsDestination}
                        onChange={(e) => setSavingsDestination(e.target.value)}
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
                    <td className="py-2 px-1 text-charcoal-600 dark:text-charcoal-400 text-xs md:text-sm whitespace-nowrap">
                      <span className="hidden md:inline">{item.spent_on}</span>
                      <span className="md:hidden">{item.spent_on.slice(5)}</span>
                    </td>
                    <td className="py-2 px-1 text-charcoal-800 dark:text-sand-200 text-xs md:text-sm">
                      <div className="max-w-[120px] md:max-w-none truncate">
                        {item.description}
                      </div>
                    </td>
                    <td className="py-2 px-1 text-center">
                      {item.savings_destination === "savings" && (
                        <span className="text-[10px] md:text-xs px-1.5 md:px-2 py-0.5 md:py-1 rounded bg-sage-100 dark:bg-sage-900 text-sage-700 dark:text-sage-200 whitespace-nowrap">
                          Savings
                        </span>
                      )}
                      {item.savings_destination === "retirement_savings" && (
                        <span className="text-[10px] md:text-xs px-1.5 md:px-2 py-0.5 md:py-1 rounded bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-200 whitespace-nowrap">
                          Retirement
                        </span>
                      )}
                    </td>
                    <td className="py-2 px-1 text-right font-medium text-xs md:text-sm whitespace-nowrap text-sage-600 dark:text-sage-400">
                      → {formatCurrency(item.amount)}
                    </td>
                    {!isReadOnly && transfersEnabled && (
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
            ))}
          </tbody>
        </table>

        {transferItems.length === 0 && (
          <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-8 text-center">
            No transfers
          </div>
        )}
      </div>
    </Card>
  );
}
