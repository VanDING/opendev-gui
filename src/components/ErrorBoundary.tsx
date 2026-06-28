import React, { type ReactNode } from 'react';
import { Button } from './ui/Button';

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  errorInfo?: React.ErrorInfo;
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // Log to console for debugging
    console.error('[ErrorBoundary] Caught error:', error.message, errorInfo.componentStack);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center h-screen bg-surface-primary">
          <div className="text-center max-w-md px-6">
            <h1 className="text-xl font-semibold text-content-primary mb-2">Something went wrong</h1>
            <p className="text-sm text-content-secondary mb-4">Please refresh the page to continue.</p>
            {this.state.error && (
              <details className="text-xs text-left mb-4 bg-surface-secondary p-3 rounded-md overflow-auto max-h-32">
                <summary className="cursor-pointer text-content-secondary mb-1">Error details</summary>
                <pre className="text-red-500 whitespace-pre-wrap">{this.state.error.message}</pre>
              </details>
            )}
            <div className="flex gap-3 justify-center">
              <Button variant="primary" onClick={() => window.location.reload()}>
                Refresh
              </Button>
              <Button variant="secondary" onClick={() => this.setState({ hasError: false, error: undefined })}>
                Try again
              </Button>
            </div>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
