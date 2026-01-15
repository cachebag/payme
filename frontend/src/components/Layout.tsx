import { ReactNode, useRef, useState } from "react";
import { Moon, Sun, LogOut, Download, Upload, Settings } from "lucide-react";
import { useTheme } from "../context/ThemeContext";
import { useAuth } from "../context/AuthContext";
import { api, UserExport } from "../api/client";
import { Modal } from "./ui/Modal";
import { Button } from "./ui/Button";

interface LayoutProps {
  children: ReactNode;
  onSettingsClick?: () => void;
}

export function Layout({ children, onSettingsClick }: LayoutProps) {
  const { isDark, toggle } = useTheme();
  const { user, logout } = useAuth();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [showImportConfirm, setShowImportConfirm] = useState(false);
  const [pendingImport, setPendingImport] = useState<UserExport | null>(null);
  const [importing, setImporting] = useState(false);

  const handleExport = async () => {
    const data = await api.exportJson();
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `payme-${user?.username}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImportClick = () => {
    fileInputRef.current?.click();
  };

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      const data = JSON.parse(text) as UserExport;
      if (data.version && data.categories && data.months) {
        setPendingImport(data);
        setShowImportConfirm(true);
      }
    } catch {
      // Invalid JSON, ignore
    }

    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  const confirmImport = async () => {
    if (!pendingImport) return;
    setImporting(true);
    try {
      await api.importJson(pendingImport);
      window.location.reload();
    } catch {
      // Import failed, ignore
    } finally {
      setImporting(false);
      setShowImportConfirm(false);
      setPendingImport(null);
    }
  };

  return (
    <div className="min-h-screen">
      <header className="sticky top-0 z-40 bg-sand-50/80 dark:bg-charcoal-950/80 backdrop-blur-md border-b border-sand-200 dark:border-charcoal-800">
        <div className="max-w-6xl mx-auto px-4 py-3 sm:py-4 flex items-center justify-between">
          <span className="text-lg sm:text-xl font-semibold tracking-tight text-charcoal-800 dark:text-sand-100">
            payme
          </span>
          {user && (
            <span className="hidden sm:inline text-sm text-charcoal-600 dark:text-charcoal-300">
              Welcome, {user.username}
            </span>
          )}
          <div className="flex items-center gap-1 sm:gap-2">
            {user && (
              <>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".json"
                  onChange={handleFileSelect}
                  className="hidden"
                />
                <button
                  onClick={handleImportClick}
                  className="p-2 sm:p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors cursor-pointer touch-manipulation"
                  title="Import data"
                  aria-label="Import data"
                >
                  <Upload size={18} />
                </button>
                <button
                  onClick={handleExport}
                  className="p-2 sm:p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors cursor-pointer touch-manipulation"
                  title="Export data"
                  aria-label="Export data"
                >
                  <Download size={18} />
                </button>
              </>
            )}
            <button
              onClick={toggle}
              className="p-2 sm:p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors cursor-pointer touch-manipulation"
              aria-label="Toggle theme"
            >
              {isDark ? <Sun size={18} /> : <Moon size={18} />}
            </button>
            {user && onSettingsClick && (
              <button
                onClick={onSettingsClick}
                className="p-2 sm:p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors cursor-pointer touch-manipulation"
                title="Settings"
                aria-label="Settings"
              >
                <Settings size={18} />
              </button>
            )}
            {user && (
              <button
                onClick={logout}
                className="p-2 sm:p-2 hover:bg-sand-200 dark:hover:bg-charcoal-800 transition-colors cursor-pointer touch-manipulation"
                aria-label="Logout"
              >
                <LogOut size={18} />
              </button>
            )}
          </div>
        </div>
      </header>
      <main className="max-w-6xl mx-auto px-4 py-4 sm:py-8">{children}</main>

      <Modal isOpen={showImportConfirm} onClose={() => setShowImportConfirm(false)} title="Import Data">
        <div className="space-y-4">
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300">
            This will replace all your current data with the imported file.
          </p>
          {pendingImport && (
            <div className="text-xs text-charcoal-500 dark:text-charcoal-400 p-3 bg-sand-100 dark:bg-charcoal-800">
              <div>{pendingImport.categories.length} categories</div>
              <div>{pendingImport.fixed_expenses.length} fixed expenses</div>
              <div>{pendingImport.months.length} months</div>
            </div>
          )}
          <div className="flex flex-col sm:flex-row gap-2">
            <Button onClick={confirmImport} disabled={importing} className="w-full sm:w-auto">
              {importing ? "Importing..." : "Replace My Data"}
            </Button>
            <Button variant="ghost" onClick={() => setShowImportConfirm(false)} className="w-full sm:w-auto">
              Cancel
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
