import React, { useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Wallet {
  public_key: string;
  private_key: number[];
  balance: number;
}

interface DappBrowserProps {
  wallet: Wallet;
}

interface TransactionRequest {
  from: string;
  to: string;
  amount: number;
  message?: string;
}

const DappBrowser: React.FC<DappBrowserProps> = ({ wallet }) => {
  const [url, setUrl] = useState('https://jup.ag');
  const [currentUrl, setCurrentUrl] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const [pendingTransaction, setPendingTransaction] = useState<TransactionRequest | null>(null);
  const webviewRef = useRef<HTMLIFrameElement>(null);

  const handleConnect = () => {
    setIsConnected(true);
    // In a real implementation, this would inject the wallet provider into the webview
    alert(`Connected to ${wallet.public_key.slice(0, 8)}...${wallet.public_key.slice(-8)}`);
  };

  const handleDisconnect = () => {
    setIsConnected(false);
    setPendingTransaction(null);
  };

  const handleNavigate = () => {
    setCurrentUrl(url);
  };

  const handleSignTransaction = async (transaction: TransactionRequest) => {
    try {
      const signature: string = await invoke('sign_transaction', {
        publicKey: wallet.public_key,
        privateKey: wallet.private_key,
        recipient: transaction.to,
        amount: Math.floor(transaction.amount * 1e9), // Convert SOL to lamports
      });
      alert(`Transaction signed: ${signature}`);
      setPendingTransaction(null);
    } catch (error) {
      console.error('Failed to sign transaction:', error);
      alert('Failed to sign transaction');
    }
  };

  const handleRejectTransaction = () => {
    setPendingTransaction(null);
    alert('Transaction rejected');
  };

  // Simulate receiving a transaction request from dapp
  const simulateTransactionRequest = () => {
    const mockTransaction: TransactionRequest = {
      from: wallet.public_key,
      to: '11111111111111111111111111111112', // System program
      amount: 0.01,
      message: 'Test transaction from dapp',
    };
    setPendingTransaction(mockTransaction);
  };

  return (
    <div className="dapp-browser">
      <div className="browser-header">
        <div className="wallet-status">
          <span className={`status-indicator ${isConnected ? 'connected' : 'disconnected'}`}></span>
          <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
          <code>{wallet.public_key.slice(0, 8)}...{wallet.public_key.slice(-8)}</code>
        </div>
        <div className="browser-controls">
          {isConnected ? (
            <button onClick={handleDisconnect} className="disconnect-btn">
              Disconnect
            </button>
          ) : (
            <button onClick={handleConnect} className="connect-btn">
              Connect Wallet
            </button>
          )}
          <button onClick={simulateTransactionRequest} className="test-btn">
            Simulate Transaction
          </button>
        </div>
      </div>

      <div className="url-bar">
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="Enter dapp URL"
        />
        <button onClick={handleNavigate}>Go</button>
      </div>

      <div className="webview-container">
        <iframe
          ref={webviewRef as any}
          src={currentUrl || 'about:blank'}
          className="webview"
          title="Dapp Browser"
          sandbox="allow-scripts allow-same-origin"
        />
      </div>

      {pendingTransaction && (
        <div className="transaction-modal">
          <div className="transaction-content">
            <h3>Transaction Request</h3>
            <div className="transaction-details">
              <p><strong>From:</strong> {pendingTransaction.from.slice(0, 8)}...{pendingTransaction.from.slice(-8)}</p>
              <p><strong>To:</strong> {pendingTransaction.to.slice(0, 8)}...{pendingTransaction.to.slice(-8)}</p>
              <p><strong>Amount:</strong> {pendingTransaction.amount} SOL</p>
              {pendingTransaction.message && <p><strong>Message:</strong> {pendingTransaction.message}</p>}
            </div>
            <div className="transaction-actions">
              <button onClick={handleRejectTransaction} className="reject-btn">
                Reject
              </button>
              <button onClick={() => handleSignTransaction(pendingTransaction)} className="approve-btn">
                Approve
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default DappBrowser;