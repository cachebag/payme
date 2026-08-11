import { ReactNode, useEffect } from "react";
import { X } from "lucide-react";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  children: ReactNode;
  title?: string;
}

export function Modal({ isOpen, onClose, children, title }: ModalProps) {
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    // Bottom sheet on phones, centered dialog from `sm:` up.
    <div className="fixed inset-0 z-50 flex items-end justify-center sm:items-center">
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />
      <div className="relative flex max-h-[calc(100dvh-3rem)] w-full max-w-lg flex-col rounded-t-2xl border border-sand-300 bg-charcoal-50 shadow-[0_18px_60px_rgb(0_0_0_/_0.18)] sm:max-h-[85dvh] sm:mx-4 sm:rounded-md dark:border-charcoal-700 dark:bg-charcoal-900">
        {/* Grab handle, phone only */}
        <div className="flex justify-center pt-2 sm:hidden" aria-hidden="true">
          <div className="h-1 w-9 rounded-full bg-sand-300 dark:bg-charcoal-700" />
        </div>
        <div className="flex items-center justify-between px-5 pt-3 pb-0 sm:pt-5">
          {title && (
            <h2 className="text-lg font-semibold text-charcoal-800 dark:text-sand-100">
              {title}
            </h2>
          )}
          <button
            onClick={onClose}
            aria-label="Close"
            className="ml-auto rounded-md p-2 hover:bg-sand-200 active:bg-sand-300 dark:hover:bg-charcoal-800 dark:active:bg-charcoal-700 transition-colors touch-manipulation sm:p-1"
          >
            <X size={20} />
          </button>
        </div>
        <div className="overflow-y-auto px-5 pt-4 pb-[calc(1.25rem+env(safe-area-inset-bottom))] sm:pb-5">
          {children}
        </div>
      </div>
    </div>
  );
}
