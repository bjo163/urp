import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";

import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "URP — Universal Radical Platform",
    template: "%s · URP"
  },
  description:
    "A next-generation business operating platform built with Rust and Next.js.",
  applicationName: "URP"
};

export const viewport: Viewport = {
  themeColor: "#111315",
  colorScheme: "dark light"
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
