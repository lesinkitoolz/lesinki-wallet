import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Wallet {
  public_key: string;
  encrypted_private_key: number[];
  balance: number;
  created_at: string;
  last_updated: string;
  network: string;
}

interface ImportWalletProps {
  onImportWallet: (wallet: Wallet) => void;
  password: string;
  network: string;
}

const ImportWallet: React.FC<ImportWalletProps> = ({ onImportWallet, password, network }) => {
  const [importType, setImportType] = useState<'seed' | 'privateKey'>('seed');
  const [input, setInput] = useState('');
  const [error, setError] = useState('');
  const [isImporting, setIsImporting] = useState(false);

  const handleImport = async () => {
    try {
      setIsImporting(true);
      setError('');
      let wallet: any;
      if (importType === 'seed') {
        wallet = await invoke('import_wallet_from_seed_phrase', { seedPhrase: input, password, network });
      } else {
        wallet = await invoke('import_wallet_from_private_key', { privateKeyHex: input, password, network });
      }
      const convertedWallet: Wallet = {
        ...wallet,
        encrypted_private_key: Array.from(wallet.encrypted_private_key)
      };
      onImportWallet(convertedWallet);
      setInput('');
    } catch (err) {
      setError(err as string);
    } finally {
      setIsImporting(false);
    }
  };

  return (
    <div className="import-wallet">
      <h2>Import Wallet</h2>
      <div className="import-options">
        <button
          className={importType === 'seed' ? 'active' : ''}
          onClick={() => setImportType('seed')}
        >
          Seed Phrase
        </button>
        <button
          className={importType === 'privateKey' ? 'active' : ''}
          onClick={() => setImportType('privateKey')}
        >
          Private Key
        </button>
      </div>

      <div className="import-form">
        <label>
          {importType === 'seed' ? 'Enter your 12 or 24 word seed phrase:' : 'Enter your private key (hex):'}
        </label>
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={importType === 'seed' ? 'word1 word2 word3...' : '0x...'}
          rows={importType === 'seed' ? 4 : 2}
        />
        {error && <div className="error-message">{error}</div>}
        <button onClick={handleImport} disabled={!input.trim() || isImporting} className={isImporting ? 'btn-loading' : ''}>
          {isImporting ? 'Importing...' : 'Import Wallet'}
        </button>
      </div>

      <div className="security-notice">
        <h3>Security Notice</h3>
        <ul>
          <li>Never share your seed phrase or private key with anyone</li>
          <li>Make sure you're importing from a trusted source</li>
          <li>Double-check your input before importing</li>
        </ul>
      </div>
    </div>
  );
};

export default ImportWallet;