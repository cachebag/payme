import { useState } from "react";
import { ChevronLeft, ChevronRight, Lock, Calendar, PieChart } from "lucide-react";
import { Month } from "../api/client";
import { Button } from "./ui/Button";

interface MonthNavProps {
  months: Month[];
  selectedMonthId: number | null;
  onSelect: (id: number) => void;
  onCreateMonth: (year: number, month: number) => void;
  onClose: () => void;
  onReopen: () => void;
  onSummary: () => void;
}

const MONTH_NAMES = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun",
  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const FULL_MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

export function MonthNav({
  months,
  selectedMonthId,
  onSelect,
  onCreateMonth,
  onClose,
  onReopen,
  onSummary,
}: MonthNavProps) {
  const [showPicker, setShowPicker] = useState(false);
  const [pickerYear, setPickerYear] = useState(new Date().getFullYear());
  
  const selectedMonth = months.find((m) => m.id === selectedMonthId);

  const canClose = !selectedMonth?.is_closed;

  const getPrevMonth = () => {
    if (!selectedMonth) return null;
    let prevYear = selectedMonth.year;
    let prevMonth = selectedMonth.month - 1;
    if (prevMonth < 1) {
      prevMonth = 12;
      prevYear -= 1;
    }
    return { year: prevYear, month: prevMonth };
  };

  const getNextMonth = () => {
    if (!selectedMonth) return null;
    let nextYear = selectedMonth.year;
    let nextMonth = selectedMonth.month + 1;
    if (nextMonth > 12) {
      nextMonth = 1;
      nextYear += 1;
    }
    return { year: nextYear, month: nextMonth };
  };

  const goPrev = () => {
    const prev = getPrevMonth();
    if (!prev) return;
    
    const existingMonth = months.find(m => m.year === prev.year && m.month === prev.month);
    if (existingMonth) {
      onSelect(existingMonth.id);
    } else {
      onCreateMonth(prev.year, prev.month);
    }
  };

  const goNext = () => {
    const next = getNextMonth();
    if (!next) return;
    
    const existingMonth = months.find(m => m.year === next.year && m.month === next.month);
    if (existingMonth) {
      onSelect(existingMonth.id);
    } else {
      onCreateMonth(next.year, next.month);
    }
  };

  const handleMonthSelect = (month: number) => {
    const existingMonth = months.find(m => m.year === pickerYear && m.month === month);
    if (existingMonth) {
      onSelect(existingMonth.id);
    } else {
      onCreateMonth(pickerYear, month);
    }
    setShowPicker(false);
  };

  const openPicker = () => {
    if (selectedMonth) {
      setPickerYear(selectedMonth.year);
    }
    setShowPicker(true);
  };

  if (!selectedMonth) return null;

  return (
    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 mb-6 sm:mb-8">
      <div className="flex items-center gap-2 sm:gap-4">
        <button
          onClick={goPrev}
          className="p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors touch-manipulation"
          aria-label="Previous month"
        >
          <ChevronLeft size={20} />
        </button>
        <button
          onClick={openPicker}
          className="text-center hover:bg-sand-200 dark:hover:bg-charcoal-800 px-3 py-1 rounded-lg transition-colors"
        >
          <div className="text-xl sm:text-2xl font-semibold text-charcoal-900 dark:text-sand-50 flex items-center gap-2 justify-center">
            {MONTH_NAMES[selectedMonth.month - 1]} {selectedMonth.year}
            <Calendar size={16} className="text-charcoal-400" />
          </div>
          <div className="text-xs text-charcoal-500 dark:text-charcoal-400 flex items-center justify-center gap-1">
            {selectedMonth.is_closed ? (
              <>
                <Lock size={12} />
                closed
              </>
            ) : (
              "active"
            )}
          </div>
        </button>
        <button
          onClick={goNext}
          className="p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors touch-manipulation"
          aria-label="Next month"
        >
          <ChevronRight size={20} />
        </button>
      </div>

      <div className="flex items-center gap-2 w-full sm:w-auto">
        <Button variant="ghost" size="sm" onClick={onSummary} className="flex-1 sm:flex-none">
          <PieChart size={16} className="mr-2" />
          Summary
        </Button>
        {canClose ? (
          <Button variant="primary" size="sm" onClick={onClose} className="flex-1 sm:flex-none">
            Close Month
          </Button>
        ) : (
          <Button variant="ghost" size="sm" onClick={onReopen} className="flex-1 sm:flex-none">
            Reopen
          </Button>
        )}
      </div>

      {showPicker && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowPicker(false)}>
          <div 
            className="bg-white dark:bg-charcoal-900 rounded-xl p-4 shadow-xl max-w-sm w-full mx-4"
            onClick={e => e.stopPropagation()}
          >
            <div className="flex items-center justify-between mb-4">
              <button
                onClick={() => setPickerYear(y => y - 1)}
                className="p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 rounded-lg transition-colors"
              >
                <ChevronLeft size={20} />
              </button>
              <span className="text-lg font-semibold text-charcoal-900 dark:text-sand-50">
                {pickerYear}
              </span>
              <button
                onClick={() => setPickerYear(y => y + 1)}
                className="p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 rounded-lg transition-colors"
              >
                <ChevronRight size={20} />
              </button>
            </div>
            <div className="grid grid-cols-3 gap-2">
              {FULL_MONTH_NAMES.map((name, idx) => {
                const monthNum = idx + 1;
                const isSelected = selectedMonth?.year === pickerYear && selectedMonth?.month === monthNum;
                const exists = months.some(m => m.year === pickerYear && m.month === monthNum);
                
                return (
                  <button
                    key={monthNum}
                    onClick={() => handleMonthSelect(monthNum)}
                    className={`
                      p-2 rounded-lg text-sm font-medium transition-colors
                      ${isSelected 
                        ? "bg-sage-600 text-white" 
                        : exists
                          ? "bg-sand-200 dark:bg-charcoal-800 text-charcoal-900 dark:text-sand-50 hover:bg-sand-300 dark:hover:bg-charcoal-700"
                          : "text-charcoal-600 dark:text-charcoal-400 hover:bg-sand-100 dark:hover:bg-charcoal-800"
                      }
                    `}
                  >
                    {name.slice(0, 3)}
                  </button>
                );
              })}
            </div>
            <div className="mt-4 flex gap-2 text-xs text-charcoal-500 dark:text-charcoal-400">
              <span className="flex items-center gap-1">
                <span className="w-3 h-3 rounded bg-sand-200 dark:bg-charcoal-800"></span>
                Existing
              </span>
              <span className="flex items-center gap-1">
                <span className="w-3 h-3 rounded border border-charcoal-300 dark:border-charcoal-600"></span>
                New
              </span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

