import React from 'react'
import ReactDOM from 'react-dom/client'
import { Toaster } from 'sonner'
import App from './App.tsx'
import './index.css'
import { ThemeProvider } from './contexts/ThemeContext'
import { ErrorBoundary } from './components/ErrorBoundary'

// Tauri IPC is always available — no connection step needed.
// Events arrive via Tauri listen(), requests via Tauri invoke().
// The legacy HTTP/WebSocket bridge is handled transparently.

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <ThemeProvider>
        <App />
      </ThemeProvider>
    </ErrorBoundary>
    <Toaster
      theme="dark"
      position="bottom-right"
      duration={4000}
      toastOptions={{
        classNames: {
          toast: '!bg-surface-floating !border-border-default !text-content-primary !rounded-lg !shadow-popover',
          success: '!border-l-intent-success',
          error: '!border-l-intent-danger',
          warning: '!border-l-intent-warning',
          info: '!border-l-accent-primary',
        },
      }}
    />
  </React.StrictMode>,
)
