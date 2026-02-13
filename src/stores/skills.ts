import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { SkillInfo, SkillDetail } from '@/types/skill';

export const useSkillsStore = defineStore('skills', () => {
  const skills = ref<SkillInfo[]>([]);
  const selectedSkillId = ref<string | null>(null);
  const skillDetail = ref<SkillDetail | null>(null);

  async function loadSkills() {
    try {
      skills.value = await invoke<SkillInfo[]>('list_skills');
    } catch (e) {
      console.error('Failed to load skills:', e);
    }
  }

  async function selectSkill(id: string) {
    selectedSkillId.value = id;
    try {
      skillDetail.value = await invoke<SkillDetail>('get_skill_content', { id });
    } catch (e) {
      console.error('Failed to load skill detail:', e);
      skillDetail.value = null;
    }
  }

  function clearSelection() {
    selectedSkillId.value = null;
    skillDetail.value = null;
  }

  return {
    skills,
    selectedSkillId,
    skillDetail,
    loadSkills,
    selectSkill,
    clearSelection,
  };
});
