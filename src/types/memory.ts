export interface MemoryStatus {
  enabled: boolean;
  serverStatus: string | null;
  uvxAvailable: boolean;
  dockerAvailable: boolean;
  redisRunning: boolean;
  ollamaRunning: boolean;
  embeddingProvider: string;
  embeddingModel: string;
  error: string | null;
}
