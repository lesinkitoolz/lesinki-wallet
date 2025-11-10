import React, { useState, useEffect } from 'react';

interface SecuritySettingsProps {
  showToast?: (message: string, type: 'success' | 'error' | 'info') => void;
}

const SecuritySettings: React.FC<SecuritySettingsProps> = ({ showToast }) => {
  const [pinEnabled, setPinEnabled] = useState(false);
  const [pin, setPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [currentPin, setCurrentPin] = useState('');
  const [biometricEnabled, setBiometricEnabled] = useState(false);
  const [autoLockTime, setAutoLockTime] = useState(5);
  const [backupReminder, setBackupReminder] = useState(true);
  const [lastBackup, setLastBackup] = useState<number | null>(null);
  const [showPinSetup, setShowPinSetup] = useState(false);
  const [showPinChange, setShowPinChange] = useState(false);

  useEffect(() => {
    loadSecuritySettings();
  }, []);

  const loadSecuritySettings = () => {
    const settings = localStorage.getItem('lesinki-security-settings');
    if (settings) {
      try {
        const parsed = JSON.parse(settings);
        setPinEnabled(parsed.pinEnabled || false);
        setBiometricEnabled(parsed.biometricEnabled || false);
        setAutoLockTime(parsed.autoLockTime || 5);
        setBackupReminder(parsed.backupReminder !== false);
        setLastBackup(parsed.lastBackup || null);
      } catch (error) {
        console.error('Failed to load security settings:', error);
      }
    }
  };

  const saveSecuritySettings = (settings: any) => {
    localStorage.setItem('lesinki-security-settings', JSON.stringify(settings));
  };

  const handlePinSetup = () => {
    if (pin.length < 4) {
      showToast?.('PIN must be at least 4 digits', 'error');
      return;
    }

    if (pin !== confirmPin) {
      showToast?.('PINs do not match', 'error');
      return;
    }

    const settings = {
      pinEnabled: true,
      pin: btoa(pin), // Simple base64 encoding (in production, use proper encryption)
      biometricEnabled,
      autoLockTime,
      backupReminder,
      lastBackup
    };

    saveSecuritySettings(settings);
    setPinEnabled(true);
    setPin('');
    setConfirmPin('');
    setShowPinSetup(false);
    showToast?.('PIN protection enabled', 'success');
  };

  const handlePinChange = () => {
    const settings = JSON.parse(localStorage.getItem('lesinki-security-settings') || '{}');
    if (currentPin !== atob(settings.pin || '')) {
      showToast?.('Current PIN is incorrect', 'error');
      return;
    }

    if (pin.length < 4) {
      showToast?.('New PIN must be at least 4 digits', 'error');
      return;
    }

    if (pin !== confirmPin) {
      showToast?.('New PINs do not match', 'error');
      return;
    }

    const currentSettings = JSON.parse(localStorage.getItem('lesinki-security-settings') || '{}');
    currentSettings.pin = btoa(pin);

    saveSecuritySettings(currentSettings);
    setCurrentPin('');
    setPin('');
    setConfirmPin('');
    setShowPinChange(false);
    showToast?.('PIN changed successfully', 'success');
  };

  const handlePinDisable = () => {
    const settings = JSON.parse(localStorage.getItem('lesinki-security-settings') || '{}');
    if (currentPin !== atob(settings.pin || '')) {
      showToast?.('PIN is incorrect', 'error');
      return;
    }

    const disableSettings = {
      pinEnabled: false,
      biometricEnabled,
      autoLockTime,
      backupReminder,
      lastBackup
    };

    saveSecuritySettings(settings);
    setPinEnabled(false);
    setCurrentPin('');
    setShowPinChange(false);
    showToast?.('PIN protection disabled', 'info');
  };

  const handleBiometricToggle = async () => {
    if (!biometricEnabled) {
      // Check if biometric authentication is available
      try {
        const available = await navigator.credentials.get({ publicKey: { challenge: new Uint8Array(32) } });
        if (!available) {
          showToast?.('Biometric authentication not available on this device', 'error');
          return;
        }
      } catch (error) {
        showToast?.('Biometric authentication not supported', 'error');
        return;
      }
    }

    const newValue = !biometricEnabled;
    setBiometricEnabled(newValue);

    const settings = JSON.parse(localStorage.getItem('lesinki-security-settings') || '{}');
    settings.biometricEnabled = newValue;
    saveSecuritySettings(settings);

    showToast?.(`Biometric authentication ${newValue ? 'enabled' : 'disabled'}`, 'success');
  };

  const handleAutoLockChange = (time: number) => {
    setAutoLockTime(time);

    const settings = JSON.parse(localStorage.getItem('lesinki-security-settings') || '{}');
    settings.autoLockTime = time;
    saveSecuritySettings(settings);

    showToast?.(`Auto-lock set to ${time} minutes`, 'info');
  };

  const handleBackupReminderToggle = () => {
    const newValue = !backupReminder;
    setBackupReminder(newValue);

    const settings = JSON.parse(localStorage.getItem('lesinki-security-settings') || '{}');
    settings.backupReminder = newValue;
    saveSecuritySettings(settings);

    showToast?.(`Backup reminders ${newValue ? 'enabled' : 'disabled'}`, 'info');
  };

  const handleManualBackup = () => {
    // In a real implementation, this would create a proper encrypted backup
    const backupData = {
      wallets: localStorage.getItem('wallets.json'),
      contacts: localStorage.getItem('lesinki-address-book'),
      settings: localStorage.getItem('lesinki-security-settings'),
      timestamp: Date.now()
    };

    const blob = new Blob([JSON.stringify(backupData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `lesinki-wallet-backup-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    setLastBackup(Date.now());
    const settings = JSON.parse(localStorage.getItem('lesinki-security-settings') || '{}');
    settings.lastBackup = Date.now();
    saveSecuritySettings(settings);

    showToast?.('Backup downloaded successfully', 'success');
  };

  const getBackupStatus = () => {
    if (!lastBackup) return 'Never';

    const daysSince = Math.floor((Date.now() - lastBackup) / (1000 * 60 * 60 * 24));
    if (daysSince === 0) return 'Today';
    if (daysSince === 1) return 'Yesterday';
    return `${daysSince} days ago`;
  };

  return (
    <div className="security-settings">
      <div className="settings-header">
        <h2>Security Settings</h2>
        <p>Configure security options for your wallet</p>
      </div>

      <div className="settings-section">
        <h3>PIN Protection</h3>
        <div className="setting-item">
          <div className="setting-info">
            <label>PIN Protection</label>
            <p>Require PIN to access your wallet</p>
          </div>
          <div className="setting-controls">
            {pinEnabled ? (
              <div className="pin-controls">
                <span className="status-enabled">Enabled</span>
                <button onClick={() => setShowPinChange(true)} className="change-btn">
                  Change PIN
                </button>
              </div>
            ) : (
              <button onClick={() => setShowPinSetup(true)} className="enable-btn">
                Set Up PIN
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="settings-section">
        <h3>Biometric Authentication</h3>
        <div className="setting-item">
          <div className="setting-info">
            <label>Biometric Unlock</label>
            <p>Use fingerprint or face recognition</p>
          </div>
          <label className="toggle">
            <input
              type="checkbox"
              checked={biometricEnabled}
              onChange={handleBiometricToggle}
              aria-label="Enable biometric authentication"
            />
            <span className="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div className="settings-section">
        <h3>Auto-Lock</h3>
        <div className="setting-item">
          <div className="setting-info">
            <label>Auto-Lock Timer</label>
            <p>Automatically lock wallet after inactivity</p>
          </div>
          <select
            value={autoLockTime}
            onChange={(e) => handleAutoLockChange(parseInt(e.target.value))}
            className="time-select"
            aria-label="Auto-lock timer"
          >
            <option value={1}>1 minute</option>
            <option value={5}>5 minutes</option>
            <option value={15}>15 minutes</option>
            <option value={30}>30 minutes</option>
            <option value={60}>1 hour</option>
            <option value={0}>Never</option>
          </select>
        </div>
      </div>

      <div className="settings-section">
        <h3>Backup & Recovery</h3>
        <div className="setting-item">
          <div className="setting-info">
            <label>Backup Reminders</label>
            <p>Remind me to backup my wallet</p>
          </div>
          <label className="toggle">
            <input
              type="checkbox"
              checked={backupReminder}
              onChange={handleBackupReminderToggle}
              aria-label="Enable backup reminders"
            />
            <span className="toggle-slider"></span>
          </label>
        </div>

        <div className="setting-item">
          <div className="setting-info">
            <label>Manual Backup</label>
            <p>Last backup: {getBackupStatus()}</p>
          </div>
          <button onClick={handleManualBackup} className="backup-btn">
            Download Backup
          </button>
        </div>
      </div>

      {/* PIN Setup Modal */}
      {showPinSetup && (
        <div className="modal-overlay" onClick={() => setShowPinSetup(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>Set Up PIN</h3>
            <div className="pin-inputs">
              <input
                type="password"
                placeholder="Enter PIN (4+ digits)"
                value={pin}
                onChange={(e) => setPin(e.target.value.replace(/\D/g, ''))}
                maxLength={6}
              />
              <input
                type="password"
                placeholder="Confirm PIN"
                value={confirmPin}
                onChange={(e) => setConfirmPin(e.target.value.replace(/\D/g, ''))}
                maxLength={6}
              />
            </div>
            <div className="modal-actions">
              <button onClick={() => setShowPinSetup(false)} className="cancel-btn">
                Cancel
              </button>
              <button onClick={handlePinSetup} className="confirm-btn">
                Set PIN
              </button>
            </div>
          </div>
        </div>
      )}

      {/* PIN Change Modal */}
      {showPinChange && (
        <div className="modal-overlay" onClick={() => setShowPinChange(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>{pinEnabled ? 'Change PIN' : 'Disable PIN'}</h3>
            <div className="pin-inputs">
              <input
                type="password"
                placeholder="Current PIN"
                value={currentPin}
                onChange={(e) => setCurrentPin(e.target.value.replace(/\D/g, ''))}
                maxLength={6}
              />
              {pinEnabled && (
                <>
                  <input
                    type="password"
                    placeholder="New PIN (4+ digits)"
                    value={pin}
                    onChange={(e) => setPin(e.target.value.replace(/\D/g, ''))}
                    maxLength={6}
                  />
                  <input
                    type="password"
                    placeholder="Confirm New PIN"
                    value={confirmPin}
                    onChange={(e) => setConfirmPin(e.target.value.replace(/\D/g, ''))}
                    maxLength={6}
                  />
                </>
              )}
            </div>
            <div className="modal-actions">
              <button onClick={() => setShowPinChange(false)} className="cancel-btn">
                Cancel
              </button>
              <button
                onClick={pinEnabled ? handlePinChange : handlePinDisable}
                className="confirm-btn"
              >
                {pinEnabled ? 'Change PIN' : 'Disable PIN'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SecuritySettings;