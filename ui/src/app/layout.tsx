import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'ProtoSol - Solana Protocol Frontend',
  description: 'Modern React frontend for ProtoSol Solana SDK',
  keywords: ['solana', 'blockchain', 'protoSol', 'typescript', 'rpc'],
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-100">
        <nav className="border-b border-slate-200 bg-white/50 backdrop-blur-sm">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between h-16">
              <div className="flex items-center">
                <div className="flex-shrink-0 flex items-center">
                  <div className="h-8 w-8 bg-gradient-to-r from-purple-500 to-blue-600 rounded-lg flex items-center justify-center">
                    <span className="text-white font-bold text-sm">P</span>
                  </div>
                  <span className="ml-3 text-xl font-semibold text-slate-900">ProtoSol</span>
                </div>
              </div>
              <div className="flex items-center space-x-4">
                <span className="text-sm text-slate-600 bg-slate-100 px-2 py-1 rounded-md">
                  Next.js 15 + App Router
                </span>
              </div>
            </div>
          </div>
        </nav>
        <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
          {children}
        </main>
      </body>
    </html>
  )
}
