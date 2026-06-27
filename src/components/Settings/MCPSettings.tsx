/**
 * MCP Settings Component - Table View
 *
 * Displays MCP servers in a table format similar to terminal /mcp list
 * with real-time WebSocket updates
 */

import { useState, useEffect, useRef } from 'react';
import { AddMCPServerModal } from './AddMCPServerModal';
import { EditMCPServerModal } from './EditMCPServerModal';
import { MCPToolsModal } from './MCPToolsModal';
import { eventBridge } from '../../api/eventBridge';
import { mcpRepository } from '../../repositories';
import type { MCPServer, MCPServerCreateRequest, MCPServerUpdateRequest, MCPTool } from '../../types/mcp';
import type { WSMessage } from '../../types';
import { Button } from '../ui/Button';

export function MCPSettings() {
  // Server list state
  const [servers, setServers] = useState<MCPServer[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Modal states
  const [showAddModal, setShowAddModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showToolsModal, setShowToolsModal] = useState(false);
  const [selectedServer, setSelectedServer] = useState<MCPServer | null>(null);
  const [selectedServerTools, setSelectedServerTools] = useState<MCPTool[]>([]);

  // Action states
  const [processingServer, setProcessingServer] = useState<string | null>(null);

  // Load servers on mount
  useEffect(() => {
    console.log('[MCPSettings] Component mounted, loading servers...');
    loadServers();
  }, []);

  // WebSocket event listener for real-time updates
  useEffect(() => {
    const handleWSMessage = (message: WSMessage) => {
      if (message.type === 'mcp:status_changed' || message.type === 'mcp_status_update') {
        const { server_name, status } = message.data;
        console.log('[MCPSettings] Status update:', { server_name, status });
        setServers(prev => prev.map(server =>
          server.name === server_name ? { ...server, status } : server
        ));
      } else if (message.type === 'mcp:servers_updated' || message.type === 'mcp_servers_update') {
        console.log('[MCPSettings] Full update:', message.data);
        setServers(message.data.servers);
      }
    };

    const unsubs: (() => void)[] = [];

    (async () => {
      const u1 = await eventBridge.on('mcp:status_changed', handleWSMessage);
      unsubs.push(u1);
      const u2 = await eventBridge.on('mcp:servers_updated', handleWSMessage);
      unsubs.push(u2);
    })();

    return () => {
      unsubs.forEach(fn => fn());
    };
  }, []);

  const loadServers = async () => {
    setIsLoading(true);
    setError(null);
    try {
      console.log('[MCPSettings] Fetching from /api/mcp/servers...');
      const response = await mcpRepository.listServers();
      console.log('[MCPSettings] API Response:', response);
      console.log('[MCPSettings] Servers loaded:', response.servers?.length || 0, 'servers');
      setServers(response.servers || []);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to load servers';
      console.error('[MCPSettings] Load error:', errorMsg, err);
      setError(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  const handleConnect = async (name: string) => {
    setProcessingServer(name);
    try {
      await mcpRepository.connectServer(name);
      await loadServers(); // Reload to update UI
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to connect');
    } finally {
      setProcessingServer(null);
    }
  };

  const handleDisconnect = async (name: string) => {
    setProcessingServer(name);
    try {
      await dismcpRepository.connectServer(name);
      await loadServers(); // Reload to update UI
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to disconnect');
    } finally {
      setProcessingServer(null);
    }
  };

  const handleTest = async (name: string) => {
    setProcessingServer(name);
    try {
      const response = await mcpRepository.connectServer(name);
      alert(response.message || 'Connection test successful');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Test failed');
    } finally {
      setProcessingServer(null);
    }
  };

  const handleViewTools = async (name: string) => {
    try {
      const serverDetail = await mcpRepository.getServer(name);
      setSelectedServerTools(serverDetail.tools);
      setSelectedServer(servers.find(s => s.name === name) || null);
      setShowToolsModal(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tools');
    }
  };

  const handleEdit = (server: MCPServer) => {
    setSelectedServer(server);
    setShowEditModal(true);
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`Remove "${name}"? This action cannot be undone.`)) return;

    try {
      await mcpRepository.deleteServer(name);
      await loadServers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove server');
    }
  };

  const handleAddServer = async (server: MCPServerCreateRequest) => {
    try {
      await mcpRepository.createServer(server);
      await loadServers();
      setShowAddModal(false);
    } catch (err) {
      throw err;
    }
  };

  const handleUpdateServer = async (name: string, update: MCPServerUpdateRequest) => {
    try {
      await mcpRepository.updateServer(name, update);
      await loadServers();
      setShowEditModal(false);
      setSelectedServer(null);
    } catch (err) {
      throw err;
    }
  };

  // Debug render
  console.log('[MCPSettings] Rendering with:', { isLoading, serversCount: servers.length, error });

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-content-primary">MCP Servers</h3>
          <p className="text-sm text-content-tertiary mt-0.5">
            Manage Model Context Protocol server connections
          </p>
        </div>
        <Button variant="primary" onClick={() => setShowAddModal(true)}>
          Add Server
        </Button>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="flex items-center justify-between px-4 py-3 bg-intent-danger-muted border border-intent-danger-muted rounded-lg">
          <div className="flex items-center gap-3">
            <svg className="w-5 h-5 text-intent-danger" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p className="text-sm text-intent-danger-fg">{error}</p>
          </div>
          <button onClick={() => setError(null)} className="text-intent-danger hover:text-intent-danger-fg">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {/* Content */}
      {isLoading ? (
        <LoadingState />
      ) : servers.length === 0 ? (
        <EmptyState />
      ) : (
        <ServerTable
          servers={servers}
          processingServer={processingServer}
          onConnect={handleConnect}
          onDisconnect={handleDisconnect}
          onTest={handleTest}
          onViewTools={handleViewTools}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      )}

      {/* Footer Info */}
      <div className="pt-4 border-t border-border-default">
        <p className="text-xs text-content-tertiary">
          <strong>Note:</strong> Connected servers are available in both terminal and web interface.
          Changes take effect immediately.
        </p>
      </div>

      {/* Modals */}
      <AddMCPServerModal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onSubmit={handleAddServer}
      />

      <EditMCPServerModal
        isOpen={showEditModal}
        server={selectedServer}
        onClose={() => {
          setShowEditModal(false);
          setSelectedServer(null);
        }}
        onSubmit={handleUpdateServer}
      />

      <MCPToolsModal
        isOpen={showToolsModal}
        serverName={selectedServer?.name || ''}
        tools={selectedServerTools}
        onClose={() => {
          setShowToolsModal(false);
          setSelectedServer(null);
          setSelectedServerTools([]);
        }}
      />
    </div>
  );
}

// ============================================================================
// Sub-components
// ============================================================================

function LoadingState() {
  return (
    <div className="space-y-3 animate-pulse">
      {[1, 2, 3].map(i => (
        <div key={i} className="bg-surface-elevated rounded-lg border border-border-default p-4">
          <div className="flex items-center gap-4">
            <div className="h-4 w-32 bg-surface-2 rounded" />
            <div className="h-3 w-20 bg-surface-2 rounded-full" />
            <div className="ml-auto flex gap-2">
              <div className="h-6 w-16 bg-surface-2 rounded-md" />
              <div className="h-6 w-16 bg-surface-2 rounded-md" />
              <div className="h-6 w-16 bg-surface-2 rounded-md" />
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="text-center py-12 bg-surface-elevated rounded-lg border-2 border-dashed border-border-default">
      <svg className="w-12 h-12 mx-auto text-content-tertiary mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 12h14M12 5l7 7-7 7" />
      </svg>
      <p className="text-sm text-content-secondary font-medium mb-1">No MCP servers configured</p>
      <p className="text-xs text-content-tertiary">
        Click "Add Server" above to add your first MCP server
      </p>
    </div>
  );
}

interface ServerTableProps {
  servers: MCPServer[];
  processingServer: string | null;
  onConnect: (name: string) => void;
  onDisconnect: (name: string) => void;
  onTest: (name: string) => void;
  onViewTools: (name: string) => void;
  onEdit: (server: MCPServer) => void;
  onDelete: (name: string) => void;
}

function ServerTable({
  servers,
  processingServer,
  onConnect,
  onDisconnect,
  onTest,
  onViewTools,
  onEdit,
  onDelete,
}: ServerTableProps) {
  return (
    <div className="bg-surface-primary border border-border-default rounded-lg overflow-x-auto">
      <table className="w-full divide-y divide-border-subtle">
        <colgroup>
          <col style={{ width: '40%' }} /> {/* Name */}
          <col style={{ width: '15%' }} /> {/* Status */}
          <col style={{ width: '15%' }} /> {/* Enabled */}
          <col style={{ width: '15%' }} /> {/* Auto-start */}
          <col style={{ width: '15%' }} /> {/* Actions */}
        </colgroup>
        <thead className="bg-surface-elevated">
          <tr>
            <th className="px-4 py-3 text-left text-sm font-semibold text-content-secondary uppercase whitespace-nowrap">
              Name
            </th>
            <th className="px-4 py-3 text-center text-sm font-semibold text-content-secondary uppercase whitespace-nowrap">
              Status
            </th>
            <th className="px-4 py-3 text-center text-sm font-semibold text-content-secondary uppercase whitespace-nowrap">
              Enabled
            </th>
            <th className="px-4 py-3 text-center text-sm font-semibold text-content-secondary uppercase whitespace-nowrap">
              Auto-start
            </th>
            <th className="px-4 py-3 text-center text-sm font-semibold text-content-secondary uppercase whitespace-nowrap">
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="bg-surface-primary">
          {servers.map((server) => (
            <ServerRow
              key={server.name}
              server={server}
              isProcessing={processingServer === server.name}
              onConnect={onConnect}
              onDisconnect={onDisconnect}
              onTest={onTest}
              onViewTools={onViewTools}
              onEdit={onEdit}
              onDelete={onDelete}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}

interface ServerRowProps {
  server: MCPServer;
  isProcessing: boolean;
  onConnect: (name: string) => void;
  onDisconnect: (name: string) => void;
  onTest: (name: string) => void;
  onViewTools: (name: string) => void;
  onEdit: (server: MCPServer) => void;
  onDelete: (name: string) => void;
}

function ServerRow({
  server,
  isProcessing,
  onConnect,
  onDisconnect,
  onTest,
  onViewTools,
  onEdit,
  onDelete,
}: ServerRowProps) {
  const isConnected = server.status === 'connected';

  const handleTest = () => {
    onTest(server.name);
  };

  return (
    <tr className="hover:bg-surface-elevated transition-colors">
      {/* Name + Action Buttons */}
      <td className="px-4 py-3">
        <div className="flex items-center gap-3">
          <div className="flex-1 min-w-0">
            <div className="text-sm font-medium text-content-primary truncate">{server.name}</div>
            <div className="text-xs text-content-tertiary">{server.config_location}</div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              size="xs"
              onClick={() => onConnect(server.name)}
              disabled={isProcessing || isConnected}
            >
              Connect
            </Button>
            <Button
              variant="secondary"
              size="xs"
              onClick={() => onDisconnect(server.name)}
              disabled={isProcessing || !isConnected}
            >
              Disconnect
            </Button>
            <Button
              variant="secondary"
              size="xs"
              onClick={handleTest}
              disabled={isProcessing}
            >
              Test
            </Button>

            {/* Tools Button */}
            <button
              onClick={() => onViewTools(server.name)}
              disabled={isProcessing || !isConnected}
              className="px-3 py-1.5 text-sm font-medium text-content-secondary bg-surface-primary border border-border-emphasis hover:bg-surface-elevated rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap"
            >
              Tools
            </button>
          </div>
        </div>
      </td>

      {/* Status */}
      <td className="px-4 py-3 text-center whitespace-nowrap">
        {isProcessing ? (
          <div className="w-4 h-4 border-2 border-border-emphasis border-t-surface-primary rounded-full animate-spin mx-auto" />
        ) : isConnected ? (
          <div className="flex items-center justify-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-intent-success" />
            <span className="text-sm font-medium text-intent-success">On</span>
          </div>
        ) : (
          <div className="flex items-center justify-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-surface-3" />
            <span className="text-sm text-content-tertiary">Off</span>
          </div>
        )}
      </td>

      {/* Enabled */}
      <td className="px-4 py-3 text-center whitespace-nowrap">
        {server.config.enabled ? (
          <svg className="w-5 h-5 text-intent-success mx-auto" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
          </svg>
        ) : (
          <span className="text-content-tertiary">-</span>
        )}
      </td>

      {/* Auto-start */}
      <td className="px-4 py-3 text-center whitespace-nowrap">
        {server.config.auto_start ? (
          <svg className="w-5 h-5 text-intent-success mx-auto" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
          </svg>
        ) : (
          <span className="text-content-tertiary">-</span>
        )}
      </td>

      {/* Actions - Dropdown only */}
      <td className="px-4 py-3 text-center whitespace-nowrap">
        <div className="flex items-center justify-center">
          <DropdownMenu
            server={server}
            isProcessing={isProcessing}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        </div>
      </td>
    </tr>
  );
}

// ============================================================================
// Dropdown Menu Component
// ============================================================================

interface DropdownMenuProps {
  server: MCPServer;
  isProcessing: boolean;
  onEdit: (server: MCPServer) => void;
  onDelete: (name: string) => void;
}

function DropdownMenu({ server, isProcessing, onEdit, onDelete }: DropdownMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isOpen]);

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Dropdown Trigger Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={isProcessing}
        className="p-2 text-content-tertiary hover:text-content-secondary hover:bg-surface-2 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        title="More actions"
      >
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
        </svg>
      </button>

      {/* Dropdown Menu */}
      {isOpen && (
        <div className="absolute right-0 mt-1 w-32 bg-surface-primary border border-border-default rounded-lg shadow-lg z-10 overflow-hidden">
          <button
            onClick={() => {
              onEdit(server);
              setIsOpen(false);
            }}
            className="w-full px-3 py-2 text-left text-sm text-content-secondary hover:bg-surface-elevated transition-colors flex items-center gap-2"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
            Edit
          </button>
          <button
            onClick={() => {
              onDelete(server.name);
              setIsOpen(false);
            }}
            className="w-full px-3 py-2 text-left text-sm text-intent-danger hover:bg-intent-danger-muted transition-colors flex items-center gap-2"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Remove
          </button>
        </div>
      )}
    </div>
  );
}
