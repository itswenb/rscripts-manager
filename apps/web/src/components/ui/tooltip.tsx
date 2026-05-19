import * as React from "react";
import { cn } from "@/lib/utils";

interface TooltipContextValue {
  open: boolean;
  setOpen: (open: boolean) => void;
}
const TooltipContext = React.createContext<TooltipContextValue>({ open: false, setOpen: () => {} });

function TooltipProvider({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}

function Tooltip({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = React.useState(false);
  return <TooltipContext.Provider value={{ open, setOpen }}><div className="relative inline-block">{children}</div></TooltipContext.Provider>;
}

function TooltipTrigger({ children, asChild }: { children: React.ReactNode; asChild?: boolean }) {
  const { setOpen } = React.useContext(TooltipContext);
  if (asChild && React.isValidElement(children)) {
    return React.cloneElement(children as React.ReactElement<{ onMouseEnter?: () => void; onMouseLeave?: () => void }>, {
      onMouseEnter: () => setOpen(true),
      onMouseLeave: () => setOpen(false),
    });
  }
  return <span onMouseEnter={() => setOpen(true)} onMouseLeave={() => setOpen(false)}>{children}</span>;
}

function TooltipContent({ className, children, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  const { open } = React.useContext(TooltipContext);
  if (!open) return null;
  return (
    <div
      className={cn(
        "absolute bottom-full left-1/2 z-50 mb-1 -translate-x-1/2 rounded-md bg-primary px-2 py-1 text-xs text-primary-foreground shadow",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}

export { TooltipProvider, Tooltip, TooltipTrigger, TooltipContent };
