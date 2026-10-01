import { useEffect, useMemo, useState } from "react";
import { Plus, RotateCcw, X } from "lucide-react";
import { api, Tag } from "../api/client";
import { PRESET_COLORS } from "../constants/colors";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";

interface TagPickerProps {
  selectedIds: number[];
  onChange: (ids: number[]) => void;
}

export function TagPicker({ selectedIds, onChange }: TagPickerProps) {
  const [tags, setTags] = useState<Tag[]>([]);
  const [query, setQuery] = useState("");
  const [error, setError] = useState("");

  const loadTags = async () => setTags(await api.tags.list());

  useEffect(() => {
    void api.tags.list().then(setTags);
  }, []);

  const normalizedQuery = query.trim().toLocaleLowerCase();
  const matching = useMemo(
    () =>
      tags
        .filter((tag) => !selectedIds.includes(tag.id))
        .filter((tag) => tag.label.toLocaleLowerCase().includes(normalizedQuery))
        .sort((a, b) => a.label.localeCompare(b.label)),
    [normalizedQuery, selectedIds, tags]
  );
  const exactMatch = tags.find(
    (tag) => tag.label.toLocaleLowerCase() === normalizedQuery
  );
  const selected = selectedIds
    .map((id) => tags.find((tag) => tag.id === id))
    .filter((tag): tag is Tag => tag !== undefined);
  const canAdd = selectedIds.length < 5;
  const nextColor = PRESET_COLORS[tags.length % PRESET_COLORS.length];

  const add = (id: number) => {
    if (!canAdd) return;
    onChange([...selectedIds, id]);
    setQuery("");
  };

  const create = async () => {
    if (!normalizedQuery || !canAdd) return;
    try {
      const tag = await api.tags.create({ label: query, color: nextColor });
      setTags((current) => [...current, tag]);
      setError("");
      add(tag.id);
    } catch {
      setError("Could not create this Tag. Refresh the list and try again.");
    }
  };

  const restoreAndAdd = async (tag: Tag) => {
    if (!canAdd) return;
    try {
      await api.tags.restore(tag.id);
      await loadTags();
      setError("");
      add(tag.id);
    } catch {
      setError("Could not restore this Tag. Try again.");
    }
  };

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-1">
        {selected.map((tag) => (
          <span
            key={tag.id}
            className="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs"
            style={{ color: tag.color, borderColor: `${tag.color}60`, backgroundColor: `${tag.color}18` }}
          >
            {tag.label}
            <button
              type="button"
              aria-label={`Remove ${tag.label}`}
              onClick={() => onChange(selectedIds.filter((id) => id !== tag.id))}
            >
              <X size={12} />
            </button>
          </span>
        ))}
      </div>
      <Input
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder={canAdd ? "Search or create Tags" : "Maximum 5 Tags"}
        aria-label="Search or create Tags"
        disabled={!canAdd}
        className="text-xs"
      />
      {canAdd && query.trim() && (
        <div className="max-h-36 space-y-1 overflow-y-auto rounded border border-sand-200 bg-charcoal-50 p-1 dark:border-charcoal-700 dark:bg-charcoal-900">
          {matching.map((tag) => (
            <Button
              key={tag.id}
              type="button"
              size="sm"
              variant="ghost"
              disabled={tag.stopped}
              onClick={() => add(tag.id)}
              className="w-full justify-start gap-2"
            >
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: tag.color }} />
              {tag.label}{tag.stopped ? " (Stopped)" : ""}
            </Button>
          ))}
          {exactMatch?.stopped ? (
            <Button
              type="button"
              size="sm"
              variant="secondary"
              onClick={() => void restoreAndAdd(exactMatch)}
              className="w-full gap-2"
            >
              <RotateCcw size={14} /> Restore and add “{exactMatch.label}”
            </Button>
          ) : !exactMatch ? (
            <Button
              type="button"
              size="sm"
              variant="secondary"
              onClick={() => void create()}
              className="w-full gap-2"
            >
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: nextColor }} />
              <Plus size={14} /> Create “{query.trim()}”
            </Button>
          ) : null}
        </div>
      )}
      {error && <p className="text-xs text-terracotta-600">{error}</p>}
    </div>
  );
}
