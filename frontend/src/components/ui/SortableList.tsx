import {
  DndContext,
  DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  UniqueIdentifier,
  closestCenter,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical } from "lucide-react";
import { CSSProperties, ElementType, ReactNode } from "react";

interface SortableListProps {
  ids: UniqueIdentifier[];
  onDragEnd: (event: DragEndEvent) => void;
  children: ReactNode;
}

export function SortableList({ ids, onDragEnd, children }: SortableListProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 6,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
      <SortableContext items={ids} strategy={verticalListSortingStrategy}>
        {children}
      </SortableContext>
    </DndContext>
  );
}

interface SortableItemProps {
  id: UniqueIdentifier;
  as?: ElementType;
  className?: string;
  style?: CSSProperties;
  children: ReactNode | ((props: ReturnType<typeof useSortable>) => ReactNode);
}

export function SortableItem({
  id,
  as: Component = "div",
  className = "",
  style,
  children,
}: SortableItemProps) {
  const sortable = useSortable({ id });
  const { isDragging, setNodeRef, transform, transition } = sortable;

  const itemStyle: CSSProperties = {
    ...style,
    transform: CSS.Transform.toString(transform),
    transition,
    zIndex: isDragging ? 20 : style?.zIndex,
    opacity: isDragging ? 0.85 : style?.opacity,
    position: isDragging ? "relative" : style?.position,
  };

  return (
    <Component
      ref={setNodeRef}
      style={itemStyle}
      className={`${className} ${isDragging ? "shadow-lg" : ""}`}
    >
      {typeof children === "function" ? children(sortable) : children}
    </Component>
  );
}

interface SortableHandleProps {
  attributes: ReturnType<typeof useSortable>["attributes"];
  listeners: ReturnType<typeof useSortable>["listeners"];
  className?: string;
}

export function SortableHandle({
  attributes,
  listeners,
  className = "",
}: SortableHandleProps) {
  return (
    <button
      type="button"
      className={`cursor-grab rounded p-1 text-charcoal-500 transition-colors hover:bg-sand-200 hover:text-charcoal-800 active:cursor-grabbing dark:text-charcoal-400 dark:hover:bg-charcoal-800 dark:hover:text-sand-200 ${className}`}
      title="Drag to reorder"
      {...attributes}
      {...listeners}
    >
      <GripVertical size={14} />
    </button>
  );
}
