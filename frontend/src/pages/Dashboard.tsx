import { Layout } from "../components/Layout";
import { MonthNav } from "../components/MonthNav";
import { Summary } from "../components/Summary";
import { SavingsCard } from "../components/SavingsCard";
import { RetirementSavingsCard } from "../components/RetirementSavingsCard";
import { IncomeSection } from "../components/IncomeSection";
import { FixedExpenses } from "../components/FixedExpenses";
import { BudgetSection } from "../components/BudgetSection";
import { ItemsSection } from "../components/ItemsSection";
import { Stats } from "../components/Stats";
import { useMonth } from "../hooks/useMonth";
import { Loader2 } from "lucide-react";

export function Dashboard() {
  const {
    summary,
    months,
    categories,
    selectedMonthId,
    loading,
    selectMonth,
    refresh,
    closeMonth,
    downloadPdf,
  } = useMonth();

  if (loading && !summary) {
    return (
      <Layout>
        <div className="flex items-center justify-center py-20">
          <Loader2 size={24} className="animate-spin text-charcoal-400" />
        </div>
      </Layout>
    );
  }

  if (!summary) {
    return (
      <Layout>
        <div className="text-center py-20 text-charcoal-500">
          Unable to load data
        </div>
      </Layout>
    );
  }

  const isReadOnly = summary.month.is_closed;

  return (
    <Layout>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <MonthNav
            months={months}
            selectedMonthId={selectedMonthId}
            onSelect={selectMonth}
            onClose={closeMonth}
            onDownloadPdf={downloadPdf}
          />
          <div className="w-36">
            <SavingsCard />
          </div>
        </div>
        <Stats />
      </div>

      <div className="space-y-6">
        <Summary
          totalIncome={summary.total_income}
          totalFixed={summary.total_fixed}
          totalSpent={summary.total_spent}
          remaining={summary.remaining}
          extraCard={<RetirementSavingsCard />}
        />

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
      </div>

      <footer className="mt-12 py-4 text-center text-xs text-charcoal-400 dark:text-charcoal-600">
        {new Date().toLocaleDateString("en-US", {
          weekday: "long",
          year: "numeric",
          month: "long",
          day: "numeric",
        })}
      </footer>
    </Layout>
  );
}

