import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import WalletList from './components/WalletList';
import CreateWallet from './components/CreateWallet';
import ImportWallet from './components/ImportWallet';
import ExportWallet from './components/ExportWallet';
import TransferTokens from './components/TransferTokens';
import DappBrowser from './components/DappBrowser';
import TransactionHistory from './components/TransactionHistory';
import PortfolioDashboard from './components/PortfolioDashboard';
import NFTGallery from './components/NFTGallery';
import AddressBook from './components/AddressBook';
import StakingInterface from './components/StakingInterface';
import SwapInterface from './components/SwapInterface';
import SecuritySettings from './components/SecuritySettings';
import PumpfunBundler from './components/PumpfunBundler';
import Toast from './components/Toast';
import './styles.css';

interface Wallet {
  public_key: string;
  encrypted_private_key: number[];
  salt: number[];
  balance: number;
  created_at: string;
  last_updated: string;
  network: string;
}

interface LegacyWallet {
  public_key: string;
  private_key: number[];
  salt: number[];
  balance: number;
  created_at: string;
  last_updated: string;
  network: string;
}

const App: React.FC = () => {
  const [wallets, setWallets] = useState<Wallet[]>([]);
  const [currentView, setCurrentView] = useState<'wallets' | 'create' | 'import' | 'export' | 'transfer' | 'dapp' | 'history' | 'portfolio' | 'nfts' | 'addressbook' | 'staking' | 'swap' | 'security' | 'pumpfun'>('portfolio');
  const [selectedWallet, setSelectedWallet] = useState<Wallet | null>(null);
  const [selectedLegacyWallet, setSelectedLegacyWallet] = useState<LegacyWallet | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  const [network, setNetwork] = useState<'mainnet' | 'devnet'>('mainnet');
  const [password, setPassword] = useState<string>('');
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>(false);

  useEffect(() => {
    loadWallets();
  }, [password]);

  useEffect(() => {
    // Set loading to false after component mounts
    const timer = setTimeout(() => setIsLoading(false), 100);
    return () => clearTimeout(timer);
  }, []);

  const loadWallets = async () => {
    if (!password) return;

    try {
      const loadedWallets: any = await invoke('load_wallets', { password });
      const convertedWallets: Wallet[] = loadedWallets.wallets.map((wallet: any) => ({
        ...wallet,
        encrypted_private_key: Array.from(wallet.encrypted_private_key),
        salt: Array.from(wallet.salt || [])
      }));
      setWallets(convertedWallets);
      setIsAuthenticated(true);
    } catch (error) {
      console.error('Failed to load wallets:', error);
      showToast('Failed to load wallets - check password', 'error');
    }
  };

  const saveWallets = async (updatedWallets: Wallet[]) => {
    if (!password) return;

    try {
      const backendWallets = {
        wallets: updatedWallets.map(wallet => ({
          ...wallet,
          encrypted_private_key: wallet.encrypted_private_key
        })),
        version: "1.0"
      };
      await invoke('save_wallets', { wallets: backendWallets, password });
      setWallets(updatedWallets);
    } catch (error) {
      console.error('Failed to save wallets:', error);
      showToast('Failed to save wallets', 'error');
    }
  };

  const showToast = (message: string, type: 'success' | 'error' | 'info') => {
    setToast({ message, type });
  };

  const hideToast = () => {
    setToast(null);
  };

  const handleCreateWallet = async () => {
    if (!password) {
      showToast('Please set a password first', 'error');
      return;
    }

    try {
      const newWallet: any = await invoke('generate_wallet', { password, network });
      const convertedWallet: Wallet = {
        ...newWallet,
        encrypted_private_key: Array.from(newWallet.encrypted_private_key),
        salt: Array.from(newWallet.salt || [])
      };
      const updatedWallets = [...wallets, convertedWallet];
      await saveWallets(updatedWallets);
      setCurrentView('wallets');
      showToast('Wallet created successfully!', 'success');
    } catch (error) {
      console.error('Failed to create wallet:', error);
      showToast('Failed to create wallet', 'error');
    }
  };

  const handleSelectWallet = (wallet: Wallet) => {
    setSelectedWallet(wallet);
    // Convert to legacy format for components that still expect it
    setSelectedLegacyWallet({
      public_key: wallet.public_key,
      private_key: [], // Will be decrypted when needed
      salt: wallet.salt,
      balance: wallet.balance,
      created_at: wallet.created_at,
      last_updated: wallet.last_updated,
      network: wallet.network
    });
  };

  const handleImportWallet = async (wallet: any) => {
    if (!password) {
      showToast('Please set a password first', 'error');
      return;
    }

    try {
      const convertedWallet: Wallet = {
        ...wallet,
        encrypted_private_key: Array.from(wallet.encrypted_private_key),
        salt: Array.from(wallet.salt || [])
      };
      const updatedWallets = [...wallets, convertedWallet];
      await saveWallets(updatedWallets);
      setCurrentView('wallets');
      showToast('Wallet imported successfully!', 'success');
    } catch (error) {
      console.error('Failed to import wallet:', error);
      showToast('Failed to import wallet', 'error');
    }
  };

  const handleExportWallet = async (wallet: Wallet, exportType: 'seed' | 'privateKey') => {
    if (!password) {
      showToast('Please set a password first', 'error');
      return;
    }

    try {
      let data;
      if (exportType === 'seed') {
        showToast('Seed phrase export is not available for security reasons', 'error');
        return;
      } else {
        data = await invoke('export_wallet_private_key', {
          publicKey: wallet.public_key,
          encryptedPrivateKey: wallet.encrypted_private_key,
          password
        });
      }
      // Copy to clipboard
      navigator.clipboard.writeText(data as string);
      showToast('Private key copied to clipboard!', 'success');
    } catch (error) {
      console.error('Failed to export wallet:', error);
      showToast('Failed to export wallet', 'error');
    }
  };

  const handleConnectDapp = () => {
    if (selectedWallet) {
      setCurrentView('dapp');
    }
  };

  if (isLoading) {
    return (
      <div className="loading-screen">
        <div className="loading-content">
          <h1>Lesinki Wallet</h1>
          <p>Loading...</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="auth-screen">
        <div className="auth-content">
          <h1>Lesinki Wallet</h1>
          <p>Enter your password to access your wallets</p>
          <input
            type="password"
            placeholder="Enter password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="password-input"
          />
          <button onClick={loadWallets} className="auth-btn">Unlock</button>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Lesinki Wallet</h1>
        <div className="header-controls">
          <div className="network-selector">
            <label htmlFor="network-select" className="sr-only">Select Network</label>
            <select
              id="network-select"
              value={network}
              onChange={(e) => setNetwork(e.target.value as 'mainnet' | 'devnet')}
              className="network-select"
            >
              <option value="mainnet">Mainnet</option>
              <option value="devnet">Devnet</option>
            </select>
          </div>
          <nav>
            <button onClick={() => setCurrentView('portfolio')} className={currentView === 'portfolio' ? 'active' : ''}>
              Portfolio
            </button>
            <button onClick={() => setCurrentView('wallets')} className={currentView === 'wallets' ? 'active' : ''}>
              Wallets
            </button>
            <button onClick={() => setCurrentView('create')} className={currentView === 'create' ? 'active' : ''}>
              Create Wallet
            </button>
            <button onClick={() => setCurrentView('import')} className={currentView === 'import' ? 'active' : ''}>
              Import Wallet
            </button>
            {selectedWallet && (
              <>
                <button onClick={() => setCurrentView('export')} className={currentView === 'export' ? 'active' : ''}>
                  Export Wallet
                </button>
                <button onClick={() => setCurrentView('transfer')} className={currentView === 'transfer' ? 'active' : ''}>
                  Transfer
                </button>
                <button onClick={() => setCurrentView('history')} className={currentView === 'history' ? 'active' : ''}>
                  History
                </button>
                <button onClick={() => setCurrentView('nfts')} className={currentView === 'nfts' ? 'active' : ''}>
                  NFTs
                </button>
                <button onClick={() => setCurrentView('addressbook')} className={currentView === 'addressbook' ? 'active' : ''}>
                  Address Book
                </button>
                <button onClick={() => setCurrentView('staking')} className={currentView === 'staking' ? 'active' : ''}>
                  Staking
                </button>
                <button onClick={() => setCurrentView('swap')} className={currentView === 'swap' ? 'active' : ''}>
                  Swap
                </button>
                <button onClick={() => setCurrentView('security')} className={currentView === 'security' ? 'active' : ''}>
                  Security
                </button>
                <button onClick={() => setCurrentView('pumpfun')} className={currentView === 'pumpfun' ? 'active' : ''}>
                  Pumpfun Bundler
                </button>
                <button onClick={handleConnectDapp} className={currentView === 'dapp' ? 'active' : ''}>
                  Dapp Browser
                </button>
              </>
            )}
          </nav>
        </div>
      </header>

      <main className="app-main">
        {currentView === 'portfolio' && isAuthenticated && (
          <PortfolioDashboard wallets={wallets} network={network} />
        )}
        {currentView === 'wallets' && (
          <WalletList
            wallets={wallets.map(w => ({
              public_key: w.public_key,
              private_key: [], // Legacy compatibility
              balance: w.balance,
              salt: w.salt,
              created_at: w.created_at,
              last_updated: w.last_updated,
              network: w.network
            }))}
            onSelectWallet={(wallet: LegacyWallet) => {
              // Find the corresponding encrypted wallet
              const encryptedWallet = wallets.find(w => w.public_key === wallet.public_key);
              if (encryptedWallet) {
                handleSelectWallet(encryptedWallet);
              }
            }}
            selectedWallet={selectedLegacyWallet}
            network={network}
          />
        )}
        {currentView === 'create' && (
          <CreateWallet onCreateWallet={handleCreateWallet} />
        )}
        {currentView === 'import' && (
          <ImportWallet onImportWallet={handleImportWallet} password={password} network={network} />
        )}
        {currentView === 'export' && selectedWallet && (
          <ExportWallet wallet={selectedWallet} onExport={handleExportWallet} />
        )}
        {currentView === 'transfer' && selectedLegacyWallet && (
          <TransferTokens wallet={selectedLegacyWallet} onTransferComplete={() => loadWallets()} network={network} showToast={showToast} />
        )}
        {currentView === 'dapp' && selectedLegacyWallet && (
          <DappBrowser wallet={selectedLegacyWallet} />
        )}
        {currentView === 'history' && selectedLegacyWallet && (
          <TransactionHistory wallet={selectedLegacyWallet} network={network} />
        )}
        {currentView === 'nfts' && selectedLegacyWallet && (
          <NFTGallery wallet={selectedLegacyWallet} network={network} />
        )}
        {currentView === 'addressbook' && (
          <AddressBook />
        )}
        {currentView === 'staking' && selectedLegacyWallet && selectedWallet && (
          <StakingInterface wallet={selectedLegacyWallet} network={network} showToast={showToast} encryptedWallet={selectedWallet} />
        )}
        {currentView === 'swap' && selectedLegacyWallet && (
          <SwapInterface wallet={selectedLegacyWallet} network={network} password={password} showToast={showToast} />
        )}
        {currentView === 'security' && (
          <SecuritySettings showToast={showToast} />
        )}
        {currentView === 'pumpfun' && (
          <PumpfunBundler
            wallets={wallets}
            password={password}
            network={network}
            showToast={showToast}
          />
        )}
      </main>
      {toast && (
        <Toast
          message={toast.message}
          type={toast.type}
          onClose={hideToast}
        />
      )}
    </div>
  );
};

export default App;