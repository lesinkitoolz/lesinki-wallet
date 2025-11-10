import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface NFT {
  mint: string;
  name: string;
  symbol: string;
  uri: string;
  image?: string;
  description?: string;
  attributes?: Array<{
    trait_type: string;
    value: string;
  }>;
  collection?: string;
}

interface NFTGalleryProps {
  wallet: {
    public_key: string;
    private_key: number[];
    balance: number;
  };
  network: 'mainnet' | 'devnet' | 'testnet';
}

const NFTGallery: React.FC<NFTGalleryProps> = ({ wallet, network }) => {
  const [nfts, setNfts] = useState<NFT[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [selectedNFT, setSelectedNFT] = useState<NFT | null>(null);
  const [filter, setFilter] = useState<'all' | 'collection'>('all');
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    loadNFTs();
  }, [wallet.public_key, network]);

  const loadNFTs = async () => {
    try {
      setLoading(true);
      setError('');

      // In a real implementation, you'd fetch NFTs from the Solana blockchain
      // For now, we'll simulate some mock NFT data
      const mockNFTs: NFT[] = [
        {
          mint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
          name: 'Lesinki Dragon #001',
          symbol: 'LSD',
          uri: 'https://arweave.net/example1',
          image: 'https://via.placeholder.com/300x300/9945ff/ffffff?text=NFT+1',
          description: 'A majestic dragon from the Lesinki collection',
          attributes: [
            { trait_type: 'Rarity', value: 'Legendary' },
            { trait_type: 'Element', value: 'Fire' },
            { trait_type: 'Level', value: '99' }
          ],
          collection: 'Lesinki Dragons'
        },
        {
          mint: 'So11111111111111111111111111111111111111112',
          name: 'Cosmic Warrior #042',
          symbol: 'CW',
          uri: 'https://arweave.net/example2',
          image: 'https://via.placeholder.com/300x300/00ff88/000000?text=NFT+2',
          description: 'A powerful warrior from the cosmos',
          attributes: [
            { trait_type: 'Class', value: 'Warrior' },
            { trait_type: 'Power', value: '8500' },
            { trait_type: 'Background', value: 'Cosmic' }
          ],
          collection: 'Cosmic Warriors'
        },
        {
          mint: '11111111111111111111111111111112',
          name: 'Digital Artifact #1337',
          symbol: 'DA',
          uri: 'https://arweave.net/example3',
          image: 'https://via.placeholder.com/300x300/ff4444/ffffff?text=NFT+3',
          description: 'A unique digital artifact',
          attributes: [
            { trait_type: 'Type', value: 'Artifact' },
            { trait_type: 'Rarity', value: 'Epic' },
            { trait_type: 'Origin', value: 'Digital Realm' }
          ]
        }
      ];

      setNfts(mockNFTs);
    } catch (err) {
      setError('Failed to load NFTs');
      console.error('Failed to load NFTs:', err);
    } finally {
      setLoading(false);
    }
  };

  const filteredNFTs = nfts.filter(nft => {
    const matchesSearch = nft.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         nft.symbol.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         nft.collection?.toLowerCase().includes(searchTerm.toLowerCase());

    if (filter === 'collection') {
      return matchesSearch && nft.collection;
    }

    return matchesSearch;
  });

  const collections = [...new Set(nfts.map(nft => nft.collection).filter(Boolean))];

  if (loading) {
    return (
      <div className="nft-gallery">
        <h2>NFT Gallery</h2>
        <div className="loading">Loading your NFTs...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="nft-gallery">
        <h2>NFT Gallery</h2>
        <div className="error-message">{error}</div>
        <button onClick={loadNFTs} className="retry-btn">Retry</button>
      </div>
    );
  }

  return (
    <div className="nft-gallery">
      <div className="gallery-header">
        <h2>NFT Gallery</h2>
        <div className="gallery-controls">
          <input
            type="text"
            placeholder="Search NFTs..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="search-input"
          />
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as 'all' | 'collection')}
            className="filter-select"
            aria-label="Filter NFTs"
          >
            <option value="all">All NFTs</option>
            <option value="collection">Collections Only</option>
          </select>
          <button onClick={loadNFTs} className="refresh-btn">↻ Refresh</button>
        </div>
      </div>

      <div className="gallery-stats">
        <div className="stat-item">
          <span className="stat-number">{nfts.length}</span>
          <span className="stat-label">Total NFTs</span>
        </div>
        <div className="stat-item">
          <span className="stat-number">{collections.length}</span>
          <span className="stat-label">Collections</span>
        </div>
        <div className="stat-item">
          <span className="stat-number">{network.charAt(0).toUpperCase() + network.slice(1)}</span>
          <span className="stat-label">Network</span>
        </div>
      </div>

      {filteredNFTs.length === 0 ? (
        <div className="no-nfts">
          <p>No NFTs found matching your criteria.</p>
          <p>Start collecting NFTs on Solana!</p>
        </div>
      ) : (
        <div className="nft-grid">
          {filteredNFTs.map((nft) => (
            <div
              key={nft.mint}
              className="nft-card"
              onClick={() => setSelectedNFT(nft)}
            >
              <div className="nft-image-container">
                <img
                  src={nft.image || 'https://via.placeholder.com/300x300/333/666?text=NFT'}
                  alt={nft.name}
                  className="nft-image"
                  onError={(e) => {
                    const target = e.target as HTMLImageElement;
                    target.src = 'https://via.placeholder.com/300x300/333/666?text=NFT';
                  }}
                />
                {nft.collection && (
                  <div className="collection-badge">{nft.collection}</div>
                )}
              </div>
              <div className="nft-info">
                <h3 className="nft-name">{nft.name}</h3>
                <p className="nft-symbol">{nft.symbol}</p>
                <div className="nft-address">
                  {nft.mint.slice(0, 8)}...{nft.mint.slice(-8)}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {selectedNFT && (
        <div className="nft-modal" onClick={() => setSelectedNFT(null)}>
          <div className="nft-modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="nft-modal-header">
              <h3>{selectedNFT.name}</h3>
              <button
                onClick={() => setSelectedNFT(null)}
                className="close-btn"
              >
                ×
              </button>
            </div>
            <div className="nft-modal-body">
              <div className="nft-modal-image">
                <img
                  src={selectedNFT.image || 'https://via.placeholder.com/400x400/333/666?text=NFT'}
                  alt={selectedNFT.name}
                  onError={(e) => {
                    const target = e.target as HTMLImageElement;
                    target.src = 'https://via.placeholder.com/400x400/333/666?text=NFT';
                  }}
                />
              </div>
              <div className="nft-modal-details">
                <div className="detail-section">
                  <h4>Description</h4>
                  <p>{selectedNFT.description || 'No description available'}</p>
                </div>

                <div className="detail-section">
                  <h4>Details</h4>
                  <div className="nft-details-grid">
                    <div className="detail-item">
                      <span className="detail-label">Symbol:</span>
                      <span className="detail-value">{selectedNFT.symbol}</span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Mint:</span>
                      <code className="detail-value">{selectedNFT.mint}</code>
                    </div>
                    {selectedNFT.collection && (
                      <div className="detail-item">
                        <span className="detail-label">Collection:</span>
                        <span className="detail-value">{selectedNFT.collection}</span>
                      </div>
                    )}
                  </div>
                </div>

                {selectedNFT.attributes && selectedNFT.attributes.length > 0 && (
                  <div className="detail-section">
                    <h4>Attributes</h4>
                    <div className="attributes-grid">
                      {selectedNFT.attributes.map((attr, index) => (
                        <div key={index} className="attribute-item">
                          <span className="attribute-type">{attr.trait_type}</span>
                          <span className="attribute-value">{attr.value}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className="nft-actions">
                  <button
                    onClick={() => navigator.clipboard.writeText(selectedNFT.mint)}
                    className="action-btn secondary"
                  >
                    Copy Mint Address
                  </button>
                  <button
                    onClick={() => window.open(`https://solscan.io/token/${selectedNFT.mint}`, '_blank')}
                    className="action-btn primary"
                  >
                    View on Solscan
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default NFTGallery;