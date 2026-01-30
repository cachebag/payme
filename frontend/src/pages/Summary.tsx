import { useState, useEffect, useRef } from "react";
import { ArrowLeft, Printer, Download, Calendar } from "lucide-react";
import { Layout } from "../components/Layout";
import { api, MonthSummary, StatsResponse, Month } from "../api/client";
import { useCurrency } from "../context/CurrencyContext";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import {
  PieChart,
  Pie,
  Cell,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area,
  Legend,
} from "recharts";

interface SummaryPageProps {
  onBack: () => void;
  initialMonthId?: number | null;
}

const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

const COLORS = [
  "#5a7d5a", "#d4694a", "#6b8e8e", "#c4a35a", "#8b6b8e",
  "#5a8e7d", "#8e6b5a", "#5a6b8e", "#8e8b5a", "#6b5a8e",
];

export function SummaryPage({ onBack, initialMonthId }: SummaryPageProps) {
  const { formatCurrency } = useCurrency();
  const [viewMode, setViewMode] = useState<"month" | "year">("month");
  const [loading, setLoading] = useState(true);
  const [months, setMonths] = useState<Month[]>([]);
  const [selectedMonthId, setSelectedMonthId] = useState<number | null>(initialMonthId ?? null);
  const [monthSummary, setMonthSummary] = useState<MonthSummary | null>(null);
  const [yearStats, setYearStats] = useState<StatsResponse | null>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const loadInitialData = async () => {
      setLoading(true);
      try {
        const [monthsList, currentMonth] = await Promise.all([
          api.months.list(),
          api.months.current(),
        ]);
        setMonths(monthsList);
        setMonthSummary(currentMonth);
        if (!initialMonthId) {
          setSelectedMonthId(currentMonth.month.id);
        }
      } finally {
        setLoading(false);
      }
    };
    loadInitialData();
  }, [initialMonthId]);

  useEffect(() => {
    if (viewMode === "month" && selectedMonthId) {
      loadMonthData(selectedMonthId);
    } else if (viewMode === "year") {
      loadYearData();
    }
  }, [viewMode, selectedMonthId]);

  const loadMonthData = async (monthId: number) => {
    setLoading(true);
    try {
      const data = await api.months.get(monthId);
      setMonthSummary(data);
    } finally {
      setLoading(false);
    }
  };

  const loadYearData = async () => {
    setLoading(true);
    try {
      const data = await api.stats.get();
      setYearStats(data);
    } finally {
      setLoading(false);
    }
  };

  const handlePrint = () => {
    window.print();
  };

  const handleDownloadImage = async () => {
    if (!contentRef.current) return;
    try {
      const html2canvas = (await import("html2canvas")).default;
      const canvas = await html2canvas(contentRef.current, {
        backgroundColor: "#faf8f5",
        scale: 2,
      });
      const link = document.createElement("a");
      link.download = viewMode === "month" 
        ? `summary-${monthSummary?.month.year}-${monthSummary?.month.month}.png`
        : `summary-${new Date().getFullYear()}.png`;
      link.href = canvas.toDataURL("image/png");
      link.click();
    } catch (error) {
      console.error("Failed to generate image:", error);
    }
  };

  const selectedMonth = months.find(m => m.id === selectedMonthId);

  const spendingByCategory = monthSummary?.items
    .filter(i => i.savings_destination === "none")
    .reduce((acc, item) => {
      const existing = acc.find(c => c.category === item.category_label);
      if (existing) {
        existing.amount += item.amount;
      } else {
        acc.push({ category: item.category_label, amount: item.amount });
      }
      return acc;
    }, [] as { category: string; amount: number }[])
    .sort((a, b) => b.amount - a.amount) || [];

  const budgetVsActual = monthSummary?.budgets
    .map(b => ({
      category: b.category_label,
      budget: b.allocated_amount,
      actual: b.spent_amount,
      diff: b.allocated_amount - b.spent_amount,
    }))
    .filter(b => b.budget > 0 || b.actual > 0)
    .sort((a, b) => b.actual - a.actual) || [];

  const topSpending = monthSummary?.items
    .filter(i => i.savings_destination === "none")
    .sort((a, b) => b.amount - a.amount)
    .slice(0, 10) || [];

  const incomeBreakdown = monthSummary?.income_entries
    .map(e => ({ name: e.label, value: e.amount })) || [];

  const monthlyTrends = yearStats?.monthly_trends
    .slice()
    .reverse()
    .map(m => ({
      name: `${MONTH_NAMES[m.month - 1].slice(0, 3)}`,
      income: m.total_income,
      spent: m.total_spent,
      net: m.net,
    })) || [];

  const yearTotals = yearStats?.monthly_trends.reduce(
    (acc, m) => ({
      income: acc.income + m.total_income,
      spent: acc.spent + m.total_spent,
      fixed: acc.fixed + m.total_fixed,
      net: acc.net + m.net,
    }),
    { income: 0, spent: 0, fixed: 0, net: 0 }
  ) || { income: 0, spent: 0, fixed: 0, net: 0 };

  const savingsRate = monthSummary 
    ? ((monthSummary.total_income - monthSummary.total_fixed - monthSummary.total_spent) / monthSummary.total_income * 100)
    : 0;

  return (
    <Layout>
      <div className="print:hidden mb-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <button
            onClick={onBack}
            className="flex items-center gap-2 text-charcoal-600 dark:text-charcoal-400 hover:text-charcoal-900 dark:hover:text-sand-100"
          >
            <ArrowLeft size={20} />
            Back to Dashboard
          </button>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" onClick={handlePrint}>
              <Printer size={16} className="mr-2" />
              Print PDF
            </Button>
            <Button variant="ghost" size="sm" onClick={handleDownloadImage}>
              <Download size={16} className="mr-2" />
              Save Image
            </Button>
          </div>
        </div>
      </div>

      <div className="print:hidden mb-6 flex flex-col sm:flex-row items-start sm:items-center gap-4">
        <div className="flex rounded-lg overflow-hidden border border-sand-300 dark:border-charcoal-700">
          <button
            onClick={() => setViewMode("month")}
            className={`px-4 py-2 text-sm font-medium transition-colors ${
              viewMode === "month"
                ? "bg-sage-600 text-white"
                : "bg-sand-100 dark:bg-charcoal-800 text-charcoal-900 dark:text-sand-300 hover:bg-sand-200 dark:hover:bg-charcoal-700"
            }`}
          >
            Single Month
          </button>
          <button
            onClick={() => setViewMode("year")}
            className={`px-4 py-2 text-sm font-medium transition-colors ${
              viewMode === "year"
                ? "bg-sage-600 text-white"
                : "bg-sand-100 dark:bg-charcoal-800 text-charcoal-900 dark:text-sand-300 hover:bg-sand-200 dark:hover:bg-charcoal-700"
            }`}
          >
            Full Year
          </button>
        </div>

        {viewMode === "month" && (
          <div className="flex items-center gap-2">
            <Calendar size={16} className="text-charcoal-500" />
            <select
              value={selectedMonthId ?? ""}
              onChange={(e) => setSelectedMonthId(Number(e.target.value))}
              className="px-3 py-2 rounded-lg border border-sand-300 dark:border-charcoal-700 bg-white dark:bg-charcoal-800 text-charcoal-900 dark:text-sand-100 text-sm"
            >
              {months.map(m => (
                <option key={m.id} value={m.id}>
                  {MONTH_NAMES[m.month - 1]} {m.year}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>

      <div ref={contentRef} className="space-y-6 bg-sand-50 dark:bg-charcoal-950 p-4 print:p-0">
        <div className="text-center mb-8 print:mb-4">
          <h1 className="text-2xl sm:text-3xl font-bold text-charcoal-900 dark:text-sand-50">
            {viewMode === "month" && selectedMonth
              ? `${MONTH_NAMES[selectedMonth.month - 1]} ${selectedMonth.year} Summary`
              : `${new Date().getFullYear()} Year Summary`}
          </h1>
          <p className="text-sm text-charcoal-500 dark:text-charcoal-400 mt-1">
            Generated on {new Date().toLocaleDateString()}
          </p>
        </div>

        {loading ? (
          <div className="py-12 text-center text-charcoal-500">Loading...</div>
        ) : viewMode === "month" && monthSummary ? (
          <>
            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3 sm:gap-4">
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Total Income</div>
                <div className="text-lg font-semibold text-sage-600 dark:text-sage-400">
                  {formatCurrency(monthSummary.total_income)}
                </div>
              </Card>
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Fixed Expenses</div>
                <div className="text-lg font-semibold text-charcoal-600 dark:text-charcoal-400">
                  {formatCurrency(monthSummary.total_fixed)}
                </div>
              </Card>
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Total Spent</div>
                <div className="text-lg font-semibold text-terracotta-600 dark:text-terracotta-400">
                  {formatCurrency(monthSummary.total_spent)}
                </div>
              </Card>
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Remaining</div>
                <div className={`text-lg font-semibold ${monthSummary.remaining >= 0 ? "text-sage-600 dark:text-sage-400" : "text-terracotta-600 dark:text-terracotta-400"}`}>
                  {formatCurrency(monthSummary.remaining)}
                </div>
              </Card>
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Savings Rate</div>
                <div className={`text-lg font-semibold ${savingsRate >= 0 ? "text-sage-600 dark:text-sage-400" : "text-terracotta-600 dark:text-terracotta-400"}`}>
                  {savingsRate.toFixed(1)}%
                </div>
              </Card>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {spendingByCategory.length > 0 && (
                <Card>
                  <h3 className="text-sm font-medium text-charcoal-700 dark:text-sand-200 mb-4">
                    Spending by Category
                  </h3>
                  <div className="h-64">
                    <ResponsiveContainer width="100%" height="100%">
                      <PieChart>
                        <Pie
                          data={spendingByCategory}
                          dataKey="amount"
                          nameKey="category"
                          cx="30%"
                          cy="50%"
                          outerRadius={70}
                        >
                          {spendingByCategory.map((_, index) => (
                            <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                          ))}
                        </Pie>
                        <Tooltip
                          formatter={(value) => formatCurrency(Number(value))}
                          contentStyle={{ backgroundColor: "#faf8f5", border: "none", fontSize: 12, color: "#1a1a1a" }}
                        />
                        <Legend layout="vertical" align="right" verticalAlign="middle" wrapperStyle={{ fontSize: 10 }} />
                      </PieChart>
                    </ResponsiveContainer>
                  </div>
                </Card>
              )}

              {incomeBreakdown.length > 0 && (
                <Card>
                  <h3 className="text-sm font-medium text-charcoal-700 dark:text-sand-200 mb-4">
                    Income Sources
                  </h3>
                  <div className="h-64">
                    <ResponsiveContainer width="100%" height="100%">
                      <PieChart>
                        <Pie
                          data={incomeBreakdown}
                          dataKey="value"
                          nameKey="name"
                          cx="30%"
                          cy="50%"
                          outerRadius={70}
                        >
                          {incomeBreakdown.map((_, index) => (
                            <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                          ))}
                        </Pie>
                        <Tooltip
                          formatter={(value) => formatCurrency(Number(value))}
                          contentStyle={{ backgroundColor: "#faf8f5", border: "none", fontSize: 12, color: "#1a1a1a" }}
                        />
                        <Legend layout="vertical" align="right" verticalAlign="middle" wrapperStyle={{ fontSize: 10 }} />
                      </PieChart>
                    </ResponsiveContainer>
                  </div>
                </Card>
              )}
            </div>

            {budgetVsActual.length > 0 && (
              <Card>
                <h3 className="text-sm font-medium text-charcoal-700 dark:text-sand-200 mb-4">
                  Budget vs Actual
                </h3>
                <div className="h-80">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={budgetVsActual} layout="vertical" margin={{ left: 80 }}>
                      <XAxis type="number" tick={{ fontSize: 10 }} />
                      <YAxis type="category" dataKey="category" tick={{ fontSize: 10 }} width={75} />
                      <Tooltip
                        formatter={(value) => formatCurrency(Number(value))}
                        contentStyle={{ backgroundColor: "#faf8f5", border: "none", fontSize: 12, color: "#1a1a1a" }}
                      />
                      <Legend />
                      <Bar dataKey="budget" fill="#5a7d5a" name="Budget" />
                      <Bar dataKey="actual" fill="#d4694a" name="Actual" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </Card>
            )}

            {topSpending.length > 0 && (
              <Card>
                <h3 className="text-sm font-medium text-charcoal-700 dark:text-sand-200 mb-4">
                  Top 10 Expenses
                </h3>
                <div className="h-80">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart
                      data={topSpending.map(i => ({ name: i.description.slice(0, 20), amount: i.amount, category: i.category_label }))}
                      layout="vertical"
                      margin={{ left: 100 }}
                    >
                      <XAxis type="number" tick={{ fontSize: 10 }} />
                      <YAxis type="category" dataKey="name" tick={{ fontSize: 10 }} width={95} />
                      <Tooltip
                        formatter={(value) => formatCurrency(Number(value))}
                        labelFormatter={(label) => {
                          const item = topSpending.find(i => i.description.slice(0, 20) === label);
                          return item ? `${item.description} (${item.category_label})` : String(label);
                        }}
                        contentStyle={{ backgroundColor: "#faf8f5", border: "none", fontSize: 12, color: "#1a1a1a" }}
                      />
                      <Bar dataKey="amount" fill="#6b8e8e" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </Card>
            )}
          </>
        ) : viewMode === "year" && yearStats ? (
          <>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 sm:gap-4">
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Total Earned</div>
                <div className="text-lg font-semibold text-sage-600 dark:text-sage-400">
                  {formatCurrency(yearTotals.income)}
                </div>
              </Card>
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Total Spent</div>
                <div className="text-lg font-semibold text-terracotta-600 dark:text-terracotta-400">
                  {formatCurrency(yearTotals.spent + yearTotals.fixed)}
                </div>
              </Card>
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Total Saved</div>
                <div className={`text-lg font-semibold ${yearTotals.net >= 0 ? "text-sage-600 dark:text-sage-400" : "text-terracotta-600 dark:text-terracotta-400"}`}>
                  {formatCurrency(yearTotals.net)}
                </div>
              </Card>
              <Card>
                <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">Avg Monthly</div>
                <div className="text-lg font-semibold text-charcoal-600 dark:text-charcoal-400">
                  {formatCurrency(yearStats.average_monthly_spending)}
                </div>
              </Card>
            </div>

            {monthlyTrends.length > 1 && (
              <Card>
                <h3 className="text-sm font-medium text-charcoal-700 dark:text-sand-200 mb-4">
                  Monthly Trends
                </h3>
                <div className="h-80">
                  <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={monthlyTrends}>
                      <XAxis dataKey="name" tick={{ fontSize: 10 }} />
                      <YAxis tick={{ fontSize: 10 }} />
                      <Tooltip
                        formatter={(value) => formatCurrency(Number(value))}
                        contentStyle={{ backgroundColor: "#faf8f5", border: "none", fontSize: 12, color: "#1a1a1a" }}
                      />
                      <Legend />
                      <Area type="monotone" dataKey="income" stroke="#5a7d5a" fill="#5a7d5a" fillOpacity={0.3} name="Income" />
                      <Area type="monotone" dataKey="spent" stroke="#d4694a" fill="#d4694a" fillOpacity={0.3} name="Spent" />
                      <Area type="monotone" dataKey="net" stroke="#6b8e8e" fill="#6b8e8e" fillOpacity={0.3} name="Net" />
                    </AreaChart>
                  </ResponsiveContainer>
                </div>
              </Card>
            )}

            {yearStats.category_comparisons.length > 0 && (
              <Card>
                <h3 className="text-sm font-medium text-charcoal-700 dark:text-sand-200 mb-4">
                  Category Spending This Month vs Last
                </h3>
                <div className="h-80">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart
                      data={yearStats.category_comparisons.map(c => ({
                        category: c.category_label,
                        current: c.current_month_spent,
                        previous: c.previous_month_spent,
                      }))}
                      layout="vertical"
                      margin={{ left: 80 }}
                    >
                      <XAxis type="number" tick={{ fontSize: 10 }} />
                      <YAxis type="category" dataKey="category" tick={{ fontSize: 10 }} width={75} />
                      <Tooltip
                        formatter={(value) => formatCurrency(Number(value))}
                        contentStyle={{ backgroundColor: "#faf8f5", border: "none", fontSize: 12, color: "#1a1a1a" }}
                      />
                      <Legend />
                      <Bar dataKey="current" fill="#5a7d5a" name="This Month" />
                      <Bar dataKey="previous" fill="#c4a35a" name="Last Month" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </Card>
            )}
          </>
        ) : (
          <div className="py-12 text-center text-charcoal-500">
            No data available
          </div>
        )}
      </div>
    </Layout>
  );
}
