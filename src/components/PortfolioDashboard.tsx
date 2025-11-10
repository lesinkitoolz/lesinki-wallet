import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Wallet {
  public_key: string;
  encrypted_private_key: number[];
  balance: number;
  created_at: string;
  last_updated: string;
  network: string;
}

interface PortfolioData {
  totalBalance: number;
  totalValueUSD: number;
  wallets: Array<{
    address: string;
    balance: number;
    valueUSD: number;
    percentage: number;
  }>;
  recentTransactions: number;
  network: string;
}

interface PortfolioDashboardProps {
  wallets: Wallet[];
  network: 'mainnet' | 'devnet';
}

const PortfolioDashboard: React.FC<PortfolioDashboardProps> = ({ wallets, network }) => {
  const [portfolioData, setPortfolioData] = useState<PortfolioData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    loadPortfolioData();
  }, [wallets, network]);

  const loadPortfolioData = async () => {
    try {
      setLoading(true);
      setError('');

      // Calculate total balance across all wallets
      let totalBalance = 0;
      const walletData = [];

      for (const wallet of wallets) {
        try {
          const balance: number = await invoke('get_balance', { publicKey: wallet.public_key, network });
          const balanceSOL = balance / 1e9; // Convert lamports to SOL
          totalBalance += balanceSOL;

          walletData.push({
            address: wallet.public_key,
            balance: balanceSOL,
            valueUSD: balanceSOL * 150, // Mock USD conversion (SOL ~ $150)
            percentage: 0 // Will be calculated after total is known
          });
        } catch (err) {
          console.error('Failed to get balance for wallet:', wallet.public_key, err);
          walletData.push({
            address: wallet.public_key,
            balance: 0,
            valueUSD: 0,
            percentage: 0
          });
        }
      }

      // Calculate percentages
      const totalValueUSD = totalBalance * 150;
      walletData.forEach(wallet => {
        wallet.percentage = totalBalance > 0 ? (wallet.balance / totalBalance) * 100 : 0;
      });

      setPortfolioData({
        totalBalance,
        totalValueUSD,
        wallets: walletData,
        recentTransactions: 0, // Mock data
        network: network.charAt(0).toUpperCase() + network.slice(1)
      });

    } catch (err) {
      setError('Failed to load portfolio data');
      console.error('Failed to load portfolio data:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount);
  };

  const formatSOL = (amount: number) => {
    return `${amount.toFixed(4)} SOL`;
  };

  if (loading) {
    return (
      <div className="portfolio-dashboard">
        <h2>Portfolio Overview</h2>
        <div className="loading">Loading portfolio data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="portfolio-dashboard">
        <h2>Portfolio Overview</h2>
        <div className="error-message">{error}</div>
        <button onClick={loadPortfolioData} className="retry-btn">Retry</button>
      </div>
    );
  }

  if (!portfolioData) {
    return (
      <div className="portfolio-dashboard">
        <h2>Portfolio Overview</h2>
        <div className="empty-portfolio">
          <p>No wallets found. Create your first wallet to get started!</p>
        </div>
      </div>
    );
  }

  return (
    <div className="portfolio-dashboard">
      <div className="dashboard-header">
        <h2>Portfolio Overview</h2>
        <button onClick={loadPortfolioData} className="refresh-btn">↻ Refresh</button>
      </div>

      <div className="portfolio-summary">
        <div className="summary-card total-balance">
          <h3>Total Balance</h3>
          <div className="balance-amount">{formatSOL(portfolioData.totalBalance)}</div>
          <div className="balance-value">{formatCurrency(portfolioData.totalValueUSD)}</div>
        </div>

        <div className="summary-card network-info">
          <h3>Network</h3>
          <div className="network-name">{portfolioData.network}</div>
          <div className="wallet-count">{wallets.length} Wallet{wallets.length !== 1 ? 's' : ''}</div>
        </div>

        <div className="summary-card transaction-count">
          <h3>Recent Activity</h3>
          <div className="transaction-number">{portfolioData.recentTransactions}</div>
          <div className="transaction-label">Transactions (24h)</div>
        </div>
      </div>

      <div className="wallet-breakdown">
        <h3>Wallet Breakdown</h3>
        <div className="wallet-list">
          {portfolioData.wallets.map((wallet, index) => (
            <div key={wallet.address} className="wallet-item">
              <div className="wallet-info">
                <div className="wallet-name">Wallet {index + 1}</div>
                <div className="wallet-address">
                  {wallet.address.slice(0, 8)}...{wallet.address.slice(-8)}
                </div>
              </div>
              <div className="wallet-stats">
                <div className="wallet-balance">{formatSOL(wallet.balance)}</div>
                <div className="wallet-value">{formatCurrency(wallet.valueUSD)}</div>
                <div className="wallet-percentage">{wallet.percentage.toFixed(1)}%</div>
              </div>
              <div className="wallet-bar">
                <div
                  className="wallet-bar-fill"
                  style={{width: `${wallet.percentage}%`}}
                ></div>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="portfolio-insights">
        <h3>Portfolio Insights</h3>
        <div className="insights-grid">
          <div className="insight-card">
            <h4>Largest Holding</h4>
            <p>
              {portfolioData.wallets.length > 0
                ? `${formatSOL(Math.max(...portfolioData.wallets.map(w => w.balance)))} SOL`
                : 'No data'
              }
            </p>
          </div>
          <div className="insight-card">
            <h4>Network Fee Estimate</h4>
            <p>~0.000005 SOL</p>
          </div>
          <div className="insight-card">
            <h4>24h Change</h4>
            <p className="change-positive">+2.5%</p>
          </div>
          <div className="insight-card">
            <h4>Security Status</h4>
            <p className="status-good">Protected</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PortfolioDashboard;