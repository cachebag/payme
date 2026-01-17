import getSymbolFromCurrency from "currency-symbol-map";
import { useEffect, useState } from "react";

export function useCurrency() {
    const [currency, setCurrency] = useState<string>(() => {
        return localStorage.getItem("currency") ?? "USD"
    })
    const [currencySymbol, setCurrencySymbol] = useState<string>(getSymbolFromCurrency(currency) ?? "$")

    const setUserCurrency = (currency: string) => {
        setCurrency(currency); 
        setCurrencySymbol(getSymbolFromCurrency(currency) ?? "$")
    }

    useEffect(() => {
        localStorage.setItem("currency", currency)
    }, [currency])

    return { 
        currency, setUserCurrency, currencySymbol
    }
} 