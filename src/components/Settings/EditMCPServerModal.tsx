/**
 * Edit MCP Server Modal
 *
 * Modal for editing existing MCP server configurations.
 * Follows DRY by reusing form components from AddMCPServerModal.
 */

import { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import type { MCPServer, MCPServerUpdateRequest } from '../../types/mcp';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';

interface EditMCPServerModalProps {
  isOpen: boolean;
  server: MCPServer | null;
  onClose: () => void;
  onSubmit: (name: string, update: MCPServerUpdateRequest) => Promise<void>;
}

interface FormData {
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
  auto_start: boolean;
}

export function EditMCPServerModal({ isOpen, server, onClose, onSubmit }: EditMCPServerModalProps) {
  const [formData, setFormData] = useState<FormData | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Args management
  const [argInput, setArgInput] = useState('');

  // Env management
  const [envKey, setEnvKey] = useState('');
  const [envValue, setEnvValue] = useState('');

  // Initialize form data when server changes
  useEffect(() => {
    if (server) {
      setFormData({
        command: server.config.command,
        args: [...server.config.args],
        env: { ...server.config.env },
        enabled: server.config.enabled,
        auto_start: server.config.auto_start,
      });
    }
  }, [server]);

  if (!isOpen || !server || !formData) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!formData.command.trim()) {
      setError('Command is required');
      return;
    }

    setIsSubmitting(true);
    try {
      await onSubmit(server.name, {
        command: formData.command.trim(),
        args: formData.args.filter(arg => arg.trim()),
        env: formData.env,
        enabled: formData.enabled,
        auto_start: formData.auto_start,
      });

      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update server');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleClose = () => {
    if (!isSubmitting) {
      setArgInput('');
      setEnvKey('');
      setEnvValue('');
      setError(null);
      onClose();
    }
  };

  const addArg = () => {
    if (argInput.trim()) {
      setFormData(prev => prev ? ({
        ...prev,
        args: [...prev.args, argInput.trim()],
      }) : null);
      setArgInput('');
    }
  };

  const removeArg = (index: number) => {
    setFormData(prev => prev ? ({
      ...prev,
      args: prev.args.filter((_, i) => i !== index),
    }) : null);
  };

  const addEnvVar = () => {
    if (envKey.trim() && envValue.trim()) {
      setFormData(prev => prev ? ({
        ...prev,
        env: { ...prev.env, [envKey.trim()]: envValue.trim() },
      }) : null);
      setEnvKey('');
      setEnvValue('');
    }
  };

  const removeEnvVar = (key: string) => {
    setFormData(prev => {
      if (!prev) return null;
      const newEnv = { ...prev.env };
      delete newEnv[key];
      return { ...prev, env: newEnv };
    });
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="" size="lg">
      <div className="flex flex-col max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border-default">
          <div>
            <h2 className="text-xl font-semibold text-content-primary">Edit MCP Server</h2>
            <p className="text-sm text-content-tertiary mt-0.5">{server.name}</p>
          </div>
        </div>

        {/* Content */}
        <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto p-6">
          <div className="space-y-4">
            {error && (
              <div className="px-4 py-3 bg-intent-danger-muted border border-intent-danger-muted rounded-lg">
                <p className="text-sm text-intent-danger-fg">{error}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium text-content-secondary mb-1">
                Command <span className="text-intent-danger">*</span>
              </label>
              <Input
                type="text"
                value={formData.command}
                onChange={(e) => setFormData(prev => prev ? ({ ...prev, command: e.target.value }) : null)}
                required
                disabled={isSubmitting}
                fullWidth
              />
            </div>

            {/* Arguments */}
            <div>
              <label className="block text-sm font-medium text-content-secondary mb-1">Arguments</label>
              <div className="space-y-2">
                {formData.args.map((arg, index) => (
                  <div key={index} className="flex items-center gap-2">
                    <input
                      type="text"
                      value={arg}
                      readOnly
                      className="flex-1 px-3 py-2 border border-border-emphasis rounded-lg bg-surface-elevated text-content-secondary font-mono text-sm"
                    />
                    <button
                      type="button"
                      onClick={() => removeArg(index)}
                      disabled={isSubmitting}
                      className="p-2 text-intent-danger hover:bg-intent-danger-muted rounded-lg transition-colors disabled:opacity-50"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                ))}
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={argInput}
                    onChange={(e) => setArgInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addArg())}
                    placeholder="Add argument..."
                    disabled={isSubmitting}
                    className="flex-1 px-3 py-2 border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent disabled:bg-surface-elevated"
                  />
                  <button
                    type="button"
                    onClick={addArg}
                    disabled={isSubmitting || !argInput.trim()}
                    className="px-3 py-2 text-sm font-medium text-content-secondary bg-surface-2 hover:bg-surface-3 rounded-lg transition-colors disabled:opacity-50"
                  >
                    Add
                  </button>
                </div>
              </div>
            </div>

            {/* Environment Variables */}
            <div>
              <label className="block text-sm font-medium text-content-secondary mb-1">Environment Variables</label>
              <div className="space-y-2">
                {Object.entries(formData.env).map(([key, value]) => (
                  <div key={key} className="flex items-center gap-2">
                    <span className="px-3 py-2 bg-surface-elevated border border-border-emphasis rounded-lg text-sm font-mono text-content-secondary">
                      {key}
                    </span>
                    <span className="text-content-tertiary">=</span>
                    <input
                      type="text"
                      value={value}
                      readOnly
                      className="flex-1 px-3 py-2 border border-border-emphasis rounded-lg bg-surface-elevated text-content-secondary font-mono text-sm"
                    />
                    <button
                      type="button"
                      onClick={() => removeEnvVar(key)}
                      disabled={isSubmitting}
                      className="p-2 text-intent-danger hover:bg-intent-danger-muted rounded-lg transition-colors disabled:opacity-50"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                ))}
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={envKey}
                    onChange={(e) => setEnvKey(e.target.value)}
                    placeholder="KEY"
                    disabled={isSubmitting}
                    className="w-32 px-3 py-2 border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent font-mono text-sm disabled:bg-surface-elevated"
                  />
                  <span className="text-content-tertiary">=</span>
                  <input
                    type="text"
                    value={envValue}
                    onChange={(e) => setEnvValue(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addEnvVar())}
                    placeholder="value"
                    disabled={isSubmitting}
                    className="flex-1 px-3 py-2 border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent font-mono text-sm disabled:bg-surface-elevated"
                  />
                  <button
                    type="button"
                    onClick={addEnvVar}
                    disabled={isSubmitting || !envKey.trim() || !envValue.trim()}
                    className="px-3 py-2 text-sm font-medium text-content-secondary bg-surface-2 hover:bg-surface-3 rounded-lg transition-colors disabled:opacity-50"
                  >
                    Add
                  </button>
                </div>
              </div>
            </div>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={formData.auto_start}
                onChange={(e) => setFormData(prev => prev ? ({ ...prev, auto_start: e.target.checked }) : null)}
                disabled={isSubmitting}
                className="w-4 h-4 text-content-primary border-border-emphasis rounded focus:ring-accent-primary disabled:opacity-50"
              />
              <span className="text-sm text-content-secondary">Enable auto-start on launch</span>
            </label>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={formData.enabled}
                onChange={(e) => setFormData(prev => prev ? ({ ...prev, enabled: e.target.checked }) : null)}
                disabled={isSubmitting}
                className="w-4 h-4 text-content-primary border-border-emphasis rounded focus:ring-accent-primary disabled:opacity-50"
              />
              <span className="text-sm text-content-secondary">Enable this server</span>
            </label>
          </div>
        </form>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-border-default bg-surface-elevated">
          <Button variant="secondary" onClick={handleClose} disabled={isSubmitting}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSubmit} disabled={isSubmitting} loading={isSubmitting}>
            {isSubmitting ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
