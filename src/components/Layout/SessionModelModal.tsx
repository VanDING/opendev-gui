import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { configRepository, sessionRepository } from '../../repositories';
import { ModelSlot } from '../Settings/ModelSlot';
import type { Provider } from '../Settings/ModelSlot';
import { toast } from 'sonner';

interface SessionModelModalProps {
  sessionId: string | null;
  sessionLabel: string;
  onClose: () => void;
}

export function SessionModelModal({ sessionId, sessionLabel, onClose }: SessionModelModalProps) {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [hasExistingOverlay, setHasExistingOverlay] = useState(false);
  // Model slots
  const [normalProvider, setNormalProvider] = useState('');
  const [normalModel, setNormalModel] = useState('');
  const [thinkingProvider, setThinkingProvider] = useState('');
  const [thinkingModel, setThinkingModel] = useState('');
  const [compactProvider, setCompactProvider] = useState('');
  const [compactModel, setCompactModel] = useState('');
  const [visionProvider, setVisionProvider] = useState('');
  const [visionModel, setVisionModel] = useState('');

  // Verification states
  type VerifyStatus = 'idle' | 'verifying' | 'success' | 'error';
  const [normalVerifyStatus, setNormalVerifyStatus] = useState<VerifyStatus>('idle');
  const [normalVerifyError, setNormalVerifyError] = useState<string>();
  const [thinkingVerifyStatus, setThinkingVerifyStatus] = useState<VerifyStatus>('idle');
  const [thinkingVerifyError, setThinkingVerifyError] = useState<string>();
  const [compactVerifyStatus, setCompactVerifyStatus] = useState<VerifyStatus>('idle');
  const [compactVerifyError, setCompactVerifyError] = useState<string>();
  const [visionVerifyStatus, setVisionVerifyStatus] = useState<VerifyStatus>('idle');
  const [visionVerifyError, setVisionVerifyError] = useState<string>();

  useEffect(() => {
    if (sessionId) {
      loadData();
    }
  }, [sessionId]);

  const loadData = async () => {
    if (!sessionId) return;
    try {
      setLoading(true);
      const [providersData, configData, overlayData] = await Promise.all([
        configRepository.listProviders(),
        configRepository.getConfig(),
        sessionRepository.getSessionModel(sessionId),
      ]);

      setProviders(providersData);

      const hasOverlay = Object.keys(overlayData).length > 0;
      setHasExistingOverlay(hasOverlay);

      // Use overlay values if set, otherwise fall back to global config
      setNormalProvider(overlayData.model_provider || configData.model_provider || '');
      setNormalModel(overlayData.model || configData.model || '');
      setThinkingProvider(overlayData.model_thinking_provider || configData.model_thinking_provider || '');
      setThinkingModel(overlayData.model_thinking || configData.model_thinking || '');
      setCompactProvider(overlayData.model_compact_provider || configData.model_compact_provider || '');
      setCompactModel(overlayData.model_compact || configData.model_compact || '');
      setVisionProvider(overlayData.model_vlm_provider || configData.model_vlm_provider || '');
      setVisionModel(overlayData.model_vlm || configData.model_vlm || '');
    } catch (error) {
      console.error('Failed to load session model data:', error);
    } finally {
      setLoading(false);
    }
  };

  const verifySingleModel = async (
    provider: string,
    model: string,
    setStatus: (status: VerifyStatus) => void,
    setError: (error: string | undefined) => void
  ): Promise<boolean> => {
    if (!provider || !model) return true; // Nothing to verify

    setStatus('verifying');
    setError(undefined);

    try {
      const result = await configRepository.verifyModel(provider, model);
      if (result.valid) {
        setStatus('success');
        return true;
      } else {
        setStatus('error');
        setError(result.error || 'Verification failed');
        return false;
      }
    } catch (err: any) {
      setStatus('error');
      setError(err.message || 'Verification failed');
      return false;
    }
  };

  const verifyAllModels = async (): Promise<boolean> => {
    setSaving(true);
    let allValid = true;

    // Run verifications in parallel
    const verifications = [
      normalProvider && normalModel
        ? verifySingleModel(normalProvider, normalModel, setNormalVerifyStatus, setNormalVerifyError)
        : Promise.resolve(true),
      thinkingProvider && thinkingModel
        ? verifySingleModel(thinkingProvider, thinkingModel, setThinkingVerifyStatus, setThinkingVerifyError)
        : Promise.resolve(true),
      compactProvider && compactModel
        ? verifySingleModel(compactProvider, compactModel, setCompactVerifyStatus, setCompactVerifyError)
        : Promise.resolve(true),
      visionProvider && visionModel
        ? verifySingleModel(visionProvider, visionModel, setVisionVerifyStatus, setVisionVerifyError)
        : Promise.resolve(true),
    ];

    const results = await Promise.all(verifications);
    allValid = results.every(Boolean);

    if (!allValid) {
      toast.error('One or more models failed verification');
      setSaving(false);
    }

    return allValid;
  };

  const handleSave = async () => {
    if (!sessionId) return;

    // Verify before saving
    const isValid = await verifyAllModels();
    if (!isValid) return;

    try {
      setSaving(true);

      await sessionRepository.updateSessionModel(
        sessionId,
        normalModel,
        normalProvider,
      );

      setHasExistingOverlay(true);
      toast.success('Session model updated');
      onClose();
    } catch (error) {
      console.error('Failed to save session model:', error);
      toast.error('Failed to save session model');
    } finally {
      setSaving(false);
    }
  };

  const handleClear = async () => {
    if (!sessionId) return;
    try {
      setSaving(true);
      await sessionRepository.clearSessionModel(sessionId);
      setHasExistingOverlay(false);

      // Reload with global defaults
      const configData = await configRepository.getConfig();
      setNormalProvider(configData.model_provider || '');
      setNormalModel(configData.model || '');
      setThinkingProvider(configData.model_thinking_provider || '');
      setThinkingModel(configData.model_thinking || '');
      setCompactProvider(configData.model_compact_provider || '');
      setCompactModel(configData.model_compact || '');
      setVisionProvider(configData.model_vlm_provider || '');
      setVisionModel(configData.model_vlm || '');

      toast.success('Session model cleared');
      onClose();
    } catch (error) {
      console.error('Failed to clear session model:', error);
      toast.error('Failed to clear session model');
    } finally {
      setSaving(false);
    }
  };

  if (!sessionId) return null;

  const modalContent = (
    <div
      className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="bg-surface-primary rounded-2xl shadow-2xl w-full max-w-lg max-h-[90vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="px-6 py-4 border-b border-border-default flex-shrink-0">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-lg font-bold text-content-primary">Session Models</h2>
              <p className="text-xs text-content-tertiary mt-0.5">{sessionLabel}</p>
            </div>
            <button
              onClick={onClose}
              className="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-surface-2 text-content-tertiary hover:text-content-secondary"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Info banner */}
          <div className="mt-3 px-3 py-2 bg-intent-warning-muted border border-intent-warning-muted rounded-lg">
            <p className="text-xs text-intent-warning">
              Override models for this session only. Changes don't affect global settings.
            </p>
          </div>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <div className="flex items-center gap-2 text-content-secondary">
                <div className="w-4 h-4 border-2 border-border-emphasis border-t-surface-muted rounded-full animate-spin" />
                <span className="text-sm">Loading...</span>
              </div>
            </div>
          ) : (
            <>
              <ModelSlot
                title="Normal Model"
                description="Standard coding tasks"
                icon={<svg className="w-5 h-5 text-content-inverse" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" /></svg>}
                providers={providers}
                selectedProvider={normalProvider}
                selectedModel={normalModel}
                onProviderChange={(p) => { setNormalProvider(p); setNormalVerifyStatus('idle'); }}
                onModelChange={(m) => { setNormalModel(m); setNormalVerifyStatus('idle'); }}
                verifyStatus={normalVerifyStatus}
                verifyError={normalVerifyError}
                onVerify={(p, m) => verifySingleModel(p, m, setNormalVerifyStatus, setNormalVerifyError)}
              />

              <ModelSlot
                title="Thinking Model"
                description="Complex reasoning (falls back to Normal)"
                icon={<svg className="w-5 h-5 text-content-inverse" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" /></svg>}
                providers={providers}
                selectedProvider={thinkingProvider}
                selectedModel={thinkingModel}
                onProviderChange={(p) => { setThinkingProvider(p); setThinkingVerifyStatus('idle'); }}
                onModelChange={(m) => { setThinkingModel(m); setThinkingVerifyStatus('idle'); }}
                optional
                notSetText="Use Normal Model"
                verifyStatus={thinkingVerifyStatus}
                verifyError={thinkingVerifyError}
                onVerify={(p, m) => verifySingleModel(p, m, setThinkingVerifyStatus, setThinkingVerifyError)}
              />

              <ModelSlot
                title="Compact Model"
                description="Context compaction (falls back to Normal)"
                icon={<svg className="w-5 h-5 text-content-inverse" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>}
                providers={providers}
                selectedProvider={compactProvider}
                selectedModel={compactModel}
                onProviderChange={(p) => { setCompactProvider(p); setCompactVerifyStatus('idle'); }}
                onModelChange={(m) => { setCompactModel(m); setCompactVerifyStatus('idle'); }}
                optional
                notSetText="Use Normal Model"
                verifyStatus={compactVerifyStatus}
                verifyError={compactVerifyError}
                onVerify={(p, m) => verifySingleModel(p, m, setCompactVerifyStatus, setCompactVerifyError)}
              />

              <ModelSlot
                title="Vision Model"
                description="Image processing (disabled if not set)"
                icon={<svg className="w-5 h-5 text-content-inverse" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>}
                providers={providers}
                selectedProvider={visionProvider}
                selectedModel={visionModel}
                onProviderChange={(p) => { setVisionProvider(p); setVisionVerifyStatus('idle'); }}
                onModelChange={(m) => { setVisionModel(m); setVisionVerifyStatus('idle'); }}
                optional
                notSetText="Vision Disabled"
                verifyStatus={visionVerifyStatus}
                verifyError={visionVerifyError}
                onVerify={(p, m) => verifySingleModel(p, m, setVisionVerifyStatus, setVisionVerifyError)}
              />
            </>
          )}
        </div>

        {/* Footer */}
        {!loading && (
          <div className="px-6 py-4 border-t border-border-default flex-shrink-0 space-y-3">
            <div className="flex gap-3">
              {hasExistingOverlay && (
                <button
                  onClick={handleClear}
                  disabled={saving}
                  className="px-4 py-2.5 border border-red-300 text-intent-danger rounded-lg hover:bg-intent-danger-muted disabled:opacity-50 disabled:cursor-not-allowed font-medium text-sm"
                >
                  Clear Overrides
                </button>
              )}
              <button
                onClick={handleSave}
                disabled={saving}
                className="flex-1 px-4 py-2.5 bg-gradient-to-r from-intent-warning to-intent-warning-hover hover:from-intent-warning-hover hover:to-intent-warning-fg text-content-inverse rounded-lg disabled:opacity-50 disabled:cursor-not-allowed font-medium text-sm shadow-md hover:shadow-lg"
              >
                {saving ? (
                  <span className="flex items-center justify-center gap-2">
                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    Saving...
                  </span>
                ) : (
                  'Save'
                )}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );

  return createPortal(modalContent, document.body);
}
