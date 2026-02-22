import * as React from "react";
import { cn } from "@/lib/utils";

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "icon";
  size?: "default" | "sm" | "lg" | "icon";
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "primary", size = "default", ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          "inline-flex items-center justify-center rounded-sm font-bold transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-black focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none active:translate-y-0 active:shadow-none",
          {
            "bg-accent-main text-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 hover:bg-accent-secondary":
              variant === "primary",
            "bg-black text-white border border-black shadow-hard-sm hover:shadow-hard hover:-translate-y-0.5 hover:bg-gray-900":
              variant === "secondary",
            "bg-transparent text-gray-700 border border-gray-300 hover:border-black hover:text-black":
              variant === "outline",
            "bg-transparent text-gray-600 border border-transparent hover:bg-gray-50 hover:border-black hover:text-black":
              variant === "ghost",
            "p-2 text-gray-500 hover:bg-black hover:text-white border border-transparent hover:border-black rounded-sm":
              variant === "icon",
            "h-10 py-2 px-4": size === "default",
            "h-9 px-3 text-xs": size === "sm",
            "h-11 px-8 text-lg": size === "lg",
            "h-10 w-10": size === "icon",
          },
          className,
        )}
        {...props}
      />
    );
  },
);
Button.displayName = "Button";

export { Button };
