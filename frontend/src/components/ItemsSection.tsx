import { useEffect, useState, useMemo } from "react";
import { Plus, Trash2, Edit2, Check, X, Search, Filter, Tags, Tag } from "lucide-react";
import { ItemWithCategory, BudgetCategory, TagSummary, api } from "../api/client";
import { Card } from "./ui/Card";
import { Input } from "./ui/Input";
import { Select } from "./ui/Select";
import { Button } from "./ui/Button";
import { ReorderControls } from "./ui/ReorderControls";
import { SortableHandle, SortableItem, SortableList } from "./ui/SortableList";
import { useCurrency } from "../context/CurrencyContext";
import { useSortableReorder } from "../hooks/useSortableReorder";
import { ManageTags } from "./ManageTags";
import { TagPicker } from "./TagPicker";

function DesktopTagChips({ tags }: { tags: TagSummary[] }) {
  if (tags.length === 0) return <span className="text-charcoal-400">–</span>;
  const names = tags.map((tag) => tag.label).join(", ");
  return (
    <div
      className="group relative flex w-fit max-w-full items-center gap-1 outline-none"
      tabIndex={0}
      aria-label={`Tags: ${names}`}
      title={names}
    >
      {tags.slice(0, 2).map((tag) => (
        <span
          key={tag.id}
          className="max-w-28 truncate rounded-md px-2 py-1 text-xs"
          style={{ color: tag.color, borderColor: `${tag.color}60`, backgroundColor: `${tag.color}18` }}
        >
          {tag.label}
        </span>
      ))}
      {tags.length > 2 && (
        <span className="rounded-md bg-sand-200 px-2 py-1 text-xs text-charcoal-500 dark:bg-charcoal-800 dark:text-charcoal-300">
          +{tags.length - 2}
        </span>
      )}
      <span className="pointer-events-none absolute left-0 top-full z-20 mt-1 hidden whitespace-nowrap rounded bg-charcoal-900 px-2 py-1 text-xs text-white shadow group-hover:block group-focus:block">
        {names}
      </span>
    </div>
  );
}

function MobileTagMenu({ tags }: { tags: TagSummary[] }) {
  if (tags.length === 0) return <span className="text-charcoal-400">–</span>;
  return (
    <details className="relative w-fit">
      <summary
        className="flex cursor-pointer list-none items-center gap-1 rounded-md border border-sand-300 px-2 py-1 text-xs text-charcoal-600 dark:border-charcoal-700 dark:text-sand-300 [&::-webkit-details-marker]:hidden"
        aria-label={`Show ${tags.length} Tags`}
      >
        <Tag size={14} />
        {tags.length}
      </summary>
      <div className="absolute left-1/2 top-full z-30 mt-2 min-w-36 -translate-x-1/2 rounded-md border border-sand-300 bg-charcoal-50 p-3 shadow-xl dark:border-charcoal-700 dark:bg-charcoal-900">
        <div className="mb-2 text-xs font-semibold text-charcoal-700 dark:text-sand-200">
          Tags <span className="ml-1 text-charcoal-400">{tags.length}</span>
        </div>
        <div className="space-y-2">
          {tags.map((tag) => (
            <div key={tag.id} className="flex items-center gap-2 whitespace-nowrap text-xs text-charcoal-700 dark:text-sand-200">
              <span className="h-3 w-3 rounded-full" style={{ backgroundColor: tag.color }} />
              {tag.label}
            </div>
          ))}
        </div>
      </div>
    </details>
  );
}

function TagFilter({
  tags,
  selectedIds,
  onChange,
}: {
  tags: TagSummary[];
  selectedIds: number[];
  onChange: (ids: number[]) => void;
}) {
  const [query, setQuery] = useState("");
  const matchingTags = [...tags]
    .filter((tag) => tag.label.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()))
    .sort((a, b) => a.label.localeCompare(b.label));

  return (
    <details className="relative">
      <summary className="flex h-9 w-40 cursor-pointer list-none items-center gap-2 rounded-md border border-sand-300 px-3 text-xs text-charcoal-700 dark:border-charcoal-700 dark:text-sand-200 [&::-webkit-details-marker]:hidden">
        <Tag size={14} className="shrink-0 text-charcoal-400" />
        <span className="truncate">{selectedIds.length ? `${selectedIds.length} Tags` : "All Tags"}</span>
      </summary>
      <div className="absolute right-0 z-30 mt-1 w-56 rounded-md border border-sand-300 bg-charcoal-50 p-2 shadow-xl dark:border-charcoal-700 dark:bg-charcoal-900">
        <Input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search Tags"
          aria-label="Search Tags"
          className="mb-2 h-8 text-xs"
        />
        <div className="max-h-48 space-y-1 overflow-y-auto">
          {matchingTags.map((tag) => (
            <label key={tag.id} className="flex cursor-pointer items-center gap-2 rounded px-2 py-1.5 text-xs hover:bg-sand-100 dark:hover:bg-charcoal-800">
              <input
                type="checkbox"
                checked={selectedIds.includes(tag.id)}
                onChange={() =>
                  onChange(
                    selectedIds.includes(tag.id)
                      ? selectedIds.filter((id) => id !== tag.id)
                      : [...selectedIds, tag.id]
                  )
                }
              />
              <span className="h-2.5 w-2.5 shrink-0 rounded-full" style={{ backgroundColor: tag.color }} />
              <span className="truncate">{tag.label}{tag.stopped ? " (Stopped)" : ""}</span>
            </label>
          ))}
          {matchingTags.length === 0 && (
            <div className="py-2 text-center text-xs text-charcoal-400">No Tags found</div>
          )}
        </div>
      </div>
    </details>
  );
}

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
  const { formatCurrency } = useCurrency();
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [description, setDescription] = useState("");
  const [amount, setAmount] = useState("");
  const [categoryId, setCategoryId] = useState<string>("");
  const [spentOn, setSpentOn] = useState(new Date().toISOString().split("T")[0]);
  const [isManagingTags, setIsManagingTags] = useState(false);
  const [tagIds, setTagIds] = useState<number[]>([]);
  const [formError, setFormError] = useState("");

  const [filterCategory, setFilterCategory] = useState<string>("all");
  const [filterTags, setFilterTags] = useState<TagSummary[]>([]);
  const [filterTagIds, setFilterTagIds] = useState<number[]>([]);
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    void api.tags.list().then(setFilterTags);
  }, []);

  const handleAdd = async () => {
    if (!description || !amount || !categoryId) return;
    try {
      await api.items.create(monthId, {
        description,
        amount: parseFloat(amount),
        category_id: parseInt(categoryId),
        spent_on: spentOn,
        savings_destination: "none",
        tag_ids: tagIds,
      });
      resetForm();
      await onUpdate();
    } catch {
      setFormError("Could not save this Spending Item. Check the selected Tags and try again.");
    }
  };

  const handleUpdate = async (id: number) => {
    if (!description || !amount) return;
    // An uncategorized item can be saved without picking a category; it stays uncategorized.
    try {
      await api.items.update(monthId, id, {
        description,
        amount: parseFloat(amount),
        ...(categoryId ? { category_id: parseInt(categoryId) } : {}),
        spent_on: spentOn,
        savings_destination: "none",
        tag_ids: tagIds,
      });
      resetForm();
      await onUpdate();
    } catch {
      setFormError("Could not save this Spending Item. Check the selected Tags and try again.");
    }
  };

  const handleDelete = async (id: number) => {
    await api.items.delete(monthId, id);
    await onUpdate();
  };

  const startEdit = (item: ItemWithCategory) => {
    setEditingId(item.id);
    setDescription(item.description);
    setAmount(item.amount.toString());
    setCategoryId(item.category_id?.toString() ?? "");
    setSpentOn(item.spent_on);
    setTagIds(item.tags.map((tag) => tag.id));
    setFormError("");
  };

  const resetForm = () => {
    setEditingId(null);
    setDescription("");
    setAmount("");
    setCategoryId("");
    setSpentOn(new Date().toISOString().split("T")[0]);
    setIsAdding(false);
    setTagIds([]);
    setFormError("");
  };

  const categoryOptions = categories.map((c) => ({ value: c.id, label: c.label }));
  const hasUncategorized = items.some((item) => item.category_id === null);
  // An uncategorized item keeps an explicit blank choice; picking a category re-labels it.
  const editCategoryOptions = categoryId
    ? categoryOptions
    : [{ value: "", label: "Uncategorized" }, ...categoryOptions];
  const filterCategoryOptions = [
    { value: "all", label: "All Categories" },
    ...(hasUncategorized ? [{ value: "uncategorized", label: "Uncategorized" }] : []),
    ...categoryOptions,
  ];

  const allSpendingItems = useMemo(() => {
    return items.filter((item) => item.savings_destination === "none");
  }, [items]);
  const {
    orderedItems: orderedSpendingItems,
    handleDragEnd: handleSpendingDragEnd,
  } = useSortableReorder(allSpendingItems, async (nextItems) => {
    await api.items.reorder(monthId, nextItems.map((item) => item.id));
    await onUpdate();
  });

  const spendingItems = useMemo(() => {
    return orderedSpendingItems
      .filter((item) => {
        const matchesCategory =
          filterCategory === "all"
            ? true
            : filterCategory === "uncategorized"
              ? item.category_id === null
              : item.category_id?.toString() === filterCategory;
        const matchesSearch =
          item.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
          (item.category_label ?? "uncategorized")
            .toLowerCase()
            .includes(searchQuery.toLowerCase()) ||
          item.tags.some((tag) => tag.label.toLowerCase().includes(searchQuery.toLowerCase()));
        const matchesTags = filterTagIds.every((id) =>
          item.tags.some((tag) => tag.id === id)
        );
        return matchesCategory && matchesSearch && matchesTags;
      });
  }, [orderedSpendingItems, filterCategory, filterTagIds, searchQuery]);

  const handleMove = async (index: number, direction: -1 | 1) => {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= spendingItems.length) return;
    const currentItem = spendingItems[index];
    const targetItem = spendingItems[nextIndex];
    const currentFullIndex = orderedSpendingItems.findIndex((item) => item.id === currentItem.id);
    const targetFullIndex = orderedSpendingItems.findIndex((item) => item.id === targetItem.id);
    if (currentFullIndex < 0 || targetFullIndex < 0) return;
    const next = [...orderedSpendingItems];
    [next[currentFullIndex], next[targetFullIndex]] = [
      next[targetFullIndex],
      next[currentFullIndex],
    ];
    await api.items.reorder(monthId, next.map((item) => item.id));
    await onUpdate();
  };

  return (
    <Card className="col-span-full">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-charcoal-700 dark:text-sand-200">
          Spending Items
        </h3>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setIsManagingTags(true)}
            aria-label="Manage Tags"
            className="p-2 md:p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 active:bg-sand-300 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
          >
            <Tags size={16} />
          </button>
          {!isReadOnly && !isAdding && (
            <button
              onClick={() => {
                setIsAdding(true);
                if (categories.length > 0) {
                  setCategoryId(categories[0].id.toString());
                }
              }}
              aria-label="Add Spending Item"
              className="p-2 md:p-1 hover:bg-sand-200 dark:hover:bg-charcoal-800 active:bg-sand-300 dark:active:bg-charcoal-700 transition-colors rounded touch-manipulation"
            >
              <Plus size={16} />
            </button>
          )}
        </div>
      </div>

      <ManageTags
        isOpen={isManagingTags}
        onClose={() => {
          setIsManagingTags(false);
          void api.tags.list().then(setFilterTags);
          void onUpdate();
        }}
      />

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
          <div className="mt-3">
            <TagPicker selectedIds={tagIds} onChange={setTagIds} />
          </div>
          {formError && <p className="mt-2 text-xs text-terracotta-600">{formError}</p>}
        </div>
      )}

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
        <div className="flex flex-wrap gap-2">
          <div className="relative w-40">
            <Filter size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-charcoal-400 z-10" />
            <Select
              options={filterCategoryOptions}
              value={filterCategory}
              onChange={(e) => setFilterCategory(e.target.value)}
              className="pl-9 h-9 text-xs"
            />
          </div>
          <TagFilter tags={filterTags} selectedIds={filterTagIds} onChange={setFilterTagIds} />
          {(searchQuery || filterCategory !== "all" || filterTagIds.length > 0) && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                setSearchQuery("");
                setFilterCategory("all");
                setFilterTagIds([]);
              }}
              className="h-9 px-2 text-[10px]"
            >
              Clear
            </Button>
          )}
        </div>
      </div>

      <div className="-mx-4 overflow-visible px-4">
        <table className="w-full table-fixed text-sm sm:table-auto">
          <thead>
            <tr className="border-b border-sand-300 dark:border-charcoal-700">
              <th className="w-14 text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs sm:w-auto md:text-sm">
                Date
              </th>
              <th className="text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs md:text-sm">
                Description
              </th>
              <th className="hidden text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs sm:table-cell md:text-sm">
                Category
              </th>
              <th className="w-14 text-left py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs sm:w-auto md:text-sm">
                Tags
              </th>
              <th className="w-20 text-right py-2 px-1 font-medium text-charcoal-600 dark:text-sand-400 text-xs sm:w-auto md:text-sm">
                Amount
              </th>
              {!isReadOnly && <th className="w-24 sm:w-28 md:w-32"></th>}
            </tr>
          </thead>
          <tbody>
            <SortableList
              ids={spendingItems.map((item) => item.id)}
              onDragEnd={handleSpendingDragEnd}
            >
              {spendingItems.map((item, index) => (
                <SortableItem
                  key={item.id}
                  id={item.id}
                  as="tr"
                  className="border-b border-sand-200 dark:border-charcoal-800 hover:bg-sand-100 dark:hover:bg-charcoal-900/50 active:bg-sand-200 dark:active:bg-charcoal-900 transition-colors"
                >
                  {({ attributes, listeners }) =>
                    editingId === item.id ? (
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
                        <td className="hidden py-2 sm:table-cell">
                          <Select
                            options={editCategoryOptions}
                            value={categoryId}
                            onChange={(e) => setCategoryId(e.target.value)}
                            className="text-xs"
                          />
                        </td>
                        <td className="py-2">
                          <div className="min-w-0 sm:min-w-48">
                            <TagPicker selectedIds={tagIds} onChange={setTagIds} />
                          </div>
                          {formError && <p className="mt-2 text-xs text-terracotta-600">{formError}</p>}
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
                          <div className="flex max-w-[150px] items-center gap-1.5 truncate md:max-w-none">
                            {/* On phones the category column is hidden; its color dot rides along here */}
                            <span
                              className="h-2 w-2 shrink-0 rounded-full sm:hidden"
                              style={{ backgroundColor: item.category_color ?? "#71717a" }}
                              title={item.category_label ?? "Uncategorized"}
                            />
                            <span className="truncate">{item.description}</span>
                          </div>
                        </td>
                        <td className="hidden py-2 px-1 sm:table-cell">
                          <span
                            className="text-[10px] md:text-xs px-1.5 md:px-2 py-0.5 md:py-1 rounded-sm border whitespace-nowrap"
                            style={{
                              backgroundColor: `${item.category_color ?? "#71717a"}20`,
                              color: item.category_color ?? "#71717a",
                              borderColor: `${item.category_color ?? "#71717a"}40`,
                            }}
                          >
                            {item.category_label ?? "Uncategorized"}
                          </span>
                        </td>
                        <td className="py-2 px-1">
                          <div className="sm:hidden">
                            <MobileTagMenu tags={item.tags} />
                          </div>
                          <div className="hidden sm:block">
                            <DesktopTagChips tags={item.tags} />
                          </div>
                        </td>
                        <td className={`py-2 px-1 text-right font-medium text-xs md:text-sm whitespace-nowrap text-terracotta-600 dark:text-terracotta-400`}>
                          {formatCurrency(item.amount)}
                        </td>
                        {!isReadOnly && (
                          <td className="py-2 px-1">
                            <div className="flex gap-0.5 md:gap-1 justify-end">
                              {spendingItems.length > 1 && (
                                <>
                                  <SortableHandle attributes={attributes} listeners={listeners} />
                                  <ReorderControls
                                    index={index}
                                    total={spendingItems.length}
                                    onMove={handleMove}
                                    className="mr-1"
                                  />
                                </>
                              )}
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
                    )
                  }
                </SortableItem>
              ))}
            </SortableList>
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
