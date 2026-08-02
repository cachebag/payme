import { DragEndEvent, UniqueIdentifier } from "@dnd-kit/core";
import { arrayMove } from "@dnd-kit/sortable";
import { useState } from "react";

interface OptimisticOrder<T> {
  source: T[];
  items: T[];
}

export function useSortableReorder<T extends { id: UniqueIdentifier }>(
  items: T[],
  onCommit: (items: T[]) => Promise<void> | void
) {
  const [optimisticOrder, setOptimisticOrder] = useState<OptimisticOrder<T> | null>(null);
  const orderedItems = optimisticOrder?.source === items ? optimisticOrder.items : items;

  const handleDragEnd = async ({ active, over }: DragEndEvent) => {
    if (!over || active.id === over.id) return;

    const oldIndex = orderedItems.findIndex((item) => item.id === active.id);
    const newIndex = orderedItems.findIndex((item) => item.id === over.id);
    if (oldIndex < 0 || newIndex < 0) return;

    const previousItems = orderedItems;
    const nextItems = arrayMove(orderedItems, oldIndex, newIndex);
    setOptimisticOrder({ source: items, items: nextItems });

    try {
      await onCommit(nextItems);
    } catch (error) {
      setOptimisticOrder({ source: items, items: previousItems });
      console.error("Failed to reorder items:", error);
    }
  };

  return {
    orderedItems,
    itemIds: orderedItems.map((item) => item.id),
    handleDragEnd,
  };
}
