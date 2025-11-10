# 📱 Installing Lesinki Wallet

Get your professional Solana wallet by downloading the app directly from GitHub releases.

## 🚀 Quick Download

**Latest Release**: Check the [Releases page](https://github.com/lesinkitoolz/lesinki-wallet/releases) for the newest version.

## 📋 System Requirements

- **Windows**: Windows 10 or later (64-bit)
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Most modern distributions (64-bit)

## 🖥️ Platform-Specific Instructions

### Windows (Recommended: MSI installer)
1. Visit the [Releases page](https://github.com/lesinkitoolz/lesinki-wallet/releases)
2. Download `Lesinki Wallet Setup.msi` for Windows
3. Run the installer and follow the setup wizard
4. Launch Lesinki Wallet from your Start menu

**Alternative: EXE installer**
- Download `lesinki-wallet_x64_en-US.exe`
- Run the executable and follow the installation prompts
- Allow the app through Windows Defender if prompted

### macOS (Recommended: DMG)
1. Visit the [Releases page](https://github.com/lesinkitoolz/lesinki-wallet/releases)
2. Download `Lesinki Wallet.dmg`
3. Double-click the DMG file to mount it
4. Drag Lesinki Wallet to your Applications folder
5. Launch from Applications or Spotlight

**Security Note**: If you get a "cannot be opened because it is from an unidentified developer" warning:
1. Right-click on the app
2. Select "Open" and then "Open" in the dialog

### Linux

#### AppImage (Recommended)
1. Visit the [Releases page](https://github.com/lesinkitoolz/lesinki-wallet/releases)
2. Download `lesinki-wallet_amd64.AppImage`
3. Make it executable: `chmod +x lesinki-wallet_amd64.AppImage`
4. Run: `./lesinki-wallet_amd64.AppImage`

#### DEB Package (Ubuntu/Debian)
1. Visit the [Releases page](https://github.com/lesinkitoolz/lesinki-wallet/releases)
2. Download `lesinki-wallet_amd64.deb`
3. Install: `sudo dpkg -i lesinki-wallet_amd64.deb`
4. Launch: `lesinki-wallet`

#### Troubleshooting Linux
If you encounter missing dependencies:
```bash
# Install required system libraries
sudo apt-get install libgtk-3-0 libappindicator3-1 librsvg2-common
```

## 🔒 Security Verification

### Windows
1. Right-click the installer
2. Select "Properties" → "Digital Signatures"
3. Verify the signature is from "lesinski toolz"

### macOS
1. Open Terminal
2. Run: `codesign -dv --verbose=4 /Applications/Lesinki\ Wallet.app`
3. Should show proper code signing information

### Linux
1. Check file permissions and source
2. Verify the download came from the official repository
3. Compare with published checksums when available

## 🛠️ Development Installation

For developers who want to build from source:

### Prerequisites
- Node.js 18+ and npm
- Rust 1.70+
- System dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `build-essential`, `libwebkit2gtk-4.0-dev`, `libssl-dev`
  - **Windows**: Visual Studio Build Tools

### Setup
```bash
# Clone the repository
git clone https://github.com/lesinski-tools/lesinki-wallet.git
cd lesinki-wallet

# Install dependencies
npm install

# Build and run
npm run tauri:build  # Build the application
npm run tauri:dev    # Run in development mode
```

## 🔄 Updating

### Automatic Updates
The app includes an update checker that will notify you when new versions are available.

### Manual Updates
1. Visit the [Releases page](https://github.com/lesinski-tools/lesinki-wallet/releases)
2. Download the latest version for your platform
3. Install over the existing version or use the built-in updater

## 🆘 Troubleshooting

### Common Issues

**App won't start**
- Check if your system meets the requirements
- Try running as administrator (Windows) or with sudo (Linux)
- Check firewall/antivirus settings

**Slow performance**
- Close other resource-intensive applications
- Check available disk space
- Restart the application

**Network connectivity issues**
- Check internet connection
- Verify firewall allows the app
- Try different network if on VPN/corporate network

**Import/backup issues**
- Ensure you have the correct backup file format
- Check file permissions
- Verify wallet address matches the backup

### Getting Help
- Create an [issue](https://github.com/lesinski-tools/lesinki-wallet/issues)
- Check the [troubleshooting guide](https://github.com/lesinski-tools/lesinki-wallet/wiki/Troubleshooting)
- Review the [FAQ](https://github.com/lesinski-tools/lesinki-wallet/wiki/FAQ)

## 🎯 Features

Once installed, you can:
- Create and manage multiple Solana wallets
- Swap tokens and use DeFi protocols
- Manage NFTs and view collections
- Stake SOL and earn rewards
- Send and receive payments
- Import/export wallet backups
- Use in multiple languages

## 📄 License

This software is licensed under the MIT License. See [LICENSE](LICENSE) file for details.

## 🔗 Links

- **GitHub Repository**: https://github.com/lesinski-tools/lesinki-wallet
- **Releases**: https://github.com/lesinski-tools/lesinki-wallet/releases
- **Documentation**: https://github.com/lesinski-tools/lesinki-wallet/wiki
- **Issues**: https://github.com/lesinski-tools/lesinki-wallet/issues

---

*Happy wallet management! 🦾*