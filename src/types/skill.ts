export interface SkillInfo {
  id: string;
  name: string;
  description: string;
  pluginName: string;
  pluginAuthor: string;
  version?: string;
  tools?: string;
}

export interface SkillDetail {
  info: SkillInfo;
  content: string;
}
