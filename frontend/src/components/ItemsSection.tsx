import { useState, useMemo } from "react";
import { Plus, Trash2, Edit2, Check, X, ArrowUpDown, ArrowUp, ArrowDown, Search, Filter } from "lucide-react";
import { ItemWithCategory, BudgetCategory, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Select } from "./ui/Select";
import { Button } from "./ui/Button";
import { useCurrency } from "../context/CurrencyContext";

interface ItemsSectionProps {
  monthId: number;
  items: ItemWithCategory[];
  categories: BudgetCategory[];
  isReadOnly: boolean;
  onUpdate: () => void;
}

type SortField = "spent_on" | "description" | "category_label" | "amount";

export function ItemsSection({
  monthId,
  items,
  categories,
  isReadOnly,
  onUpdate,
}: ItemsSectionProps) {
  const { formatCurrency } = useCurrency();
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [description, setDescription] = useState("");
  const [amount, setAmount] = useState("");
  const [categoryId, setCategoryId] = useState<string>("");
  const [spentOn, setSpentOn] = useState(new Date().toISOString().split("T")[0]);

  // Sorting and Filtering State
  const [sortField, setSortField] = useState<SortField>("spent_on");
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">("desc");
  const [filterCategory, setFilterCategory] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");

  const handleAdd = async () => {
    if (!description || !amount || !categoryId) return;
    await api.items.create(monthId, {
      description,
      amount: parseFloat(amount),
      category_id: parseInt(categoryId),
      spent_on: spentOn,
      savings_destination: "none",
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
      savings_destination: "none",
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
  };

  const resetForm = () => {
    setEditingId(null);
    setDescription("");
    setAmount("");
    setCategoryId("");
    setSpentOn(new Date().toISOString().split("T")[0]);
    setIsAdding(false);
  };

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortDirection("asc");
    }
  };

  const categoryOptions = categories.map((c) => ({ value: c.id, label: c.label }));
  const filterCategoryOptions = [
    { value: "all", label: "All Categories" },
    ...categoryOptions,
  ];

  const spendingItems = useMemo(() => {
    return items
      .filter((item) => item.savings_destination === "none")
      .filter((item) => {
        const matchesCategory =
          filterCategory === "all" || item.category_id.toString() === filterCategory;
        const matchesSearch =
          item.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
          item.category_label.toLowerCase().includes(searchQuery.toLowerCase());
        return matchesCategory && matchesSearch;
      })
      .sort((a, b) => {
        let aValue: any = a[sortField];
        let bValue: any = b[sortField];

        if (sortField === "amount") {
          aValue = Number(aValue);
          bValue = Number(bValue);
        }

        if (aValue < bValue) return sortDirection === "asc" ? -1 : 1;
        if (aValue > bValue) return sortDirection === "asc" ? 1 : -1;
        return 0;
      });
  }, [items, sortField, sortDirection, filterCategory, searchQuery]);

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return <ArrowUpDown size={14} className="ml-1 opacity-20" />;
    return sortDirection === "asc" ? (
      <ArrowUp size={14} className="ml-1 text-sage-600" />
    ) : (
      <ArrowDown size={14} className="ml-1 text-sage-600" />
    );
  };

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
            className="p-2 md:p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 active:bg-sand-300 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
          >
            <Plus size={16} />
          </button>
        )}
      </div>

      {isAdding && categories.length === 0 && (
        <div className="mb-4 p-4 bg-sand-100 dark:bg-charcoal-800 text-center rounded-lg">
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300 mb-1">
            No budget categories yet.
          </p>
          <p className="text-xs text-charcoal-400 dark:text-charcoal-500">
            Add some in the Budget section first.
          </p>
          <button
            onClick={resetForm}
            className="mt-3 px-4 py-2 text-xs text-charcoal-500 hover:text-charcoal-700 dark:hover:text-charcoal-300 hover:bg-sand-200 dark:hover:bg-charcoal-700 active:bg-sand-300 dark:active:bg-charcoal-600 transition-colors rounded touch-manipulation"
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

      {/* Filtering and Search Controls */}
      <div className="flex flex-col md:flex-row gap-2 mb-4 bg-sand-100/50 dark:bg-charcoal-800/50 p-2 rounded-lg border border-sand-200 dark:border-charcoal-700">
        <div className="relative flex-1">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-charcoal-400" />
          <Input
            placeholder="Search spending..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 h-9 text-xs"
          />
        </div>
        <div className="flex gap-2">
          <div className="relative w-40">
            <Filter size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-charcoal-400 z-10" />
            <Select
              options={filterCategoryOptions}
              value={filterCategory}
              onChange={(e) => setFilterCategory(e.target.value)}
              className="pl-9 h-9 text-xs"
            />
          </div>
          {(searchQuery || filterCategory !== "all") && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                setSearchQuery("");
                setFilterCategory("all");
              }}
              className="h-9 px-2 text-[10px]"
            >
              Clear
            </Button>
          )}
        </div>
      </div>

      <div className="overflow-x-auto -mx-4 px-4">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-sand-300 dark:border-charcoal-700">
              <th
                className="text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm cursor-pointer hover:text-charcoal-900 dark:hover:text-sand-200 transition-colors select-none"
                onClick={() => handleSort("spent_on")}
              >
                <div className="flex items-center">
                  Date <SortIcon field="spent_on" />
                </div>
              </th>
              <th
                className="text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm cursor-pointer hover:text-charcoal-900 dark:hover:text-sand-200 transition-colors select-none"
                onClick={() => handleSort("description")}
              >
                <div className="flex items-center">
                  Description <SortIcon field="description" />
                </div>
              </th>
              <th
                className="text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm cursor-pointer hover:text-charcoal-900 dark:hover:text-sand-200 transition-colors select-none"
                onClick={() => handleSort("category_label")}
              >
                <div className="flex items-center">
                  Category <SortIcon field="category_label" />
                </div>
              </th>
              <th
                className="text-right py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm cursor-pointer hover:text-charcoal-900 dark:hover:text-sand-200 transition-colors select-none"
                onClick={() => handleSort("amount")}
              >
                <div className="flex items-center justify-end">
                  Amount <SortIcon field="amount" />
                </div>
              </th>
              {!isReadOnly && <th className="w-16 md:w-20"></th>}
            </tr>
          </thead>
          <tbody>
            {spendingItems.map((item) => (
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
                    <td className="py-2 px-1">
                      <span className="text-[10px] md:text-xs px-1.5 md:px-2 py-0.5 md:py-1 bg-sand-200 dark:bg-charcoal-800 text-charcoal-600 dark:text-sand-400 whitespace-nowrap">
                        {item.category_label}
                      </span>
                    </td>
                    <td className={`py-2 px-1 text-right font-medium text-xs md:text-sm whitespace-nowrap text-terracotta-600 dark:text-terracotta-400`}>
                      {formatCurrency(item.amount)}
                    </td>
                    {!isReadOnly && (
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

        {spendingItems.length === 0 && (
          <div className="text-sm text-charcoal-400 dark:text-charcoal-600 py-8 text-center">
            No spending items
          </div>
        )}
      </div>
    </Card>
  );
}
