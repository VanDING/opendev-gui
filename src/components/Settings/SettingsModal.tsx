import { useState } from 'react';
import { Cpu, Server, Zap, Palette, Shield, Lock, Globe, Database } from 'lucide-react';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { ModelSettings } from './ModelSettings';
import { MCPSettings } from './MCPSettings';
import { ThemeSettings } from './ThemeSettings';
import { SkillsSettings } from './SkillsSettings';
import { PrivacySettings } from './PrivacySettings';
import { PermissionsSettings } from './PermissionsSettings';
import { SandboxSettings } from './SandboxSettings';
import { MemorySettings } from './MemorySettings';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  initialTab?: TabId;
}

type TabId = 'model' | 'mcp' | 'skills' | 'theme' | 'privacy' | 'permissions' | 'sandbox' | 'memory';

interface TabConfig {
  id: TabId;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  description: string;
}

const tabs: TabConfig[] = [
  {
    id: 'model',
    label: 'Model',
    icon: Cpu,
    description: 'Configure AI model and provider settings'
  },
  {
    id: 'mcp',
    label: 'MCP Servers',
    icon: Server,
    description: 'Manage Model Context Protocol servers'
  },
  {
    id: 'skills',
    label: 'Skills',
    icon: Zap,
    description: 'Manage agent skills and knowledge'
  },
  {
    id: 'permissions',
    label: 'Permissions',
    icon: Lock,
    description: 'Manage tool permission rules'
  },
  {
    id: 'sandbox',
    label: 'Sandbox',
    icon: Globe,
    description: 'Configure sandbox and network policies'
  },
  {
    id: 'memory',
    label: 'Memory',
    icon: Database,
    description: 'View and manage memories'
  },
  {
    id: 'theme',
    label: 'Theme',
    icon: Palette,
    description: 'Choose your preferred theme'
  },
  {
    id: 'privacy',
    label: 'Privacy',
    icon: Shield,
    description: 'Manage privacy and data collection'
  },
];

export function SettingsModal({ isOpen, onClose, initialTab }: SettingsModalProps) {
  const [activeTab, setActiveTab] = useState<TabId>(initialTab || 'model');
  const [config] = useState<{ shadowed_env_vars?: string[] }>({});

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Settings" size="xl">
      <div className="flex flex-1 overflow-hidden" style={{ height: '80vh' }}>
        {/* Vertical Sidebar Navigation */}
        <div className="w-56 border-r border-border-default bg-surface-elevated flex flex-col overflow-y-auto flex-shrink-0">
          <nav className="p-3 space-y-1">
            {tabs.map(tab => {
              const Icon = tab.icon;
              const isActiveTab = activeTab === tab.id;

              return (
                <Button
                  key={tab.id}
                  variant="ghost"
                  onClick={() => setActiveTab(tab.id)}
                  className={`w-full justify-start rounded-lg px-3 py-2 ${
                    isActiveTab
                      ? 'bg-surface-primary text-content-primary shadow-sm'
                      : 'text-content-tertiary hover:text-content-secondary'
                  }`}
                >
                  <Icon className={`w-4 h-4 flex-shrink-0 ${isActiveTab ? 'text-content-primary' : 'text-content-tertiary'}`} />
                  <span className="text-sm font-medium">{tab.label}</span>
                </Button>
              );
            })}
          </nav>

          {/* Sidebar Footer — About */}
          <div className="mt-auto border-t border-border-subtle px-4 py-3">
            <p className="text-xs text-content-tertiary flex items-center gap-1.5">
              <span className="size-1.5 rounded-full bg-surface-3" />
              OpenDev v0.1.0
            </p>
          </div>
        </div>

        {/* Content Area */}
        <div className="flex-1 overflow-y-auto bg-surface-primary">
          <div className="m-4 border border-border-default rounded-lg overflow-hidden">
            <div className="p-6">
              {activeTab === 'model' && <ModelSettings />}
              {activeTab === 'mcp' && <MCPSettings />}
              {activeTab === 'skills' && <SkillsSettings />}
              {activeTab === 'permissions' && <PermissionsSettings />}
              {activeTab === 'sandbox' && <SandboxSettings />}
              {activeTab === 'memory' && <MemorySettings />}
              {activeTab === 'theme' && <ThemeSettings />}
              {activeTab === 'privacy' && (
                <PrivacySettings config={config} />
              )}
            </div>
          </div>
        </div>
      </div>
    </Modal>
  );
}
