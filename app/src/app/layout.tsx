import { NavBar } from "@/components/roots/NavBar";
import { Providers } from "@/components/roots/Providers";
import "@/styles/globals.css";
import type { Metadata } from "next";
import { Saira } from "next/font/google";

const sans = Saira({
  variable: "--font-sans",
  subsets: ["latin"],
});



export const metadata: Metadata = {
  title: { default: "Caduceus", template: "%s | Caduceus" },
  description: "An open-source alternative to Typst App.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning className={sans.variable}>
      <body className="min-h-screen bg-background font-sans text-foreground antialiased">
        <Providers>
          <div className="relative h-screen">
            <NavBar />
            {children}
            <footer>
              <p className="text-sm text-muted-foreground">
                © {new Date().getFullYear()} Your Company Name
              </p>
            </footer>
          </div>
        </Providers>
      </body>
    </html>
  );
}
