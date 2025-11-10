import React, { useState, useEffect } from 'react';
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

interface BundleWallet extends Wallet {
  isAged: boolean;
  agingTransactions: number;
}

interface TokenMetadata {
  name: string;
  symbol: string;
  description: string;
  logo: string;
  website: string;
  telegram: string;
  twitter: string;
}

interface PumpfunBundlerProps {
  wallets: Wallet[];
  password: string;
  network: string;
  showToast: (message: string, type: 'success' | 'error' | 'info') => void;
}

const PumpfunBundler: React.FC<PumpfunBundlerProps> = ({ wallets, password, network, showToast }) => {
  const [devWallet, setDevWallet] = useState<Wallet | null>(null);
  const [fundingWallet, setFundingWallet] = useState<Wallet | null>(null);
  const [bundleWallets, setBundleWallets] = useState<BundleWallet[]>([]);
  const [selectedSwapDapp, setSelectedSwapDapp] = useState<'photon' | 'jupiter' | 'orca' | 'raydium'>('jupiter');
  const [tokenMetadata, setTokenMetadata] = useState<TokenMetadata>({
    name: '',
    symbol: '',
    description: '',
    logo: '',
    website: '',
    telegram: '',
    twitter: ''
  });
  const [tokenAddress, setTokenAddress] = useState<string>('');
  const [bundleDelay, setBundleDelay] = useState<boolean>(false);
  const [bundlerTip, setBundlerTip] = useState<number>(0.001);
  const [launchProgress, setLaunchProgress] = useState<number>(0);
  const [autoSellPrice, setAutoSellPrice] = useState<number>(0);
  const [sellPercentage, setSellPercentage] = useState<number>(100);
  const [isLaunching, setIsLaunching] = useState<boolean>(false);

  // Wallet Management States
  const [showWalletManagement, setShowWalletManagement] = useState<boolean>(false);
  const [newWalletName, setNewWalletName] = useState<string>('');
  const [importPrivateKey, setImportPrivateKey] = useState<string>('');

  useEffect(() => {
    // Load dev and funding wallets from localStorage or initialize
    const savedDevWallet = localStorage.getItem('pumpfun_dev_wallet');
    const savedFundingWallet = localStorage.getItem('pumpfun_funding_wallet');
    const savedBundleWallets = localStorage.getItem('pumpfun_bundle_wallets');

    if (savedDevWallet) {
      setDevWallet(JSON.parse(savedDevWallet));
    }
    if (savedFundingWallet) {
      setFundingWallet(JSON.parse(savedFundingWallet));
    }
    if (savedBundleWallets) {
      setBundleWallets(JSON.parse(savedBundleWallets));
    }
  }, []);

  const saveWalletsToStorage = () => {
    if (devWallet) localStorage.setItem('pumpfun_dev_wallet', JSON.stringify(devWallet));
    if (fundingWallet) localStorage.setItem('pumpfun_funding_wallet', JSON.stringify(fundingWallet));
    localStorage.setItem('pumpfun_bundle_wallets', JSON.stringify(bundleWallets));
  };

  const handleSetDevWallet = (wallet: Wallet) => {
    setDevWallet(wallet);
    saveWalletsToStorage();
    showToast('Dev wallet set successfully', 'success');
  };

  const handleSetFundingWallet = (wallet: Wallet) => {
    setFundingWallet(wallet);
    saveWalletsToStorage();
    showToast('Funding wallet set successfully', 'success');
  };

  const handleCreateBundleWallet = async () => {
    if (!password) {
      showToast('Password required', 'error');
      return;
    }

    try {
      const newWallet: any = await invoke('generate_wallet', { password, network });
      const bundleWallet: BundleWallet = {
        ...newWallet,
        encrypted_private_key: Array.from(newWallet.encrypted_private_key),
        salt: Array.from(newWallet.salt || []),
        isAged: false,
        agingTransactions: 0
      };
      setBundleWallets([...bundleWallets, bundleWallet]);
      saveWalletsToStorage();
      showToast('Bundle wallet created', 'success');
    } catch (error) {
      showToast('Failed to create bundle wallet', 'error');
    }
  };

  const handleImportBundleWallet = async () => {
    if (!password || !importPrivateKey) {
      showToast('Password and private key required', 'error');
      return;
    }

    try {
      const importedWallet: any = await invoke('import_wallet_from_private_key', {
        privateKey: importPrivateKey,
        password,
        network
      });
      const bundleWallet: BundleWallet = {
        ...importedWallet,
        encrypted_private_key: Array.from(importedWallet.encrypted_private_key),
        salt: Array.from(importedWallet.salt || []),
        isAged: false,
        agingTransactions: 0
      };
      setBundleWallets([...bundleWallets, bundleWallet]);
      saveWalletsToStorage();
      setImportPrivateKey('');
      showToast('Bundle wallet imported', 'success');
    } catch (error) {
      showToast('Failed to import bundle wallet', 'error');
    }
  };

  const handleAgeWallet = async (wallet: BundleWallet) => {
    if (wallet.isAged) return;

    setIsLaunching(true); // Reuse loading state
    try {
      // Convert BundleWallet to backend Wallet format
      const backendWallet: any = {
        public_key: wallet.public_key,
        encrypted_private_key: wallet.encrypted_private_key,
        salt: wallet.salt,
        balance: wallet.balance,
        created_at: wallet.created_at,
        last_updated: wallet.last_updated,
        network: wallet.network
      };

      await invoke('age_wallet', { wallet: backendWallet, password, network });

      const updatedWallet = { ...wallet, isAged: true, agingTransactions: 5 };
      setBundleWallets(bundleWallets.map(w => w.public_key === wallet.public_key ? updatedWallet : w));
      saveWalletsToStorage();
      showToast(`Wallet ${wallet.public_key.slice(0, 8)}... aged successfully`, 'success');
    } catch (error) {
      showToast('Failed to age wallet', 'error');
    } finally {
      setIsLaunching(false);
    }
  };

  const handleAgeAllWallets = async () => {
    const unagedWallets = bundleWallets.filter(w => !w.isAged);
    for (const wallet of unagedWallets) {
      await handleAgeWallet(wallet);
    }
  };

  const handleLaunchToken = async () => {
    if (!devWallet || !fundingWallet || bundleWallets.length === 0) {
      showToast('Dev wallet, funding wallet, and bundle wallets required', 'error');
      return;
    }

    setIsLaunching(true);
    setLaunchProgress(0);

    try {
      // Step 1: Create token (10% progress)
      setLaunchProgress(10);
      const tokenAddressResult = await invoke('create_pump_fun_token', {
        devWallet,
        tokenMetadata,
        password,
        network
      });
      setTokenAddress(tokenAddressResult as string);
      showToast('Token created successfully', 'info');

      // Step 2: Fund bundle wallets (30% progress)
      setLaunchProgress(30);
      const bundleAddresses = bundleWallets.map(w => w.public_key);
      const fundingAmount = 1000000; // 0.001 SOL per wallet in lamports

      await invoke('fund_bundle_wallets', {
        fundingWallet,
        bundleWallets: bundleAddresses,
        amountPerWallet: fundingAmount,
        password,
        network
      });
      showToast('Bundle wallets funded', 'info');

      // Step 3: Execute bundle buys (70% progress)
      setLaunchProgress(70);
      const backendBundleWallets = bundleWallets.map(w => ({
        public_key: w.public_key,
        encrypted_private_key: w.encrypted_private_key,
        salt: w.salt,
        balance: w.balance,
        created_at: w.created_at,
        last_updated: w.last_updated,
        network: w.network
      }));

      await invoke('execute_bundle_buy', {
        bundleWallets: backendBundleWallets,
        tokenAddress: tokenAddressResult as string,
        amountPerWallet: 100000, // 0.0001 SOL per buy
        password,
        swapDapp: selectedSwapDapp,
        network
      });
      showToast('Bundle buys executed', 'info');

      // Step 4: Finalize (100% progress)
      setLaunchProgress(100);
      showToast('Token launched successfully!', 'success');
    } catch (error) {
      console.error('Launch error:', error);
      showToast('Failed to launch token', 'error');
    } finally {
      setIsLaunching(false);
    }
  };

  const handleSellNow = async () => {
    if (!tokenAddress || bundleWallets.length === 0) {
      showToast('No token or bundle wallets available', 'error');
      return;
    }

    try {
      const backendBundleWallets = bundleWallets.map(w => ({
        public_key: w.public_key,
        encrypted_private_key: w.encrypted_private_key,
        salt: w.salt,
        balance: w.balance,
        created_at: w.created_at,
        last_updated: w.last_updated,
        network: w.network
      }));

      await invoke('sell_token_bundle', {
        bundleWallets: backendBundleWallets,
        tokenAddress,
        percentage: sellPercentage,
        password,
        swapDapp: selectedSwapDapp,
        network
      });

      showToast(`Sold ${sellPercentage}% of bundle positions`, 'info');
    } catch (error) {
      showToast('Failed to sell bundle positions', 'error');
    }
  };

  const handleSellDev = async () => {
    if (!tokenAddress || !devWallet) {
      showToast('No token or dev wallet available', 'error');
      return;
    }

    try {
      // Convert dev wallet to backend format
      const backendDevWallet = {
        public_key: devWallet.public_key,
        encrypted_private_key: devWallet.encrypted_private_key,
        salt: devWallet.salt,
        balance: devWallet.balance,
        created_at: devWallet.created_at,
        last_updated: devWallet.last_updated,
        network: devWallet.network
      };

      await invoke('sell_token_bundle', {
        bundleWallets: [backendDevWallet],
        tokenAddress,
        percentage: sellPercentage,
        password,
        swapDapp: selectedSwapDapp,
        network
      });

      showToast(`Sold ${sellPercentage}% of dev position`, 'info');
    } catch (error) {
      showToast('Failed to sell dev position', 'error');
    }
  };

  // Auto-sell monitoring
  useEffect(() => {
    if (!autoSellPrice || autoSellPrice <= 0 || !tokenAddress) return;

    const checkPriceAndSell = async () => {
      try {
        const currentPrice = await invoke('get_token_price', { tokenAddress }) as number;
        if (currentPrice >= autoSellPrice) {
          await handleSellNow();
          showToast(`Auto-sell triggered at price ${currentPrice}`, 'success');
        }
      } catch (error) {
        console.error('Price check failed:', error);
      }
    };

    const interval = setInterval(checkPriceAndSell, 30000); // Check every 30 seconds
    return () => clearInterval(interval);
  }, [autoSellPrice, tokenAddress]);

  return (
    <div className="pumpfun-bundler">
      {/* Dev and Funding Wallets Section - At the top */}
      <div className="wallet-section">
        <div className="wallet-row">
          <div className="wallet-card">
            <h3>Dev Wallet</h3>
            {devWallet ? (
              <div>
                <p>{devWallet.public_key.slice(0, 8)}...{devWallet.public_key.slice(-8)}</p>
                <p>Balance: {devWallet.balance} SOL</p>
              </div>
            ) : (
              <p>Not set</p>
            )}
            <select
              onChange={(e) => {
                const wallet = wallets.find(w => w.public_key === e.target.value);
                if (wallet) handleSetDevWallet(wallet);
              }}
              title="Select Dev Wallet"
            >
              <option value="">Select Dev Wallet</option>
              {wallets.map(wallet => (
                <option key={wallet.public_key} value={wallet.public_key}>
                  {wallet.public_key.slice(0, 8)}...{wallet.public_key.slice(-8)}
                </option>
              ))}
            </select>
          </div>

          <div className="wallet-card">
            <h3>Funding Wallet</h3>
            {fundingWallet ? (
              <div>
                <p>{fundingWallet.public_key.slice(0, 8)}...{fundingWallet.public_key.slice(-8)}</p>
                <p>Balance: {fundingWallet.balance} SOL</p>
              </div>
            ) : (
              <p>Not set</p>
            )}
            <select
              onChange={(e) => {
                const wallet = wallets.find(w => w.public_key === e.target.value);
                if (wallet) handleSetFundingWallet(wallet);
              }}
              title="Select Funding Wallet"
            >
              <option value="">Select Funding Wallet</option>
              {wallets.map(wallet => (
                <option key={wallet.public_key} value={wallet.public_key}>
                  {wallet.public_key.slice(0, 8)}...{wallet.public_key.slice(-8)}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Wallet Management Submenu */}
      <div className="wallet-management">
        <button onClick={() => setShowWalletManagement(!showWalletManagement)}>
          Wallet Management {showWalletManagement ? '▼' : '▶'}
        </button>

        {showWalletManagement && (
          <div className="wallet-management-content">
            <div className="wallet-actions">
              <button onClick={handleCreateBundleWallet}>Create New Bundle Wallet</button>
              <div className="import-section">
                <input
                  type="text"
                  placeholder="Private Key"
                  value={importPrivateKey}
                  onChange={(e) => setImportPrivateKey(e.target.value)}
                />
                <button onClick={handleImportBundleWallet}>Import Bundle Wallet</button>
              </div>
            </div>

            {/* Swap Dapp Selection Sidebar */}
            <div className="swap-dapp-sidebar">
              <h4>Select Swap Dapp</h4>
              <div className="dapp-options">
                {['photon', 'jupiter', 'orca', 'raydium'].map(dapp => (
                  <label key={dapp}>
                    <input
                      type="radio"
                      value={dapp}
                      checked={selectedSwapDapp === dapp}
                      onChange={(e) => setSelectedSwapDapp(e.target.value as any)}
                    />
                    {dapp.charAt(0).toUpperCase() + dapp.slice(1)}
                  </label>
                ))}
              </div>
            </div>

            <div className="bundle-wallets-list">
              <h4>Bundle Wallets ({bundleWallets.length})</h4>
              <button onClick={handleAgeAllWallets} className="age-all-btn">Age All Wallets</button>
              {bundleWallets.map(wallet => (
                <div key={wallet.public_key} className="bundle-wallet-item">
                  <div className="wallet-info">
                    <span>{wallet.public_key.slice(0, 8)}...{wallet.public_key.slice(-8)}</span>
                    <span className={wallet.isAged ? 'aged' : 'not-aged'}>
                      {wallet.isAged ? 'Aged' : 'Not Aged'}
                    </span>
                  </div>
                  {!wallet.isAged && (
                    <button onClick={() => handleAgeWallet(wallet)}>Age Wallet</button>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Launcher Section */}
      <div className="launcher-section">
        <h2>Launcher</h2>
        <div className="metadata-section">
          <h3>Token Metadata</h3>
          <div className="metadata-grid">
            <input
              type="text"
              placeholder="Token Name"
              value={tokenMetadata.name}
              onChange={(e) => setTokenMetadata({...tokenMetadata, name: e.target.value})}
            />
            <input
              type="text"
              placeholder="Symbol"
              value={tokenMetadata.symbol}
              onChange={(e) => setTokenMetadata({...tokenMetadata, symbol: e.target.value})}
            />
            <textarea
              placeholder="Description"
              value={tokenMetadata.description}
              onChange={(e) => setTokenMetadata({...tokenMetadata, description: e.target.value})}
            />
            <input
              type="text"
              placeholder="Logo URL"
              value={tokenMetadata.logo}
              onChange={(e) => setTokenMetadata({...tokenMetadata, logo: e.target.value})}
            />
            <input
              type="text"
              placeholder="Website"
              value={tokenMetadata.website}
              onChange={(e) => setTokenMetadata({...tokenMetadata, website: e.target.value})}
            />
            <input
              type="text"
              placeholder="Telegram"
              value={tokenMetadata.telegram}
              onChange={(e) => setTokenMetadata({...tokenMetadata, telegram: e.target.value})}
            />
            <input
              type="text"
              placeholder="Twitter (X)"
              value={tokenMetadata.twitter}
              onChange={(e) => setTokenMetadata({...tokenMetadata, twitter: e.target.value})}
            />
          </div>
        </div>

        <div className="token-preview">
          <h4>Token Address Preview</h4>
          <p>{tokenAddress || 'Address will be generated on launch'}</p>
        </div>

        <div className="bundle-options">
          <label>
            <input
              type="checkbox"
              checked={bundleDelay}
              onChange={(e) => setBundleDelay(e.target.checked)}
            />
            Delay Bundle with Jito Advance Latest Features
          </label>
          <div className="bundler-tip">
            <label>Instant Bundle with Bundler Jito Tip:</label>
            <input
              type="number"
              step="0.001"
              value={bundlerTip}
              onChange={(e) => setBundlerTip(parseFloat(e.target.value))}
              title="Bundler Tip Amount"
            />
            <span>SOL</span>
          </div>
        </div>
      </div>

      {/* Launch Progress and Controls */}
      <div className="launch-controls">
        <div className="launch-progress-section">
          <h4>Launch Progress</h4>
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${launchProgress}%` }}></div>
          </div>
          <span>{launchProgress}%</span>
        </div>

        <div className="sell-controls-section">
          <div className="auto-sell">
            <label>Auto Sell at Price:</label>
            <input
              type="number"
              value={autoSellPrice}
              onChange={(e) => setAutoSellPrice(parseFloat(e.target.value))}
              placeholder="0.00"
              title="Auto Sell Price"
            />
            <span>SOL</span>
          </div>

          <div className="sell-actions">
            <button onClick={handleSellNow} className="sell-now-btn">Sell Now</button>
            <div className="sell-percentage">
              <input
                type="number"
                min="1"
                max="100"
                value={sellPercentage}
                onChange={(e) => setSellPercentage(parseInt(e.target.value))}
                title="Sell Percentage"
              />
              <span>%</span>
            </div>
            <button onClick={handleSellDev} className="sell-dev-btn">Sell Dev</button>
          </div>
        </div>

        <button
          onClick={handleLaunchToken}
          disabled={isLaunching}
          className="launch-btn"
        >
          {isLaunching ? 'Launching...' : 'Launch Token'}
        </button>
      </div>
    </div>
  );
};

export default PumpfunBundler;