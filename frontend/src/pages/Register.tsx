import { useState, FormEvent } from "react";
import { useAuth } from "../context/AuthContext";
import { Input } from "../components/ui/Input";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Select } from "../components/ui/Select";
import { useTranslation } from "react-i18next";
import { LanguageSwitcher } from "../components/LanguageSwitcher";
import * as currencies from "@dinero.js/currencies"; 
import getSymbolFromCurrency from "currency-symbol-map";

interface RegisterProps {
  onSwitchToLogin: () => void;
}

export const getCurrencyList = () => { 
    return Object.values(currencies).map((c) => ({
        value: c.code,
        label: c.code + " " + getSymbolFromCurrency(c.code)
    }))
}


export function Register({ onSwitchToLogin }: RegisterProps) {
  const { register } = useAuth();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [selectedCurrency, setSelectedCurrency] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const { t } = useTranslation(); 

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError("");

    if (password !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }

    if (password.length < 6) {
      setError("Password must be at least 6 characters");
      return;
    }

    setLoading(true);

    try {
      await register(username, password, selectedCurrency);
    } catch {
      setError("Username already exists");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-sand-100 to-sand-200 dark:from-charcoal-950 dark:to-charcoal-900 p-4">
      <div className=" absolute left-1 top-0">
        <LanguageSwitcher/>
      </div>
      <Card className="w-full max-w-sm">
        <h1 className="text-2xl font-semibold text-center mb-8 text-charcoal-800 dark:text-sand-100">
          payme
        </h1>
        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            type="text"
            placeholder={t("register.username")}
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
          />
          <Input
            type="password"
            placeholder={t("register.password")}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
          />
          <Input
            type="password"
            placeholder={t("register.confirm_password")}
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            required
          />

          {error && (
            <div className="text-sm text-terracotta-600 dark:text-terracotta-400">
              {error}
            </div>
          )}

          <Select defaultValue="USD" label={t("register.select_currency")} options={getCurrencyList()} onChange={(e) => setSelectedCurrency(e.target.value)}></Select>
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "..." : t("register.create_account")}
          </Button>        
        </form>

        <div className="mt-6 text-center">
          <button
            onClick={onSwitchToLogin}
            className="text-sm text-charcoal-500 hover:text-charcoal-700 dark:text-charcoal-400 dark:hover:text-sand-300"
          >
            {t("register.already_have_account")}?
          </button>
        </div>
      </Card>
    </div>
  );
}

