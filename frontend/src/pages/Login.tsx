import { useState, FormEvent } from "react";
import { useAuth } from "../context/AuthContext";
import { Input } from "../components/ui/Input";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { useTranslation } from "react-i18next";
import { LanguageSwitcher } from "../components/LanguageSwitcher";
interface LoginProps {
  onSwitchToRegister: () => void;
}

export function Login({ onSwitchToRegister }: LoginProps) {
  const { login } = useAuth();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const { t } = useTranslation();


  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      await login(username, password);
    } catch {
      setError("Invalid credentials");
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
            placeholder={t("login.username")}
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
          />
          <Input
            type="password"
            placeholder={t("login.password")}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
          />

          {error && (
            <div className="text-sm text-terracotta-600 dark:text-terracotta-400">
              {error}
            </div>
          )}

          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "..." : t("login.sign_in")}
          </Button>
        </form>

        <div className="mt-6 text-center">
          <button
            onClick={onSwitchToRegister}
            className="text-sm text-charcoal-500 hover:text-charcoal-700 dark:text-charcoal-400 dark:hover:text-sand-300"
          >
            {t("login.create_account")}
          </button>
        </div>
      </Card>
    </div>
  );
}

