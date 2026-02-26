export interface ProxyStatus {
  running: boolean;
  port: number;
}

export interface ManagedConfigPreview {
  toolId: string;
  toolName: string;
  configPath: string;
  content: string;
  strategy: string;
}
