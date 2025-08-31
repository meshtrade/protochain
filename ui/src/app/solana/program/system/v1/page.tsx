export default function SystemProgramV1Page() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-slate-900">System Program Service v1</h1>
        <p className="mt-1 text-sm text-slate-600">
          Core Solana system program operations - all return composable SolanaInstruction
        </p>
      </div>

      <div className="bg-white shadow rounded-lg p-6">
        <h2 className="text-lg font-medium text-slate-900 mb-4">Core Operations</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mb-6">
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">Create</h3>
            <p className="text-xs text-slate-600 mt-1">Create new account</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">Transfer</h3>
            <p className="text-xs text-slate-600 mt-1">Transfer SOL</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">Allocate</h3>
            <p className="text-xs text-slate-600 mt-1">Allocate space</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">Assign</h3>
            <p className="text-xs text-slate-600 mt-1">Change owner</p>
          </div>
        </div>

        <h2 className="text-lg font-medium text-slate-900 mb-4">Extended Operations</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">CreateWithSeed</h3>
            <p className="text-xs text-slate-600 mt-1">Seed-based account creation</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">InitializeNonceAccount</h3>
            <p className="text-xs text-slate-600 mt-1">Initialize nonce account</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">AuthorizeNonceAccount</h3>
            <p className="text-xs text-slate-600 mt-1">Change nonce authority</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">WithdrawNonceAccount</h3>
            <p className="text-xs text-slate-600 mt-1">Withdraw from nonce</p>
          </div>
        </div>
      </div>

      <div className="bg-green-50 border border-green-200 rounded-md p-4">
        <p className="text-sm text-green-800">
          <span className="font-medium">Composable Design:</span> All methods return SolanaInstruction that can be added to draft transactions.
        </p>
      </div>
    </div>
  )
}