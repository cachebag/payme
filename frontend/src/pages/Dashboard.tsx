import { useState } from "react";
import { Layout } from "../components/Layout";
import { MonthNav } from "../components/MonthNav";
import { Summary } from "../components/Summary";
import { SavingsCard } from "../components/SavingsCard";
import { RetirementSavingsCard } from "../components/RetirementSavingsCard";
import { CustomSavingsGoals } from "../components/CustomSavingsGoals";
import { TransfersCard } from "../components/TransfersCard";
import { VarianceModal } from "../components/VarianceModal";
import { IncomeSection } from "../components/IncomeSection";
import { FixedExpenses } from "../components/FixedExpenses";
import { BudgetSection } from "../components/BudgetSection";
import { ItemsSection } from "../components/ItemsSection";
import { Stats } from "../components/Stats";
import { useMonth } from "../hooks/useMonth";
import { useUIPreferences } from "../context/UIPreferencesContext";
import { Loader2 } from "lucide-react";

interface DashboardProps {
  onSettingsClick: () => void;
  onSummaryClick: (monthId?: number) => void;
}

export function Dashboard({ onSettingsClick, onSummaryClick }: DashboardProps) {
  const [showVarianceModal, setShowVarianceModal] = useState(false);
  const { transfersEnabled } = useUIPreferences();
  const {
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
    refreshTrigger,
  } = useMonth();

  if (loading && !summary) {
    return (
      <Layout onSettingsClick={onSettingsClick}>
        <div className="flex items-center justify-center py-20">
          <Loader2 size={24} className="animate-spin text-charcoal-400" />
        </div>
      </Layout>
    );
  }

  if (!summary) {
    return (
      <Layout onSettingsClick={onSettingsClick}>
        <div className="text-center py-20 text-charcoal-500">
          Unable to load data
        </div>
      </Layout>
    );
  }

  const isReadOnly = summary.month.is_closed;

  return (
    <Layout onSettingsClick={onSettingsClick}>
      <div className="space-y-4 mb-4">
        <MonthNav
          months={months}
          selectedMonthId={selectedMonthId}
          onSelect={selectMonth}
          onCreateMonth={createMonth}
          onClose={closeMonth}
          onReopen={reopenMonth}
          onSummary={() => onSummaryClick(selectedMonthId ?? undefined)}
        />
        <div className="hidden lg:flex justify-end">
          <Stats />
        </div>
      </div>

      <div className="space-y-6">
        <Summary
          totalIncome={summary.total_income}
          totalFixed={summary.total_fixed}
          totalSpent={summary.total_spent}
          remaining={summary.remaining}
          extraCard={<RetirementSavingsCard refreshTrigger={refreshTrigger} />}
        />

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <SavingsCard 
            remaining={summary.remaining}
            refreshTrigger={refreshTrigger}
          />
          <CustomSavingsGoals />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <IncomeSection
            monthId={summary.month.id}
            entries={summary.income_entries}
            isReadOnly={isReadOnly}
            onUpdate={refresh}
          />
          <FixedExpenses
            expenses={summary.fixed_expenses}
            onUpdate={refresh}
          />
          <BudgetSection
            monthId={summary.month.id}
            budgets={summary.budgets}
            categories={categories}
            isReadOnly={isReadOnly}
            onUpdate={refresh}
          />
        </div>

        <ItemsSection
          monthId={summary.month.id}
          items={summary.items}
          categories={categories}
          isReadOnly={isReadOnly}
          onUpdate={refresh}
        />

        {(transfersEnabled || summary.items.some(item => item.savings_destination === "savings" || item.savings_destination === "retirement_savings")) && (
          <TransfersCard 
            monthId={summary.month.id}
            items={summary.items}
            categories={categories}
            isReadOnly={isReadOnly}
            onUpdate={refresh}
          />
        )}
      </div>

      <footer className="mt-12 py-4 text-center text-xs text-charcoal-400 dark:text-charcoal-600">
        {new Date().toLocaleDateString("en-US", {
          weekday: "long",
          year: "numeric",
          month: "long",
          day: "numeric",
        })}
      </footer>

      <VarianceModal
        isOpen={showVarianceModal}
        onClose={() => setShowVarianceModal(false)}
        budgets={summary.budgets}
        totalIncome={summary.total_income}
        totalFixed={summary.total_fixed}
        totalBudgeted={summary.total_budgeted}
      />
    </Layout>
  );
}

