export interface Provider {
  id: string;
  name: string;
  description: string;
  models: Model[];
}

export interface Model {
  id: string;
  name: string;
  description: string;
}

export interface ModelSlotProps {
  title: string;
  description: string;
  icon: React.ReactNode;
  providers: Provider[];
  selectedProvider: string;
  selectedModel: string;
  onProviderChange: (provider: string) => void;
  onModelChange: (model: string) => void;
  optional?: boolean;
  notSetText?: string;
  verifyStatus?: 'idle' | 'verifying' | 'success' | 'error';
  verifyError?: string;
  onVerify?: (provider: string, model: string) => void;
}

export function ModelSlot({
  title,
  description,
  icon,
  providers,
  selectedProvider,
  selectedModel,
  onProviderChange,
  onModelChange,
  optional = false,
  notSetText = "Not configured",
  verifyStatus = 'idle',
  verifyError,
  onVerify
}: ModelSlotProps) {
  const currentProvider = providers.find(p => p.id === selectedProvider);
  const availableModels = currentProvider?.models || [];

  return (
    <div className="border border-border-default rounded-lg p-4 bg-gradient-to-br from-surface-primary to-surface-elevated">
      {/* Header */}
      <div className="flex items-center gap-3 mb-4">
        <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-accent-primary to-intent-purple-hover flex items-center justify-center shadow-md flex-shrink-0">
          {icon}
        </div>
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <h3 className="text-base font-semibold text-content-primary">{title}</h3>
            {optional && (
              <span className="text-xs px-2 py-0.5 bg-surface-3 text-content-secondary rounded-full">
                Optional
              </span>
            )}
          </div>
          <p className="text-xs text-content-secondary mt-0.5">{description}</p>
        </div>
      </div>

      {/* Provider Selection */}
      <div className="space-y-3">
        <div>
          <label className="block text-xs font-medium text-content-secondary mb-1.5">
            Provider
          </label>
          <select
            value={selectedProvider || ''}
            onChange={(e) => {
              const newProvider = e.target.value;
              onProviderChange(newProvider);
              // Reset model selection when provider changes
              const provider = providers.find(p => p.id === newProvider);
              if (provider && provider.models.length > 0) {
                onModelChange(provider.models[0].id);
              }
            }}
            className="w-full px-3 py-2 text-sm border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent bg-surface-primary"
          >
            {optional && (
              <option value="">{notSetText}</option>
            )}
            {providers.map(provider => (
              <option key={provider.id} value={provider.id}>
                {provider.name}
              </option>
            ))}
          </select>
        </div>

        {/* Model Selection */}
        {selectedProvider && (
          <div>
            <label className="block text-xs font-medium text-content-secondary mb-1.5">
              Model
            </label>
            <select
              value={selectedModel || ''}
              onChange={(e) => onModelChange(e.target.value)}
              className="w-full px-3 py-2 text-sm border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent bg-surface-primary"
              disabled={availableModels.length === 0}
            >
              {availableModels.map(model => (
                <option key={model.id} value={model.id}>
                  {model.name}
                </option>
              ))}
            </select>
            {availableModels.find(m => m.id === selectedModel) && (
              <p className="mt-1.5 text-xs text-content-tertiary">
                {availableModels.find(m => m.id === selectedModel)?.description}
              </p>
            )}

            {/* Verification Status Area */}
            {selectedProvider && selectedModel && onVerify && (
              <div className="mt-3 flex items-center justify-between border-t border-border-subtle pt-3">
                <div className="flex-1 pr-4">
                  {verifyStatus === 'success' && (
                    <div className="flex items-center gap-1.5 text-intent-success">
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg>
                      <span className="text-xs font-medium">Model verified successfully</span>
                    </div>
                  )}
                  {verifyStatus === 'error' && (
                    <div className="flex flex-col gap-0.5">
                      <div className="flex items-center gap-1.5 text-intent-danger">
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                        <span className="text-xs font-medium">Verification failed</span>
                      </div>
                      {verifyError && (
                        <span className="text-xs text-intent-danger break-words line-clamp-2" title={verifyError}>
                          {verifyError}
                        </span>
                      )}
                    </div>
                  )}
                  {verifyStatus === 'idle' && (
                    <span className="text-xs text-content-tertiary">Not verified</span>
                  )}
                </div>
                <button
                  onClick={() => onVerify(selectedProvider, selectedModel)}
                  disabled={verifyStatus === 'verifying'}
                  className="px-3 py-1.5 text-xs font-medium text-content-secondary bg-surface-2 hover:bg-surface-3 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5 flex-shrink-0"
                >
                  {verifyStatus === 'verifying' ? (
                    <>
                      <div className="w-3 h-3 border-2 border-border-emphasis border-t-transparent rounded-full animate-spin" />
                      Verifying...
                    </>
                  ) : (
                    'Verify'
                  )}
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
