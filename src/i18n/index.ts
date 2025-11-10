// i18n configuration for Lesinki Wallet
// This will be initialized when dependencies are installed

// Mock i18n object for development - will be replaced by actual i18next
const i18n = {
  language: 'en',
  t: (key: string, params?: any) => key,
  changeLanguage: (lng: string) => Promise.resolve(true),
  use: (plugin: any) => i18n,
  init: (config: any) => Promise.resolve(i18n),
  isInitialized: true
};

// Mock language configuration
export const languages = [
  { code: 'en', name: 'English', rtl: false },
  { code: 'es', name: 'Español', rtl: false },
  { code: 'fr', name: 'Français', rtl: false },
  { code: 'de', name: 'Deutsch', rtl: false },
  { code: 'zh-CN', name: '中文 (简体)', rtl: false },
  { code: 'zh-TW', name: '中文 (繁體)', rtl: false },
  { code: 'ja', name: '日本語', rtl: false },
  { code: 'ko', name: '한국어', rtl: false },
  { code: 'pt', name: 'Português', rtl: false },
  { code: 'ru', name: 'Русский', rtl: false },
  { code: 'ar', name: 'العربية', rtl: true }
];

// Helper functions for localization
export const i18nHelpers = {
  // Check if language is RTL
  isRTL: (language: string): boolean => {
    const lang = languages.find(l => l.code === language);
    return lang?.rtl || false;
  },

  // Get language direction
  getDirection: (language: string): 'ltr' | 'rtl' => {
    return i18nHelpers.isRTL(language) ? 'rtl' : 'ltr';
  },

  // Get display name for language
  getDisplayName: (language: string): string => {
    const lang = languages.find(l => l.code === language);
    return lang?.name || language;
  },

  // Get all supported languages
  getSupportedLanguages: () => {
    return languages;
  },

  // Change language with persistence
  changeLanguage: async (language: string): Promise<void> => {
    await i18n.changeLanguage(language);
    localStorage.setItem('i18nextLng', language);
    
    // Update document direction
    const direction = i18nHelpers.getDirection(language);
    document.documentElement.dir = direction;
    document.documentElement.lang = language;
  },

  // Initialize with detected language
  initialize: (): void => {
    const savedLanguage = localStorage.getItem('i18nextLng') || 'en';
    const direction = i18nHelpers.getDirection(savedLanguage);
    
    document.documentElement.dir = direction;
    document.documentElement.lang = savedLanguage;
  }
};

// Export default i18n instance
export default i18n;

// Note: To fully enable i18n, install these dependencies:
// npm install i18next react-i18next i18next-browser-languagedetector
// 
// Then replace this file with the full i18n configuration including:
// - Real i18next setup
// - Translation files for all languages
// - Proper initialization