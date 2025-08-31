import type { Metadata } from 'next'
import './globals.css'
import Sidebar from '../components/Sidebar'

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
        <div className="flex h-screen">
          {/* Sidebar */}
          <Sidebar />
          
          {/* Main Content Area */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* Top Navigation Bar */}
            <header className="border-b border-slate-200 bg-white/50 backdrop-blur-sm">
              <div className="px-4 sm:px-6 lg:px-8">
                <div className="flex justify-between items-center h-16">
                  <div className="flex items-center">
                    <h1 className="text-lg font-semibold text-slate-900">
                      Dashboard
                    </h1>
                  </div>
                  <div className="flex items-center space-x-4">
                    <span className="text-sm text-slate-600 bg-slate-100 px-2 py-1 rounded-md">
                      Next.js 15 + App Router
                    </span>
                  </div>
                </div>
              </div>
            </header>
            
            {/* Page Content */}
            <main className="flex-1 overflow-auto">
              <div className="p-4 sm:p-6 lg:p-8">
                {children}
              </div>
            </main>
          </div>
        </div>
      </body>
    </html>
  )
}
