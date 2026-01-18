import { createContext, useContext, useEffect, useState, ReactNode, useCallback } from "react";

export interface Currency {
  code: string;
  symbol: string;
  name: string;
  locale: string;
  position: "before" | "after";
}

export const SUPPORTED_CURRENCIES: Currency[] = [
  { code: "USD", symbol: "$", name: "US Dollar", locale: "en-US", position: "before" },
  { code: "EUR", symbol: "€", name: "Euro", locale: "de-DE", position: "after" },
  { code: "GBP", symbol: "£", name: "British Pound", locale: "en-GB", position: "before" },
  { code: "JPY", symbol: "¥", name: "Japanese Yen", locale: "ja-JP", position: "before" },
  { code: "CAD", symbol: "CA$", name: "Canadian Dollar", locale: "en-CA", position: "before" },
  { code: "AUD", symbol: "A$", name: "Australian Dollar", locale: "en-AU", position: "before" },
  { code: "CHF", symbol: "CHF", name: "Swiss Franc", locale: "de-CH", position: "after" },
  { code: "CNY", symbol: "¥", name: "Chinese Yuan", locale: "zh-CN", position: "before" },
  { code: "INR", symbol: "₹", name: "Indian Rupee", locale: "en-IN", position: "before" },
  { code: "MXN", symbol: "MX$", name: "Mexican Peso", locale: "es-MX", position: "before" },
  { code: "BRL", symbol: "R$", name: "Brazilian Real", locale: "pt-BR", position: "before" },
  { code: "KRW", symbol: "₩", name: "South Korean Won", locale: "ko-KR", position: "before" },
  { code: "MYR", symbol: "RM", name: "Malaysian Ringgit", locale: "ms-MY", position: "before" },
  { code: "EGP", symbol: "EGP", name: "Egyptian Pound", locale: "en-EG", position: "before" },
  { code: "SAR", symbol: "SAR", name: "Saudi Riyal", locale: "en-SA", position: "before" },
];

interface CurrencyContextType {
  currency: Currency;
  setCurrency: (code: string) => void;
  formatCurrency: (value: number, options?: FormatOptions) => string;
  formatCurrencyCompact: (value: number) => string;
  getCurrencySymbol: () => string;
}

interface FormatOptions {
  showSymbol?: boolean;
  absolute?: boolean;
}

const STORAGE_KEY = "currency";
const DEFAULT_CURRENCY_CODE = "USD";

function getDefaultCurrency(): Currency {
  // Try to detect from browser locale
  const browserLocale = navigator.language || "en-US";

  // Map common locales to currencies
  const localeMap: Record<string, string> = {
    "en-US": "USD",
    "en-GB": "GBP",
    "de-DE": "EUR",
    "fr-FR": "EUR",
    "es-ES": "EUR",
    "it-IT": "EUR",
    "ja-JP": "JPY",
    "en-CA": "CAD",
    "en-AU": "AUD",
    "de-CH": "CHF",
    "zh-CN": "CNY",
    "en-IN": "INR",
    "es-MX": "MXN",
    "pt-BR": "BRL",
    "ko-KR": "KRW",
    "ms-MY": "MYR",
    "en-MY": "MYR",
    "en-EG": "EGP",
    "en-SA": "SAR",
  };

  const detectedCode = localeMap[browserLocale] || DEFAULT_CURRENCY_CODE;
  return SUPPORTED_CURRENCIES.find(c => c.code === detectedCode) || SUPPORTED_CURRENCIES[0];
}

function getCurrencyByCode(code: string): Currency {
  return SUPPORTED_CURRENCIES.find(c => c.code === code) || SUPPORTED_CURRENCIES[0];
}

const CurrencyContext = createContext<CurrencyContextType | undefined>(undefined);

export function CurrencyProvider({ children }: { children: ReactNode }) {
  const [currency, setCurrencyState] = useState<Currency>(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const found = SUPPORTED_CURRENCIES.find(c => c.code === stored);
      if (found) return found;
    }
    return getDefaultCurrency();
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, currency.code);
  }, [currency]);

  const setCurrency = useCallback((code: string) => {
    const newCurrency = getCurrencyByCode(code);
    setCurrencyState(newCurrency);
  }, []);

  const formatCurrency = useCallback((value: number, options: FormatOptions = {}): string => {
    const { showSymbol = true, absolute = false } = options;
    const displayValue = absolute ? Math.abs(value) : value;

    try {
      // Use Intl.NumberFormat for proper locale-aware formatting
      const formatter = new Intl.NumberFormat(currency.locale, {
        style: showSymbol ? "currency" : "decimal",
        currency: currency.code,
        minimumFractionDigits: currency.code === "JPY" || currency.code === "KRW" ? 0 : 2,
        maximumFractionDigits: currency.code === "JPY" || currency.code === "KRW" ? 0 : 2,
      });

      return formatter.format(displayValue);
    } catch {
      // Fallback formatting if Intl fails
      const numStr = displayValue.toFixed(currency.code === "JPY" || currency.code === "KRW" ? 0 : 2);
      if (!showSymbol) return numStr;
      return currency.position === "before"
        ? `${currency.symbol}${numStr}`
        : `${numStr} ${currency.symbol}`;
    }
  }, [currency]);

  const formatCurrencyCompact = useCallback((value: number): string => {
    try {
      const formatter = new Intl.NumberFormat(currency.locale, {
        style: "currency",
        currency: currency.code,
        notation: "compact",
        minimumFractionDigits: 0,
        maximumFractionDigits: 1,
      });
      return formatter.format(value);
    } catch {
      // Fallback
      if (Math.abs(value) >= 1000000) {
        return `${currency.symbol}${(value / 1000000).toFixed(1)}M`;
      } else if (Math.abs(value) >= 1000) {
        return `${currency.symbol}${(value / 1000).toFixed(1)}K`;
      }
      return formatCurrency(value);
    }
  }, [currency, formatCurrency]);

  const getCurrencySymbol = useCallback((): string => {
    return currency.symbol;
  }, [currency]);

  return (
    <CurrencyContext.Provider
      value={{
        currency,
        setCurrency,
        formatCurrency,
        formatCurrencyCompact,
        getCurrencySymbol
      }}
    >
      {children}
    </CurrencyContext.Provider>
  );
}

export function useCurrency() {
  const context = useContext(CurrencyContext);
  if (!context) {
    throw new Error("useCurrency must be used within CurrencyProvider");
  }
  return context;
}
