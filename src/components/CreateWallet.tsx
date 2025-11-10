import React from 'react';

interface CreateWalletProps {
  onCreateWallet: () => void;
}

const CreateWallet: React.FC<CreateWalletProps> = ({ onCreateWallet }) => {
  const [isCreating, setIsCreating] = React.useState(false);

  const handleCreate = async () => {
    setIsCreating(true);
    try {
      await onCreateWallet();
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <div className="create-wallet">
      <h2>Create New Wallet</h2>
      <p>Create a new Solana wallet with a randomly generated keypair.</p>
      <div className="create-wallet-actions">
        <button onClick={handleCreate} className={`create-btn ${isCreating ? 'btn-loading' : ''}`} disabled={isCreating}>
          {isCreating ? 'Creating...' : 'Generate New Wallet'}
        </button>
      </div>
      <div className="security-notice">
        <h3>Security Notice</h3>
        <ul>
          <li>Keep your private key secure and never share it</li>
          <li>Backup your wallet information in a safe place</li>
          <li>This wallet is stored locally on your device</li>
        </ul>
      </div>
    </div>
  );
};

export default CreateWallet;