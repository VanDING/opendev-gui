import { useEffect, useState } from 'react';
import { useChatStore } from '../../stores/chat';
import { mcpRepository } from '../../repositories';
import { Modal } from '../ui/Modal';

interface StatusDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

interface MCPServer {
  name: string;
  status: string;
  config: { enabled: boolean };
  tools_count: number;
}

export function StatusDialog({ isOpen, onClose }: StatusDialogProps) {
  const status = useChatStore(state => state.status);
  const currentSessionId = useChatStore(state => state.currentSessionId);
  const sessionMessages = useChatStore(state => {
    const sid = state.currentSessionId;
    return sid ? state.sessionStates[sid]?.messages?.length ?? 0 : 0;
  });

  const [mcpServers, setMcpServers] = useState<MCPServer[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setLoading(true);
    mcpRepository.listServers()
      .then(data => setMcpServers(data?.servers || []))
      .catch(() => setMcpServers([]))
      .finally(() => setLoading(false));
  }, [isOpen]);

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Session Status" size="md">
      <div className="px-4 py-3 space-y-4 max-h-[60vh] overflow-y-auto">
          {/* Model Info */}
          <section>
            <h3 className="text-xs font-semibold uppercase text-content-tertiary mb-2">Model</h3>
            <div className="text-sm text-content-secondary space-y-1">
              <div className="flex justify-between">
                <span className="text-content-tertiary">Provider</span>
                <span className="font-mono">{status?.model_provider || '—'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-content-tertiary">Model</span>
                <span className="font-mono">{status?.model || '—'}</span>
              </div>
            </div>
          </section>

          {/* Session Info */}
          <section>
            <h3 className="text-xs font-semibold uppercase text-content-tertiary mb-2">Session</h3>
            <div className="text-sm text-content-secondary space-y-1">
              <div className="flex justify-between">
                <span className="text-content-tertiary">ID</span>
                <span className="font-mono">{currentSessionId || '—'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-content-tertiary">Messages</span>
                <span>{sessionMessages}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-content-tertiary">Mode</span>
                <span className="capitalize">{status?.mode || '—'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-content-tertiary">Autonomy</span>
                <span>{status?.autonomy_level || '—'}</span>
              </div>
              {status?.session_cost != null && (
                <div className="flex justify-between">
                  <span className="text-content-tertiary">Cost</span>
                  <span className="font-mono">
                    {status.session_cost < 0.01
                      ? `$${status.session_cost.toFixed(4)}`
                      : `$${status.session_cost.toFixed(2)}`}
                  </span>
                </div>
              )}
              {status?.context_usage_pct != null && (
                <div className="flex justify-between">
                  <span className="text-content-tertiary">Context Usage</span>
                  <span>{Math.round(status.context_usage_pct)}%</span>
                </div>
              )}
            </div>
          </section>

          {/* MCP Servers */}
          <section>
            <h3 className="text-xs font-semibold uppercase text-content-tertiary mb-2">MCP Servers</h3>
            {loading ? (
              <div className="text-sm text-content-tertiary">Loading...</div>
            ) : mcpServers.length === 0 ? (
              <div className="text-sm text-content-tertiary">No MCP servers configured</div>
            ) : (
              <div className="space-y-1.5">
                {mcpServers.map(server => (
                  <div key={server.name} className="flex items-center gap-2 text-sm">
                    <span className={`w-2 h-2 rounded-full flex-shrink-0 ${
                      server.status === 'connected' ? 'bg-intent-success-muted' : 'bg-surface-muted'
                    }`} />
                    <span className="text-content-secondary font-mono">{server.name}</span>
                    <span className="text-content-tertiary text-xs">
                      {server.status === 'connected' ? `connected (${server.tools_count} tools)` : 'disconnected'}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </section>

          {/* Working Directory */}
          {status?.working_dir && (
            <section>
              <h3 className="text-xs font-semibold uppercase text-content-tertiary mb-2">Working Directory</h3>
              <div className="text-sm text-content-secondary font-mono break-all">{status.working_dir}</div>
              {status.git_branch && (
                <div className="text-sm text-content-tertiary mt-1">Branch: {status.git_branch}</div>
              )}
            </section>
          )}
      </div>
    </Modal>
  );
}
