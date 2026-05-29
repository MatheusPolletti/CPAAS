"use client";

import * as React from "react";

import { NavRoutes } from "@/components/sidebar/nav-routes";
import { NavUser } from "@/components/sidebar/nav-user";
import { CompanyHeader } from "@/components/sidebar/company-header";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar";
import { MessageSquare, MessagesSquare, Phone, Users } from "lucide-react";
import { Session } from "next-auth";

const data = {
  projects: [
    {
      name: "SMS",
      url: "/sms",
      icon: <MessageSquare />,
    },
    {
      name: "Contatos",
      url: "/contacts",
      icon: <Users />,
    },
    {
      name: "Whatsapp",
      url: "/whatsapp",
      icon: <MessagesSquare />,
    },
    {
      name: "Ligação",
      url: "/call",
      icon: <Phone />,
    },
  ],
};

export function AppSidebar({
  ...props
}: React.ComponentProps<typeof Sidebar> & { session: Session | null }) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <CompanyHeader />
      </SidebarHeader>
      <SidebarContent>
        <NavRoutes projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser session={props.session} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
