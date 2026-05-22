import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/sidebar/app-sidebar";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth-options";

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const session = await getServerSession(authOptions);

  return (
    <SidebarProvider>
      <AppSidebar session={session} />
      <SidebarInset>{children}</SidebarInset>
    </SidebarProvider>
  );
}
