import * as React from "react";
import { cn } from "@/lib/utils";

interface SelectContextValue {
  value: string;
  onValueChange: (value: string) => void;
  open: boolean;
  setOpen: (open: boolean) => void;
}
const SelectContext = React.createContext<SelectContextValue>({ value: "", onValueChange: () => {}, open: false, setOpen: () => {} });

function Select({ value, onValueChange, children }: { value?: string; onValueChange?: (value: string) => void; children: React.ReactNode }) {
  const [internalValue, setInternalValue] = React.useState(value ?? "");
  const [open, setOpen] = React.useState(false);
  const val = value !== undefined ? value : internalValue;
  const setVal = onValueChange ?? setInternalValue;
  return (
    <SelectContext.Provider value={{ value: val, onValueChange: setVal, open, setOpen }}>
      <div className="relative">{children}</div>
    </SelectContext.Provider>
  );
}

function SelectTrigger({ className, children, ...props }: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  const { open, setOpen } = React.useContext(SelectContext);
  return (
    <button
      type="button"
      className={cn(
        "flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
        className,
      )}
      onClick={() => setOpen(!open)}
      {...props}
    >
      {children}
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="opacity-50"><path d="m6 9 6 6 6-6"/></svg>
    </button>
  );
}

function SelectValue({ placeholder }: { placeholder?: string }) {
  const { value } = React.useContext(SelectContext);
  const [label, setLabel] = React.useState<string>("");
  const ctx = React.useContext(SelectContext);
  // We'll render the value label via SelectItem registration
  return <span className={value ? "" : "text-muted-foreground"}>{label || value || placeholder}</span>;
}

function SelectContent({ className, children, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  const { open, setOpen } = React.useContext(SelectContext);
  if (!open) return null;
  return (
    <>
      <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
      <div
        className={cn(
          "absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border bg-popover text-popover-foreground shadow-md",
          className,
        )}
        {...props}
      >
        {children}
      </div>
    </>
  );
}

function SelectItem({ value, children, className, ...props }: React.HTMLAttributes<HTMLDivElement> & { value: string }) {
  const ctx = React.useContext(SelectContext);
  return (
    <div
      className={cn(
        "relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent hover:text-accent-foreground",
        ctx.value === value && "bg-accent",
        className,
      )}
      onClick={() => { ctx.onValueChange(value); ctx.setOpen(false); }}
      {...props}
    >
      {children}
    </div>
  );
}

function SelectGroup({ children }: { children: React.ReactNode }) {
  return <div>{children}</div>;
}

function SelectLabel({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("px-2 py-1.5 text-xs font-semibold text-muted-foreground", className)} {...props} />;
}

export { Select, SelectTrigger, SelectValue, SelectContent, SelectItem, SelectGroup, SelectLabel };
