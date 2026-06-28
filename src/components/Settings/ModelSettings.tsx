import { useState, useEffect } from 'react';
import { Cpu, Brain, LayoutGrid, Eye, EyeOff } from 'lucide-react';
import { configRepository } from '../../repositories';
import { ModelSlot } from './ModelSlot';
import type { Provider } from './ModelSlot';
import { toast } from 'sonner';
import { Button } from '../ui/Button';

interface Config {
  model_provider: string;
  model: string;
  model_thinking_provider?: string | null;
  model_thinking?: string | null;
  model_vlm_provider?: string | null;
  model_vlm?: string | null;
  model_compact_provider?: string | null;
  model_compact?: string | null;
  temperature: number;
  api_key?: string | null;
  api_base_url?: string | null;
}

export function ModelSettings() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  // Normal model
  const [normalProvider, setNormalProvider] = useState<string>('');
  const [normalModel, setNormalModel] = useState<string>('');

  // Thinking model
  const [thinkingProvider, setThinkingProvider] = useState<string>('');
  const [thinkingModel, setThinkingModel] = useState<string>('');

  // Vision model
  const [visionProvider, setVisionProvider] = useState<string>('');
  const [visionModel, setVisionModel] = useState<string>('');

  // Compact model
  const [compactProvider, setCompactProvider] = useState<string>('');
  const [compactModel, setCompactModel] = useState<string>('');

  // Other settings
  const [temperature, setTemperature] = useState<number>(0.7);

  // API configuration
  const [apiKey, setApiKey] = useState('');
  const [apiBaseUrl, setApiBaseUrl] = useState('');
  const [apiKeyDirty, setApiKeyDirty] = useState(false);
  const [apiBaseUrlDirty, setApiBaseUrlDirty] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [verifyState, setVerifyState] = useState<'idle' | 'verifying' | 'success' | 'error'>('idle');
  const [verifyMessage, setVerifyMessage] = useState('');
  const [shadowedEnvVars, setShadowedEnvVars] = useState<string[]>([]);

  // Map of provider IDs to their known environment variable names
  const PROVIDER_ENV_VARS: Record<string, string> = {
    openai: 'OPENAI_API_KEY',
    anthropic: 'ANTHROPIC_API_KEY',
    google: 'GOOGLE_API_KEY',
    gemini: 'GEMINI_API_KEY',
    groq: 'GROQ_API_KEY',
    mistral: 'MISTRAL_API_KEY',
    deepseek: 'DEEPSEEK_API_KEY',
    openrouter: 'OPENROUTER_API_KEY',
    together: 'TOGETHER_API_KEY',
    xai: 'XAI_API_KEY',
    fireworks: 'FIREWORKS_API_KEY',
    cohere: 'COHERE_API_KEY',
    perplexity: 'PERPLEXITY_API_KEY',
    deepinfra: 'DEEPINFRA_API_KEY',
    azure_openai: 'AZURE_OPENAI_API_KEY',
  };

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const [providersData, configData] = await Promise.all([
        configRepository.listProviders(),
        configRepository.getConfig(),
      ]);

      setProviders(providersData);
      setConfig(configData);

      // Normal model
      setNormalProvider(configData.model_provider);
      setNormalModel(configData.model);

      // Thinking model
      setThinkingProvider(configData.model_thinking_provider || '');
      setThinkingModel(configData.model_thinking || '');

      // Vision model
      setVisionProvider(configData.model_vlm_provider || '');
      setVisionModel(configData.model_vlm || '');

      // Compact model
      setCompactProvider(configData.model_compact_provider || '');
      setCompactModel(configData.model_compact || '');

      // Other settings
      setTemperature(configData.temperature ?? 0.7);

      // Shadowed env vars (env vars that override keyring-stored secrets)
      setShadowedEnvVars(configData.shadowed_env_vars || []);

      // API config — keep empty initially so user explicitly sets it
      setApiKey('');
      setApiBaseUrl(configData.api_base_url || '');
      setApiKeyDirty(false);
      setApiBaseUrlDirty(false);
    } catch (error) {
      console.error('Failed to load settings:', error);
      toast.error('Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setSaving(true);

      const payload: Record<string, any> = {
        model_provider: normalProvider,
        model: normalModel,
        model_thinking_provider: thinkingProvider || null,
        model_thinking: thinkingModel || null,
        model_vlm_provider: visionProvider || null,
        model_vlm: visionModel || null,
        temperature,
      };

      if (apiKeyDirty) payload.api_key = apiKey;
      if (apiBaseUrlDirty) payload.api_base_url = apiBaseUrl;

      await configRepository.updateConfig(payload);

      // Dispatch custom event to notify other components
      window.dispatchEvent(new CustomEvent('config-updated', {
        detail: {
          model_provider: normalProvider,
          model: normalModel,
          temperature,
        }
      }));

      // Reset dirty flags after save
      setApiKeyDirty(false);
      setApiBaseUrlDirty(false);

      toast.success('Settings saved successfully');

      // If API key was saved, show keyring toast
      if (apiKeyDirty && apiKey) {
        toast.success('API key saved to system keyring', {
          description: 'For enhanced security, your API key is now stored in the OS keyring.',
        });
      }
    } catch (error) {
      console.error('Failed to save settings:', error);
      toast.error('Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  const handleVerify = async () => {
    const provider = normalProvider;
    const model = normalModel;
    if (!provider || !model) {
      toast.error('Select a provider and model first');
      return;
    }
    setVerifyState('verifying');
    setVerifyMessage('');
    try {
      const result = await configRepository.verifyModel(provider, model);
      if (result.valid) {
        setVerifyState('success');
        setVerifyMessage('Connection verified successfully');
      } else {
        setVerifyState('error');
        setVerifyMessage(result.error || 'Verification failed');
      }
    } catch (e: any) {
      setVerifyState('error');
      setVerifyMessage(e.message || 'Verification failed');
    }
  };

  if (loading) {
    return (
      <div className="space-y-6 animate-pulse">
        <div className="bg-accent-primary-muted rounded-lg p-4 h-24" />
        {[1, 2, 3, 4].map(i => (
          <div key={i} className="border border-border-default rounded-lg p-4 space-y-3">
            <div className="flex items-center gap-3">
              <div className="w-6 h-6 rounded bg-surface-2" />
              <div className="flex-1 space-y-1">
                <div className="h-4 w-32 bg-surface-2 rounded" />
                <div className="h-3 w-48 bg-surface-2 rounded" />
              </div>
            </div>
            <div className="h-9 bg-surface-2 rounded-lg" />
            <div className="h-9 bg-surface-2 rounded-lg" />
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header Info */}
      <div className="bg-accent-primary-muted border border-accent-primary-muted rounded-lg p-4">
        <div className="flex items-start gap-3">
          <svg className="w-5 h-5 text-accent-primary flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div className="flex-1">
            <h4 className="text-sm font-semibold text-accent-primary-fg mb-1">Model System</h4>
            <p className="text-xs text-accent-primary leading-relaxed">
              Configure different models for different tasks: <strong>Normal</strong> for standard coding,
              <strong> Thinking</strong> for complex reasoning, <strong>Compact</strong> for context compaction
              summaries, and <strong>Vision</strong> for image processing.
              Optional models fall back: Thinking → Normal, Compact → Normal, Vision → disabled.
            </p>
          </div>
        </div>
      </div>

      {/* Normal Model */}
      <ModelSlot
        title="Normal Model"
        description="For standard coding tasks and general-purpose operations"
        icon={<Cpu className="w-5 h-5 text-accent-primary" />}
        providers={providers}
        selectedProvider={normalProvider}
        selectedModel={normalModel}
        onProviderChange={setNormalProvider}
        onModelChange={setNormalModel}
      />

      {/* Thinking Model */}
      <ModelSlot
        title="Thinking Model"
        description="For complex reasoning and planning tasks (falls back to Normal if not set)"
        icon={<Brain className="w-5 h-5 text-accent-primary" />}
        providers={providers}
        selectedProvider={thinkingProvider}
        selectedModel={thinkingModel}
        onProviderChange={setThinkingProvider}
        onModelChange={setThinkingModel}
        optional
        notSetText="Use Normal Model"
      />

      {/* Compact Model */}
      <ModelSlot
        title="Compact Model"
        description="For context compaction summaries (falls back to Normal)"
        icon={<LayoutGrid className="w-5 h-5 text-accent-primary" />}
        providers={providers}
        selectedProvider={compactProvider}
        selectedModel={compactModel}
        onProviderChange={setCompactProvider}
        onModelChange={setCompactModel}
        optional
        notSetText="Use Normal Model"
      />

      {/* Vision Model */}
      <ModelSlot
        title="Vision Model"
        description="For image processing and multi-modal tasks (vision unavailable if not set)"
        icon={<Eye className="w-5 h-5 text-accent-primary" />}
        providers={providers}
        selectedProvider={visionProvider}
        selectedModel={visionModel}
        onProviderChange={setVisionProvider}
        onModelChange={setVisionModel}
        optional
        notSetText="Vision Disabled"
      />

      {/* API Configuration */}
      <div className="border-t border-border-default pt-6 space-y-4">
        <h3 className="text-sm font-semibold text-content-primary">API Configuration</h3>

        <div>
          <label className="block text-sm font-medium text-content-primary mb-2">
            API Key
            {config?.api_key && (
              <span className="text-content-tertiary font-normal ml-1">
                (currently configured — enter new value to replace)
              </span>
            )}
          </label>
          <div className="relative">
            <input
              type={showApiKey ? 'text' : 'password'}
              value={apiKey}
              onChange={(e) => { setApiKey(e.target.value); setApiKeyDirty(true); }}
              placeholder={config?.api_key ? 'Leave blank to keep current key' : 'Enter API key'}
              className="w-full px-3 py-2 text-sm border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent bg-surface-primary pr-10"
            />
            <button
              type="button"
              onClick={() => setShowApiKey(!showApiKey)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-content-tertiary hover:text-content-secondary transition-colors"
              tabIndex={-1}
            >
              {showApiKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </button>
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium text-content-primary mb-2">
            Base URL <span className="text-content-tertiary font-normal">(optional)</span>
          </label>
          <input
            type="text"
            value={apiBaseUrl}
            onChange={(e) => { setApiBaseUrl(e.target.value); setApiBaseUrlDirty(true); }}
            placeholder={config?.api_base_url || 'https://api.example.com/v1'}
            className="w-full px-3 py-2 text-sm border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent bg-surface-primary"
          />
        </div>

        <div className="flex items-center gap-3">
          <Button
            variant="secondary"
            size="sm"
            onClick={handleVerify}
            disabled={verifyState === 'verifying'}
            loading={verifyState === 'verifying'}
          >
            Test Connection
          </Button>

          {verifyState === 'success' && (
            <span className="flex items-center gap-1.5 text-xs text-intent-success">
              <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg>
              {verifyMessage}
            </span>
          )}
          {verifyState === 'error' && (
            <span className="flex items-center gap-1.5 text-xs text-intent-danger">
              <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
              {verifyMessage}
            </span>
          )}
        </div>
      </div>

      {/* Shadowed env var warning — shown if an env var overrides the keyring for the selected provider */}
      {(() => {
        const shadowedEnvVar = PROVIDER_ENV_VARS[normalProvider]
          ? shadowedEnvVars.find(v => v === PROVIDER_ENV_VARS[normalProvider])
          : undefined;
        return shadowedEnvVar ? (
          <div className="mt-3 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md">
            <div className="flex items-start gap-2">
              <span className="text-lg">🔒</span>
              <div>
                <p className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                  Environment variable overrides keyring
                </p>
                <p className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                  <code>{shadowedEnvVar}</code> is set in your environment.
                  This key was saved to the system keyring but the environment variable
                  takes precedence. To use the keyring value, unset the environment variable.
                </p>
                <p className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                  Run <code>opendev secret doctor</code> in your terminal for a full report.
                </p>
              </div>
            </div>
          </div>
        ) : null;
      })()}

      {/* Global Settings */}
      <div className="border-t border-border-default pt-6 space-y-4">
        <h3 className="text-sm font-semibold text-content-primary">Global Settings</h3>

        {/* Temperature */}
        <div>
          <label className="block text-sm font-medium text-content-primary mb-2">
            Temperature: {temperature.toFixed(2)}
          </label>
          <input
            type="range"
            min="0"
            max="2"
            step="0.1"
            value={temperature}
            onChange={(e) => setTemperature(parseFloat(e.target.value))}
            className="w-full h-2 bg-surface-3 rounded-lg appearance-none cursor-pointer"
          />
          <div className="flex justify-between text-xs text-content-tertiary mt-1">
            <span>Precise</span>
            <span>Balanced</span>
            <span>Creative</span>
          </div>
        </div>
      </div>

      {/* Save Button */}
      <div className="pt-4 border-t border-border-default">
        <Button
          variant="primary"
          onClick={handleSave}
          disabled={saving}
          loading={saving}
          fullWidth
        >
          {saving ? 'Saving...' : 'Save Changes'}
        </Button>
      </div>
    </div>
  );
}
