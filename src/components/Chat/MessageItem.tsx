import React from 'react';
import { MarkdownContent } from './MarkdownContent';
import { ToolCallMessage } from './ToolCallMessage';
import { ThinkingBlock } from './ThinkingBlock';

interface MessageItemProps {
  message: any; // Using any for now to simplify, ideally should import ChatMessage from types
  index: number;
  isNewMessage: boolean;
  prevMessageCount: number;
  thinkingLevel: string;
  isLoading: boolean;
  isLastMessage: boolean;
}

export const MessageItem = React.memo(function MessageItem({
  message,
  index,
  isNewMessage,
  prevMessageCount,
  thinkingLevel,
  isLoading,
  isLastMessage
}: MessageItemProps) {
  // Stagger animation for new messages
  const staggerStyle = isNewMessage
    ? { animationDelay: `${(index - prevMessageCount) * 50}ms`, animationFillMode: 'both' as const }
    : undefined;

  // Render tool calls with special component
  if (message.role === 'tool_call') {
    const hasResult = message.tool_result != null && Object.keys(message.tool_result).length > 0;
    return (
      <div className={`animate-slide-up`} style={{ ...(message.depth ? { marginLeft: `${message.depth * 1.5}rem` } : {}), ...staggerStyle }}>
        <ToolCallMessage message={message} hasResult={hasResult} />
      </div>
    );
  }

  // Render thinking blocks (only when thinking level is not Off)
  if (message.role === 'thinking') {
    if (thinkingLevel === 'Off') return null;
    const isLastThinking = isLoading && isLastMessage;
    return <ThinkingBlock content={message.content} level={message.metadata?.level} isActive={isLastThinking} />;
  }

  const isUser = message.role === 'user';
  const isOptimistic = message.isOptimistic === true;

  return (
    <div className="animate-slide-up" style={staggerStyle}>
      {isUser ? (
        <div className={`bg-surface-2 border border-border-default/15 rounded-lg px-4 py-3${isOptimistic ? ' opacity-70' : ''}`}>
          <div className="flex items-start gap-3">
            <span className="text-accent-primary font-mono text-sm font-bold flex-shrink-0">#</span>
            <div className="flex-1 prose prose-sm max-w-none code-hover">
              <MarkdownContent content={message.content} />
            </div>
          </div>
        </div>
      ) : (
        <div className="bg-surface-primary border border-border-default/15 rounded-lg px-4 py-3">
          <div className="flex items-start gap-3">
            <span className="text-content-tertiary font-mono text-sm font-medium flex-shrink-0">&#10095;</span>
            <div className="flex-1 prose prose-sm max-w-none code-hover">
              <MarkdownContent content={message.content} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
});
