export default function TransactionV1Page() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-slate-900">Transaction Service v1</h1>
        <p className="mt-1 text-sm text-slate-600">
          Complete transaction lifecycle management: compile → sign → submit
        </p>
      </div>

      <div className="bg-white shadow rounded-lg p-6">
        <h2 className="text-lg font-medium text-slate-900 mb-4">Available Methods</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">CompileTransaction</h3>
            <p className="text-xs text-slate-600 mt-1">DRAFT → COMPILED</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">EstimateTransaction</h3>
            <p className="text-xs text-slate-600 mt-1">Fee calculation</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">SimulateTransaction</h3>
            <p className="text-xs text-slate-600 mt-1">Dry run</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">SignTransaction</h3>
            <p className="text-xs text-slate-600 mt-1">COMPILED → SIGNED</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">SubmitTransaction</h3>
            <p className="text-xs text-slate-600 mt-1">SIGNED → SUBMITTED</p>
          </div>
          <div className="border border-slate-200 rounded-md p-3">
            <h3 className="text-sm font-medium text-slate-900">GetTransaction</h3>
            <p className="text-xs text-slate-600 mt-1">Fetch by signature</p>
          </div>
        </div>
      </div>

      <div className="bg-amber-50 border border-amber-200 rounded-md p-4">
        <p className="text-sm text-amber-800">
          <span className="font-medium">State Machine:</span> This service implements a strict transaction state machine with instruction composition support.
        </p>
      </div>
    </div>
  )
}