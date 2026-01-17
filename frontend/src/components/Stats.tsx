import { useState, useEffect } from "react";
import { BarChart3, TrendingUp, TrendingDown } from "lucide-react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { api, StatsResponse } from "../api/client";
import { Modal } from "./ui/Modal";
import { Button } from "./ui/Button";
import { useTranslation } from 'react-i18next';
import { useCurrency } from "../hooks/useCurrency";

const MONTH_NAMES = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun",
  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

export function Stats() {
  const [isOpen, setIsOpen] = useState(false);
  const [stats, setStats] = useState<StatsResponse | null>(null);
  const [loading, setLoading] = useState(false);

  const { t } = useTranslation();
  const { currencySymbol }  = useCurrency();

  const loadStats = async () => {
    setLoading(true);
    try {
      const data = await api.stats.get();
      setStats(data);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen && !stats) {
      loadStats();
    }
  }, [isOpen, stats]);

  const trendData = stats?.monthly_trends
    .slice()
    .reverse()
    .map((m) => ({
      name: `${MONTH_NAMES[m.month - 1]} ${m.year}`,
      income: m.total_income,
      spent: m.total_spent,
      net: m.net,
    })) || [];

  return (
    <>
      <Button variant="ghost" size="sm" onClick={() => setIsOpen(true)}>
        <BarChart3 size={16} className="mr-2" />
        {t("stats.main_label")}
      </Button>

      <Modal isOpen={isOpen} onClose={() => setIsOpen(false)} title="Statistics">
        {loading ? (
          <div className="py-8 text-center text-charcoal-500">{t("stats.text.loading")}...</div>
        ) : stats ? (
          <div className="space-y-6">
            {trendData.length > 1 && (
              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-sand-100 dark:bg-charcoal-800">
                  <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">
                    {t("stats.text.avg_monthly_spending")}
                  </div>
                  <div className="text-lg font-semibold text-terracotta-600 dark:text-terracotta-400">
                    {stats.average_monthly_spending.toFixed(2)}{currencySymbol}
                  </div>
                </div>
                <div className="p-4 bg-sand-100 dark:bg-charcoal-800">
                  <div className="text-xs text-charcoal-500 dark:text-charcoal-400 mb-1">
                    {t("stats.text.avg_monthly_income")}
                  </div>
                  <div className="text-lg font-semibold text-sage-600 dark:text-sage-400">
                    {stats.average_monthly_income.toFixed(2)}{currencySymbol}
                  </div>
                </div>
              </div>
            )}

            {trendData.length > 1 && (
              <div>
                <h4 className="text-sm font-medium mb-3 text-charcoal-700 dark:text-sand-200">
                  {t("stats.text.montly_trends")}
                </h4>
                <div className="h-48">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={trendData}>
                      <XAxis
                        dataKey="name"
                        tick={{ fontSize: 10 }}
                        stroke="currentColor"
                        className="text-charcoal-400"
                      />
                      <YAxis
                        tick={{ fontSize: 10 }}
                        stroke="currentColor"
                        className="text-charcoal-400"
                      />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: "var(--color-sand-100)",
                          border: "none",
                          fontSize: 12,
                        }}
                      />
                      <Line
                        type="monotone"
                        dataKey="income"
                        stroke="#5a7d5a"
                        strokeWidth={2}
                        dot={false}
                      />
                      <Line
                        type="monotone"
                        dataKey="spent"
                        stroke="#d4694a"
                        strokeWidth={2}
                        dot={false}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>
            )}

            {trendData.length > 1 && stats.category_comparisons.length > 0 && (
              <div>
                <h4 className="text-sm font-medium mb-3 text-charcoal-700 dark:text-sand-200">
                  {t("stats.text.category_comparsion_vs_(last_month)")}
                </h4>
                <div className="space-y-2">
                  {stats.category_comparisons.map((cat) => (
                    <div
                      key={cat.category_id}
                      className="flex items-center justify-between py-2 border-b border-sand-200 dark:border-charcoal-700"
                    >
                      <span className="text-sm text-charcoal-700 dark:text-sand-300">
                        {cat.category_label}
                      </span>
                      <div className="flex items-center gap-3">
                        <span className="text-sm text-charcoal-600 dark:text-charcoal-400">
                          {cat.current_month_spent.toFixed(2)}{currencySymbol}
                        </span>
                        {cat.change_amount !== 0 && (
                          <div
                            className={`flex items-center gap-1 text-xs ${
                              cat.change_amount > 0
                                ? "text-terracotta-500"
                                : "text-sage-500"
                            }`}
                          >
                            {cat.change_amount > 0 ? (
                              <TrendingUp size={12} />
                            ) : (
                              <TrendingDown size={12} />
                            )}
                            {cat.change_percent !== null
                              ? `${Math.abs(cat.change_percent).toFixed(0)}%`
                              : `${Math.abs(cat.change_amount).toFixed(0)}`}
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {trendData.length <= 1 && (
              <div className="py-8 text-center">
                <p className="text-charcoal-600 dark:text-charcoal-300 mb-2">
                  {t("stats.text.welcome_seems_this_is_your_first_month_on_payme")}.
                </p>
                <p className="text-sm text-charcoal-400 dark:text-charcoal-500">
                  {t("stats.text.check_back_in_here_next_month")};)
                </p>
              </div>
            )}
          </div>
        ) : (
          <div className="py-8 text-center">
            <p className="text-charcoal-600 dark:text-charcoal-300 mb-2">
              {t("stats.text.welcome_seems_this_is_your_first_month_on_payme")}.
            </p>
            <p className="text-sm text-charcoal-400 dark:text-charcoal-500">
              {t("stats.text.check_back_in_here_next_month")};)
            </p>
          </div>
        )}
      </Modal>
    </>
  );
}

