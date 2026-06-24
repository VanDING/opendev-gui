/**
 * MCP Tools Browser Modal - Master-Detail Layout
 *
 * Elegant modal for browsing MCP server tools with detailed parameter information.
 * Uses a master-detail pattern for optimal information architecture.
 */

import { useState, useMemo, useEffect, useDeferredValue } from 'react';
import { Search, Clipboard, Check, Wrench } from 'lucide-react';
import type { MCPTool } from '../../types/mcp';
import { Modal } from '../ui/Modal';

interface MCPToolsModalProps {
  isOpen: boolean;
  serverName: string;
  tools: MCPTool[];
  onClose: () => void;
}

export function MCPToolsModal({ isOpen, serverName, tools, onClose }: MCPToolsModalProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTool, setSelectedTool] = useState<MCPTool | null>(null);
  const [copiedText, setCopiedText] = useState<string | null>(null);

  // ⚡ Bolt Performance Optimization:
  // Use useDeferredValue instead of useDebounce for local array filtering.
  // This prevents artificial UI lag (waiting 300ms) and allows React to prioritize
  // typing updates while rendering the filtered list in the background.
  const deferredSearchQuery = useDeferredValue(searchQuery);

  // Reset state when modal opens or tools change
  useEffect(() => {
    if (isOpen) {
      setSearchQuery('');
      setSelectedTool(tools[0] || null);
      setCopiedText(null);
    }
  }, [isOpen, tools]);

  // Filter tools based on search query
  const filteredTools = useMemo(() => {
    if (!deferredSearchQuery.trim()) return tools;

    // ⚡ Bolt Performance Optimization:
    // Precompute a case-insensitive RegExp instead of repeatedly invoking .toLowerCase()
    // inside the filter loop. This prevents O(N) redundant string allocations.
    const queryRegex = new RegExp(deferredSearchQuery.trim().replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
    return tools.filter(
      tool =>
        queryRegex.test(tool.name) ||
        queryRegex.test(tool.description)
    );
  }, [tools, deferredSearchQuery]);

  // Auto-select first tool when filtered list changes
  useEffect(() => {
    if (filteredTools.length > 0 && !filteredTools.find(t => t.name === selectedTool?.name)) {
      setSelectedTool(filteredTools[0]);
    } else if (filteredTools.length === 0) {
      setSelectedTool(null);
    }
  }, [filteredTools, selectedTool]);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedText(text);
    setTimeout(() => setCopiedText(null), 2000);
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} className="max-w-6xl h-[85vh]">
      <div className="flex flex-col h-full overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-gradient-to-r from-surface-elevated to-surface-primary">
          <div>
            <h2 className="text-xl font-semibold text-content-primary">
              Tools from {serverName}
            </h2>
            <p className="text-sm text-content-tertiary mt-0.5">
              {filteredTools.length} {filteredTools.length === 1 ? 'tool' : 'tools'} available
            </p>
          </div>
        </div>

        {/* Search Bar */}
        <div className="px-6 py-3 border-b border-border-subtle bg-surface-elevated">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-content-tertiary" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search tools by name or description..."
              className="w-full pl-10 pr-4 py-2.5 border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent bg-surface-primary"
            />
          </div>
        </div>

        {/* Master-Detail Layout */}
        <div className="flex-1 flex overflow-hidden">
          {/* Master: Tools List (Left Sidebar) */}
          <div className="w-80 border-r border-border-default bg-surface-elevated overflow-y-auto">
            {filteredTools.length === 0 ? (
              <EmptyState searchQuery={searchQuery} />
            ) : (
              <div className="p-2">
                {filteredTools.map((tool) => (
                  <ToolListItem
                    key={tool.name}
                    tool={tool}
                    isSelected={selectedTool?.name === tool.name}
                    onClick={() => setSelectedTool(tool)}
                  />
                ))}
              </div>
            )}
          </div>

          {/* Detail: Tool Details (Right Panel) */}
          <div className="flex-1 overflow-y-auto bg-surface-primary">
            {selectedTool ? (
              <ToolDetails
                tool={selectedTool}
                serverName={serverName}
                copiedText={copiedText}
                onCopy={copyToClipboard}
              />
            ) : (
              <div className="flex items-center justify-center h-full text-content-tertiary">
                <div className="text-center">
                  <Wrench className="w-16 h-16 mx-auto mb-3 opacity-20" />
                  <p className="text-sm">Select a tool to view details</p>
                </div>
              </div>
            )}
          </div>
      </div>
      </div>
    </Modal>
  );
}

// ============================================================================
// Sub-components
// ============================================================================

interface EmptyStateProps {
  searchQuery: string;
}

function EmptyState({ searchQuery }: EmptyStateProps) {
  return (
    <div className="text-center py-12 px-4">
      <div className="text-content-tertiary mb-2">
        <svg className="w-12 h-12 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
      </div>
      <p className="text-sm text-content-secondary font-medium mb-1">
        {searchQuery ? 'No tools found' : 'No tools available'}
      </p>
      {searchQuery && (
        <p className="text-xs text-content-tertiary">
          Try a different search term
        </p>
      )}
    </div>
  );
}

interface ToolListItemProps {
  tool: MCPTool;
  isSelected: boolean;
  onClick: () => void;
}

function ToolListItem({ tool, isSelected, onClick }: ToolListItemProps) {
  const paramCount = tool.inputSchema?.properties ? Object.keys(tool.inputSchema.properties).length : 0;

  return (
    <button
      onClick={onClick}
      className={`w-full text-left p-3 rounded-lg transition-all ${
        isSelected
          ? 'bg-surface-primary shadow-md border border-border-default'
          : 'hover:bg-surface-primary/50 border border-transparent'
      }`}
    >
      <div className="flex items-start gap-2">
        <Wrench className={`w-4 h-4 mt-0.5 flex-shrink-0 ${
          isSelected ? 'text-content-primary' : 'text-content-tertiary'
        }`} />
        <div className="flex-1 min-w-0">
          <h4 className={`text-sm font-medium truncate ${
            isSelected ? 'text-content-primary' : 'text-content-secondary'
          }`}>
            {tool.name}
          </h4>
          <p className="text-xs text-content-tertiary mt-0.5 line-clamp-2">
            {tool.description}
          </p>
          {paramCount > 0 && (
            <div className="mt-1.5">
              <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-surface-2 text-content-secondary">
                {paramCount} {paramCount === 1 ? 'parameter' : 'parameters'}
              </span>
            </div>
          )}
        </div>
      </div>
    </button>
  );
}

interface ToolDetailsProps {
  tool: MCPTool;
  serverName: string;
  copiedText: string | null;
  onCopy: (text: string) => void;
}

function ToolDetails({ tool, serverName, copiedText, onCopy }: ToolDetailsProps) {
  const fullName = `mcp__${serverName}__${tool.name}`;
  const properties = tool.inputSchema?.properties || {};
  const required = tool.inputSchema?.required || [];
  const hasParameters = Object.keys(properties).length > 0;

  return (
    <div className="p-6">
      {/* Tool Header */}
      <div className="mb-6">
        <div className="flex items-start gap-3 mb-3">
          <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-surface-2 to-surface-primary flex items-center justify-center shadow-md flex-shrink-0">
            <Wrench className="w-5 h-5 text-content-inverse" />
          </div>
          <div className="flex-1">
            <h3 className="text-xl font-semibold text-content-primary">{tool.name}</h3>
            <p className="text-sm text-content-secondary mt-1 leading-relaxed">{tool.description}</p>
          </div>
        </div>

        {/* Full Tool Name */}
        <div className="mt-4">
          <label className="block text-xs font-medium text-content-tertiary mb-1.5">Full Tool Name</label>
          <div className="flex items-center gap-2">
            <code className="flex-1 px-3 py-2.5 bg-surface-elevated border border-border-default rounded-lg text-sm font-mono text-content-primary">
              {fullName}
            </code>
            <button
              onClick={() => onCopy(fullName)}
              className="p-2.5 text-content-secondary hover:text-content-primary hover:bg-surface-2 rounded-lg transition-colors border border-border-default"
              title="Copy to clipboard"
            >
              {copiedText === fullName ? (
                <Check className="w-4 h-4 text-intent-success" />
              ) : (
                <Clipboard className="w-4 h-4" />
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Parameters Section */}
      <div className="border-t border-border-default pt-6">
        <h4 className="text-sm font-semibold text-content-primary mb-4">Parameters</h4>

        {!hasParameters ? (
          <div className="text-center py-8 bg-surface-elevated rounded-lg border border-border-default">
            <p className="text-sm text-content-tertiary">This tool doesn't require any parameters</p>
          </div>
        ) : (
          <div className="space-y-4">
            {Object.entries(properties).map(([paramName, paramSchema]) => (
              <ParameterCard
                key={paramName}
                name={paramName}
                schema={paramSchema}
                isRequired={required.includes(paramName)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

interface ParameterCardProps {
  name: string;
  schema: any;
  isRequired: boolean;
}

function ParameterCard({ name, schema, isRequired }: ParameterCardProps) {
  const getTypeDisplay = (schema: any): string => {
    if (schema.enum) {
      return `enum: ${schema.enum.join(' | ')}`;
    }
    if (schema.type === 'array') {
      const itemType = schema.items?.type || 'any';
      return `array<${itemType}>`;
    }
    return schema.type || 'any';
  };

  return (
    <div className="bg-surface-primary border border-border-default rounded-lg p-4 hover:border-border-emphasis transition-colors">
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2">
          <code className="text-sm font-semibold text-content-primary">{name}</code>
          {isRequired && (
            <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-intent-danger-muted text-intent-danger">
              Required
            </span>
          )}
        </div>
        <span className="text-xs font-mono text-content-tertiary bg-surface-2 px-2 py-1 rounded">
          {getTypeDisplay(schema)}
        </span>
      </div>

      {schema.description && (
        <p className="text-sm text-content-secondary leading-relaxed">{schema.description}</p>
      )}

      {schema.enum && (
        <div className="mt-2 pt-2 border-t border-border-subtle">
          <p className="text-xs text-content-tertiary mb-1">Allowed values:</p>
          <div className="flex flex-wrap gap-1">
            {schema.enum.map((value: string) => (
              <code key={value} className="text-xs bg-surface-2 text-content-secondary px-2 py-0.5 rounded">
                {value}
              </code>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
