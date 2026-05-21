"use client";

import { SessionProvider } from "next-auth/react";
import { ReactNode } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";

const Providers = ({ children }: { children: ReactNode }) => (
  <SessionProvider>
    <TooltipProvider delayDuration={200}>{children}</TooltipProvider>
  </SessionProvider>
);

export default Providers;
