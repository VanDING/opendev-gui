import { useState } from 'react';
import { Cpu, Server, Info, Palette } from 'lucide-react';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { ModelSettings } from './ModelSettings';
import { MCPSettings } from './MCPSettings';
import { ThemeSettings } from './ThemeSettings';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  initialTab?: TabId;
}

type TabId = 'model' | 'mcp' | 'theme' | 'about';

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
    id: 'theme',
    label: 'Theme',
    icon: Palette,
    description: 'Choose your preferred theme'
  },
  {
    id: 'about',
    label: 'About',
    icon: Info,
    description: 'Version and license'
  },
];

export function SettingsModal({ isOpen, onClose, initialTab }: SettingsModalProps) {
  const [activeTab, setActiveTab] = useState<TabId>(initialTab || 'model');

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Settings" size="lg">
      <div className="flex-1 flex overflow-hidden" style={{ minHeight: '60vh' }}>
        {/* Vertical Sidebar Navigation */}
        <div className="w-56 border-r border-border-default bg-surface-elevated overflow-y-auto">
          <nav className="p-3 space-y-1">
            {tabs.map(tab => {
              const Icon = tab.icon;
              const isActiveTab = activeTab === tab.id;

              return (
                <Button
                  key={tab.id}
                  variant="ghost"
                  onClick={() => setActiveTab(tab.id)}
                  className={`w-full justify-start ${isActiveTab ? 'bg-surface-primary text-content-primary shadow-sm' : ''}`}
                >
                  <Icon className={`w-5 h-5 flex-shrink-0 ${isActiveTab ? 'text-content-primary' : 'text-content-tertiary'}`} />
                  <span className="text-sm font-medium">{tab.label}</span>
                </Button>
              );
            })}
          </nav>
        </div>

        {/* Content Area */}
        <div className="flex-1 overflow-y-auto bg-surface-primary">
          <div className="p-6">
            {activeTab === 'model' && <ModelSettings />}
            {activeTab === 'mcp' && <MCPSettings />}
            {activeTab === 'theme' && <ThemeSettings />}
            {activeTab === 'about' && (
              <div className="text-center py-12">
                <Info className="w-12 h-12 mx-auto text-content-tertiary mb-3" />
                <p className="text-sm text-content-secondary font-medium mb-1">OpenDev GUI</p>
                <p className="text-xs text-content-tertiary">v0.1.0</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </Modal>
  );
}
