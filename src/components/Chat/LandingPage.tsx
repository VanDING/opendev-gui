import { useState, useEffect, useRef, KeyboardEvent } from 'react';
import { useChatStore } from '../../stores/chat';
import { apiClient } from '../../api/client';
import { NewSessionModal } from '../Layout/NewSessionModal';
import { Button } from '../ui/Button';
import { useWorkspaces } from '../../hooks/useWorkspaces';
import { MatrixRain } from './MatrixRain';
import { QuickActions } from './QuickActions';

const FADE_MS = 400;

export function LandingPage() {
  const [input, setInput] = useState('');
  const [selectedWorkspace, setSelectedWorkspace] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [isFading, setIsFading] = useState(false);
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

  const isConnected = useChatStore(state => state.isConnected);
  const loadSession = useChatStore(state => state.loadSession);
  const sendMessage = useChatStore(state => state.sendMessage);
  const bumpSessionList = useChatStore(state => state.bumpSessionList);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  useEffect(() => {
    if (workspaces.length > 0 && !selectedWorkspace) {
      setSelectedWorkspace(workspaces[0].path);
    }
  }, [workspaces, selectedWorkspace]);

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

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 200) + 'px';
    }
  }, [input]);

  const handleSend = async () => {
    if (!input.trim() || isCreating || isFading || !isConnected) return;
    if (!selectedWorkspace) {
      setError('Select a workspace first');
      return;
    }

    setIsFading(true);
    await new Promise<void>((resolve) => setTimeout(resolve, FADE_MS));

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
      setIsFading(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleFileUpload = (accept: string) => {
    setShowPlusMenu(false);
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
    e.target.value = '';
  };

  const removeFile = (index: number) => {
    setAttachedFiles(prev => prev.filter((_, i) => i !== index));
  };

  const selectedProject = workspaces.find(w => w.path === selectedWorkspace);

  const handleQuickAction = (command: string) => {
    setInput(command + ' ');
    textareaRef.current?.focus();
  };

  const inputDisabled = isCreating || !isConnected || isFading;

  return (
    <div
      className={[
        'relative flex flex-col items-center h-full px-6 bg-surface-elevated',
        'overflow-y-auto overflow-x-hidden',
        'transition-opacity ease-out',
        isFading ? 'opacity-0' : 'opacity-100',
      ].join(' ')}
      style={{ transitionDuration: `${FADE_MS}ms` }}
    >
      {/* Background: dynamic matrix rain (now contains OPENDEV + box + subtitle) */}
      <MatrixRain />

      {/* Spacer pushes the input card below the canvas-rendered OPENDEV + box + subtitle.
          Sized to match canvas: ~20vh to top of logo + ~5 cell heights of logo+padding + 1 subtitle. */}
      <div className="shrink-0 min-h-[clamp(220px,28vh,320px)]" aria-hidden />

      {/* Centered input card */}
      <div className="relative z-10 w-full max-w-2xl">
        <div className="rounded-md border border-border-default bg-surface-elevated shadow-xs">
          {/* Prompt + textarea */}
          <div className="relative px-4 pt-4 pb-3">
            <span
              aria-hidden
              className="absolute left-4 top-4 font-mono text-base text-accent-primary select-none pointer-events-none"
            >
              &gt;
            </span>
            <textarea
              ref={textareaRef}
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="What are you working on?"
              disabled={inputDisabled}
              className="w-full bg-transparent text-content-primary placeholder-content-tertiary resize-none border-0 focus:outline-none focus:ring-0 text-base leading-relaxed pl-5 disabled:opacity-50 disabled:cursor-not-allowed"
              rows={3}
              style={{ minHeight: '80px' }}
            />

            {attachedFiles.length > 0 && (
              <div className="flex flex-wrap gap-2 mt-2 pl-5">
                {attachedFiles.map((file, i) => (
                  <span
                    key={i}
                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-surface-2 text-content-secondary text-xs border border-border-subtle font-mono"
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
          <div className="flex items-center justify-between px-4 py-2.5 border-t border-border-subtle">
            {/* Left: context hints */}
            <div className="flex items-center gap-3 text-xs font-mono text-content-tertiary">
              <div className="relative" ref={plusMenuRef}>
                <button
                  onClick={() => setShowPlusMenu(!showPlusMenu)}
                  className="inline-flex items-center gap-1 hover:text-content-primary transition-colors"
                  title="Attach files"
                >
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
                  </svg>
                  <span>Add context</span>
                </button>
                {showPlusMenu && (
                  <div className="absolute bottom-full left-0 mb-2 w-48 bg-surface-elevated border border-border-default rounded-md shadow-md overflow-hidden z-50 animate-fade-in">
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
              <span className="text-border-default">·</span>
              <span>@ to mention</span>
              <span className="text-border-default">·</span>
              <span>/ for commands</span>
            </div>

            {/* Right: project picker + send */}
            <div className="flex items-center gap-2">
              {workspaces.length === 0 ? (
                <Button variant="secondary" onClick={() => setIsNewSessionOpen(true)}>
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  Select workspace…
                </Button>
              ) : (
                <div className="relative" ref={workspaceMenuRef}>
                  <button
                    onClick={() => setShowWorkspacePicker(!showWorkspacePicker)}
                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-2xs font-mono text-content-tertiary hover:text-content-primary hover:bg-surface-2 transition-colors"
                  >
                    <span className="uppercase tracking-wider">Project:</span>
                    <span className="text-content-primary font-medium">{selectedProject?.projectName || 'Select'}</span>
                    <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                    </svg>
                  </button>
                  {showWorkspacePicker && (
                    <div className="absolute bottom-full right-0 mb-2 w-64 bg-surface-elevated border border-border-default rounded-md shadow-md overflow-hidden z-50 animate-fade-in">
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
                            <span className="text-content-tertiary text-xs truncate font-mono">{w.path}</span>
                          </button>
                        ))}
                      </div>
                      <div className="border-t border-border-subtle">
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
                          New workspace…
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              )}

              <Button
                variant="primary"
                onClick={handleSend}
                disabled={inputDisabled || !input.trim()}
                title="Send (Enter)"
                className="px-3"
              >
                {isCreating || isFading ? (
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

        {error && (
          <p className="mt-3 text-sm text-intent-danger text-center animate-fade-in">{error}</p>
        )}

        {/* kbd hint row — three discrete blocks */}
        <div className="mt-4 flex items-center justify-center gap-4 text-xs text-content-tertiary font-mono">
          <span className="inline-flex items-center gap-1.5">
            <kbd className="kbd">Enter</kbd>
            <span>to send</span>
          </span>
          <span className="text-border-emphasis">|</span>
          <span className="inline-flex items-center gap-1.5">
            <kbd className="kbd">Shift + Enter</kbd>
            <span>for new line</span>
          </span>
          <span className="text-border-emphasis">|</span>
          <span className="inline-flex items-center gap-1.5">
            <kbd className="kbd">/help</kbd>
            <span>for commands</span>
          </span>
        </div>
      </div>

      {/* Quick actions */}
      <div className="relative z-10 w-full max-w-4xl mt-14 mb-12">
        <QuickActions
          onSelect={handleQuickAction}
          onBrowseAll={() => textareaRef.current?.focus()}
        />
      </div>

      <input
        ref={fileInputRef}
        type="file"
        className="hidden"
        onChange={handleFileChange}
      />

      <NewSessionModal
        isOpen={isNewSessionOpen}
        onClose={() => setIsNewSessionOpen(false)}
      />
    </div>
  );
}
