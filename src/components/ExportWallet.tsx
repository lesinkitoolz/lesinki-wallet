import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Wallet {
  public_key: string;
  encrypted_private_key: number[];
  salt: number[];
  balance: number;
  created_at: string;
  last_updated: string;
  network: string;
}

interface ExportWalletProps {
  wallet: Wallet;
  onExport: (wallet: Wallet, exportType: 'seed' | 'privateKey') => void;
}

const ExportWallet: React.FC<ExportWalletProps> = ({ wallet, onExport }) => {
  const [exportType, setExportType] = useState<'seed' | 'privateKey'>('privateKey');
  const [exportedData, setExportedData] = useState('');
  const [error, setError] = useState('');
  const [showData, setShowData] = useState(false);

  const handleExport = async () => {
    try {
      setError('');
      onExport(wallet, exportType);
      setExportedData('Data copied to clipboard!');
      setShowData(true);
    } catch (err) {
      setError(err as string);
    }
  };

  const copyToClipboard = () => {
    navigator.clipboard.writeText(exportedData);
    alert('Copied to clipboard!');
  };

  return (
    <div className="export-wallet">
      <h2>Export Wallet</h2>
      <p>Export your wallet data. Keep this information secure!</p>

      <div className="export-options">
        <button
          className={exportType === 'privateKey' ? 'active' : ''}
          onClick={() => setExportType('privateKey')}
        >
          Private Key
        </button>
        <button
          className={exportType === 'seed' ? 'active' : ''}
          onClick={() => setExportType('seed')}
        >
          Seed Phrase
        </button>
      </div>

      <div className="export-actions">
        <button onClick={handleExport} className="export-btn">
          Export {exportType === 'seed' ? 'Seed Phrase' : 'Private Key'}
        </button>
      </div>

      {error && <div className="error-message">{error}</div>}

      {showData && (
        <div className="exported-data">
          <h3>Your {exportType === 'seed' ? 'Seed Phrase' : 'Private Key'}:</h3>
          <div className="data-display">
            <code>{exportedData}</code>
          </div>
          <button onClick={copyToClipboard} className="copy-btn">
            Copy to Clipboard
          </button>
        </div>
      )}

      <div className="security-warning">
        <h3>⚠️ Security Warning</h3>
        <ul>
          <li>Never share this information with anyone</li>
          <li>Anyone with this data can access your funds</li>
          <li>Store this securely offline</li>
          <li>This action cannot be undone</li>
        </ul>
      </div>
    </div>
  );
};

export default ExportWallet;