export const SUPPORTED_CURRENCIES = {
  USD: { symbol: '$', name: 'US Dollar', locale: 'en-US' },
  EUR: { symbol: '€', name: 'Euro', locale: 'en-EU' },
  GBP: { symbol: '£', name: 'British Pound', locale: 'en-GB' },
  JPY: { symbol: '¥', name: 'Japanese Yen', locale: 'ja-JP' },
  CAD: { symbol: 'C$', name: 'Canadian Dollar', locale: 'en-CA' },
  AUD: { symbol: 'A$', name: 'Australian Dollar', locale: 'en-AU' },
  CHF: { symbol: 'CHF', name: 'Swiss Franc', locale: 'de-CH' },
  CNY: { symbol: '¥', name: 'Chinese Yuan', locale: 'zh-CN' },
  INR: { symbol: '₹', name: 'Indian Rupee', locale: 'en-IN' },
} as const;

export type Currency = keyof typeof SUPPORTED_CURRENCIES;

export function formatCurrency(
  amount: number,
  currency: Currency = 'USD',
  showSymbol: boolean = true
): string {
  const currencyInfo = SUPPORTED_CURRENCIES[currency];
  
  const formatted = new Intl.NumberFormat(currencyInfo.locale, {
    style: 'currency',
    currency: currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount);

  if (!showSymbol) {
    return formatted.replace(/[^\d,.-]/g, '').trim();
  }

  return formatted;
}

export function getCurrencySymbol(currency: Currency = 'USD'): string {
  return SUPPORTED_CURRENCIES[currency].symbol;
}