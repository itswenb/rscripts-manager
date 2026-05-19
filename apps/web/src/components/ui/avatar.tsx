import * as React from "react";
import { cn } from "@/lib/utils";

function Avatar({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("relative flex h-8 w-8 shrink-0 overflow-hidden rounded-full", className)} {...props} />;
}

function AvatarImage({ className, ...props }: React.ImgHTMLAttributes<HTMLImageElement>) {
  return <img className={cn("aspect-square h-full w-full", className)} {...props} />;
}

function AvatarFallback({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn("flex h-full w-full items-center justify-center rounded-full bg-muted text-xs font-medium", className)} {...props} />
  );
}

export { Avatar, AvatarImage, AvatarFallback };
