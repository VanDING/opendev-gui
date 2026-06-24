import { useState, useCallback } from 'react';
import { TopBar } from '../components/Layout/TopBar';
import { SessionsSidebar } from '../components/Layout/SessionsSidebar';
import { ChatInterface } from '../components/Chat/ChatInterface';
import { DetailPanel } from '../components/Chat/DetailPanel';
import { ApprovalDialog } from '../components/ApprovalDialog';
import { AskUserDialog } from '../components/Chat/AskUserDialog';
import { PlanApprovalDialog } from '../components/Chat/PlanApprovalDialog';
import { CommandPalette } from '../components/Chat/CommandPalette';
import { StatusDialog } from '../components/Chat/StatusDialog';
export function ChatPage() {
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [statusDialogOpen, setStatusDialogOpen] = useState(false);
  const [detailCollapsed, setDetailCollapsed] = useState(false);

  const openCommandPalette = useCallback(() => setCommandPaletteOpen(true), []);
  const closeCommandPalette = useCallback(() => setCommandPaletteOpen(false), []);
  const openStatusDialog = useCallback(() => setStatusDialogOpen(true), []);
  const closeStatusDialog = useCallback(() => setStatusDialogOpen(false), []);
  const toggleDetail = useCallback(() => setDetailCollapsed(c => !c), []);

  return (
    <div className="h-screen flex flex-col bg-surface-elevated">
      <TopBar onOpenCommandPalette={openCommandPalette} detailCollapsed={detailCollapsed} onToggleDetail={toggleDetail} />
      <div className="flex-1 flex overflow-hidden">
        <SessionsSidebar />
        <main className="flex-1 flex flex-col overflow-hidden bg-surface-primary">
          <ChatInterface />
        </main>
        <DetailPanel collapsed={detailCollapsed} onToggle={toggleDetail} />
      </div>

      {/* Modals */}
      <ApprovalDialog />
      <AskUserDialog />
      <PlanApprovalDialog />
      <CommandPalette
        isOpen={commandPaletteOpen}
        onClose={closeCommandPalette}
        onOpenStatus={openStatusDialog}
      />
      <StatusDialog isOpen={statusDialogOpen} onClose={closeStatusDialog} />
    </div>
  );
}
