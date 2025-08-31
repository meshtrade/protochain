export default function RPCClientV1Page() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-slate-900">RPC Client Service v1</h1>
        <p className="mt-1 text-sm text-slate-600">
          Direct Solana RPC client calls with minimal viable operations
        </p>
      </div>

      <div className="bg-white shadow rounded-lg p-6">
        <h2 className="text-lg font-medium text-slate-900 mb-4">Available Methods</h2>
        <div className="space-y-3">
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">GetMinimumBalanceForRentExemption</h3>
            <p className="text-xs text-slate-600 mt-1">
              Calculate minimum balance required for rent exemption based on data length
            </p>
          </div>
        </div>
      </div>

      <div className="bg-indigo-50 border border-indigo-200 rounded-md p-4">
        <p className="text-sm text-indigo-800">
          <span className="font-medium">Minimal Scope:</span> This service provides essential RPC operations as specified in the scope limiter.
        </p>
      </div>
    </div>
  )
}