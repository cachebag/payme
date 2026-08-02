import { ButtonHTMLAttributes, ReactNode } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "danger" | "ghost";
  size?: "sm" | "md" | "lg";
  children: ReactNode;
}

export function Button({
  variant = "primary",
  size = "md",
  children,
  className = "",
  ...props
}: ButtonProps) {
  const baseStyles =
    "inline-flex items-center justify-center rounded-md border font-medium transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-50 touch-manipulation";

  const variants = {
    primary:
      "border-sage-600 bg-sage-600 text-white hover:bg-sage-700 active:bg-sage-800 dark:border-sage-500 dark:bg-sage-500 dark:text-charcoal-950 dark:hover:bg-sage-400 dark:active:bg-sage-600",
    secondary:
      "border-sand-300 bg-sand-100 text-charcoal-800 hover:bg-sand-200 active:bg-sand-300 dark:border-charcoal-700 dark:bg-charcoal-800 dark:text-sand-200 dark:hover:bg-charcoal-700 dark:active:bg-charcoal-600",
    danger:
      "border-terracotta-600 bg-terracotta-600 text-white hover:bg-terracotta-700 active:bg-terracotta-800 dark:border-terracotta-600 dark:bg-terracotta-600 dark:text-white dark:hover:bg-terracotta-500 dark:active:bg-terracotta-700",
    ghost:
      "border-transparent bg-transparent text-charcoal-600 hover:bg-sand-200 hover:text-charcoal-900 active:bg-sand-300 dark:text-charcoal-400 dark:hover:bg-charcoal-800 dark:hover:text-sand-200 dark:active:bg-charcoal-700",
  };

  const sizes = {
    sm: "min-h-9 px-3 py-1.5 text-sm md:text-xs",
    md: "min-h-10 px-4 py-2 text-base md:text-sm",
    lg: "min-h-11 px-5 py-2.5 text-lg md:text-base",
  };

  return (
    <button
      className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
}
