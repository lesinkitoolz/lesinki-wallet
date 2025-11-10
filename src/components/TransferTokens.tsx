import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Wallet {
  public_key: string;
  private_key: number[];
  balance: number;
}

interface TransferTokensProps {
  wallet: Wallet;
  onTransferComplete: () => void;
  network?: 'mainnet' | 'devnet' | 'testnet';
  showToast?: (message: string, type: 'success' | 'error' | 'info') => void;
}

const TransferTokens: React.FC<TransferTokensProps> = ({ wallet, onTransferComplete, network = 'mainnet', showToast }) => {
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [tokenMint, setTokenMint] = useState('');
  const [isSOL, setIsSOL] = useState(true);
  const [useJito, setUseJito] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  const handleTransfer = async () => {
    if (!recipient || !amount) {
      setError('Please fill in all required fields');
      return;
    }

    try {
      setLoading(true);
      setError('');
      setSuccess('');

      const amountLamports = Math.floor(parseFloat(amount) * 1e9); // Convert SOL to lamports

      const signature = useJito ? await invoke('send_bundle_transaction', {
        public_key: wallet.public_key,
        private_key: wallet.private_key,
        recipient,
        amount: amountLamports,
        network,
      }) : await invoke('transfer_tokens', {
        fromPublicKey: wallet.public_key,
        fromPrivateKey: wallet.private_key,
        toPublicKey: recipient,
        amount: amountLamports,
        tokenMint: isSOL ? null : tokenMint || null,
        network,
      });

      setSuccess(`Transaction successful! Signature: ${signature}`);
      setRecipient('');
      setAmount('');
      setTokenMint('');
      onTransferComplete();
      showToast?.('Transaction completed successfully!', 'success');
    } catch (err) {
      setError(err as string);
      showToast?.('Transaction failed', 'error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="transfer-tokens">
      <h2>Transfer Tokens</h2>

      <div className="transfer-form">
        <div className="token-type-selector">
          <button
            className={isSOL ? 'active' : ''}
            onClick={() => setIsSOL(true)}
          >
            SOL
          </button>
          <button
            className={!isSOL ? 'active' : ''}
            onClick={() => setIsSOL(false)}
          >
            SPL Token
          </button>
        </div>

        <div className="form-group">
          <label>Recipient Address:</label>
          <input
            type="text"
            value={recipient}
            onChange={(e) => setRecipient(e.target.value)}
            placeholder="Enter recipient's Solana address"
          />
        </div>

        <div className="form-group">
          <label>Amount ({isSOL ? 'SOL' : 'Tokens'}):</label>
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder={`Enter amount in ${isSOL ? 'SOL' : 'tokens'}`}
            step="0.000000001"
            min="0"
          />
        </div>

        {!isSOL && (
          <div className="form-group">
            <label>Token Mint Address:</label>
            <input
              type="text"
              value={tokenMint}
              onChange={(e) => setTokenMint(e.target.value)}
              placeholder="Enter token mint address"
            />
          </div>
        )}

        <div className="form-group">
          <label>
            <input
              type="checkbox"
              checked={useJito}
              onChange={(e) => setUseJito(e.target.checked)}
            />
            Use Jito Bundler (MEV Protection)
          </label>
        </div>

        {error && <div className="error-message">{error}</div>}
        {success && <div className="success-message">{success}</div>}

        <button
          onClick={handleTransfer}
          disabled={loading || !recipient || !amount}
          className={`transfer-btn ${loading ? 'btn-loading' : ''}`}
        >
          {loading ? 'Processing...' : 'Transfer'}
        </button>
      </div>

      <div className="transfer-info">
        <h3>Transfer Information</h3>
        <ul>
          <li>From: {wallet.public_key.slice(0, 8)}...{wallet.public_key.slice(-8)}</li>
          <li>Network: {network.charAt(0).toUpperCase() + network.slice(1)} {network === 'mainnet' ? 'Beta' : ''}</li>
          <li>Fee: ~0.000005 SOL (estimated)</li>
          <li>Token: {isSOL ? 'SOL' : 'SPL Token'}</li>
          {!isSOL && tokenMint && <li>Token Mint: {tokenMint.slice(0, 8)}...{tokenMint.slice(-8)}</li>}
        </ul>
      </div>
    </div>
  );
};

export default TransferTokens;