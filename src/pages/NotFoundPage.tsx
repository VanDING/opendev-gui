import { Link } from 'react-router-dom';
import { Button } from '../components/ui/Button';

export function NotFoundPage() {
  return (
    <div className="flex items-center justify-center h-screen bg-surface-primary">
      <div className="text-center">
        <h1 className="text-6xl font-bold text-content-primary mb-4">404</h1>
        <p className="text-lg text-content-secondary mb-6">Page not found</p>
        <Link to="/chat">
          <Button variant="secondary">Back to Chat</Button>
        </Link>
      </div>
    </div>
  );
}
