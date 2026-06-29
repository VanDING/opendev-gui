import { create } from 'zustand';

const TRUST_FILE = '.opendev/trust.json';

function trustFilePath(): string {
  const home = process.env.HOME || process.env.USERPROFILE || '';
  return `${home}/${TRUST_FILE}`;
}

interface TrustState {
  /** Map of project directory hashes → trust status */
  trustedProjects: Record<string, boolean>;
  /** Mark a project hash as trusted */
  trustProject: (hash: string) => void;
  /** Check if a project hash has been trusted */
  isProjectTrusted: (hash: string) => boolean;
  /** Load trusted projects from disk */
  loadTrustedProjects: () => void;
}

/** Generate a simple hash from a project path string. */
export function hashProjectPath(path: string): string {
  let hash = 0;
  for (let i = 0; i < path.length; i++) {
    const char = path.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash).toString(16).padStart(8, '0');
}

export const useTrustStore = create<TrustState>((set, get) => ({
  trustedProjects: {},

  trustProject: (hash: string) => {
    set(state => {
      const updated = { ...state.trustedProjects, [hash]: true };
      // Persist to disk
      try {
        const fs = require('fs');
        const dir = require('path').dirname(trustFilePath());
        if (!fs.existsSync(dir)) {
          fs.mkdirSync(dir, { recursive: true });
        }
        fs.writeFileSync(trustFilePath(), JSON.stringify(updated, null, 2), 'utf-8');
      } catch {
        // Silently fail — trust is ephemeral if we can't write
      }
      return { trustedProjects: updated };
    });
  },

  isProjectTrusted: (hash: string) => {
    return !!get().trustedProjects[hash];
  },

  loadTrustedProjects: () => {
    try {
      const fs = require('fs');
      if (fs.existsSync(trustFilePath())) {
        const raw = fs.readFileSync(trustFilePath(), 'utf-8');
        const data = JSON.parse(raw);
        if (typeof data === 'object' && data !== null) {
          set({ trustedProjects: data });
        }
      }
    } catch {
      // Silently fail
    }
  },
}));
