import { SelectHTMLAttributes, forwardRef } from "react";

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  options: { value: string | number; label: string }[];
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, options, className = "", ...props }, ref) => {
    return (
      <div className="flex flex-col gap-1">
        {label && (
          <label className="text-xs text-charcoal-500 dark:text-charcoal-400">
            {label}
          </label>
        )}
        <select
          ref={ref}
          className={`w-full cursor-pointer rounded-md border border-sand-300 bg-charcoal-50 px-3 py-2.5 text-base text-charcoal-900 outline-none transition-colors focus:border-sage-500 md:py-2 md:text-sm dark:border-charcoal-700 dark:bg-charcoal-950/60 dark:text-sand-100 dark:focus:border-sage-500 touch-manipulation ${className}`}
          {...props}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
    );
  }
);
