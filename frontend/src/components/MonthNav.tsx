import { ChevronLeft, ChevronRight, FileDown, Lock } from "lucide-react";
import { Month } from "../api/client";
import { Button } from "./ui/Button";
import { useTranslation } from "react-i18next";

interface MonthNavProps {
  months: Month[];
  selectedMonthId: number | null;
  onSelect: (id: number) => void;
  onClose: () => void;
  onDownloadPdf: () => void;
}

export function MonthNav({
  months,
  selectedMonthId,
  onSelect,
  onClose,
  onDownloadPdf,
}: MonthNavProps) {
  const selectedMonth = months.find((m) => m.id === selectedMonthId);
  const currentIndex = months.findIndex((m) => m.id === selectedMonthId);

  const { t } = useTranslation();

  const MONTH_NAMES = [
    t("month.name.jan"), t("month.name.feb"), t("month.name.mar"), t("month.name.apr"), t("month.name.may"), t("month.name.jun"),
    t("month.name.jul"), t("month.name.aug"), t("month.name.sep"), t("month.name.oct"), t("month.name.nov"), t("month.name.dec"),
  ];

  const now = new Date();
  const isCurrentCalendarMonth =
    selectedMonth?.year === now.getFullYear() &&
    selectedMonth?.month === now.getMonth() + 1;
  const isLastDay = now.getDate() === new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate();
  const canClose = isCurrentCalendarMonth && isLastDay && !selectedMonth?.is_closed;

  const goPrev = () => {
    if (currentIndex < months.length - 1) {
      onSelect(months[currentIndex + 1].id);
    }
  };

  const goNext = () => {
    if (currentIndex > 0) {
      onSelect(months[currentIndex - 1].id);
    }
  };

  if (!selectedMonth) return null;

  return (
    <div className="flex items-center justify-between mb-8">
      <div className="flex items-center gap-4">
        <button
          onClick={goPrev}
          disabled={currentIndex >= months.length - 1}
          className="p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 disabled:opacity-30 transition-colors"
        >
          <ChevronLeft size={20} />
        </button>
        <div className="text-center">
          <div className="text-2xl font-semibold text-charcoal-900 dark:text-sand-50">
            {MONTH_NAMES[selectedMonth.month - 1]} {selectedMonth.year}
          </div>
          <div className="text-xs text-charcoal-500 dark:text-charcoal-400 flex items-center justify-center gap-1">
            {selectedMonth.is_closed ? (
              <>
                <Lock size={12} />
                {t("month.text.closed")}
              </>
            ) : (
              t("month.text.active")
            )}
          </div>
        </div>
        <button
          onClick={goNext}
          disabled={currentIndex <= 0}
          className="p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 disabled:opacity-30 transition-colors"
        >
          <ChevronRight size={20} />
        </button>
      </div>

      <div className="flex items-center gap-2">
        {selectedMonth.is_closed && (
          <Button variant="ghost" size="sm" onClick={onDownloadPdf}>
            <FileDown size={16} className="mr-2" />
            PDF
          </Button>
        )}
        {canClose && (
          <Button variant="primary" size="sm" onClick={onClose}>
            {t("month.button.close_month")}
          </Button>
        )}
      </div>
    </div>
  );
}

