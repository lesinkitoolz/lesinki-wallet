import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Transaction {
  signature: string;
  timestamp: number;
  amount: number;
  type: 'send' | 'receive';
  counterparty: string;
  status: 'confirmed' | 'pending' | 'failed';
}

interface TransactionHistoryProps {
  wallet: {
    public_key: string;
    private_key: number[];
    balance: number;
  };
  network: 'mainnet' | 'devnet' | 'testnet';
}

const TransactionHistory: React.FC<TransactionHistoryProps> = ({ wallet, network }) => {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    fetchTransactionHistory();
  }, [wallet.public_key, network]);

  const fetchTransactionHistory = async () => {
    try {
      setLoading(true);
      setError('');
      // In a real implementation, you'd fetch transaction history from Solana RPC
      // For now, we'll simulate some mock data
      const mockTransactions: Transaction[] = [
        {
          signature: '5K...8L',
          timestamp: Date.now() - 86400000,
          amount: 0.5,
          type: 'receive',
          counterparty: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
          status: 'confirmed'
        },
        {
          signature: '3M...9P',
          timestamp: Date.now() - 172800000,
          amount: 1.2,
          type: 'send',
          counterparty: 'So11111111111111111111111111111111111111112',
          status: 'confirmed'
        }
      ];
      setTransactions(mockTransactions);
    } catch (err) {
      setError('Failed to load transaction history');
      console.error('Failed to fetch transaction history:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatAmount = (amount: number, type: 'send' | 'receive') => {
    const sign = type === 'send' ? '-' : '+';
    return `${sign}${amount.toFixed(4)} SOL`;
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString();
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'confirmed': return '#00ff88';
      case 'pending': return '#ffa500';
      case 'failed': return '#ff4444';
      default: return '#666';
    }
  };

  if (loading) {
    return (
      <div className="transaction-history">
        <h2>Transaction History</h2>
        <div className="loading">Loading transactions...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="transaction-history">
        <h2>Transaction History</h2>
        <div className="error-message">{error}</div>
        <button onClick={fetchTransactionHistory} className="retry-btn">Retry</button>
      </div>
    );
  }

  return (
    <div className="transaction-history">
      <div className="history-header">
        <h2>Transaction History</h2>
        <button onClick={fetchTransactionHistory} className="refresh-btn">↻ Refresh</button>
      </div>

      {transactions.length === 0 ? (
        <p>No transactions found for this wallet.</p>
      ) : (
        <div className="transaction-list">
          {transactions.map((tx, index) => (
            <div key={index} className="transaction-item">
              <div className="transaction-main">
                <div className="transaction-type">
                  <span className={`type-indicator ${tx.type}`}></span>
                  <span>{tx.type === 'send' ? 'Sent' : 'Received'}</span>
                </div>
                <div className={`transaction-amount ${tx.type === 'send' ? 'send' : 'receive'}`}>
                  {formatAmount(tx.amount, tx.type)}
                </div>
              </div>
              <div className="transaction-details">
                <div className="transaction-counterparty">
                  {tx.type === 'send' ? 'To:' : 'From:'} {tx.counterparty.slice(0, 8)}...{tx.counterparty.slice(-8)}
                </div>
                <div className="transaction-meta">
                  <span className="transaction-date">{formatDate(tx.timestamp)}</span>
                  <span className={`transaction-status ${tx.status}`}>
                    {tx.status}
                  </span>
                </div>
              </div>
              <div className="transaction-signature">
                <code>{tx.signature}</code>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default TransactionHistory;