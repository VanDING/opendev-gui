import type { Transport } from './Transport';

export function createMCPServerRepository(transport: Transport) {
  return {
    listServers: () =>
      transport.invoke<{ servers: any[] }>('list_mcp_servers'),
    getServer: (name: string) =>
      transport.invoke<any>('get_mcp_server', { name }),
    createServer: (server: {
      name: string;
      command: string;
      args?: string[];
      env?: Record<string, string>;
      enabled?: boolean;
      auto_start?: boolean;
    }) => transport.invoke<any>('create_mcp_server', { req: server }),
    updateServer: (name: string, update: any) =>
      transport.invoke<any>('update_mcp_server', { name, req: update }),
    deleteServer: (name: string) =>
      transport.invoke<any>('delete_mcp_server', { name }),
    connectServer: (name: string) =>
      transport.invoke<any>('connect_mcp_server', { name }),
    disconnectServer: (name: string) =>
      transport.invoke<any>('disconnect_mcp_server', { name }),
  };
}
