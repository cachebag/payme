import { useState } from "react";
import { Plus, Trash2, Edit2, Check, X } from "lucide-react";
import { ItemWithCategory, BudgetCategory, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Select } from "./ui/Select";
import { Button } from "./ui/Button";

interface ItemsSectionProps {
  monthId: number;
  items: ItemWithCategory[];
  categories: BudgetCategory[];
  isReadOnly: boolean;
  onUpdate: () => void;
}

export function ItemsSection({
  monthId,
  items,
  categories,
  isReadOnly,
  onUpdate,
}: ItemsSectionProps) {
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [description, setDescription] = useState("");
  const [amount, setAmount] = useState("");
  const [categoryId, setCategoryId] = useState<string>("");
  const [spentOn, setSpentOn] = useState(new Date().toISOString().split("T")[0]);
  const [addToSavings, setAddToSavings] = useState(false);

  const handleAdd = async () => {
    if (!description || !amount || !categoryId) return;
    await api.items.create(monthId, {
      description,
      amount: parseFloat(amount),
      category_id: parseInt(categoryId),
      spent_on: spentOn,
      add_to_savings: addToSavings,
    });
    resetForm();
    await onUpdate();
  };

  const handleUpdate = async (id: number) => {
    if (!description || !amount || !categoryId) return;
    await api.items.update(monthId, id, {
      description,
      amount: parseFloat(amount),
      category_id: parseInt(categoryId),
      spent_on: spentOn,
      add_to_savings: addToSavings,
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
    setCategoryId(item.category_id.toString());
    setSpentOn(item.spent_on);
    setAddToSavings(item.add_to_savings);
  };

  const resetForm = () => {
    setEditingId(null);
    setDescription("");
    setAmount("");
    setCategoryId("");
    setSpentOn(new Date().toISOString().split("T")[0]);
    setAddToSavings(false);
    setIsAdding(false);
  };

  const categoryOptions = categories.map((c) => ({ value: c.id, label: c.label }));

  return (
    <Card className="col-span-full">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
          Spending Items
        </h3>
        {!isReadOnly && !isAdding && (
          <button
            onClick={() => {
              setIsAdding(true);
              if (categories.length > 0) {
                setCategoryId(categories[0].id.toString());
              }
            }}
            className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors"
          >
            <Plus size={16} />
          </button>
        )}
      </div>

      {isAdding && categories.length === 0 && (
        <div className="mb-4 p-4 bg-sand-100 dark:bg-charcoal-800 text-center">
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300 mb-1">
            No budget categories yet.
          </p>
          <p className="text-xs text-charcoal-400 dark:text-charcoal-500">
            Add some in the Budget section first.
          </p>
          <button
            onClick={resetForm}
            className="mt-3 text-xs text-charcoal-500 hover:text-charcoal-700 dark:hover:text-charcoal-300"
          >
            Close
          </button>
        </div>
      )}

      {isAdding && categories.length > 0 && (
        <div className="mb-4 p-4 bg-sand-100 dark:bg-charcoal-800">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
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
            <Select
              options={categoryOptions}
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value)}
            />
            <Input
              type="date"
              value={spentOn}
              onChange={(e) => setSpentOn(e.target.value)}
            />
          </div>
          <div className="flex items-center gap-2 mt-3 mb-3">
            <input
              type="checkbox"
              id="add-to-savings"
              checked={addToSavings}
              onChange={(e) => setAddToSavings(e.target.checked)}
              className="rounded"
            />
            <label htmlFor="add-to-savings" className="text-sm text-charcoal-700 dark:text-sand-300">
              Add to savings
            </label>
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

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-sand-300 dark:border-charcoal-700">
              <th className="text-left py-2 font-medium text-charcoal-600 dark:text-sand-400">
                Date
              </th>
              <th className="text-left py-2 font-medium text-charcoal-600 dark:text-sand-400">
                Description
              </th>
              <th className="text-left py-2 font-medium text-charcoal-600 dark:text-sand-400">
                Category
              </th>
              <th className="text-right py-2 font-medium text-charcoal-600 dark:text-sand-400">
                Amount
              </th>
              <th className="text-center py-2 font-medium text-charcoal-600 dark:text-sand-400">
                Savings
              </th>
              {!isReadOnly && <th className="w-20"></th>}
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <tr
                key={item.id}
                className="border-b border-sand-200 dark:border-charcoal-800 hover:bg-sand-100 dark:hover:bg-charcoal-900/50"
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
                        options={categoryOptions}
                        value={categoryId}
                        onChange={(e) => setCategoryId(e.target.value)}
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
                      <div className="flex items-center gap-1">
                        <input
                          type="checkbox"
                          checked={addToSavings}
                          onChange={(e) => setAddToSavings(e.target.checked)}
                          className="rounded w-3 h-3"
                        />
                      </div>
                    </td>
                    <td className="py-2">
                      <div className="flex gap-1 justify-end">
                        <button
                          onClick={() => handleUpdate(item.id)}
                          className="p-1 text-sage-600 hover:bg-sage-100 dark:hover:bg-charcoal-800"
                        >
                          <Check size={14} />
                        </button>
                        <button
                          onClick={resetForm}
                          className="p-1 text-charcoal-500 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                        >
                          <X size={14} />
                        </button>
                      </div>
                    </td>
                  </>
                ) : (
                  <>
                    <td className="py-2 text-charcoal-600 dark:text-charcoal-400">
                      {item.spent_on}
                    </td>
                    <td className="py-2 text-charcoal-800 dark:text-sand-200">
                      {item.description}
                    </td>
                    <td className="py-2">
                      <span className="text-xs px-2 py-1 bg-sand-200 dark:bg-charcoal-800 text-charcoal-600 dark:text-sand-400">
                        {item.category_label}
                      </span>
                    </td>
                    <td className={`py-2 text-right font-medium ${
                      item.add_to_savings 
                        ? 'text-sage-600 dark:text-sage-400' 
                        : 'text-terracotta-600 dark:text-terracotta-400'
                    }`}>
                      {item.add_to_savings && '→ '} ${item.amount.toFixed(2)}
                    </td>
                    <td className="py-2 text-center">
                      {item.add_to_savings && (
                        <span className="text-xs px-2 py-1 rounded bg-sage-100 dark:bg-sage-900 text-sage-700 dark:text-sage-200">
                          ✓
                        </span>
                      )}
                    </td>
                    {!isReadOnly && (
                      <td className="py-2">
                        <div className="flex gap-1 justify-end">
                          <button
                            onClick={() => startEdit(item)}
                            className="p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800"
                          >
                            <Edit2 size={14} />
                          </button>
                          <button
                            onClick={() => handleDelete(item.id)}
                            className="p-1 text-terracotta-500 hover:bg-terracotta-100 dark:hover:bg-charcoal-800"
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

        {items.length === 0 && (
          <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-8 text-center">
            No spending items
          </div>
        )}
      </div>
    </Card>
  );
}

