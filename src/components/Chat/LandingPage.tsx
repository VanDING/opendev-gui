import { useState, useEffect, useRef, KeyboardEvent } from 'react';
import { useChatStore } from '../../stores/chat';
import { apiClient } from '../../api/client';
import { HaloSpinner } from '../ui/HaloSpinner';
import { NewSessionModal } from '../Layout/NewSessionModal';
import { Button } from '../ui/Button';
import { useWorkspaces } from '../../hooks/useWorkspaces';

export function LandingPage() {
  const [input, setInput] = useState('');
  const [selectedWorkspace, setSelectedWorkspace] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showPlusMenu, setShowPlusMenu] = useState(false);
  const [showWorkspacePicker, setShowWorkspacePicker] = useState(false);
  const [attachedFiles, setAttachedFiles] = useState<File[]>([]);
  const [isNewSessionOpen, setIsNewSessionOpen] = useState(false);

  const { workspaceOptions: workspaces, fetchSessions } = useWorkspaces();

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const plusMenuRef = useRef<HTMLDivElement>(null);
  const workspaceMenuRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const fileAcceptRef = useRef<string>('');

  const isConnected = useChatStore(state => state.isConnected);
  const loadSession = useChatStore(state => state.loadSession);
  const sendMessage = useChatStore(state => state.sendMessage);
  const bumpSessionList = useChatStore(state => state.bumpSessionList);

  // Fetch workspaces on mount
  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  // Auto-select first workspace when list loads
  useEffect(() => {
    if (workspaces.length > 0 && !selectedWorkspace) {
      setSelectedWorkspace(workspaces[0].path);
    }
  }, [workspaces, selectedWorkspace]);

  // Click-outside to dismiss menus
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (plusMenuRef.current && !plusMenuRef.current.contains(e.target as Node)) {
        setShowPlusMenu(false);
      }
      if (workspaceMenuRef.current && !workspaceMenuRef.current.contains(e.target as Node)) {
        setShowWorkspacePicker(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 200) + 'px';
    }
  }, [input]);

  const handleSend = async () => {
    if (!input.trim() || isCreating || !isConnected) return;
    if (!selectedWorkspace) {
      setError('Select a workspace first');
      return;
    }

    setIsCreating(true);
    setError(null);

    try {
      const result = await apiClient.createSession(selectedWorkspace);
      bumpSessionList();
      const sessionId = result.id;
      await loadSession(sessionId);
      sendMessage(input.trim());
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create session');
      setIsCreating(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleFileUpload = (accept: string) => {
    fileAcceptRef.current = accept;
    setShowPlusMenu(false);
    // Trigger after state update
    setTimeout(() => {
      if (fileInputRef.current) {
        fileInputRef.current.accept = accept;
        fileInputRef.current.click();
      }
    }, 0);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files) {
      setAttachedFiles(prev => [...prev, ...Array.from(files)]);
    }
    // Reset input so the same file can be re-selected
    e.target.value = '';
  };

  const removeFile = (index: number) => {
    setAttachedFiles(prev => prev.filter((_, i) => i !== index));
  };

  const selectedProject = workspaces.find(w => w.path === selectedWorkspace);

  return (
    <div className="relative flex flex-col items-center justify-center h-full px-6 bg-surface-elevated overflow-hidden">
      {/* Background watermark layer */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
        <span className="text-5xl md:text-7xl font-mono font-bold tracking-wider text-bg-300 animate-breathe select-none">
          OpenDev
        </span>
        <HaloSpinner />
      </div>

      {/* Centered input card */}
      <div className="relative z-10 w-full max-w-2xl animate-fade-in">
        <h2 className="text-2xl font-semibold text-content-primary mb-6 text-center">
          What are you working on?
        </h2>
        <div className="rounded-2xl border border-border-default/20 bg-surface-primary shadow-lg">
          {/* Textarea area */}
          <div className="px-5 pt-5 pb-2 rounded-t-2xl">
            <textarea
              ref={textareaRef}
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="How can I help you today?"
              disabled={isCreating || !isConnected}
              className="w-full bg-transparent text-content-primary placeholder-content-tertiary resize-none border-0 focus:outline-none focus:ring-0 text-base leading-relaxed disabled:opacity-50 disabled:cursor-not-allowed"
              rows={3}
              style={{ minHeight: '80px' }}
            />

            {/* Attached file chips */}
            {attachedFiles.length > 0 && (
              <div className="flex flex-wrap gap-2 mt-2">
                {attachedFiles.map((file, i) => (
                  <span
                    key={i}
                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-surface-2 text-content-secondary text-xs border border-border-default/15"
                  >
                    <svg className="w-3.5 h-3.5 text-content-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
                    </svg>
                    {file.name}
                    <button
                      onClick={() => removeFile(i)}
                      className="ml-0.5 text-content-tertiary hover:text-intent-danger"
                    >
                      <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </span>
                ))}
              </div>
            )}
          </div>

          {/* Bottom utility bar */}
          <div className="flex items-center justify-between px-4 py-3 border-t border-border-default/10 rounded-b-2xl">
            {/* Left: + button */}
            <div className="relative" ref={plusMenuRef}>
              <button
                onClick={() => setShowPlusMenu(!showPlusMenu)}
                className="w-8 h-8 rounded-full flex items-center justify-center bg-surface-2 hover:bg-surface-3 text-content-secondary hover:text-content-primary transition-colors"
                title="Attach files"
              >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
                </svg>
              </button>

              {/* Plus menu popover */}
              {showPlusMenu && (
                <div className="absolute bottom-full left-0 mb-2 w-48 bg-surface-primary border border-border-default/20 rounded-xl shadow-lg overflow-hidden z-50 animate-fade-in">
                  <button
                    onClick={() => handleFileUpload('.png,.jpg,.jpeg,.gif,.webp')}
                    className="w-full px-4 py-2.5 text-left text-sm text-content-primary hover:bg-surface-2 flex items-center gap-2.5"
                  >
                    <svg className="w-4 h-4 text-content-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                    Upload image
                  </button>
                  <button
                    onClick={() => handleFileUpload('.pdf,.docx')}
                    className="w-full px-4 py-2.5 text-left text-sm text-content-primary hover:bg-surface-2 flex items-center gap-2.5"
                  >
                    <svg className="w-4 h-4 text-content-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    Upload document
                  </button>
                </div>
              )}
            </div>

            {/* Center-right: Workspace badge */}
            <div className="flex items-center gap-2">
              {workspaces.length === 0 ? (
                <Button
                  variant="secondary"
                  onClick={() => setIsNewSessionOpen(true)}
                >
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  Select workspace...
                </Button>
              ) : (
                <div className="relative" ref={workspaceMenuRef}>
                  <button
                    onClick={() => setShowWorkspacePicker(!showWorkspacePicker)}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-content-secondary hover:text-content-primary bg-surface-2 hover:bg-surface-3 transition-colors"
                  >
                    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    {selectedProject?.projectName || 'Select workspace'}
                    <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                    </svg>
                  </button>

                  {/* Workspace dropdown */}
                  {showWorkspacePicker && (
                    <div className="absolute bottom-full right-0 mb-2 w-64 bg-surface-primary border border-border-default/20 rounded-xl shadow-lg overflow-hidden z-50 animate-fade-in">
                      <div className="max-h-48 overflow-y-auto py-1">
                        {workspaces.map(w => (
                          <button
                            key={w.path}
                            onClick={() => {
                              setSelectedWorkspace(w.path);
                              setShowWorkspacePicker(false);
                            }}
                            className={`w-full px-4 py-2.5 text-left text-sm hover:bg-surface-2 flex flex-col ${
                              w.path === selectedWorkspace ? 'bg-surface-2' : ''
                            }`}
                            title={w.path}
                          >
                            <span className="text-content-primary font-medium">{w.projectName}</span>
                            <span className="text-content-tertiary text-xs truncate">{w.path}</span>
                          </button>
                        ))}
                      </div>
                      <div className="border-t border-border-default/10">
                        <button
                          onClick={() => {
                            setShowWorkspacePicker(false);
                            setIsNewSessionOpen(true);
                          }}
                          className="w-full px-4 py-2.5 text-left text-sm text-accent-primary hover:bg-surface-2 flex items-center gap-2"
                        >
                          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                          </svg>
                          New workspace...
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              )}

              {/* Send button */}
              <Button
                variant="primary"
                onClick={handleSend}
                disabled={!input.trim() || isCreating || !isConnected}
                title="Send (Enter)"
              >
                {isCreating ? (
                  <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                  </svg>
                ) : (
                  <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z" />
                  </svg>
                )}
              </Button>
            </div>
          </div>
        </div>

        {/* Error message */}
        {error && (
          <p className="mt-3 text-sm text-intent-danger text-center animate-fade-in">{error}</p>
        )}

        {/* Hint text */}
        <p className="mt-4 text-xs text-content-tertiary text-center">
          <kbd className="px-1.5 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs">Enter</kbd> to send
          {' '}&middot;{' '}
          <kbd className="px-1.5 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs">Shift + Enter</kbd> for new line
          {' '}&middot; auto-creates a session
        </p>
      </div>

      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        className="hidden"
        onChange={handleFileChange}
      />

      {/* New workspace modal */}
      <NewSessionModal
        isOpen={isNewSessionOpen}
        onClose={() => setIsNewSessionOpen(false)}
      />
    </div>
  );
}
