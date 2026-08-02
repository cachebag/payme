import { ChevronDown, ChevronUp } from "lucide-react";
import { InputHTMLAttributes, MutableRefObject, forwardRef, useRef } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, className = "", type = "text", min, max, step, disabled, readOnly, ...props }, ref) => {
    const inputRef = useRef<HTMLInputElement | null>(null);
    const isNumber = type === "number";
    const isDate = type === "date";
    const inputType = isNumber ? "text" : type;
    const isCounterDisabled = disabled || readOnly;

    const setInputRef = (node: HTMLInputElement | null) => {
      inputRef.current = node;
      if (typeof ref === "function") {
        ref(node);
      } else if (ref) {
        (ref as MutableRefObject<HTMLInputElement | null>).current = node;
      }
    };

    const toNumber = (value: string | number | undefined) => {
      if (value === undefined || value === "") return undefined;
      const parsed = Number(value);
      return Number.isFinite(parsed) ? parsed : undefined;
    };

    const formatValue = (value: number, stepValue: number) => {
      const stepText = String(stepValue);
      const decimals = stepText.includes(".") ? stepText.split(".")[1].length : 0;
      if (decimals === 0) return String(value);
      return value.toFixed(decimals).replace(/\.?0+$/, "");
    };

    const changeNumber = (direction: -1 | 1) => {
      const input = inputRef.current;
      if (!input || isCounterDisabled) return;

      const stepValue = toNumber(step) ?? 1;
      const minValue = toNumber(min);
      const maxValue = toNumber(max);
      const currentValue = toNumber(input.value) ?? minValue ?? 0;
      let nextValue = currentValue + stepValue * direction;

      if (minValue !== undefined) nextValue = Math.max(minValue, nextValue);
      if (maxValue !== undefined) nextValue = Math.min(maxValue, nextValue);

      const valueSetter = Object.getOwnPropertyDescriptor(
        window.HTMLInputElement.prototype,
        "value"
      )?.set;

      if (valueSetter) {
        valueSetter.call(input, formatValue(nextValue, stepValue));
      } else {
        input.value = formatValue(nextValue, stepValue);
      }

      input.dispatchEvent(new Event("input", { bubbles: true }));
    };

    const input = (
      <input
        ref={setInputRef}
        type={inputType}
        inputMode={isNumber ? "decimal" : props.inputMode}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        readOnly={readOnly}
        className={`w-full rounded-md border border-sand-300 bg-charcoal-50 px-3 py-2.5 text-base text-charcoal-900 outline-none transition-colors placeholder:text-charcoal-400 focus:border-sage-500 md:py-2 md:text-sm dark:border-charcoal-700 dark:bg-charcoal-950/60 dark:text-sand-100 dark:placeholder:text-charcoal-500 dark:focus:border-sage-500 touch-manipulation ${className} ${isNumber ? "pr-8" : ""} ${isDate ? "min-w-[11rem] pr-9" : ""}`}
        {...props}
      />
    );

    return (
      <div className="flex flex-col gap-1">
        {label && (
          <label className="text-xs text-charcoal-500 dark:text-charcoal-400">
            {label}
          </label>
        )}
        {isNumber ? (
          <div className="relative">
            {input}
            <div className="absolute right-1 top-1/2 flex -translate-y-1/2 flex-col">
              <button
                type="button"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => changeNumber(1)}
                disabled={isCounterDisabled}
                className="rounded-sm p-0.5 text-charcoal-400 transition-colors hover:bg-sand-200 hover:text-charcoal-700 disabled:pointer-events-none disabled:opacity-30 dark:hover:bg-charcoal-800 dark:hover:text-sand-200"
                title="Increase"
              >
                <ChevronUp size={12} />
              </button>
              <button
                type="button"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => changeNumber(-1)}
                disabled={isCounterDisabled}
                className="rounded-sm p-0.5 text-charcoal-400 transition-colors hover:bg-sand-200 hover:text-charcoal-700 disabled:pointer-events-none disabled:opacity-30 dark:hover:bg-charcoal-800 dark:hover:text-sand-200"
                title="Decrease"
              >
                <ChevronDown size={12} />
              </button>
            </div>
          </div>
        ) : (
          input
        )}
      </div>
    );
  }
);
