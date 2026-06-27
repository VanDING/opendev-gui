import { useEffect, useState } from 'react';
import { useChatStore } from '../../stores/chat';
import { } from '../../repositories';
import { MessageList } from './MessageList';
import { QueueBar } from './QueueBar';
import { InputBox } from './InputBox';
import { LandingPage } from './LandingPage';
import { StatusBar } from './StatusBar';

export function ChatInterface() {
  const error = useChatStore(state => {
    const sid = state.currentSessionId;
    return sid ? state.sessionStates[sid]?.error ?? null : null;
  });
  const currentSessionId = useChatStore(state => state.currentSessionId);
  const loadSession = useChatStore(state => state.loadSession);
  const [bridgeChecked, setBridgeChecked] = useState(false);

  // Bridge info is always disabled in desktop mode
  useEffect(() => {
    setBridgeChecked(true);
  }, [loadSession]);

  // Brief null render while checking bridge info (imperceptible)
  if (!bridgeChecked && !currentSessionId) {
    return null;
  }

  if (!currentSessionId) {
    return <LandingPage />;
  }

  return (
    <div className="flex flex-col h-full relative animate-fade-in">
      {error && (
        <div className="bg-intent-danger-muted border border-intent-danger-muted text-intent-danger-fg px-4 py-3 mx-6 mt-4 rounded-lg">
          <strong className="font-semibold">Error:</strong> {error}
        </div>
      )}

      <MessageList />
      <QueueBar />
      <InputBox />
      <StatusBar />
    </div>
  );
}
