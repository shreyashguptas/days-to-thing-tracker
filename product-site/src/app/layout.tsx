import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "DaysTracker - Never Forget What Matters",
  description:
    "The ambient task tracker that keeps your life organized. A beautiful display that quietly tracks your recurring tasks - always visible, always on, always keeping you on track.",
  keywords: [
    "task tracker",
    "recurring tasks",
    "habit tracker",
    "smart home",
    "productivity",
    "home organization",
    "ambient display",
  ],
  openGraph: {
    title: "DaysTracker - Never Forget What Matters",
    description:
      "The ambient task tracker that keeps your life organized without the noise.",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "DaysTracker - Never Forget What Matters",
    description:
      "The ambient task tracker that keeps your life organized without the noise.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="scroll-smooth">
      <body className="antialiased">
        {children}
      </body>
    </html>
  );
}
