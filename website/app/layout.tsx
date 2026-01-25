import type { Metadata } from 'next'
import '../styles/globals.css'

export const metadata: Metadata = {
  title: 'AnyCowork - Open-Source AI Assistant Platform',
  description: 'Build powerful AI agents with multi-provider support, federation, and seamless integrations. Local-first, open-source, and extensible.',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <head>
        <link rel="icon" type="image/svg+xml" href="/logo.svg" />
        <link rel="alternate icon" href="/favicon.ico" />
      </head>
      <body suppressHydrationWarning>
        {children}
      </body>
    </html>
  )
}
