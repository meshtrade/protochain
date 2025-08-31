import { Suspense } from 'react'
import { ProtoSolDashboard } from '@/components/ProtoSolDashboard'

// Temporary mock for ProtoSol API until workspace is resolved
const mockProtoSolApi = {
  VERSION: '1.0.0',
  SDK_NAME: 'ProtoSol SDK (Mock)',
  run: () => console.log('Mock ProtoSol SDK running...')
}

export default function HomePage() {
  return (
    <>
      <div className="text-center py-12 animate-fade-in">
        <div className="max-w-3xl mx-auto space-y-6">
          <div className="text-sm font-medium text-slate-500 tracking-wider uppercase">
            Welcome to the Prototype
          </div>
          <h1 className="text-4xl sm:text-6xl font-bold bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600 bg-clip-text text-transparent">
            ProtoSol UI Demo
          </h1>
          <p className="text-xl text-slate-600 max-w-2xl mx-auto leading-relaxed">
            Experience the latest in blockchain development with Next.js 15, TypeScript, and the ProtoSol SDK.
            Built with modern approaches and cutting-edge technologies.
          </p>
          <div className="flex flex-wrap justify-center gap-3 pt-4">
            <span className="px-3 py-1 text-sm bg-purple-100 text-purple-800 rounded-full">
              Next.js 15
            </span>
            <span className="px-3 py-1 text-sm bg-blue-100 text-blue-800 rounded-full">
              App Router
            </span>
            <span className="px-3 py-1 text-sm bg-indigo-100 text-indigo-800 rounded-full">
              TypeScript
            </span>
            <span className="px-3 py-1 text-sm bg-cyan-100 text-cyan-800 rounded-full">
              Tailwind CSS
            </span>
            <span className="px-3 py-1 text-sm bg-emerald-100 text-emerald-800 rounded-full">
              ProtoSol SDK
            </span>
          </div>
        </div>
      </div>

      <div className="mt-16">
        <Suspense fallback={<LoadingSkeleton />}>
          <ProtoSolDashboard />
        </Suspense>
      </div>
    </>
  )
}

function LoadingSkeleton() {
  return (
    <div className="max-w-md mx-auto bg-white rounded-2xl shadow-lg p-6 animate-pulse">
      <div className="h-4 bg-slate-200 rounded w-3/4 mb-4 mx-auto"></div>
      <div className="space-y-3">
        <div className="h-3 bg-slate-200 rounded"></div>
        <div className="h-3 bg-slate-200 rounded w-5/6"></div>
        <div className="h-3 bg-slate-200 rounded w-4/6"></div>
      </div>
    </div>
  )
}
