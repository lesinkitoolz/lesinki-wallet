import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Wallet {
  public_key: string;
  private_key: number[];
  salt: number[];
  balance: number;
  created_at: string;
  last_updated: string;
  network: string;
}

interface WalletListProps {
  wallets: Wallet[];
  onSelectWallet: (wallet: Wallet) => void;
  selectedWallet: Wallet | null;
  network: 'mainnet' | 'devnet';
}

const WalletList: React.FC<WalletListProps> = ({ wallets, onSelectWallet, selectedWallet, network }) => {
  const [balances, setBalances] = useState<{ [key: string]: number }>({});

  useEffect(() => {
    const fetchBalances = async () => {
      const newBalances: { [key: string]: number } = {};
      for (const wallet of wallets) {
        try {
          const balance: number = await invoke('get_balance', { publicKey: wallet.public_key, network });
          newBalances[wallet.public_key] = balance;
        } catch (error) {
          console.error('Failed to fetch balance for', wallet.public_key, error);
          newBalances[wallet.public_key] = 0;
        }
      }
      setBalances(newBalances);
    };

    if (wallets.length > 0) {
      fetchBalances();
    }
  }, [wallets, network]);

  const formatBalance = (balance: number) => {
    return (balance / 1e9).toFixed(4); // Convert lamports to SOL
  };

  const refreshBalances = async () => {
    const newBalances: { [key: string]: number } = {};
    for (const wallet of wallets) {
      try {
        const balance: number = await invoke('get_balance', { publicKey: wallet.public_key, network });
        newBalances[wallet.public_key] = balance;
      } catch (error) {
        console.error('Failed to fetch balance for', wallet.public_key, error);
        newBalances[wallet.public_key] = 0;
      }
    }
    setBalances(newBalances);
  };

  return (
    <div className="wallet-list">
      <div className="wallet-list-header">
        <h2>Your Wallets</h2>
        {wallets.length > 0 && (
          <button onClick={refreshBalances} className="refresh-btn">
            ↻ Refresh Balances
          </button>
        )}
      </div>
      {wallets.length === 0 ? (
        <p>No wallets found. Create your first wallet!</p>
      ) : (
        <div className="wallet-grid">
          {wallets.map((wallet, index) => (
            <div
              key={wallet.public_key}
              className={`wallet-card ${selectedWallet?.public_key === wallet.public_key ? 'selected' : ''}`}
              onClick={() => onSelectWallet(wallet)}
            >
              <div className="wallet-header">
                <h3>Wallet {index + 1}</h3>
                <div className="wallet-balance">
                  {balances[wallet.public_key] !== undefined ? (
                    <span>{formatBalance(balances[wallet.public_key])} SOL</span>
                  ) : (
                    <span className="loading-spinner"></span>
                  )}
                </div>
              </div>
              <div className="wallet-address">
                <code>{wallet.public_key.slice(0, 8)}...{wallet.public_key.slice(-8)}</code>
              </div>
              <div className="wallet-actions">
                <button onClick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(wallet.public_key); }}>
                  Copy Address
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default WalletList;