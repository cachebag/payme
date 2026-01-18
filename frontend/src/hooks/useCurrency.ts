import { useAuth } from "../context/AuthContext";
import { formatCurrency, Currency } from "../lib/currency";

export function useCurrency() {
  const { user } = useAuth();
  const currency = (user?.currency as Currency) || "USD";

  const format = (amount: number) => formatCurrency(amount, currency);

  return { currency, format };
}