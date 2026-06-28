import React, { useState } from 'react';
import { Button } from '../ui/Button';

interface PrivacySettingsProps {
  config: {
    shadowed_env_vars?: string[];
  };
}

export function PrivacySettings({ config }: PrivacySettingsProps) {
  const [sentryEnabled, setSentryEnabled] = useState(false);
  const [metricsEnabled, setMetricsEnabled] = useState(false);
  const [recordFullPayload, setRecordFullPayload] = useState(false);
  const [recordToolArgs, setRecordToolArgs] = useState(false);

  const openLogsDir = () => {
    // Instruct user how to find logs
    const logPath = process.platform === 'darwin'
      ? '~/Library/Application Support/com.opendev-opendev/logs/'
      : '~/.opendev/logs/';
    console.log(`[Frontend] View logs at: ${logPath}`);
    // In Tauri, we could use shell.open
    alert(`View local logs at:\n${logPath}`);
  };

  const clearAllData = () => {
    if (window.confirm('Are you sure? This will clear all local data including settings, sessions, and logs. This cannot be undone.')) {
      alert('Clear all data: restart the application to complete.');
    }
  };

  return (
    <div className="space-y-6" data-testid="privacy-settings">
      <h2 className="text-lg font-semibold text-content-primary">Privacy & Data</h2>
      <p className="text-sm text-content-secondary">
        OpenDev respects your privacy. No data is collected automatically.
        All options below are opt-in.
      </p>

      <div className="space-y-4">
        {/* Sentry toggle */}
        <div className="flex items-center justify-between p-3 bg-surface-secondary rounded-md">
          <div>
            <p className="text-sm font-medium text-content-primary">Send anonymous error reports</p>
            <p className="text-xs text-content-secondary mt-1">
              Helps improve OpenDev. No personal data is collected.
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              className="sr-only peer"
              checked={sentryEnabled}
              onChange={(e) => setSentryEnabled(e.target.checked)}
            />
            <div className="w-9 h-5 bg-gray-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
          </label>
        </div>

        {/* Metrics toggle */}
        <div className="flex items-center justify-between p-3 bg-surface-secondary rounded-md">
          <div>
            <p className="text-sm font-medium text-content-primary">Allow performance metrics</p>
            <p className="text-xs text-content-secondary mt-1">
              Anonymized usage data to improve performance.
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              className="sr-only peer"
              checked={metricsEnabled}
              onChange={(e) => setMetricsEnabled(e.target.checked)}
            />
            <div className="w-9 h-5 bg-gray-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
          </label>
        </div>

        {/* Full payload toggle */}
        <div className="flex items-center justify-between p-3 bg-surface-secondary rounded-md">
          <div>
            <p className="text-sm font-medium text-content-primary">Record full LLM payloads</p>
            <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
              ⚠️ May include your prompts in plain text in debug logs
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              className="sr-only peer"
              checked={recordFullPayload}
              onChange={(e) => setRecordFullPayload(e.target.checked)}
            />
            <div className="w-9 h-5 bg-gray-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
          </label>
        </div>

        {/* Tool args toggle */}
        <div className="flex items-center justify-between p-3 bg-surface-secondary rounded-md">
          <div>
            <p className="text-sm font-medium text-content-primary">Record tool arguments</p>
            <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
              ⚠️ May include file contents and credentials in debug logs
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              className="sr-only peer"
              checked={recordToolArgs}
              onChange={(e) => setRecordToolArgs(e.target.checked)}
            />
            <div className="w-9 h-5 bg-gray-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
          </label>
        </div>
      </div>

      <div className="flex gap-3 pt-4 border-t border-border-primary">
        <Button variant="secondary" onClick={openLogsDir}>
          View local logs
        </Button>
        <Button variant="destructive" onClick={clearAllData}>
          Clear all local data
        </Button>
      </div>

      {config.shadowed_env_vars && config.shadowed_env_vars.length > 0 && (
        <div className="p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md">
          <p className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
            🔒 Environment variables override keyring secrets
          </p>
          <ul className="mt-1 text-xs text-yellow-600 dark:text-yellow-400 list-disc list-inside">
            {config.shadowed_env_vars.map(ev => (
              <li key={ev}><code>{ev}</code> is set in environment</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
