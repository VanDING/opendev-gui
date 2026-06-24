import { HaloSpinner } from '../ui/HaloSpinner';

export function WelcomeScreen() {
  return (
    <div className="relative flex items-center justify-center h-full px-6 bg-surface-elevated overflow-hidden">
      {/* Background watermark layer */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
        {/* "OpenDev" breathing text */}
        <span className="text-5xl md:text-7xl font-mono font-bold tracking-wider text-surface-3 animate-breathe select-none">
          OpenDev
        </span>
        {/* Orbiting braille halo ring */}
        <HaloSpinner />
      </div>
      {/* Foreground welcome content */}
      <div className="relative z-10 text-center">
        <div className="w-16 h-16 mx-auto mb-6 rounded-full bg-surface-2 flex items-center justify-center">
          <svg className="w-8 h-8 text-content-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
          </svg>
        </div>
        <h2 className="text-xl font-semibold text-content-primary mb-2">Welcome to OpenDev</h2>
        <p className="text-sm text-content-secondary">Start a conversation with your AI coding assistant</p>
      </div>
    </div>
  );
}
