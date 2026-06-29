import { useState } from 'react';

/**
 * Sandbox settings panel — configure sandbox backends, network/filesystem policies.
 */
export function SandboxSettings() {
  const [sandboxEnabled, setSandboxEnabled] = useState(true);
  const [backend, setBackend] = useState('auto');
  const [networkAllowed, setNetworkAllowed] = useState(true);
  const [writeAllowed, setWriteAllowed] = useState(true);
  const [allowedDomains, setAllowedDomains] = useState('');

  return (
    <div className="space-y-5">
      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Sandbox Backend</h3>
        <p className="text-xs text-content-tertiary mb-2">
          Choose the isolation mechanism for executing shell commands.
        </p>
        <div className="flex items-center gap-3 mb-3">
          <label className="flex items-center gap-2 text-sm text-content-secondary cursor-pointer">
            <input
              type="checkbox"
              checked={sandboxEnabled}
              onChange={e => setSandboxEnabled(e.target.checked)}
              className="rounded border-border-default/30"
            />
            Enable sandbox isolation
          </label>
        </div>
        {sandboxEnabled && (
          <select
            value={backend}
            onChange={e => setBackend(e.target.value)}
            className="w-full px-3 py-1.5 text-sm bg-surface-primary border border-border-default/20 rounded-lg text-content-secondary"
          >
            <option value="auto">Auto-detect (recommended)</option>
            <option value="seatbelt">Seatbelt (macOS)</option>
            <option value="bwrap">Bubblewrap (Linux)</option>
            <option value="landlock">Landlock (Linux)</option>
            <option value="none">No sandbox</option>
          </select>
        )}
      </div>

      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Network Policy</h3>
        <div className="space-y-2">
          <label className="flex items-center gap-2 text-sm text-content-secondary cursor-pointer">
            <input
              type="checkbox"
              checked={networkAllowed}
              onChange={e => setNetworkAllowed(e.target.checked)}
              className="rounded border-border-default/30"
            />
            Allow outbound network access
          </label>
          {networkAllowed && (
            <div>
              <label className="text-xs text-content-tertiary">Allowed domains (one per line, * for all):</label>
              <textarea
                value={allowedDomains}
                onChange={e => setAllowedDomains(e.target.value)}
                placeholder="api.example.com&#10;*.github.com"
                className="w-full mt-1 px-3 py-1.5 text-sm bg-surface-primary border border-border-default/20 rounded-lg text-content-primary placeholder-content-tertiary resize-none"
                rows={3}
              />
            </div>
          )}
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Filesystem Policy</h3>
        <label className="flex items-center gap-2 text-sm text-content-secondary cursor-pointer">
          <input
            type="checkbox"
            checked={writeAllowed}
            onChange={e => setWriteAllowed(e.target.checked)}
            className="rounded border-border-default/30"
          />
          Allow writes to workspace directory
        </label>
        <p className="text-xs text-content-tertiary mt-1">
          When disabled, all file writes are denied. Reads are always allowed within the workspace.
        </p>
      </div>
    </div>
  );
}
