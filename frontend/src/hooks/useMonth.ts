import { useState, useEffect, useCallback } from "react";
import { api, MonthSummary, Month, BudgetCategory } from "../api/client";

export function useMonth() {
  const [summary, setSummary] = useState<MonthSummary | null>(null);
  const [months, setMonths] = useState<Month[]>([]);
  const [categories, setCategories] = useState<BudgetCategory[]>([]);
  const [selectedMonthId, setSelectedMonthId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  const loadMonths = useCallback(async () => {
    const [monthsList, cats] = await Promise.all([
      api.months.list(),
      api.categories.list(),
    ]);
    setMonths(monthsList);
    setCategories(cats);
  }, []);

  const loadMonth = useCallback(async (monthId: number | null) => {
    setLoading(true);
    try {
      const data = monthId
        ? await api.months.get(monthId)
        : await api.months.current();
      setSummary(data);
      setSelectedMonthId(data.month.id);
    } finally {
      setLoading(false);
    }
  }, []);

  const refresh = useCallback(async () => {
    await loadMonths();
    await loadMonth(selectedMonthId);
    setRefreshTrigger(prev => prev + 1);
  }, [loadMonths, loadMonth, selectedMonthId]);

  useEffect(() => {
    loadMonths().then(() => loadMonth(null));
  }, [loadMonths, loadMonth]);

  const selectMonth = (monthId: number) => {
    setSelectedMonthId(monthId);
    loadMonth(monthId);
  };

  const createMonth = async (year: number, month: number) => {
    setLoading(true);
    try {
      const data = await api.months.create(year, month);
      await loadMonths();
      setSummary(data);
      setSelectedMonthId(data.month.id);
    } finally {
      setLoading(false);
    }
  };

  const closeMonth = async () => {
    if (!selectedMonthId) return;
    await api.months.close(selectedMonthId);
    await refresh();
  };

  const reopenMonth = async () => {
    if (!selectedMonthId) return;
    await api.months.reopen(selectedMonthId);
    await refresh();
  };

  const downloadPdf = async () => {
    if (!selectedMonthId) return;
    const blob = await api.months.downloadPdf(selectedMonthId);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `month-${summary?.month.year}-${summary?.month.month}.pdf`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return {
    summary,
    months,
    categories,
    selectedMonthId,
    loading,
    selectMonth,
    createMonth,
    refresh,
    closeMonth,
    reopenMonth,
    downloadPdf,
    refreshTrigger,
  };
}

