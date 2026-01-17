import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { api } from "../api/client";
import getSymbolFromCurrency from "currency-symbol-map";
import { useCurrency } from "../hooks/useCurrency";
interface User {
  id: number;
  username: string;
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  register: (username: string, password: string, currency: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const { setUserCurrency } = useCurrency();

  useEffect(() => {
    api.auth
      .me()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setLoading(false));
  }, []);

  const login = async (username: string, password: string) => {
    const user = await api.auth.login(username, password);
    setUser(user);
    setUserCurrency(user.currency)
  };

  const register = async (username: string, password: string, currency: string) => {
    await api.auth.register(username, password, currency);
    await login(username, password);
  };

  const logout = async () => {
    try {
      await api.auth.logout();
    } catch {
      // Logout failed, ignore
    } finally {
      setUser(null);
      localStorage.removeItem("currency"); 
    }
  };

  return (
    <AuthContext.Provider value={{ user, loading, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth must be used within AuthProvider");
  return context;
}

