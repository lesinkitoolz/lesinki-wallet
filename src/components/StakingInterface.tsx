import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface StakingAccount {
  stakeAccount: string;
  validator: string;
  amount: number;
  status: 'activating' | 'active' | 'deactivating' | 'inactive';
  activationEpoch?: number;
  deactivationEpoch?: number;
}

interface Validator {
  votePubkey: string;
  nodePubkey: string;
  stake: number;
  commission: number;
  lastVote: number;
  rootSlot: number;
}

interface StakingInterfaceProps {
  wallet: {
    public_key: string;
    private_key: number[];
    salt: number[];
    balance: number;
    created_at: string;
    last_updated: string;
    network: string;
  };
  network: 'mainnet' | 'devnet' | 'testnet';
  showToast?: (message: string, type: 'success' | 'error' | 'info') => void;
  encryptedWallet?: {
    public_key: string;
    encrypted_private_key: number[];
    salt: number[];
    balance: number;
    created_at: string;
    last_updated: string;
    network: string;
  };
}

const StakingInterface: React.FC<StakingInterfaceProps> = ({ wallet, network, showToast, encryptedWallet }) => {
  const [stakingAccounts, setStakingAccounts] = useState<StakingAccount[]>([]);
  const [validators, setValidators] = useState<Validator[]>([]);
  const [selectedValidator, setSelectedValidator] = useState<string>('');
  const [stakeAmount, setStakeAmount] = useState('');
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'stake' | 'unstake' | 'accounts'>('accounts');
  const [rewards, setRewards] = useState<{ epoch: number; amount: number }[]>([]);

  useEffect(() => {
    loadStakingData();
  }, [wallet.public_key, network]);

  const loadStakingData = async () => {
    try {
      setLoading(true);

      // Load real staking accounts from Solana
      const stakingAccountsResult: any = await invoke('get_staking_accounts', {
        publicKey: wallet.public_key,
        network
      });

      const realStakingAccounts: StakingAccount[] = stakingAccountsResult.accounts.map((account: any) => ({
        stakeAccount: account.stake_account,
        validator: account.validator,
        amount: account.amount / 1e9, // Convert lamports to SOL
        status: account.status,
        activationEpoch: account.activation_epoch,
        deactivationEpoch: account.deactivation_epoch
      }));
      setStakingAccounts(realStakingAccounts);

      // Load real validators from Solana (get_vote_accounts)
      // For now, keep some mock validators as fallback
      const mockValidators: Validator[] = [
        {
          votePubkey: 'CertusOne11111111111111111111111111111111',
          nodePubkey: 'CertusOne11111111111111111111111111111111',
          stake: 1250000,
          commission: 8,
          lastVote: 185000000,
          rootSlot: 184999000
        },
        {
          votePubkey: 'Lido11111111111111111111111111111111111111',
          nodePubkey: 'Lido11111111111111111111111111111111111111',
          stake: 2100000,
          commission: 5,
          lastVote: 185000100,
          rootSlot: 184999100
        },
        {
          votePubkey: 'Jito11111111111111111111111111111111111111',
          nodePubkey: 'Jito11111111111111111111111111111111111111',
          stake: 950000,
          commission: 6,
          lastVote: 185000050,
          rootSlot: 184999050
        }
      ];
      setValidators(mockValidators);

      // Load real rewards history
      const rewardsResult: any = await invoke('get_staking_rewards', {
        publicKey: wallet.public_key,
        network,
        epochs: 30
      });

      const realRewards = rewardsResult.rewards.map((reward: any) => ({
        epoch: reward.epoch,
        amount: reward.amount / 1e9 // Convert lamports to SOL
      }));
      setRewards(realRewards);

    } catch (error) {
      console.error('Failed to load staking data:', error);
      showToast?.('Failed to load staking data', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleStake = async () => {
    if (!selectedValidator || !stakeAmount) {
      showToast?.('Please select a validator and enter amount', 'error');
      return;
    }

    const amount = parseFloat(stakeAmount);
    if (amount <= 0) {
      showToast?.('Please enter a valid amount', 'error');
      return;
    }

    try {
      setLoading(true);

      // Call real Solana staking function from backend
      const signature = await invoke('delegate_stake', {
        wallet: encryptedWallet, // Pass the actual wallet with encrypted key
        password: '', // TODO: Get password from context/state
        validator: selectedValidator,
        amount: Math.floor(amount * 1e9), // Convert SOL to lamports
        network
      });

      showToast?.(`Successfully staked ${amount} SOL with validator. Tx: ${signature}`, 'success');
      setStakeAmount('');
      setSelectedValidator('');

      // Refresh data
      await loadStakingData();

    } catch (error) {
      console.error('Staking failed:', error);
      showToast?.('Staking failed', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleUnstake = async (stakeAccount: string) => {
    try {
      setLoading(true);

      // Call real Solana unstaking function from backend
      const signature = await invoke('deactivate_stake', {
        wallet: encryptedWallet, // Pass the actual wallet with encrypted key
        password: '', // TODO: Get password from context/state
        stakeAccountAddress: stakeAccount,
        network
      });

      showToast?.(`Unstaking initiated successfully. Tx: ${signature}`, 'success');

      // Refresh data
      await loadStakingData();

    } catch (error) {
      console.error('Unstaking failed:', error);
      showToast?.('Unstaking failed', 'error');
    } finally {
      setLoading(false);
    }
  };

  const formatSOL = (amount: number) => {
    return `${amount.toFixed(4)} SOL`;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return '#00ff88';
      case 'activating': return '#ffa500';
      case 'deactivating': return '#ff6b6b';
      case 'inactive': return '#666';
      default: return '#666';
    }
  };

  const totalStaked = stakingAccounts.reduce((sum, account) => sum + account.amount, 0);
  const totalRewards = rewards.reduce((sum, reward) => sum + reward.amount, 0);

  return (
    <div className="staking-interface">
      <div className="staking-header">
        <h2>Staking Dashboard</h2>
        <div className="staking-stats">
          <div className="stat-item">
            <span className="stat-value">{formatSOL(totalStaked)}</span>
            <span className="stat-label">Total Staked</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{formatSOL(totalRewards)}</span>
            <span className="stat-label">Total Rewards</span>
          </div>
          <div className="stat-item">
            <span className="stat-value">{stakingAccounts.length}</span>
            <span className="stat-label">Active Stakes</span>
          </div>
        </div>
      </div>

      <div className="staking-tabs">
        <button
          className={activeTab === 'accounts' ? 'active' : ''}
          onClick={() => setActiveTab('accounts')}
        >
          My Stakes
        </button>
        <button
          className={activeTab === 'stake' ? 'active' : ''}
          onClick={() => setActiveTab('stake')}
        >
          Stake SOL
        </button>
        <button
          className={activeTab === 'unstake' ? 'active' : ''}
          onClick={() => setActiveTab('unstake')}
        >
          Unstake
        </button>
      </div>

      <div className="staking-content">
        {activeTab === 'accounts' && (
          <div className="staking-accounts">
            <h3>Your Staking Accounts</h3>
            {stakingAccounts.length === 0 ? (
              <p className="no-stakes">No active staking accounts. Start staking to earn rewards!</p>
            ) : (
              <div className="accounts-list">
                {stakingAccounts.map((account, index) => (
                  <div key={index} className="staking-account-card">
                    <div className="account-header">
                      <div className="account-info">
                        <h4>Stake Account {index + 1}</h4>
                        <div className="validator-address">
                          Validator: {account.validator.slice(0, 8)}...{account.validator.slice(-8)}
                        </div>
                      </div>
                      <div className="account-status">
                        <span
                          className={`status-badge status-${account.status}`}
                        >
                          {account.status}
                        </span>
                      </div>
                    </div>

                    <div className="account-details">
                      <div className="detail-item">
                        <span className="detail-label">Staked Amount:</span>
                        <span className="detail-value">{formatSOL(account.amount)}</span>
                      </div>
                      {account.activationEpoch && (
                        <div className="detail-item">
                          <span className="detail-label">Activation Epoch:</span>
                          <span className="detail-value">{account.activationEpoch}</span>
                        </div>
                      )}
                    </div>

                    <div className="account-actions">
                      <button
                        onClick={() => handleUnstake(account.stakeAccount)}
                        className="unstake-btn"
                        disabled={loading}
                      >
                        Unstake
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}

            <div className="rewards-section">
              <h4>Rewards History</h4>
              <div className="rewards-list">
                {rewards.map((reward, index) => (
                  <div key={index} className="reward-item">
                    <span>Epoch {reward.epoch}</span>
                    <span>+{formatSOL(reward.amount)}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'stake' && (
          <div className="staking-form">
            <h3>Stake Your SOL</h3>

            <div className="form-section">
              <h4>Select Validator</h4>
              <div className="validator-list">
                {validators.map((validator) => (
                  <div
                    key={validator.votePubkey}
                    className={`validator-card ${selectedValidator === validator.votePubkey ? 'selected' : ''}`}
                    onClick={() => setSelectedValidator(validator.votePubkey)}
                  >
                    <div className="validator-info">
                      <div className="validator-name">
                        Validator {validator.votePubkey.slice(0, 8)}...
                      </div>
                      <div className="validator-details">
                        <span>Commission: {validator.commission}%</span>
                        <span>Stake: {formatSOL(validator.stake)}</span>
                      </div>
                    </div>
                    <div className="validator-status">
                      <div className="uptime-indicator">●</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="form-section">
              <h4>Staking Amount</h4>
              <div className="amount-input">
                <input
                  type="number"
                  value={stakeAmount}
                  onChange={(e) => setStakeAmount(e.target.value)}
                  placeholder="Enter amount to stake"
                  step="0.000000001"
                  min="0"
                />
                <span className="sol-label">SOL</span>
              </div>
              <div className="balance-info">
                Available: {formatSOL(wallet.balance / 1e9)}
              </div>
            </div>

            <div className="staking-info">
              <h4>Staking Information</h4>
              <ul>
                <li>Minimum stake: 1 SOL (plus rent exemption)</li>
                <li>Cooldown period: ~2-3 days for unstaking</li>
                <li>Rewards are distributed daily</li>
                <li>You can unstake at any time</li>
              </ul>
            </div>

            <button
              onClick={handleStake}
              className="stake-btn"
              disabled={loading || !selectedValidator || !stakeAmount}
            >
              {loading ? 'Staking...' : 'Stake SOL'}
            </button>
          </div>
        )}

        {activeTab === 'unstake' && (
          <div className="unstaking-section">
            <h3>Unstake Your SOL</h3>
            <p>Select a staking account to initiate the unstaking process.</p>

            <div className="unstake-accounts">
              {stakingAccounts
                .filter(account => account.status === 'active')
                .map((account, index) => (
                  <div key={index} className="unstake-account-card">
                    <div className="account-summary">
                      <div>
                        <strong>{formatSOL(account.amount)}</strong> staked with validator {account.validator.slice(0, 8)}...
                      </div>
                      <button
                        onClick={() => handleUnstake(account.stakeAccount)}
                        className="unstake-btn"
                        disabled={loading}
                      >
                        Initiate Unstake
                      </button>
                    </div>
                  </div>
                ))}
            </div>

            <div className="unstake-warning">
              <h4>⚠️ Important Notes</h4>
              <ul>
                <li>Unstaking takes 2-3 days to complete</li>
                <li>You won't earn rewards during the unstaking period</li>
                <li>You can cancel unstaking within the cooldown period</li>
              </ul>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default StakingInterface;