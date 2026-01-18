import { Modal } from "./ui/Modal";
import { MonthlyBudgetWithCategory } from "../api/client";
import { TrendingUp, TrendingDown, AlertCircle, PartyPopper } from "lucide-react";
import { useCurrency } from "../context/CurrencyContext";

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
  const { formatCurrency } = useCurrency();
  const overBudget: BudgetVariance[] = [];
  const underBudget: BudgetVariance[] = [];
  const unplanned: BudgetVariance[] = [];

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
    <Modal isOpen={isOpen} onClose={onClose} title="Budget Analysis">
      <div className="space-y-6">
        {isOnTrack ? (
          <div className="flex items-center gap-3 p-4 bg-sage-100 dark:bg-sage-900/30 rounded">
            <PartyPopper className="text-sage-600 shrink-0" size={24} />
            <div>
              <p className="font-semibold text-sage-700 dark:text-sage-400">
                {netVariance < 0 ? "You're ahead of budget!" : "You're right on track!"}
              </p>
              {netVariance < 0 && (
                <p className="text-sm text-sage-600 dark:text-sage-500">
                  You've saved {formatCurrency(Math.abs(netVariance))} more than planned across your categories.
                </p>
              )}
              {underBudget.length > 0 && (
                <p className="text-sm text-sage-600 dark:text-sage-500 mt-1">
                  Great discipline! {underBudget.length} {underBudget.length === 1 ? "category is" : "categories are"} under budget.
                </p>
              )}
            </div>
          </div>
        ) : (
          <div className="flex items-center gap-3 p-4 bg-terracotta-100 dark:bg-terracotta-900/30 rounded">
            <AlertCircle className="text-terracotta-600 shrink-0" size={24} />
            <div>
              <p className="font-semibold text-terracotta-700 dark:text-terracotta-400">
                You're {formatCurrency(totalOverspend + totalUnplanned + incomeShortfall)} over budget
              </p>
              <p className="text-sm text-terracotta-600 dark:text-terracotta-500">
                Here's what's affecting your projected savings:
              </p>
            </div>
          </div>
        )}

        {overBudget.length > 0 && (
          <div>
            <h3 className="text-sm font-medium text-charcoal-600 dark:text-charcoal-400 mb-2 flex items-center gap-2">
              <TrendingUp size={16} className="text-terracotta-500" />
              Budget Overruns
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
                      +{formatCurrency(item.variance)}
                    </span>
                    <span className="text-charcoal-500 dark:text-charcoal-500 text-xs ml-2">
                      ({formatCurrency(item.spent)} / {formatCurrency(item.allocated)})
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
              Unplanned Spending
            </h3>
            <div className="space-y-2">
              {unplanned.map((item) => (
                <div
                  key={item.label}
                  className="flex justify-between items-center p-2 bg-amber-50 dark:bg-amber-900/20 rounded text-sm"
                >
                  <span className="text-charcoal-700 dark:text-charcoal-300">{item.label}</span>
                  <span className="text-amber-600 dark:text-amber-400 font-medium">
                    {formatCurrency(item.spent)}
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
              Income Shortfall
            </h3>
            <div className="p-2 bg-terracotta-50 dark:bg-terracotta-900/20 rounded text-sm">
              <p className="text-charcoal-700 dark:text-charcoal-300">
                Income is <span className="font-medium text-terracotta-600 dark:text-terracotta-400">{formatCurrency(incomeShortfall)}</span> less than needed to cover expenses
              </p>
              <p className="text-xs text-charcoal-500 mt-1">
                Income: {formatCurrency(totalIncome)} | Needed: {formatCurrency(incomeNeeded)}
              </p>
            </div>
          </div>
        )}

        {underBudget.length > 0 && !isOnTrack && (
          <div>
            <h3 className="text-sm font-medium text-charcoal-600 dark:text-charcoal-400 mb-2 flex items-center gap-2">
              <TrendingDown size={16} className="text-sage-500" />
              Under Budget (Good!)
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
                      -{formatCurrency(Math.abs(item.variance))}
                    </span>
                    <span className="text-charcoal-500 dark:text-charcoal-500 text-xs ml-2">
                      ({formatCurrency(item.spent)} / {formatCurrency(item.allocated)})
                    </span>
                  </div>
                </div>
              ))}
              {underBudget.length > 3 && (
                <p className="text-xs text-charcoal-500 text-center">
                  +{underBudget.length - 3} more categories under budget
                </p>
              )}
            </div>
          </div>
        )}

        <div className="pt-4 border-t border-sand-300 dark:border-charcoal-700">
          <div className="flex justify-between text-sm">
            <span className="text-charcoal-600 dark:text-charcoal-400">Total over budget:</span>
            <span className="text-terracotta-600 dark:text-terracotta-400 font-medium">
              +{formatCurrency(totalOverspend + totalUnplanned)}
            </span>
          </div>
          <div className="flex justify-between text-sm mt-1">
            <span className="text-charcoal-600 dark:text-charcoal-400">Total under budget:</span>
            <span className="text-sage-600 dark:text-sage-400 font-medium">
              -{formatCurrency(totalSaved)}
            </span>
          </div>
          <div className="flex justify-between text-sm mt-2 pt-2 border-t border-sand-200 dark:border-charcoal-800">
            <span className="font-medium text-charcoal-700 dark:text-charcoal-300">Net impact:</span>
            <span className={`font-semibold ${netVariance > 0 ? "text-terracotta-600 dark:text-terracotta-400" : "text-sage-600 dark:text-sage-400"}`}>
              {netVariance > 0 ? "+" : "-"}{formatCurrency(Math.abs(netVariance))}
            </span>
          </div>
        </div>
      </div>
    </Modal>
  );
}

