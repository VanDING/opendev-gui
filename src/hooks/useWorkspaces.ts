import { useState, useCallback } from 'react';
import { apiClient } from '../api/client';
import type { Session } from '../types';

export interface WorkspaceGroup {
  path: string;
  sessions: Session[];
  mostRecent: Session;
}

export interface WorkspaceOption {
  path: string;
  projectName: string;
}

const getProjectName = (path: string): string => {
  const parts = path.replace(/\/$/, '').split('/');
  return parts[parts.length - 1] || path;
};

function groupByWorkspace(sessions: Session[]): WorkspaceGroup[] {
  const groups: Record<string, Session[]> = {};

  sessions.forEach(session => {
    const wd = session.working_dir || session.working_directory;
    if (!wd || wd.trim() === '') return;
    if (!groups[wd]) groups[wd] = [];
    groups[wd].push(session);
  });

  return Object.entries(groups).map(([path, sessions]) => {
    const sorted = sessions.sort((a, b) =>
      new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
    );
    return {
      path,
      sessions: sorted,
      mostRecent: sorted[0],
    };
  }).sort((a, b) =>
    new Date(b.mostRecent.updated_at).getTime() - new Date(a.mostRecent.updated_at).getTime()
  );
}

export function useWorkspaces() {
  const [workspaces, setWorkspaces] = useState<WorkspaceGroup[]>([]);
  const [workspaceOptions, setWorkspaceOptions] = useState<WorkspaceOption[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const fetchSessions = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await apiClient.listSessions();
      const grouped = groupByWorkspace(data);
      setWorkspaces(grouped);

      const recency: Record<string, number> = {};
      const seen: Record<string, string> = {};
      for (const s of data) {
        const wd = s.working_dir || s.working_directory;
        if (!wd || !wd.trim()) continue;
        const t = new Date(s.updated_at).getTime();
        if (!recency[wd] || t > recency[wd]) recency[wd] = t;
        seen[wd] = wd;
      }
      const sorted = Object.keys(seen)
        .sort((a, b) => (recency[b] || 0) - (recency[a] || 0))
        .map(path => ({ path, projectName: getProjectName(path) }));
      setWorkspaceOptions(sorted);
    } catch (error) {
      console.error('Failed to fetch sessions:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { workspaces, workspaceOptions, isLoading, fetchSessions };
}
