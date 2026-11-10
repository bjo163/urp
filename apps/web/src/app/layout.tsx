import type { Metadata, Viewport } from 'next';
import type { ReactNode } from 'react';
import './globals.css';

export const metadata: Metadata = {
  title: {
    default: 'URP — Universal Radical Platform',
    template: '%s · URP',
  },
  description: 'A radical, modular business operating platform built with Rust and Next.js.',
  applicationName: 'URP',
};

export const viewport: Viewport = {
  themeColor: '#10110f',
  colorScheme: 'dark light',
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="id">
      <body>{children}</body>
    </html>
  );
}
