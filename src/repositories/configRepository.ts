import type { Transport } from './Transport';

export function createConfigRepository(transport: Transport) {
  return {
    getConfig: () => transport.invoke<any>('get_app_config'),
    updateConfig: (config: any) => transport.invoke<void>('update_app_config', { req: config }),
    listProviders: () => transport.invoke<any[]>('list_model_providers'),
    verifyModel: (provider: string, model: string) =>
      transport.invoke<{ valid: boolean; error?: string }>('verify_model', {
        req: { provider, model },
      }),
    setMode: (mode: string) =>
      transport.invoke<void>('set_operation_mode', { req: { mode } }),
    setAutonomy: (level: string) =>
      transport.invoke<void>('set_autonomy_level', { req: { level } }),
  };
}
