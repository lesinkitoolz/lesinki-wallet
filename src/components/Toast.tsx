import React, { useEffect } from 'react';

interface ToastProps {
  message: string;
  type: 'success' | 'error' | 'info';
  onClose: () => void;
  duration?: number;
}

const Toast: React.FC<ToastProps> = ({ message, type, onClose, duration = 5000 }) => {
  useEffect(() => {
    const timer = setTimeout(onClose, duration);
    return () => clearTimeout(timer);
  }, [onClose, duration]);

  const getToastStyles = () => {
    switch (type) {
      case 'success':
        return 'border-green-500 bg-green-900/20 text-green-400';
      case 'error':
        return 'border-red-500 bg-red-900/20 text-red-400';
      case 'info':
        return 'border-blue-500 bg-blue-900/20 text-blue-400';
      default:
        return 'border-gray-500 bg-gray-900/20 text-gray-400';
    }
  };

  return (
    <div className={`fixed top-4 right-4 z-50 p-4 rounded-lg border backdrop-blur-sm ${getToastStyles()} animate-slide-in`}>
      <div className="flex items-center justify-between">
        <span>{message}</span>
        <button
          onClick={onClose}
          className="ml-4 text-current hover:opacity-75"
        >
          ×
        </button>
      </div>
    </div>
  );
};

export default Toast;