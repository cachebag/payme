import { Modal } from "./ui/Modal";
import { MonthlyBudgetWithCategory } from "../api/client";
import { TrendingUp, TrendingDown, AlertCircle, PartyPopper } from "lucide-react";
import { useCurrency } from "../hooks/useCurrency";
import { useTranslation } from "react-i18next";

interface VarianceModalProps {
  isOpen: boolean;
  onClose: () => void;
  budgets: MonthlyBudgetWithCategory[];
  totalIncome: number;
  totalFixed: number;
  totalBudgeted: number;
}

interface BudgetVariance {
  label: string;
  allocated: number;
  spent: number;
  variance: number;
  isUnplanned: boolean;
}

export function VarianceModal({
  isOpen,
  onClose,
  budgets,
  totalIncome,
  totalFixed,
  totalBudgeted,
}: VarianceModalProps) {
  const overBudget: BudgetVariance[] = [];
  const underBudget: BudgetVariance[] = [];
  const unplanned: BudgetVariance[] = [];

  const { currencySymbol } = useCurrency();

  const { t } = useTranslation();

  budgets.forEach((b) => {
    const variance = b.spent_amount - b.allocated_amount;
    const item: BudgetVariance = {
      label: b.category_label,
      allocated: b.allocated_amount,
      spent: b.spent_amount,
      variance,
      isUnplanned: b.allocated_amount === 0 && b.spent_amount > 0,
    };

    if (item.isUnplanned) {
      unplanned.push(item);
    } else if (variance > 0) {
      overBudget.push(item);
    } else if (variance < 0) {
      underBudget.push(item);
    }
  });

  overBudget.sort((a, b) => b.variance - a.variance);
  unplanned.sort((a, b) => b.spent - a.spent);
  underBudget.sort((a, b) => a.variance - b.variance);

  const totalOverspend = overBudget.reduce((sum, b) => sum + b.variance, 0);
  const totalUnplanned = unplanned.reduce((sum, b) => sum + b.spent, 0);
  const totalSaved = underBudget.reduce((sum, b) => sum + Math.abs(b.variance), 0);

  const incomeNeeded = totalFixed + totalBudgeted;
  const incomeShortfall = incomeNeeded > totalIncome ? incomeNeeded - totalIncome : 0;

  const netVariance = totalOverspend + totalUnplanned - totalSaved;
  const isOnTrack = netVariance <= 0 && incomeShortfall === 0;


  return (
    <Modal isOpen={isOpen} onClose={onClose} title={t("variance.modal.budget_analysis")}>
      <div className="space-y-6">
        {isOnTrack ? (
          <div className="flex items-center gap-3 p-4 bg-sage-100 dark:bg-sage-900/30 rounded">
            <PartyPopper className="text-sage-600 shrink-0" size={24} />
            <div>
              <p className="font-semibold text-sage-700 dark:text-sage-400">
                {netVariance > 0 ? t("variance.text.you_are_ahead_of_budget")+"!" : t("variance.text.you_are_right_on_track") + "!"}
              </p>
              {netVariance < 0 && (
                <p className="text-sm text-sage-600 dark:text-sage-500">
                  {t("variance.text.you_saved")} {Math.abs(netVariance).toFixed(2)}{currencySymbol} {t("variance.text.more_than_planned_across_your_categories")}.
                </p>
              )}
              {underBudget.length > 0 && (
                <p className="text-sm text-sage-600 dark:text-sage-500 mt-1">
                  {t("variance.text.great_discipline")}! {underBudget.length} {underBudget.length === 1 ? t("variance.text.category_is") : t("variance.text.categories_are")} {t("variance.text.under_budget")}.
                </p>
              )}
            </div>
          </div>
        ) : (
          <div className="flex items-center gap-3 p-4 bg-terracotta-100 dark:bg-terracotta-900/30 rounded">
            <AlertCircle className="text-terracotta-600 shrink-0" size={24} />
            <div>
              <p className="font-semibold text-terracotta-700 dark:text-terracotta-400">
                {t("variance.text.you_are")} {(totalOverspend + totalUnplanned + incomeShortfall).toFixed(2)}{currencySymbol} {t("variance.text.over_budget")}
              </p>
              <p className="text-sm text-terracotta-600 dark:text-terracotta-500">
                {t("variance.text.heres_whats_affecting_your_projected_savings")}:
              </p>
            </div>
          </div>
        )}

        {overBudget.length > 0 && (
          <div>
            <h3 className="text-sm font-medium text-charcoal-600 dark:text-charcoal-400 mb-2 flex items-center gap-2">
              <TrendingUp size={16} className="text-terracotta-500" />
              {t("variance.text.budget_overruns")}
            </h3>
            <div className="space-y-2">
              {overBudget.map((item) => (
                <div
                  key={item.label}
                  className="flex justify-between items-center p-2 bg-terracotta-50 dark:bg-terracotta-900/20 rounded text-sm"
                >
                  <span className="text-charcoal-700 dark:text-charcoal-300">{item.label}</span>
                  <div className="text-right">
                    <span className="text-terracotta-600 dark:text-terracotta-400 font-medium">
                      + {item.variance.toFixed(2)}{currencySymbol}
                    </span>
                    <span className="text-charcoal-500 dark:text-charcoal-500 text-xs ml-2">
                      ({item.spent.toFixed(2)}{currencySymbol} / {item.allocated.toFixed(2)}{currencySymbol})
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {unplanned.length > 0 && (
          <div>
            <h3 className="text-sm font-medium text-charcoal-600 dark:text-charcoal-400 mb-2 flex items-center gap-2">
              <AlertCircle size={16} className="text-amber-500" />
              {t("variance.text.unplaned_spending")}
            </h3>
            <div className="space-y-2">
              {unplanned.map((item) => (
                <div
                  key={item.label}
                  className="flex justify-between items-center p-2 bg-amber-50 dark:bg-amber-900/20 rounded text-sm"
                >
                  <span className="text-charcoal-700 dark:text-charcoal-300">{item.label}</span>
                  <span className="text-amber-600 dark:text-amber-400 font-medium">
                    {item.spent.toFixed(2)}{currencySymbol}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        {incomeShortfall > 0 && (
          <div>
            <h3 className="text-sm font-medium text-charcoal-600 dark:text-charcoal-400 mb-2 flex items-center gap-2">
              <TrendingDown size={16} className="text-terracotta-500" />
              {t("variance.text.income_shortfall")}
            </h3>
            <div className="p-2 bg-terracotta-50 dark:bg-terracotta-900/20 rounded text-sm">
              <p className="text-charcoal-700 dark:text-charcoal-300">
                {t("variance.text.income_is")}<span className="font-medium text-terracotta-600 dark:text-terracotta-400">${incomeShortfall.toFixed(2)}</span> {t("variance.text.less_than_needed_to_cover_expenses")}
              </p>
              <p className="text-xs text-charcoal-500 mt-1">
                {t("variance.text.income")}: {totalIncome.toFixed(2)}{currencySymbol} | {t("variance.text.needed")}: {incomeNeeded.toFixed(2)}{currencySymbol}
              </p>
            </div>
          </div>
        )}

        {underBudget.length > 0 && !isOnTrack && (
          <div>
            <h3 className="text-sm font-medium text-charcoal-600 dark:text-charcoal-400 mb-2 flex items-center gap-2">
              <TrendingDown size={16} className="text-sage-500" />
              {t("variance.text.under_budget_(good!)")}
            </h3>
            <div className="space-y-2">
              {underBudget.slice(0, 3).map((item) => (
                <div
                  key={item.label}
                  className="flex justify-between items-center p-2 bg-sage-50 dark:bg-sage-900/20 rounded text-sm"
                >
                  <span className="text-charcoal-700 dark:text-charcoal-300">{item.label}</span>
                  <div className="text-right">
                    <span className="text-sage-600 dark:text-sage-400 font-medium">
                      -{Math.abs(item.variance).toFixed(2)}{currencySymbol}
                    </span>
                    <span className="text-charcoal-500 dark:text-charcoal-500 text-xs ml-2">
                      ({item.spent.toFixed(2)}{currencySymbol} / {item.allocated.toFixed(2)}{currencySymbol})
                    </span>
                  </div>
                </div>
              ))}
              {underBudget.length > 3 && (
                <p className="text-xs text-charcoal-500 text-center">
                  +{underBudget.length - 3} {t("variance.text.more_categories_under_budget")}
                </p>
              )}
            </div>
          </div>
        )}

        <div className="pt-4 border-t border-sand-300 dark:border-charcoal-700">
          <div className="flex justify-between text-sm">
            <span className="text-charcoal-600 dark:text-charcoal-400">{t("variance.text.total_over_budget")}:</span>
            <span className="text-terracotta-600 dark:text-terracotta-400 font-medium">
              +{(totalOverspend + totalUnplanned).toFixed(2)}{currencySymbol}
            </span>
          </div>
          <div className="flex justify-between text-sm mt-1">
            <span className="text-charcoal-600 dark:text-charcoal-400">{t("variance.text.total_under_budget")}:</span>
            <span className="text-sage-600 dark:text-sage-400 font-medium">
              -{totalSaved.toFixed(2)}{currencySymbol}
            </span>
          </div>
          <div className="flex justify-between text-sm mt-2 pt-2 border-t border-sand-200 dark:border-charcoal-800">
            <span className="font-medium text-charcoal-700 dark:text-charcoal-300">{t("variance.text.net_impact")}:</span>
            <span className={`font-semibold ${netVariance > 0 ? "text-terracotta-600 dark:text-terracotta-400" : "text-sage-600 dark:text-sage-400"}`}>
              {netVariance > 0 ? "+" : "-"}{Math.abs(netVariance).toFixed(2)}{currencySymbol}
            </span>
          </div>
        </div>
      </div>
    </Modal>
  );
}

