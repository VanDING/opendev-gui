import type { Transport } from './Transport';

export function createFileRepository(transport: Transport) {
  return {
    browseDirectory: (path: string, show_hidden?: boolean) =>
      transport.invoke<any>('browse_directory', {
        req: { path, show_hidden },
      }),
    verifyPath: (path: string) =>
      transport.invoke<any>('verify_path', { req: { path } }),
    listFiles: (query?: string) =>
      transport.invoke<{ files: any[] }>('list_workspace_files', { query }),
  };
}
