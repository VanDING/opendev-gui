import { useState, useEffect } from 'react';
import { Modal } from '../ui/Modal';

/**
 * Information about an MCP server discovered in the project's .mcp.json.
 */
export interface McpServerInfo {
  /** Unique server identifier/name. */
  id: string;
  /** Human-readable server name. */
  name: string;
  /** The command used to start the server (e.g. "npx", "python"). */
  command: string;
  /** Arguments passed to the command. */
  args: string[];
  /** Environment variables (secrets should be masked with ****). */
  env?: Record<string, string>;
  /** Optional description provided by the server. */
  description?: string;
}

interface McpApprovalDialogProps {
  /** List of discovered MCP servers. */
  servers: McpServerInfo[];
  /** Called when the user approves one or more servers. */
  onApprove: (selectedIds: string[]) => void;
  /** Called when the user denies all servers. */
  onDeny: () => void;
  /** Whether the dialog is open. */
  isOpen: boolean;
}

/**
 * Dialog shown on session start when project `.mcp.json` MCP servers are found.
 *
 * Allows users to review each server's configuration (with secrets masked),
 * and choose to enable all, enable selected, or disable all servers.
 */
export function McpApprovalDialog({
  servers,
  onApprove,
  onDeny,
  isOpen,
}: McpApprovalDialogProps) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() =>
    new Set(servers.map(s => s.id))
  );

  // Reset selection when servers change.
  useEffect(() => {
    setSelectedIds(new Set(servers.map(s => s.id)));
  }, [servers]);

  const toggleServer = (id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const handleEnableAll = () => {
    onApprove(servers.map(s => s.id));
  };

  const handleEnableSelected = () => {
    onApprove(Array.from(selectedIds));
  };

  const handleDenyAll = () => {
    onDeny();
  };

  /** Mask secret-ish env values (anything containing key, secret, token, pass). */
  const maskSecretValue = (key: string, value: string): string => {
    const keyLower = key.toLowerCase();
    if (
      keyLower.includes('secret') ||
      keyLower.includes('token') ||
      keyLower.includes('key') ||
      keyLower.includes('password') ||
      keyLower.includes('pass') ||
      keyLower.includes('credential') ||
      keyLower.includes('auth')
    ) {
      return '****';
    }
    return value;
  };

  const allSelected = selectedIds.size === servers.length;

  return (
    <Modal isOpen={isOpen} onClose={handleDenyAll} title="MCP Servers Require Approval" size="xl">
      <div className="px-6 py-4">
        <p className="text-sm text-content-secondary mb-4">
          The project contains MCP server configuration that will be started with this session.
          Review each server below and choose which to enable.
        </p>

        {/* Server list */}
        <div className="space-y-3 max-h-[50vh] overflow-y-auto">
          {servers.map(server => (
            <div
              key={server.id}
              className={`border rounded-lg p-4 transition-colors cursor-pointer ${
                selectedIds.has(server.id)
                  ? 'border-accent-secondary/50 bg-accent-secondary-muted/10'
                  : 'border-border-default/15 hover:border-accent-secondary/30'
              }`}
              onClick={() => toggleServer(server.id)}
            >
              <div className="flex items-start gap-3">
                {/* Checkbox */}
                <div
                  className={`mt-0.5 w-5 h-5 rounded border-2 flex items-center justify-center flex-shrink-0 transition-colors ${
                    selectedIds.has(server.id)
                      ? 'border-accent-secondary bg-accent-secondary'
                      : 'border-border-default/30'
                  }`}
                >
                  {selectedIds.has(server.id) && (
                    <svg className="w-3.5 h-3.5 text-content-inverse" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                    </svg>
                  )}
                </div>

                {/* Server info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-semibold text-content-primary">
                      {server.name}
                    </span>
                    {server.description && (
                      <span className="text-xs text-content-tertiary truncate">
                        — {server.description}
                      </span>
                    )}
                  </div>

                  {/* Command display */}
                  <div className="flex items-center gap-1.5 text-xs font-mono text-content-secondary bg-surface-2 rounded px-2 py-1 mb-2 overflow-x-auto">
                    <span className="text-content-tertiary">$</span>
                    <span>{server.command}</span>
                    {server.args.map((arg, i) => (
                      <span key={i} className="text-content-tertiary/80">{arg}</span>
                    ))}
                  </div>

                  {/* Environment variables */}
                  {server.env && Object.keys(server.env).length > 0 && (
                    <div className="space-y-0.5">
                      <span className="text-xs text-content-tertiary uppercase tracking-wider font-semibold">
                        Environment
                      </span>
                      {Object.entries(server.env).map(([key, value]) => (
                        <div key={key} className="flex items-center gap-2 text-xs font-mono">
                          <span className="text-content-secondary">{key}</span>
                          <span className="text-content-tertiary">=</span>
                          <span className="text-content-tertiary">
                            {maskSecretValue(key, value)}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Action buttons */}
      <div className="border-t border-border-default/15 px-6 py-4">
        <div className="flex items-center gap-3">
          <button
            onClick={handleEnableAll}
            className="flex-1 px-4 py-2.5 text-sm font-medium text-content-inverse bg-accent-secondary rounded-lg hover:bg-accent-secondary/90 transition-colors"
          >
            Enable All
          </button>
          <button
            onClick={handleEnableSelected}
            disabled={selectedIds.size === 0}
            className={`flex-1 px-4 py-2.5 text-sm font-medium rounded-lg border-2 transition-colors ${
              selectedIds.size > 0
                ? 'text-content-primary border-accent-secondary/50 hover:bg-accent-secondary-muted/30'
                : 'text-content-tertiary border-border-default/15 cursor-not-allowed'
            }`}
          >
            Enable Selected ({selectedIds.size})
          </button>
          <button
            onClick={handleDenyAll}
            className="flex-1 px-4 py-2.5 text-sm font-medium text-content-secondary border border-border-default/30 rounded-lg hover:bg-surface-elevated transition-colors"
          >
            Disable All
          </button>
        </div>

        {/* Status summary */}
        <div className="mt-3 text-xs text-content-tertiary text-center">
          {allSelected
            ? `All ${servers.length} server${servers.length !== 1 ? 's' : ''} will be enabled`
            : selectedIds.size === 0
              ? 'No servers will be enabled'
              : `${selectedIds.size} of ${servers.length} server${servers.length !== 1 ? 's' : ''} selected`}
        </div>
      </div>
    </Modal>
  );
}
