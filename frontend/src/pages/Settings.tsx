import { useState } from "react";
import { Layout } from "../components/Layout";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Select } from "../components/ui/Select";
import { Modal } from "../components/ui/Modal";
import { useAuth } from "../context/AuthContext";
import { useCurrency, SUPPORTED_CURRENCIES } from "../context/CurrencyContext";
import { useUIPreferences } from "../context/UIPreferencesContext";
import { api } from "../api/client";
import { ArrowLeft, Info } from "lucide-react";

interface SettingsProps {
  onBack: () => void;
}

export function Settings({ onBack }: SettingsProps) {
  const { user, logout, updateUsername } = useAuth();
  const { currency, setCurrency, formatCurrency } = useCurrency();
  const { transfersEnabled, setTransfersEnabled } = useUIPreferences();
  const [newUsername, setNewUsername] = useState(user?.username || "");
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [deletePassword, setDeletePassword] = useState("");
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [usernameLoading, setUsernameLoading] = useState(false);
  const [passwordLoading, setPasswordLoading] = useState(false);
  const [deleteLoading, setDeleteLoading] = useState(false);
  const [usernameError, setUsernameError] = useState("");
  const [passwordError, setPasswordError] = useState("");
  const [deleteError, setDeleteError] = useState("");
  const [usernameSuccess, setUsernameSuccess] = useState(false);
  const [passwordSuccess, setPasswordSuccess] = useState(false);
  const [currencySuccess, setCurrencySuccess] = useState(false);
  const [selectedCurrency, setSelectedCurrency] = useState(currency.code);
  const [showTransfersModal, setShowTransfersModal] = useState(false);


  const handleChangeUsername = async (e: React.FormEvent) => {
    e.preventDefault();
    setUsernameError("");
    setUsernameSuccess(false);

    if (newUsername.length < 3 || newUsername.length > 32) {
      setUsernameError("Username must be 3-32 characters");
      return;
    }

    setUsernameLoading(true);
    try {
      const response = await api.auth.changeUsername(newUsername);
      updateUsername(response.username);
      setUsernameSuccess(true);
      setTimeout(() => setUsernameSuccess(false), 3000);
    } catch {
      setUsernameError("Failed to change username. It may already be taken.");
    } finally {
      setUsernameLoading(false);
    }
  };

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    setPasswordError("");
    setPasswordSuccess(false);

    if (newPassword.length < 6 || newPassword.length > 128) {
      setPasswordError("Password must be 6-128 characters");
      return;
    }

    if (newPassword !== confirmPassword) {
      setPasswordError("Passwords do not match");
      return;
    }

    setPasswordLoading(true);
    try {
      await api.auth.changePassword(currentPassword, newPassword);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      setPasswordSuccess(true);
      setTimeout(() => setPasswordSuccess(false), 3000);
    } catch {
      setPasswordError("Failed to change password. Check your current password.");
    } finally {
      setPasswordLoading(false);
    }
  };

  const handleClearData = async () => {
    setDeleteError("");

    if (deletePassword.length < 6) {
      setDeleteError("Please enter your password");
      return;
    }

    setDeleteLoading(true);
    try {
      await api.auth.clearAllData(deletePassword);
      await logout();
    } catch {
      setDeleteError("Failed to clear data. Check your password.");
      setDeleteLoading(false);
    }
  };

  const handleSaveCurrency = () => {
    setCurrency(selectedCurrency);
    setCurrencySuccess(true);
  };

  return (
    <Layout>
      <div className="max-w-2xl mx-auto">
        <button
          onClick={onBack}
          className="mb-4 sm:mb-6 flex items-center gap-2 text-sm text-charcoal-600 dark:text-charcoal-400 hover:text-charcoal-900 dark:hover:text-sand-100 transition-colors touch-manipulation"
        >
          <ArrowLeft size={16} />
          Back to Dashboard
        </button>

        <h1 className="text-xl sm:text-2xl font-semibold mb-6 sm:mb-8 text-charcoal-800 dark:text-sand-100">
          Settings
        </h1>

        <div className="space-y-6 sm:space-y-8">
          <div className="bg-sand-100 dark:bg-charcoal-900 p-4 sm:p-6 border border-sand-200 dark:border-charcoal-800">
            <h2 className="text-base sm:text-lg font-medium mb-4 text-charcoal-800 dark:text-sand-100">
              Currency
            </h2>
            <div className="space-y-4">
              <Select
                label="Display Currency"
                value={selectedCurrency}
                onChange={(e) => setSelectedCurrency(e.target.value)}
                options={SUPPORTED_CURRENCIES.map((c) => ({
                  value: c.code,
                  label: `${c.symbol} ${c.code} - ${c.name}`,
                }))}
              />
              <p className="text-xs text-charcoal-500 dark:text-charcoal-400">
                All monetary values will be displayed in {currency.name} ({currency.symbol}).
                <br />
                Example: {formatCurrency(1234.56)}
              </p>
              {currencySuccess && (
                <p className="text-sm text-sage-600">Currency changed successfully</p>
              )}
              <Button onClick={handleSaveCurrency} disabled={selectedCurrency === currency.code}>
                Save Currency
              </Button>
            </div>
          </div>

          <div className="bg-sand-100 dark:bg-charcoal-900 p-4 sm:p-6 border border-sand-200 dark:border-charcoal-800">
            <h2 className="text-base sm:text-lg font-medium mb-4 text-charcoal-800 dark:text-sand-100">
              Transferred Items
            </h2>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <label className="text-sm font-medium text-charcoal-700 dark:text-sand-300">
                      Enable Transferred Items
                    </label>
                    <button
                      onClick={() => setShowTransfersModal(true)}
                      className="p-0.5 hover:bg-sand-200 dark:hover:bg-charcoal-700 rounded transition-colors touch-manipulation"
                      title="How to use transfers"
                    >
                      <Info size={14} className="text-charcoal-400 hover:text-charcoal-600 dark:hover:text-charcoal-300" />
                    </button>
                  </div>
                  <p className="text-xs text-charcoal-500 dark:text-charcoal-400">
                    Allow adding, editing, and deleting transferred items
                  </p>
                </div>
                <button
                  onClick={() => setTransfersEnabled(!transfersEnabled)}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    transfersEnabled
                      ? "bg-sage-600 dark:bg-sage-500"
                      : "bg-charcoal-300 dark:bg-charcoal-600"
                  }`}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                      transfersEnabled ? "translate-x-6" : "translate-x-1"
                    }`}
                  />
                </button>
              </div>
            </div>
          </div>

          <div className="bg-sand-100 dark:bg-charcoal-900 p-4 sm:p-6 border border-sand-200 dark:border-charcoal-800">
            <h2 className="text-base sm:text-lg font-medium mb-4 text-charcoal-800 dark:text-sand-100">
              Change Username
            </h2>
            <form onSubmit={handleChangeUsername} className="space-y-4">
              <Input
                label="New Username"
                type="text"
                value={newUsername}
                onChange={(e) => setNewUsername(e.target.value)}
                placeholder="Enter new username"
                disabled={usernameLoading}
              />
              {usernameError && (
                <p className="text-sm text-terracotta-600">{usernameError}</p>
              )}
              {usernameSuccess && (
                <p className="text-sm text-sage-600">Username changed successfully</p>
              )}
              <Button type="submit" disabled={usernameLoading || newUsername === user?.username}>
                {usernameLoading ? "Saving..." : "Save Username"}
              </Button>
            </form>
          </div>

          <div className="bg-sand-100 dark:bg-charcoal-900 p-4 sm:p-6 border border-sand-200 dark:border-charcoal-800">
            <h2 className="text-base sm:text-lg font-medium mb-4 text-charcoal-800 dark:text-sand-100">
              Change Password
            </h2>
            <form onSubmit={handleChangePassword} className="space-y-4">
              <Input
                label="Current Password"
                type="password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                placeholder="Enter current password"
                disabled={passwordLoading}
              />
              <Input
                label="New Password"
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="Enter new password"
                disabled={passwordLoading}
              />
              <Input
                label="Confirm New Password"
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Confirm new password"
                disabled={passwordLoading}
              />
              {passwordError && (
                <p className="text-sm text-terracotta-600">{passwordError}</p>
              )}
              {passwordSuccess && (
                <p className="text-sm text-sage-600">Password changed successfully</p>
              )}
              <Button type="submit" disabled={passwordLoading}>
                {passwordLoading ? "Changing..." : "Change Password"}
              </Button>
            </form>
          </div>

          <div className="bg-terracotta-50 dark:bg-charcoal-900 p-4 sm:p-6 border-2 border-terracotta-300 dark:border-terracotta-800">
            <h2 className="text-base sm:text-lg font-medium mb-2 text-terracotta-800 dark:text-terracotta-300">
              Danger Zone
            </h2>
            <p className="text-sm text-charcoal-600 dark:text-charcoal-400 mb-4">
              This action cannot be undone. All your data will be permanently deleted.
            </p>
            <Button variant="danger" onClick={() => setShowDeleteModal(true)}>
              Clear All Data
            </Button>
          </div>
        </div>
      </div>

      <Modal
        isOpen={showDeleteModal}
        onClose={() => {
          setShowDeleteModal(false);
          setDeletePassword("");
          setDeleteError("");
        }}
        title="Clear All Data"
      >
        <div className="space-y-4">
          <p className="text-sm text-charcoal-600 dark:text-charcoal-300">
            This will permanently delete all your data including:
          </p>
          <ul className="text-sm text-charcoal-600 dark:text-charcoal-300 list-disc list-inside space-y-1">
            <li>All months and transactions</li>
            <li>All budget categories</li>
            <li>All fixed expenses</li>
            <li>All income entries</li>
            <li>Your account and settings</li>
          </ul>
          <p className="text-sm font-medium text-terracotta-700 dark:text-terracotta-400">
            This action cannot be undone.
          </p>
          <Input
            label="Confirm your password"
            type="password"
            value={deletePassword}
            onChange={(e) => setDeletePassword(e.target.value)}
            placeholder="Enter your password"
            disabled={deleteLoading}
          />
          {deleteError && (
            <p className="text-sm text-terracotta-600">{deleteError}</p>
          )}
          <div className="flex flex-col sm:flex-row gap-2">
            <Button variant="danger" onClick={handleClearData} disabled={deleteLoading} className="w-full sm:w-auto">
              {deleteLoading ? "Deleting..." : "Yes, Delete Everything"}
            </Button>
            <Button
              variant="ghost"
              onClick={() => {
                setShowDeleteModal(false);
                setDeletePassword("");
                setDeleteError("");
              }}
              disabled={deleteLoading}
              className="w-full sm:w-auto"
            >
              Cancel
            </Button>
          </div>
        </div>
      </Modal>

      <Modal isOpen={showTransfersModal} onClose={() => setShowTransfersModal(false)}>
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-charcoal-800 dark:text-sand-100">
            How to Use Transferred Items
          </h2>
          
          <div className="space-y-3 text-sm text-charcoal-600 dark:text-charcoal-300">
            <div>
              <p className="font-medium text-charcoal-700 dark:text-sand-300 mb-1">What are transfers?</p>
              <p>Track portions of your budgeted spending that you plan to transfer to savings or retirement accounts instead of spending.</p>
            </div>
            
            <div>
              <p className="font-medium text-charcoal-700 dark:text-sand-300 mb-1">Enable/Disable behavior:</p>
              <ul className="space-y-1">
                <li><span className="font-medium">When enabled:</span> You can add, edit, and delete transfers.</li>
                <li><span className="font-medium">When disabled:</span> View only. Card hides once all transfers are deleted.</li>
              </ul>
            </div>
            
            <div>
              <p className="font-medium text-charcoal-700 dark:text-sand-300 mb-1">When to use transfers:</p>
              <ul className="list-disc list-inside space-y-1">
                <li>You have extra budget left and want to save it</li>
                <li>You want to contribute to retirement beyond regular deductions</li>
                <li>You want to track discretionary savings separately</li>
              </ul>
            </div>
            
            <div>
              <p className="font-medium text-charcoal-700 dark:text-sand-300 mb-1">How to add a transfer:</p>
              <ol className="list-decimal list-inside space-y-1 text-xs">
                <li>Open the "Transferred Items" card on the dashboard</li>
                <li>Click the <span className="inline-block">+</span> button to create a transfer</li>
                <li>Fill in the description, amount, and date</li>
                <li>Choose the destination: Savings or Retirement</li>
                <li>Click confirm</li>
              </ol>
            </div>
          </div>

          <div className="flex gap-2 pt-4">
            <Button
              onClick={() => setShowTransfersModal(false)}
              className="w-full"
            >
              Got it
            </Button>
          </div>
        </div>
      </Modal>
    </Layout>
  );
}
