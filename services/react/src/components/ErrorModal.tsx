import { useEffect } from "react";

interface ErrorModalProps {
  isOpen: boolean;
  title: string;
  message: string;
  buttonText?: string;
  onClose: () => void;
  autoCloseMs?: number;
}

export function ErrorModal({
  isOpen,
  title,
  message,
  buttonText = "OK",
  onClose,
  autoCloseMs,
}: ErrorModalProps) {
  useEffect(() => {
    if (isOpen && autoCloseMs) {
      const timer = setTimeout(onClose, autoCloseMs);
      return () => clearTimeout(timer);
    }
  }, [isOpen, autoCloseMs, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-2xl p-6 w-full max-w-sm mx-4 text-center">
        <div className="w-16 h-16 mx-auto mb-4 bg-red-100 rounded-full flex items-center justify-center">
          <svg
            className="w-8 h-8 text-red-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </div>
        <h2 className="text-xl font-bold text-gray-900 mb-2">{title}</h2>
        <p className="text-gray-600 mb-6">{message}</p>
        <button
          onClick={onClose}
          className="w-full px-4 py-2 text-sm font-bold text-white bg-red-600 hover:bg-red-700 rounded-lg transition"
        >
          {buttonText}
        </button>
      </div>
    </div>
  );
}
