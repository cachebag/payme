import { useState } from "react";
import { useAuth } from "./context/AuthContext";
import { Login } from "./pages/Login";
import { Register } from "./pages/Register";
import { Dashboard } from "./pages/Dashboard";
import { Settings } from "./pages/Settings";
import { SummaryPage } from "./pages/Summary";
import { Loader2 } from "lucide-react";

export default function App() {
  const { user, loading } = useAuth();
  const [showRegister, setShowRegister] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showSummary, setShowSummary] = useState(false);
  const [summaryMonthId, setSummaryMonthId] = useState<number | null>(null);

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-sand-50 dark:bg-charcoal-950">
        <Loader2 size={32} className="animate-spin text-sage-500" />
      </div>
    );
  }

  if (!user) {
    return showRegister ? (
      <Register onSwitchToLogin={() => setShowRegister(false)} />
    ) : (
      <Login onSwitchToRegister={() => setShowRegister(true)} />
    );
  }

  if (showSettings) {
    return <Settings onBack={() => setShowSettings(false)} />;
  }

  if (showSummary) {
    return (
      <SummaryPage
        onBack={() => setShowSummary(false)}
        initialMonthId={summaryMonthId}
      />
    );
  }

  return (
    <Dashboard
      onSettingsClick={() => setShowSettings(true)}
      onSummaryClick={(monthId) => {
        setSummaryMonthId(monthId ?? null);
        setShowSummary(true);
      }}
    />
  );
}

