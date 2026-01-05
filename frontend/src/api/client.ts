const BASE_URL = "/api";

async function request<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const response = await fetch(`${BASE_URL}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
    credentials: "include",
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

export const api = {
  auth: {
    register: (username: string, password: string) =>
      request<{ id: number; username: string }>("/auth/register", {
        method: "POST",
        body: JSON.stringify({ username, password }),
      }),
    login: (username: string, password: string) =>
      request<{ id: number; username: string }>("/auth/login", {
        method: "POST",
        body: JSON.stringify({ username, password }),
      }),
    logout: () => request<void>("/auth/logout", { method: "POST" }),
    me: () => request<{ id: number; username: string }>("/auth/me"),
  },

  months: {
    list: () => request<Month[]>("/months"),
    current: () => request<MonthSummary>("/months/current"),
    get: (id: number) => request<MonthSummary>(`/months/${id}`),
    close: (id: number) => request<Month>(`/months/${id}/close`, { method: "POST" }),
    downloadPdf: async (id: number) => {
      const response = await fetch(`${BASE_URL}/months/${id}/pdf`, {
        credentials: "include",
      });
      return response.blob();
    },
  },

  fixedExpenses: {
    list: () => request<FixedExpense[]>("/fixed-expenses"),
    create: (data: { label: string; amount: number }) =>
      request<FixedExpense>("/fixed-expenses", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    update: (id: number, data: { label?: string; amount?: number }) =>
      request<FixedExpense>(`/fixed-expenses/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (id: number) =>
      request<void>(`/fixed-expenses/${id}`, { method: "DELETE" }),
  },

  categories: {
    list: () => request<BudgetCategory[]>("/categories"),
    create: (data: { label: string; default_amount: number }) =>
      request<BudgetCategory>("/categories", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    update: (id: number, data: { label?: string; default_amount?: number }) =>
      request<BudgetCategory>(`/categories/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (id: number) =>
      request<void>(`/categories/${id}`, { method: "DELETE" }),
  },

  budgets: {
    list: (monthId: number) => request<MonthlyBudget[]>(`/months/${monthId}/budgets`),
    update: (monthId: number, budgetId: number, amount: number) =>
      request<MonthlyBudget>(`/months/${monthId}/budgets/${budgetId}`, {
        method: "PUT",
        body: JSON.stringify({ allocated_amount: amount }),
      }),
  },

  income: {
    list: (monthId: number) => request<IncomeEntry[]>(`/months/${monthId}/income`),
    create: (monthId: number, data: { label: string; amount: number }) =>
      request<IncomeEntry>(`/months/${monthId}/income`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    update: (
      monthId: number,
      incomeId: number,
      data: { label?: string; amount?: number }
    ) =>
      request<IncomeEntry>(`/months/${monthId}/income/${incomeId}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (monthId: number, incomeId: number) =>
      request<void>(`/months/${monthId}/income/${incomeId}`, { method: "DELETE" }),
  },

  items: {
    list: (monthId: number) => request<ItemWithCategory[]>(`/months/${monthId}/items`),
    create: (
      monthId: number,
      data: { category_id: number; description: string; amount: number; spent_on: string }
    ) =>
      request<Item>(`/months/${monthId}/items`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    update: (
      monthId: number,
      itemId: number,
      data: {
        category_id?: number;
        description?: string;
        amount?: number;
        spent_on?: string;
      }
    ) =>
      request<Item>(`/months/${monthId}/items/${itemId}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (monthId: number, itemId: number) =>
      request<void>(`/months/${monthId}/items/${itemId}`, { method: "DELETE" }),
  },

  stats: {
    get: () => request<StatsResponse>("/stats"),
  },

  exportDb: async () => {
    const response = await fetch(`${BASE_URL}/export`, {
      credentials: "include",
    });
    return response.blob();
  },

  exportJson: async () => {
    return request<UserExport>("/export/json");
  },

  importJson: async (data: UserExport) => {
    return request<void>("/import/json", {
      method: "POST",
      body: JSON.stringify(data),
    });
  },

  savings: {
    get: () => request<{ savings: number }>("/savings"),
    update: (savings: number) =>
      request<{ savings: number }>("/savings", {
        method: "PUT",
        body: JSON.stringify({ savings }),
      }),
  },

  retirementSavings: {
    get: () => request<{ retirement_savings: number }>("/retirement-savings"),
    update: (retirement_savings: number) =>
      request<{ retirement_savings: number }>("/retirement-savings", {
        method: "PUT",
        body: JSON.stringify({ retirement_savings }),
      }),
  },
};

export interface UserExport {
  version: number;
  savings?: number;
  retirement_savings?: number;
  fixed_expenses: { label: string; amount: number }[];
  categories: { label: string; default_amount: number }[];
  months: {
    year: number;
    month: number;
    is_closed: boolean;
    income_entries: { label: string; amount: number }[];
    budgets: { category_label: string; allocated_amount: number }[];
    items: { category_label: string; description: string; amount: number; spent_on: string }[];
  }[];
}

export interface Month {
  id: number;
  user_id: number;
  year: number;
  month: number;
  is_closed: boolean;
  closed_at: string | null;
}

export interface FixedExpense {
  id: number;
  user_id: number;
  label: string;
  amount: number;
}

export interface BudgetCategory {
  id: number;
  user_id: number;
  label: string;
  default_amount: number;
}

export interface MonthlyBudget {
  id: number;
  month_id: number;
  category_id: number;
  allocated_amount: number;
}

export interface MonthlyBudgetWithCategory {
  id: number;
  month_id: number;
  category_id: number;
  category_label: string;
  allocated_amount: number;
  spent_amount: number;
}

export interface IncomeEntry {
  id: number;
  month_id: number;
  label: string;
  amount: number;
}

export interface Item {
  id: number;
  month_id: number;
  category_id: number;
  description: string;
  amount: number;
  spent_on: string;
}

export interface ItemWithCategory extends Item {
  category_label: string;
}

export interface MonthSummary {
  month: Month;
  income_entries: IncomeEntry[];
  fixed_expenses: FixedExpense[];
  budgets: MonthlyBudgetWithCategory[];
  items: ItemWithCategory[];
  total_income: number;
  total_fixed: number;
  total_budgeted: number;
  total_spent: number;
  remaining: number;
}

export interface CategoryStats {
  category_id: number;
  category_label: string;
  current_month_spent: number;
  previous_month_spent: number;
  change_amount: number;
  change_percent: number | null;
}

export interface MonthlyStats {
  year: number;
  month: number;
  total_income: number;
  total_spent: number;
  total_fixed: number;
  net: number;
}

export interface StatsResponse {
  category_comparisons: CategoryStats[];
  monthly_trends: MonthlyStats[];
  average_monthly_spending: number;
  average_monthly_income: number;
}

