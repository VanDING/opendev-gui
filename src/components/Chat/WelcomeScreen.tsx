import { MatrixRain } from './MatrixRain';

export function WelcomeScreen() {
  return (
    <div className="relative flex flex-col items-center justify-center h-full px-6 bg-surface-elevated overflow-hidden">
      <MatrixRain />

      <div className="relative z-10 mt-10 text-center max-w-md">
        <p className="text-sm text-content-secondary font-mono">
          Start a conversation with your AI coding assistant
        </p>
      </div>
    </div>
  );
}
