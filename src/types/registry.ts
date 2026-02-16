export interface RegistryServerSummary {
  id: string;
  displayName: string;
  description?: string;
  version?: string;
  iconUrl?: string;
  transportTypes: string[];
  registryType?: string;
  requiresConfig: boolean;
  hasRemote: boolean;
  repositoryUrl?: string;
  installed: boolean;
  stars?: number;
}

export interface RegistrySearchResult {
  servers: RegistryServerSummary[];
  hasMore: boolean;
}

export interface MarketplaceServerDetail {
  id: string;
  name: string;
  description?: string;
  repositoryUrl?: string;
  stars?: number;
  version?: string;
  command?: string;
  args: string[];
  envVars: MarketplaceEnvVar[];
  runtime?: string;
}

export interface MarketplaceEnvVar {
  name: string;
  defaultValue: string;
  isRequired: boolean;
  isSecret: boolean;
}

export interface RuntimeDeps {
  npx: boolean;
  uvx: boolean;
  docker: boolean;
}
