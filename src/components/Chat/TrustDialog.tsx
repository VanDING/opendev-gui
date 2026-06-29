import { useState } from 'react';
import { Modal } from '../ui/Modal';
import { useTrustStore, hashProjectPath } from '../../stores/trust';

/**
 * Information about a potential trust indicator for the project.
 */
export interface TrustIndicator {
  /** Category label (e.g. "MCP Servers", "Project Hooks"). */
  category: string;
  /** Individual item names under this category. */
  items: string[];
}

interface TrustDialogProps {
  /** The project name shown in the dialog title. */
  projectName: string;
  /** The project directory path (used for hashing). */
  projectPath: string;
  /** Trust indicators to display. */
  indicators: TrustIndicator[];
  /** Whether the dialog is open. */
  isOpen: boolean;
  /** Called when the dialog closes (either trust or don't trust). */
  onClose: () => void;
  /** Called when the user chooses to exit the application. */
  onExit: () => void;
}

/**
 * Trust dialog shown before the first session in a project.
 *
 * Displays project-level trust indicators and allows the user to
 * trust the project, deny trust, or exit the application.
 */
export function TrustDialog({
  projectName,
  projectPath,
  indicators,
  isOpen,
  onClose,
  onExit,
}: TrustDialogProps) {
  const trustProject = useTrustStore(state => state.trustProject);
  const [showDetails, setShowDetails] = useState(false);

  const handleTrust = () => {
    const hash = hashProjectPath(projectPath);
    trustProject(hash);
    onClose();
  };

  const handleDontTrust = () => {
    onClose();
  };

  const filteredIndicators = indicators.filter(ind => ind.items.length > 0);

  return (
    <Modal isOpen={isOpen} onClose={handleDontTrust} title={`Trust Project "${projectName}"?`} size="lg">
      <div className="px-6 py-4">
        <p className="text-sm text-content-secondary mb-4">
          This project has requested additional permissions and configurations.
          Review the indicators below before deciding whether to trust it.
        </p>

        {/* Trust indicators */}
        {filteredIndicators.length > 0 ? (
          <div className="space-y-3">
            {filteredIndicators.map(indicator => (
              <div
                key={indicator.category}
                className="border border-border-default/15 rounded-lg p-3"
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-content-primary">
                    {indicator.category}
                  </span>
                  <span className="text-xs font-mono px-2 py-0.5 rounded bg-intent-warning-muted/20 text-intent-warning">
                    {indicator.items.length} found
                  </span>
                </div>
                <ul className="space-y-1">
                  {indicator.items.map(item => (
                    <li key={item} className="text-sm text-content-secondary font-mono flex items-center gap-2">
                      <span className="w-1.5 h-1.5 rounded-full bg-content-tertiary flex-shrink-0" />
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-content-tertiary py-4 text-center border border-border-default/15 rounded-lg">
            No trust indicators found for this project.
          </div>
        )}

        {/* Expand/collapse for raw config details */}
        {filteredIndicators.length > 0 && (
          <button
            onClick={() => setShowDetails(!showDetails)}
            className="mt-3 text-xs text-accent-secondary hover:text-accent-secondary/80 transition-colors"
          >
            {showDetails ? 'Hide details' : 'Show details'}
          </button>
        )}

        {showDetails && (
          <div className="mt-2 p-3 bg-surface-2 rounded-lg text-xs font-mono text-content-tertiary max-h-32 overflow-y-auto">
            {filteredIndicators.map(ind => (
              <div key={ind.category} className="mb-2">
                <span className="text-content-secondary"># {ind.category}</span>
                {ind.items.map(item => (
                  <div key={item} className="pl-4">- {item}</div>
                ))}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Action buttons */}
      <div className="border-t border-border-default/15 px-6 py-4">
        <div className="flex items-center gap-3">
          <button
            onClick={handleTrust}
            className="flex-1 px-4 py-2.5 text-sm font-medium text-content-inverse bg-accent-secondary rounded-lg hover:bg-accent-secondary/90 transition-colors"
          >
            Trust and Continue
          </button>
          <button
            onClick={handleDontTrust}
            className="flex-1 px-4 py-2.5 text-sm font-medium text-content-secondary border border-border-default/30 rounded-lg hover:bg-surface-elevated transition-colors"
          >
            Don't Trust
          </button>
          <button
            onClick={onExit}
            className="px-4 py-2.5 text-sm font-medium text-intent-danger border border-intent-danger/30 rounded-lg hover:bg-intent-danger-muted/10 transition-colors"
          >
            Exit
          </button>
        </div>

        {/* Info text */}
        <p className="mt-3 text-xs text-content-tertiary text-center">
          Trusting stores your decision locally. You can change it later in Settings.
        </p>
      </div>
    </Modal>
  );
}
