import { useState, useEffect, useRef } from 'react';
import { ChevronDown, Settings, Folder, Plus } from 'lucide-react';
import { useChatStore } from '../../stores/chat';
import type { Session } from '../../types';
import { SettingsModal } from '../Settings/SettingsModal';
import { NewSessionModal } from './NewSessionModal';
import { DeleteConfirmModal } from './DeleteConfirmModal';
import { SessionModelModal } from './SessionModelModal';
import { sessionRepository } from '../../repositories';
import { Button } from '../ui/Button';
import { Modal } from '../ui/Modal';
import { useWorkspaces } from '../../hooks/useWorkspaces';
import type { WorkspaceGroup } from '../../hooks/useWorkspaces';

const getProjectName = (path: string): string => {
  const parts = path.replace(/\/$/, '').split('/');
  return parts[parts.length - 1] || path;
};

export function SessionsSidebar() {
  const { workspaces, isLoading, fetchSessions } = useWorkspaces();
  const [expandedWorkspaces, setExpandedWorkspaces] = useState<Set<string>>(new Set());
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isNewSessionOpen, setIsNewSessionOpen] = useState(false);
  const [deleteWorkspace, setDeleteWorkspace] = useState<WorkspaceGroup | null>(null);
  const [deleteSessionId, setDeleteSessionId] = useState<string | null>(null);
  const [showCollapsedContent, setShowCollapsedContent] = useState(false);
  const [sessionModelSessionId, setSessionModelSessionId] = useState<string | null>(null);
  const [sessionModelLabel, setSessionModelLabel] = useState('');
  const swapTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Get loadSession + sidebar state from chat store
  const loadSession = useChatStore(state => state.loadSession);
  const currentSessionId = useChatStore(state => state.currentSessionId);
  const sessionListVersion = useChatStore(state => state.sessionListVersion);
  const runningSessions = useChatStore(state => state.runningSessions);
  const sessionStates = useChatStore(state => state.sessionStates);
  const isCollapsed = useChatStore(state => state.sidebarCollapsed);
  const toggleSidebar = useChatStore(state => state.toggleSidebar);

  // Disable "New Chat" when the current session has no messages yet
  const currentSessionIsEmpty = currentSessionId !== null && (
    (sessionStates[currentSessionId]?.messages ?? []).length === 0
  );

  // Per-workspace check: only disable "New Session" in the workspace
  // that contains the current empty session (not globally).
  const workspaceHasCurrentEmptySession = (workspace: WorkspaceGroup): boolean => {
    if (currentSessionId === null) return false;
    const isCurrentInWorkspace = workspace.sessions.some(s => s.id === currentSessionId);
    if (!isCurrentInWorkspace) return false;
    return (sessionStates[currentSessionId]?.messages ?? []).length === 0;
  };

  useEffect(() => {
    fetchSessions();
  }, [sessionListVersion, fetchSessions]);

  // Delayed content swap: clips naturally via overflow-hidden
  useEffect(() => {
    if (swapTimerRef.current !== null) {
      clearTimeout(swapTimerRef.current);
      swapTimerRef.current = null;
    }

    if (isCollapsed) {
      // COLLAPSING: keep expanded content while width shrinks, swap at ~250ms
      swapTimerRef.current = setTimeout(() => {
        setShowCollapsedContent(true);
        swapTimerRef.current = null;
      }, 250);
    } else {
      // EXPANDING: swap to expanded content immediately, revealed as width grows
      setShowCollapsedContent(false);
    }

    return () => {
      if (swapTimerRef.current !== null) {
        clearTimeout(swapTimerRef.current);
        swapTimerRef.current = null;
      }
    };
  }, [isCollapsed]);

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const toggleWorkspace = (workspacePath: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setExpandedWorkspaces(prev => {
      const next = new Set(prev);
      if (next.has(workspacePath)) {
        next.delete(workspacePath);
      } else {
        next.add(workspacePath);
      }
      return next;
    });
  };

  const handleSessionClick = async (session: Session, e: React.MouseEvent) => {
    e.stopPropagation();
    await loadSession(session.id);
  };

  const handleNewWorkspace = () => {
    setIsNewSessionOpen(true);
  };

  const handleNewSessionInWorkspace = async (workspacePath: string, e: React.MouseEvent) => {
    e.stopPropagation();

    try {
      const result = await sessionRepository.createSession(workspacePath);

      // Refresh sessions list
      await fetchSessions();

      // Load the new session
      const sessionId = result.id;
      if (sessionId) {
        await loadSession(sessionId);
      }
    } catch (error) {
      console.error('[SessionsSidebar] Failed to create session:', error);
      alert('Failed to create new session');
    }
  };

  const handleDeleteWorkspace = (workspace: WorkspaceGroup, e: React.MouseEvent) => {
    e.stopPropagation();
    setDeleteWorkspace(workspace);
  };

  const handleDeleteSession = (sessionId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setDeleteSessionId(sessionId);
  };

  const confirmDeleteSession = async () => {
    if (!deleteSessionId) return;

    try {
      await sessionRepository.deleteSession(deleteSessionId);

      // Clean up per-session state
      const { sessionStates: currentStates } = useChatStore.getState();
      const updated = { ...currentStates };
      delete updated[deleteSessionId];
      useChatStore.setState({ sessionStates: updated });

      // If deleting the currently viewed session, clear it
      if (deleteSessionId === useChatStore.getState().currentSessionId) {
        useChatStore.setState({ currentSessionId: null, hasWorkspace: false });
      }

      // Refresh the sessions list
      await fetchSessions();

      setDeleteSessionId(null);
    } catch (error) {
      console.error('Failed to delete session:', error);
      alert('Failed to delete session');
    }
  };

  const confirmDelete = async () => {
    if (!deleteWorkspace) return;

    try {
      const currentSid = useChatStore.getState().currentSessionId;
      let needsClearCurrent = false;

      // Delete all sessions for this workspace
      for (const session of deleteWorkspace.sessions) {
        await sessionRepository.deleteSession(session.id);

        // Clean up per-session state
        const { sessionStates: currentStates } = useChatStore.getState();
        const updated = { ...currentStates };
        delete updated[session.id];
        useChatStore.setState({ sessionStates: updated });

        if (session.id === currentSid) {
          needsClearCurrent = true;
        }
      }

      if (needsClearCurrent) {
        useChatStore.setState({ currentSessionId: null, hasWorkspace: false });
      }

      // Refresh the sessions list
      await fetchSessions();

      // Remove from expanded workspaces if it was expanded
      setExpandedWorkspaces(prev => {
        const next = new Set(prev);
        next.delete(deleteWorkspace.path);
        return next;
      });

      setDeleteWorkspace(null);
    } catch (error) {
      console.error('Failed to delete workspace:', error);
      alert('Failed to delete workspace');
    }
  };

  const getSessionLabel = (session: Session): string => {
    return session.title || session.id.substring(0, 8);
  };

  return (
    <aside
      className="h-full flex flex-col relative overflow-hidden flex-shrink-0 bg-surface-elevated border-r border-border-default"
      style={{
        width: isCollapsed ? 64 : 320,
        transition: 'width 300ms cubic-bezier(0.25, 0.46, 0.45, 0.94)',
      }}
    >
      {showCollapsedContent ? (
        <div className="min-w-[64px] flex flex-col h-full animate-content-fade">
          {/* Collapsed Workspace Icons */}
          <div className="flex-1 overflow-y-auto py-3 space-y-2 flex flex-col items-center">
            {workspaces.slice(0, 5).map((workspace) => {
              const hasActiveSession = workspace.sessions.some(s => s.id === currentSessionId);
              const hasRunningSession = workspace.sessions.some(s => runningSessions.has(s.id));
              const projectName = getProjectName(workspace.path);

              return (
                <div
                  key={workspace.path}
                  className="relative group"
                  title={`${projectName} (${workspace.sessions.length} sessions)`}
                >
                  <button
                    onClick={() => {
                      toggleSidebar();
                      setTimeout(() => {
                        setExpandedWorkspaces(prev => new Set([...prev, workspace.path]));
                      }, 100);
                    }}
                    className={`w-10 h-10 rounded-md flex items-center justify-center ${
                      hasActiveSession
                        ? 'bg-accent-magenta-muted border border-accent-magenta'
                        : 'bg-surface-primary hover:bg-surface-2 border border-border-default'
                    }`}
                  >
                    <Folder className={`w-5 h-5 ${hasActiveSession ? 'text-accent-magenta' : 'text-content-tertiary'}`} />
                  </button>
                  {hasRunningSession && (
                    <div className="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-[1.5px] border-accent-magenta-muted border-t-accent-magenta animate-spin" />
                  )}

                  {/* Tooltip */}
                  <div className="absolute left-full ml-2 top-1/2 transform -translate-y-1/2 bg-accent-primary text-content-inverse text-xs rounded-lg px-3 py-2 whitespace-nowrap opacity-60 group-hover:opacity-100 pointer-events-none z-50 shadow-lg">
                    <div className="font-medium text-sm mb-1">{projectName}</div>
                    <div className="text-content-tertiary text-xs">{workspace.sessions.length} session{workspace.sessions.length !== 1 ? 's' : ''}</div>
                    {hasActiveSession && <div className="text-accent-magenta text-xs mt-1 font-semibold">Active</div>}
                    <div className="absolute right-full top-1/2 transform -translate-y-1/2 border-4 border-transparent border-r-surface-primary"></div>
                  </div>
                </div>
              );
            })}

            {/* New Workspace Button (Collapsed) */}
            <button
              onClick={() => {
                if (currentSessionIsEmpty) return;
                toggleSidebar();
                setTimeout(() => setIsNewSessionOpen(true), 100);
              }}
              disabled={currentSessionIsEmpty}
              className={`w-10 h-10 rounded-lg flex items-center justify-center text-content-inverse shadow-md transition-all ${
                currentSessionIsEmpty
                  ? 'bg-surface-3 cursor-not-allowed opacity-50'
                  : 'bg-accent-primary hover:bg-accent-primary-hover hover:shadow-lg'
              }`}
              title={currentSessionIsEmpty ? 'Send a message before starting a new session' : 'Start Conversation'}
            >
              <Plus className="w-5 h-5" />
            </button>
          </div>

          {/* Collapsed Footer */}
          <div className="p-2 border-t border-border-default bg-surface-elevated">
            <Button
              variant="ghost"
              onClick={() => setIsSettingsOpen(true)}
              title="Settings"
              className="w-full rounded-xl"
            >
              <Settings className="w-5 h-5" />
            </Button>
          </div>
        </div>
      ) : (
        <div className="min-w-[320px] flex flex-col h-full animate-content-fade">
          {/* Compact New Chat Header */}
          <div className="flex items-center justify-between px-4 py-3 border-b border-border-default">
            <Button
              variant="primary"
              onClick={currentSessionIsEmpty ? undefined : handleNewWorkspace}
              disabled={currentSessionIsEmpty}
              title={currentSessionIsEmpty ? 'Send a message before starting a new session' : undefined}
              className="flex-1"
            >
              <Plus className="w-4 h-4" />
              <span>New Chat</span>
            </Button>
          </div>

          {/* Workspaces Header */}
          <div className="px-5 py-4 border-b border-border-subtle">
            <h2 className="text-xs font-semibold text-content-tertiary uppercase tracking-wider">Workspaces</h2>
          </div>

          {/* Workspaces List */}
          <div className="flex-1 overflow-y-auto px-4 py-3">
            {isLoading ? (
              <div className="space-y-3 px-0 py-3">
                <div className="skeleton-shimmer h-16 rounded-xl" />
                <div className="skeleton-shimmer h-16 rounded-xl" />
                <div className="skeleton-shimmer h-16 rounded-xl" />
              </div>
            ) : workspaces.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
                <div className="w-16 h-16 rounded-full bg-surface-2 flex items-center justify-center mb-4">
                  <svg className="w-8 h-8 text-content-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                </div>
                <h3 className="text-sm font-medium text-content-primary mb-1">No workspaces yet</h3>
                <p className="text-xs text-content-tertiary max-w-[200px]">
                  Start a conversation to create your first workspace
                </p>
              </div>
            ) : (
              <div className="space-y-3 animate-fade-in">
                {workspaces.map((workspace) => {
                  const isExpanded = expandedWorkspaces.has(workspace.path);
                  const hasActiveSession = workspace.sessions.some(s => s.id === currentSessionId);
                  const projectName = getProjectName(workspace.path);

                  return (
                    <div
                      key={workspace.path}
                      className="relative w-full rounded-xl bg-surface-primary border border-border-default hover:border-border-emphasis hover:shadow-sm"
                    >
                      {/* Workspace Header */}
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleWorkspace(workspace.path, e);
                        }}
                        className="w-full px-4 py-3.5 text-left group cursor-pointer hover:bg-surface-2/50 rounded-t-xl"
                      >
                        <div className="flex items-start gap-2 pr-10">
                          <ChevronDown
                            className={`mt-0.5 w-4 h-4 flex-shrink-0 text-content-tertiary ${
                              isExpanded ? 'rotate-0' : '-rotate-90'
                            }`}
                            style={{ transition: 'transform 200ms ease' }}
                          />

                          <div className="mt-0.5 w-4 h-4 rounded flex-shrink-0 flex items-center justify-center bg-surface-2 group-hover:bg-surface-3">
                            <svg className="w-2.5 h-2.5 text-content-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                            </svg>
                          </div>

                          <div className="flex-1 min-w-0">
                            <h3 className="text-sm font-semibold text-content-primary truncate" title={workspace.path}>
                              {projectName}
                            </h3>
                            <div className="flex items-center justify-between text-xs mt-1">
                              <span className="text-content-tertiary truncate" title={workspace.path}>
                                {formatDate(workspace.mostRecent.updated_at)}
                              </span>
                              <span className={`ml-2 px-1.5 py-0.5 rounded-full text-xs flex-shrink-0 ${
                                hasActiveSession
                                  ? 'bg-accent-magenta-muted text-accent-magenta font-medium'
                                  : 'bg-surface-3 text-content-secondary'
                              }`}>
                                {workspace.sessions.length}
                              </span>
                            </div>
                          </div>
                        </div>
                      </button>

                      {/* Delete Button */}
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteWorkspace(workspace, e);
                        }}
                        className="absolute top-3.5 right-3 w-7 h-7 rounded-md flex items-center justify-center hover:bg-intent-danger-muted/80 text-content-tertiary hover:text-intent-danger bg-surface-primary shadow-sm z-10 delete-glow"
                        title="Delete workspace"
                      >
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>

                      {/* Sessions List (Animated expand/collapse) */}
                      <div
                        className="overflow-hidden transition-all duration-300"
                        style={{
                          maxHeight: isExpanded ? '1000px' : '0px',
                          opacity: isExpanded ? 1 : 0,
                        }}
                      >
                        <div className="px-4 pb-3 space-y-1.5 border-t border-border-subtle pt-2">
                          {/* Add New Session Button */}
                          <button
                            onClick={workspaceHasCurrentEmptySession(workspace) ? undefined : (e) => handleNewSessionInWorkspace(workspace.path, e)}
                            disabled={workspaceHasCurrentEmptySession(workspace)}
                            title={workspaceHasCurrentEmptySession(workspace) ? 'Send a message before starting a new session' : undefined}
                            className={`w-full px-4 py-3 rounded-lg text-left border-2 border-dashed flex items-center gap-2 ${
                              workspaceHasCurrentEmptySession(workspace)
                                ? 'bg-surface-elevated border-border-default text-content-tertiary cursor-not-allowed'
                                : 'cursor-pointer bg-intent-warning-muted hover:bg-intent-warning-muted border-intent-warning hover:border-intent-warning-hover text-intent-warning'
                            }`}
                          >
                            <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                            </svg>
                            <span className="text-xs font-medium">New Session</span>
                          </button>

                          {/* Sessions List */}
                          {workspace.sessions.map((session) => {
                            const isActiveSession = currentSessionId === session.id;
                            const sessionLabel = getSessionLabel(session);
                            const isRunning = runningSessions.has(session.id);
                            const sState = sessionStates[session.id];
                            const needsAttention = !!(sState?.pendingApproval || sState?.pendingAskUser);

                            return (
                              <div key={session.id} className="relative group">
                                <button
                                  onClick={(e) => handleSessionClick(session, e)}
                                  className={`w-full px-4 py-3 pr-10 rounded-lg text-left cursor-pointer ${
                                    isActiveSession
                                      ? 'bg-accent-magenta-muted border-l-4 border-accent-magenta animate-border-breathe-magenta'
                                      : 'bg-surface-primary border border-border-default hover:border-accent-magenta hover:bg-accent-magenta-muted/30 hover:scale-[1.01] hover:shadow-sm transition-all duration-200'
                                  }`}
                                >
                                  <div className="flex items-center gap-1.5">
                                    {isRunning && (
                                      <div className="w-3.5 h-3.5 rounded-full border-2 border-accent-magenta-muted border-t-accent-magenta animate-spin flex-shrink-0" />
                                    )}
                                    {needsAttention && !isRunning && (
                                      <div className="w-4 h-4 rounded-full bg-accent-magenta text-content-inverse text-[9px] font-bold flex items-center justify-center flex-shrink-0">!</div>
                                    )}
                                    <div className={`text-xs font-medium truncate ${
                                      isActiveSession ? 'text-accent-magenta' : 'text-content-primary'
                                    }`} title={session.title || session.id}>
                                      {sessionLabel}
                                    </div>
                                    {needsAttention && isRunning && (
                                      <div className="w-4 h-4 rounded-full bg-accent-magenta text-content-inverse text-[9px] font-bold flex items-center justify-center flex-shrink-0">!</div>
                                    )}
                                    {session.has_session_model && (
                                      <span className="w-2 h-2 rounded-full bg-intent-purple-muted flex-shrink-0" title="Custom model" />
                                    )}
                                  </div>
                                  <div className="flex items-center justify-between text-xs mt-1">
                                    <span className={`${
                                      isActiveSession ? 'text-accent-magenta' : 'text-content-tertiary'
                                    }`}>
                                      {formatDate(session.updated_at)}
                                    </span>
                                    <span className={`${
                                      isActiveSession ? 'text-accent-magenta' : 'text-content-tertiary'
                                    }`}>
                                      {session.message_count} msgs
                                    </span>
                                  </div>
                                </button>

                                {/* Session Action Buttons */}
                                <div className="absolute top-1.5 right-1.5 flex gap-0.5 opacity-60 group-hover:opacity-100 z-10">
                                  {/* Session Model Button */}
                                  <button
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      setSessionModelSessionId(session.id);
                                      setSessionModelLabel(getSessionLabel(session));
                                    }}
                                    className="w-6 h-6 rounded flex items-center justify-center hover:bg-accent-magenta-muted text-content-tertiary hover:text-accent-magenta"
                                    title="Session models"
                                  >
                                    <Settings className="w-3.5 h-3.5" />
                                  </button>
                                  {/* Delete Session Button */}
                                  <button
                                    onClick={(e) => handleDeleteSession(session.id, e)}
                                    className="w-6 h-6 rounded flex items-center justify-center hover:bg-intent-danger-muted/80 text-content-tertiary hover:text-intent-danger delete-glow"
                                    title="Delete session"
                                  >
                                    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                    </svg>
                                  </button>
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {/* Footer - Expanded */}
          <div className="p-4 border-t border-border-default bg-surface-elevated">
            <Button
              variant="ghost"
              onClick={() => setIsSettingsOpen(true)}
              className="w-full rounded-xl"
            >
              <Settings className="w-4 h-4" />
              <span>Settings</span>
            </Button>
          </div>
        </div>
      )}

      {/* ===== MODALS (always accessible, outside conditional blocks) ===== */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
      />

      <NewSessionModal
        isOpen={isNewSessionOpen}
        onClose={() => setIsNewSessionOpen(false)}
      />

      <DeleteConfirmModal
        isOpen={deleteWorkspace !== null}
        workspacePath={deleteWorkspace?.path || ''}
        onConfirm={confirmDelete}
        onCancel={() => setDeleteWorkspace(null)}
      />

      <SessionModelModal
        sessionId={sessionModelSessionId}
        sessionLabel={sessionModelLabel}
        onClose={() => {
          setSessionModelSessionId(null);
          fetchSessions();
        }}
      />

      <Modal
        isOpen={deleteSessionId !== null}
        onClose={() => setDeleteSessionId(null)}
        title="Delete Session"
        size="sm"
      >
        <div className="p-6">
          <p className="text-sm text-content-tertiary mb-5">
            Are you sure you want to delete session <strong className="text-content-secondary">{deleteSessionId?.substring(0, 8)}</strong>?
            <br />
            This action cannot be undone.
          </p>
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={() => setDeleteSessionId(null)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={confirmDeleteSession}>
              Delete
            </Button>
          </div>
        </div>
      </Modal>
    </aside>
  );
}
