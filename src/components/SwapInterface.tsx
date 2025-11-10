import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Token {
  symbol: string;
  name: string;
  mint: string;
  decimals: number;
  logoURI?: string;
  price?: number;
}

interface SwapQuote {
  inputAmount: number;
  outputAmount: number;
  priceImpact: number;
  fee: number;
  route: string[];
}

interface SwapInterfaceProps {
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
  password: string;
  showToast?: (message: string, type: 'success' | 'error' | 'info') => void;
}

const SwapInterface: React.FC<SwapInterfaceProps> = ({ wallet, network, password, showToast }) => {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [fromToken, setFromToken] = useState<Token | null>(null);
  const [toToken, setToToken] = useState<Token | null>(null);
  const [fromAmount, setFromAmount] = useState('');
  const [toAmount, setToAmount] = useState('');
  const [swapQuote, setSwapQuote] = useState<SwapQuote | null>(null);
  const [loading, setLoading] = useState(false);
  const [slippage, setSlippage] = useState(0.5);
  const [showTokenSelector, setShowTokenSelector] = useState<'from' | 'to' | null>(null);
  const [tokenSearch, setTokenSearch] = useState('');
  const [showConfirmation, setShowConfirmation] = useState(false);
  const [balances, setBalances] = useState<{[key: string]: number}>({});

  useEffect(() => {
    loadTokens();
    loadBalances();
  }, [network]);

  useEffect(() => {
    if (fromAmount && fromToken && toToken) {
      getSwapQuote();
    }
  }, [fromAmount, fromToken, toToken, slippage]);

  const loadTokens = async () => {
   try {
     // Load real token list from Jupiter API
     const tokenList: any = await invoke('get_token_list', { network });

     const realTokens: Token[] = tokenList.tokens.map((token: any) => ({
       symbol: token.symbol,
       name: token.name,
       mint: token.mint,
       decimals: token.decimals,
       logoURI: token.logo_uri,
       price: token.price
     }));

     setTokens(realTokens);

      // Set default tokens
      const solToken = realTokens.find(t => t.symbol === 'SOL');
      const usdcToken = realTokens.find(t => t.symbol === 'USDC');

      if (solToken) setFromToken(solToken);
      if (usdcToken) setToToken(usdcToken);

    } catch (error) {
      console.error('Failed to load tokens:', error);
      showToast?.('Failed to load tokens', 'error');
    }
  };

  const loadBalances = async () => {
    try {
      const tokenBalances: any = await invoke('get_token_balances', {
        wallet: wallet.public_key,
        network
      });

      const balanceMap: {[key: string]: number} = {};
      tokenBalances.forEach((balance: any) => {
        balanceMap[balance.mint] = balance.amount / Math.pow(10, balance.decimals);
      });

      setBalances(balanceMap);
    } catch (error) {
      console.error('Failed to load balances:', error);
    }
  };

  const getSwapQuote = async () => {
    if (!fromAmount || !fromToken || !toToken) return;

    try {
      setLoading(true);

      // Convert amount to smallest unit (lamports for SOL, etc.)
      const inputAmount = Math.floor(parseFloat(fromAmount) * Math.pow(10, fromToken.decimals));
      const slippageBps = Math.floor(slippage * 100); // Convert to basis points

      // Get quote from Jupiter API
      const quote: any = await invoke('get_jupiter_quote', {
        inputMint: fromToken.mint,
        outputMint: toToken.mint,
        amount: inputAmount.toString(),
        slippageBps
      });

      const realQuote: SwapQuote = {
        inputAmount: parseFloat(fromAmount),
        outputAmount: parseInt(quote.other_amount_threshold) / Math.pow(10, toToken.decimals),
        priceImpact: parseFloat(quote.price_impact_pct),
        fee: quote.platform_fee ? parseInt(quote.platform_fee.amount) / Math.pow(10, 9) : 0, // Assume SOL fee
        route: quote.route_plan.map((route: any) => route.swap_info.label || 'Unknown DEX')
      };

      setSwapQuote(realQuote);
      setToAmount(realQuote.outputAmount.toFixed(6));

    } catch (error) {
      console.error('Failed to get swap quote:', error);
      showToast?.('Failed to get swap quote', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleSwap = async () => {
    if (!swapQuote || !fromToken || !toToken) {
      showToast?.('Invalid swap parameters', 'error');
      return;
    }

    try {
      setLoading(true);

      // Get fresh quote for execution
      const inputAmount = Math.floor(parseFloat(fromAmount) * Math.pow(10, fromToken.decimals));
      const slippageBps = Math.floor(slippage * 100);

      const quote: any = await invoke('get_jupiter_quote', {
        inputMint: fromToken.mint,
        outputMint: toToken.mint,
        amount: inputAmount.toString(),
        slippageBps
      });

      // Execute the swap
      const signature = await invoke('execute_jupiter_swap', {
        quoteResponse: JSON.stringify(quote),
        userPublicKey: wallet.public_key,
        wallet: wallet,
        password: password,
        network: network
      });

      showToast?.(`Successfully swapped ${fromAmount} ${fromToken.symbol} for ${toAmount} ${toToken.symbol}. Tx: ${signature}`, 'success');

      // Reset form
      setFromAmount('');
      setToAmount('');
      setSwapQuote(null);
      setShowConfirmation(false);

      // Refresh balances
      loadBalances();

    } catch (error) {
      console.error('Swap failed:', error);
      showToast?.('Swap failed', 'error');
    } finally {
      setLoading(false);
    }
  };

  const switchTokens = () => {
    const tempFrom = fromToken;
    const tempTo = toToken;
    const tempAmount = fromAmount;

    setFromToken(tempTo);
    setToToken(tempFrom);
    setFromAmount(toAmount);
    setToAmount(tempAmount);
    setSwapQuote(null);
  };

  const filteredTokens = tokens.filter(token => {
    const matchesSearch = token.symbol.toLowerCase().includes(tokenSearch.toLowerCase()) ||
                         token.name.toLowerCase().includes(tokenSearch.toLowerCase()) ||
                         token.mint.toLowerCase().includes(tokenSearch.toLowerCase());
    const notSelected = !showTokenSelector ||
                       (showTokenSelector === 'from' && token.mint !== toToken?.mint) ||
                       (showTokenSelector === 'to' && token.mint !== fromToken?.mint);
    return matchesSearch && notSelected;
  });

  return (
    <div className="swap-interface">
      <div className="swap-header">
        <h2>Token Swap</h2>
        <div className="network-indicator">
          Network: {network.charAt(0).toUpperCase() + network.slice(1)}
        </div>
      </div>

      <div className="swap-card">
        <div className="swap-input-section">
          <div className="token-input">
            <div className="input-header">
              <label>From</label>
              {fromToken && balances[fromToken.mint] && (
                <span className="balance">Balance: {balances[fromToken.mint].toFixed(6)} {fromToken.symbol}</span>
              )}
            </div>
            <div className="input-row">
              <input
                type="number"
                value={fromAmount}
                onChange={(e) => setFromAmount(e.target.value)}
                placeholder="0.00"
                step="0.000001"
                min="0"
              />
              {fromToken && balances[fromToken.mint] && (
                <button
                  className="max-btn"
                  onClick={() => setFromAmount(balances[fromToken.mint].toString())}
                >
                  MAX
                </button>
              )}
              <button
                className="token-selector"
                onClick={() => setShowTokenSelector('from')}
              >
                {fromToken ? (
                  <div className="token-info">
                    {fromToken.logoURI && (
                      <img src={fromToken.logoURI} alt={fromToken.symbol} className="token-logo" />
                    )}
                    <span>{fromToken.symbol}</span>
                  </div>
                ) : (
                  'Select Token'
                )}
                <span className="dropdown-arrow">▼</span>
              </button>
            </div>
          </div>

          <div className="swap-divider">
            <button onClick={switchTokens} className="switch-btn">
              ⇅
            </button>
          </div>

          <div className="token-input">
            <div className="input-header">
              <label>To</label>
              {toToken && balances[toToken.mint] && (
                <span className="balance">Balance: {balances[toToken.mint].toFixed(6)} {toToken.symbol}</span>
              )}
            </div>
            <div className="input-row">
              <input
                type="number"
                value={toAmount}
                readOnly
                placeholder="0.00"
              />
              <button
                className="token-selector"
                onClick={() => setShowTokenSelector('to')}
              >
                {toToken ? (
                  <div className="token-info">
                    {toToken.logoURI && (
                      <img src={toToken.logoURI} alt={toToken.symbol} className="token-logo" />
                    )}
                    <span>{toToken.symbol}</span>
                  </div>
                ) : (
                  'Select Token'
                )}
                <span className="dropdown-arrow">▼</span>
              </button>
            </div>
          </div>
        </div>

        <div className="swap-settings">
          <div className="setting-item">
            <label>Slippage Tolerance:</label>
            <div className="slippage-options">
              {[0.1, 0.5, 1.0].map(value => (
                <button
                  key={value}
                  className={slippage === value ? 'active' : ''}
                  onClick={() => setSlippage(value)}
                >
                  {value}%
                </button>
              ))}
              <label htmlFor="custom-slippage" className="sr-only">Custom Slippage Percentage</label>
              <input
                id="custom-slippage"
                type="number"
                value={slippage}
                onChange={(e) => setSlippage(parseFloat(e.target.value))}
                step="0.1"
                min="0.1"
                max="50"
                className="custom-slippage"
              />
            </div>
          </div>
        </div>

        {swapQuote && (
          <div className="swap-quote">
            <h4>Swap Summary</h4>
            <div className="quote-details">
              <div className="quote-row">
                <span>Rate:</span>
                <span>1 {fromToken?.symbol} ≈ {(swapQuote.outputAmount / swapQuote.inputAmount).toFixed(6)} {toToken?.symbol}</span>
              </div>
              <div className="quote-row">
                <span>Price Impact:</span>
                <span className={swapQuote.priceImpact > 1 ? 'high-impact' : 'low-impact'}>
                  {swapQuote.priceImpact.toFixed(2)}%
                </span>
              </div>
              <div className="quote-row">
                <span>Fee:</span>
                <span>${swapQuote.fee.toFixed(4)}</span>
              </div>
              <div className="quote-row">
                <span>Minimum Received:</span>
                <span>{(swapQuote.outputAmount * (1 - slippage / 100)).toFixed(6)} {toToken?.symbol}</span>
              </div>
              <div className="quote-row">
                <span>Route:</span>
                <span>{swapQuote.route.join(' → ')}</span>
              </div>
            </div>
          </div>
        )}

        <button
          onClick={() => setShowConfirmation(true)}
          className="swap-btn"
          disabled={loading || !fromAmount || !swapQuote}
        >
          {loading ? 'Swapping...' : 'Swap Tokens'}
        </button>

        {showConfirmation && swapQuote && (
          <div className="confirmation-modal" onClick={() => setShowConfirmation(false)}>
            <div className="modal-content" onClick={(e) => e.stopPropagation()}>
              <h3>Confirm Swap</h3>
              <div className="confirmation-details">
                <div className="confirm-row">
                  <span>You pay:</span>
                  <span>{fromAmount} {fromToken?.symbol}</span>
                </div>
                <div className="confirm-row">
                  <span>You receive:</span>
                  <span>≈ {toAmount} {toToken?.symbol}</span>
                </div>
                <div className="confirm-row">
                  <span>Price Impact:</span>
                  <span className={swapQuote.priceImpact > 1 ? 'high-impact' : 'low-impact'}>
                    {swapQuote.priceImpact.toFixed(2)}%
                  </span>
                </div>
                <div className="confirm-row">
                  <span>Minimum Received:</span>
                  <span>{(swapQuote.outputAmount * (1 - slippage / 100)).toFixed(6)} {toToken?.symbol}</span>
                </div>
                <div className="confirm-row">
                  <span>Network Fee:</span>
                  <span>≈ ${swapQuote.fee.toFixed(4)}</span>
                </div>
                <div className="confirm-row">
                  <span>Route:</span>
                  <span>{swapQuote.route.join(' → ')}</span>
                </div>
              </div>
              <div className="confirmation-buttons">
                <button onClick={() => setShowConfirmation(false)} className="cancel-btn">
                  Cancel
                </button>
                <button onClick={handleSwap} className="confirm-btn" disabled={loading}>
                  {loading ? 'Swapping...' : 'Confirm Swap'}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {showTokenSelector && (
        <div className="token-modal" onClick={() => setShowTokenSelector(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>Select Token</h3>
            <input
              type="text"
              placeholder="Search tokens..."
              value={tokenSearch}
              onChange={(e) => setTokenSearch(e.target.value)}
              className="token-search"
            />
            <div className="token-list">
              {filteredTokens.map(token => (
                <div
                  key={token.mint}
                  className="token-option"
                  onClick={() => {
                    if (showTokenSelector === 'from') {
                      setFromToken(token);
                    } else {
                      setToToken(token);
                    }
                    setShowTokenSelector(null);
                    setSwapQuote(null);
                    setToAmount('');
                    setTokenSearch('');
                  }}
                >
                  <div className="token-info">
                    {token.logoURI && (
                      <img src={token.logoURI} alt={token.symbol} className="token-logo" />
                    )}
                    <div className="token-details">
                      <div className="token-symbol">{token.symbol}</div>
                      <div className="token-name">{token.name}</div>
                    </div>
                  </div>
                  {token.price && (
                    <div className="token-price">${token.price.toFixed(4)}</div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SwapInterface;