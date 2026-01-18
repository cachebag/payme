export const CURRENCY_SYMBOLS: Record<string, string> = {
  USD: "$",
  EUR: "€",
  GBP: "£",
  JPY: "¥",
  CAD: "C$",
  AUD: "A$",
  CHF: "CHF",
  CNY: "¥",
  INR: "₹",
  MXN: "$",
  BRL: "R$",
  ZAR: "R",
};

export const CURRENCY_POSITIONS: Record<string, "before" | "after"> = {
  USD: "before",
  EUR: "after",
  GBP: "before",
  JPY: "before",
  CAD: "before",
  AUD: "before",
  CHF: "after",
  CNY: "before",
  INR: "before",
  MXN: "before",
  BRL: "before",
  ZAR: "before",
};

export const DECIMAL_PLACES: Record<string, number> = {
  USD: 2,
  EUR: 2,
  GBP: 2,
  JPY: 0,
  CAD: 2,
  AUD: 2,
  CHF: 2,
  CNY: 2,
  INR: 2,
  MXN: 2,
  BRL: 2,
  ZAR: 2,
};

export function formatCurrency(
  amount: number,
  currency: string = "USD"
): string {
  const symbol = CURRENCY_SYMBOLS[currency] || "$";
  const position = CURRENCY_POSITIONS[currency] || "before";
  const decimals = DECIMAL_PLACES[currency] ?? 2;

  const formattedAmount = Math.abs(amount).toFixed(decimals);
  const isNegative = amount < 0 ? "-" : "";

  if (position === "before") {
    return `${isNegative}${symbol}${formattedAmount}`;
  } else {
    return `${isNegative}${formattedAmount} ${symbol}`;
  }
}

export function getCurrencySymbol(currency: string = "USD"): string {
  return CURRENCY_SYMBOLS[currency] || "$";
}
