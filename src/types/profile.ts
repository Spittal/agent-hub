export interface ProfileFeatures {
  memory: boolean;
  memoryDb?: number;
  discovery: boolean;
}

export interface Profile {
  id: string;
  name: string;
  features: ProfileFeatures;
  integrationIds: string[];
  serverIds: string[];
  skillIds: string[];
  pluginIds: string[];
  directoryPaths: string[];
  sortOrder: number;
}

export interface CreateProfileInput {
  name: string;
  integrationIds?: string[];
  serverIds?: string[];
  skillIds?: string[];
  pluginIds?: string[];
  features?: ProfileFeatures;
}

export interface UpdateProfileInput {
  name?: string;
  features?: ProfileFeatures;
  integrationIds?: string[];
  serverIds?: string[];
  skillIds?: string[];
  pluginIds?: string[];
  directoryPaths?: string[];
  sortOrder?: number;
}

export interface ProfileExportServer {
  name: string;
  transport: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  headers?: Record<string, string>;
}

export interface ProfileExportSkill {
  name: string;
  skillId: string;
  source: string;
}

export interface ProfileExport {
  agentHubProfile: string;
  name: string;
  features: ProfileFeatures;
  integrations: string[];
  servers: ProfileExportServer[];
  skills: ProfileExportSkill[];
  plugins: string[];
  directoryPaths: string[];
}

export interface ImportResult {
  profile: Profile;
  matchedServers: number;
  createdServers: number;
  matchedSkills: number;
  unmatchedSkills: string[];
  matchedPlugins: number;
  unmatchedPlugins: string[];
}
