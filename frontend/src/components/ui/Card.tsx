import { ReactNode } from "react";

interface CardProps {
  children: ReactNode;
  className?: string;
}

export function Card({ children, className = "" }: CardProps) {
  return (
    <div
      className={`rounded-md border border-sand-300 bg-charcoal-50 p-4 shadow-[0_10px_30px_rgb(0_0_0_/_0.08)] dark:border-charcoal-700 dark:bg-charcoal-900 dark:shadow-[0_10px_30px_rgb(0_0_0_/_0.18)] ${className}`}
    >
      {children}
    </div>
  );
}
