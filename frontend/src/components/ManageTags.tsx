import { useEffect, useMemo, useState } from "react";
import { Check, Edit2, Plus, RotateCcw, StopCircle, X } from "lucide-react";
import { ApiError, api, Tag } from "../api/client";
import { PRESET_COLORS } from "../constants/colors";
import { Button } from "./ui/Button";
import { ConfirmDialog } from "./ui/ConfirmDialog";
import { Input } from "./ui/Input";
import { Modal } from "./ui/Modal";

interface ManageTagsProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ManageTags({ isOpen, onClose }: ManageTagsProps) {
  const [tags, setTags] = useState<Tag[]>([]);
  const [label, setLabel] = useState("");
  const [color, setColor] = useState(PRESET_COLORS[0]);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [pendingStop, setPendingStop] = useState<Tag | null>(null);
  const [error, setError] = useState("");

  const loadTags = async () => setTags(await api.tags.list());

  useEffect(() => {
    if (isOpen) void api.tags.list().then(setTags);
  }, [isOpen]);

  const sorted = useMemo(
    () => [...tags].sort((a, b) => a.label.localeCompare(b.label)),
    [tags]
  );
  const active = sorted.filter((tag) => !tag.stopped);
  const stopped = sorted.filter((tag) => tag.stopped);

  const resetForm = () => {
    setLabel("");
    setColor(PRESET_COLORS[0]);
    setEditingId(null);
    setError("");
  };

  const save = async () => {
    if (!label.trim()) return;
    try {
      if (editingId === null) await api.tags.create({ label, color });
      else await api.tags.update(editingId, { label, color });
      resetForm();
      await loadTags();
    } catch (cause) {
      const restorable =
        cause instanceof ApiError
          ? tags.find((tag) => tag.id === cause.restorableTagId)
          : undefined;
      setError(
        restorable
          ? `Stopped Tag “${restorable.label}” already uses this name. Restore it below.`
          : "A Tag with this label already exists or is invalid."
      );
    }
  };

  const edit = (tag: Tag) => {
    setEditingId(tag.id);
    setLabel(tag.label);
    setColor(tag.color);
    setError("");
  };

  const stop = async () => {
    if (!pendingStop) return;
    await api.tags.stop(pendingStop.id);
    setPendingStop(null);
    await loadTags();
  };

  const restore = async (id: number) => {
    await api.tags.restore(id);
    await loadTags();
  };

  const list = (items: Tag[], isStopped = false) => (
    <div className="space-y-2">
      {items.map((tag) => (
        <div
          key={tag.id}
          className="flex items-center gap-2 rounded border border-sand-200 p-2 dark:border-charcoal-700"
        >
          <span className="h-3 w-3 shrink-0 rounded-full" style={{ backgroundColor: tag.color }} />
          <span className="min-w-0 flex-1 truncate text-sm text-charcoal-700 dark:text-sand-200">
            {tag.label}
          </span>
          <span className="text-xs text-charcoal-400">{tag.usage_count} items</span>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => edit(tag)}
            aria-label={`Edit ${tag.label}`}
            className="h-9 px-2"
          >
            <Edit2 size={14} />
          </Button>
          {isStopped ? (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => void restore(tag.id)}
              aria-label={`Restore ${tag.label}`}
              className="h-9 px-2 text-sage-600"
            >
              <RotateCcw size={14} />
            </Button>
          ) : (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setPendingStop(tag)}
              aria-label={`Stop ${tag.label}`}
              className="h-9 px-2 text-terracotta-600"
            >
              <StopCircle size={14} />
            </Button>
          )}
        </div>
      ))}
      {items.length === 0 && <p className="text-xs text-charcoal-400">None</p>}
    </div>
  );

  return (
    <>
      <Modal isOpen={isOpen} onClose={onClose} title="Manage Tags">
        <div className="space-y-5">
          <div className="space-y-2 rounded bg-sand-100 p-3 dark:bg-charcoal-800">
            <div className="flex gap-2">
              <Input
                value={label}
                maxLength={50}
                placeholder="Tag name"
                aria-label="Tag name"
                onChange={(event) => setLabel(event.target.value)}
              />
              <Button size="sm" onClick={() => void save()}>
                {editingId === null ? <Plus size={16} /> : <Check size={16} />}
              </Button>
              {editingId !== null && (
                <Button size="sm" variant="ghost" onClick={resetForm}>
                  <X size={16} />
                </Button>
              )}
            </div>
            <div className="flex flex-wrap gap-2" aria-label="Tag color">
              {PRESET_COLORS.map((preset) => (
                <button
                  key={preset}
                  type="button"
                  aria-label={`Select color ${preset}`}
                  aria-pressed={color === preset}
                  onClick={() => setColor(preset)}
                  className={`h-7 w-7 rounded-full border-2 ${color === preset ? "border-charcoal-800 dark:border-sand-100" : "border-transparent"}`}
                  style={{ backgroundColor: preset }}
                />
              ))}
            </div>
            {error && <p className="text-xs text-terracotta-600">{error}</p>}
          </div>

          <section>
            <h3 className="mb-2 text-xs font-semibold uppercase text-charcoal-500">Active Tags</h3>
            {list(active)}
          </section>
          <section>
            <h3 className="mb-2 text-xs font-semibold uppercase text-charcoal-500">Stopped Tags</h3>
            {list(stopped, true)}
          </section>
        </div>
      </Modal>

      <ConfirmDialog
        isOpen={pendingStop !== null}
        title="Stop Tag?"
        message={`${pendingStop?.label ?? "This Tag"} is used by ${pendingStop?.usage_count ?? 0} Spending Items. Existing Tag Assignments will remain.`}
        confirmLabel="Stop Tag"
        danger
        onConfirm={() => void stop()}
        onCancel={() => setPendingStop(null)}
      />
    </>
  );
}
