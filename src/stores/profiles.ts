import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  Profile,
  CreateProfileInput,
  UpdateProfileInput,
  ProfileExport,
  ImportResult,
} from '@/types/profile';

export const useProfilesStore = defineStore('profiles', () => {
  const profiles = ref<Profile[]>([]);
  const loading = ref(false);

  async function loadProfiles() {
    loading.value = true;
    try {
      profiles.value = await invoke<Profile[]>('list_profiles');
    } catch (e) {
      console.error('Failed to load profiles:', e);
    } finally {
      loading.value = false;
    }
  }

  async function createProfile(input: CreateProfileInput): Promise<Profile> {
    const profile = await invoke<Profile>('create_profile', { input });
    profiles.value.push(profile);
    return profile;
  }

  async function updateProfile(id: string, input: UpdateProfileInput): Promise<Profile> {
    const updated = await invoke<Profile>('update_profile', { id, input });
    const idx = profiles.value.findIndex(p => p.id === id);
    if (idx !== -1) profiles.value[idx] = updated;
    return updated;
  }

  async function deleteProfile(id: string) {
    await invoke('delete_profile', { id });
    profiles.value = profiles.value.filter(p => p.id !== id);
  }

  async function addDirectory(profileId: string, directory: string): Promise<Profile> {
    const updated = await invoke<Profile>('add_directory_to_profile', { profileId, directory });
    const idx = profiles.value.findIndex(p => p.id === profileId);
    if (idx !== -1) profiles.value[idx] = updated;
    return updated;
  }

  async function removeDirectory(profileId: string, directory: string): Promise<Profile> {
    const updated = await invoke<Profile>('remove_directory_from_profile', { profileId, directory });
    const idx = profiles.value.findIndex(p => p.id === profileId);
    if (idx !== -1) profiles.value[idx] = updated;
    return updated;
  }

  async function exportProfile(profileId: string): Promise<ProfileExport> {
    return invoke<ProfileExport>('export_profile', { profileId });
  }

  async function importProfile(data: ProfileExport): Promise<ImportResult> {
    const result = await invoke<ImportResult>('import_profile', { data });
    profiles.value.push(result.profile);
    return result;
  }

  async function importProfileFromFile(path: string): Promise<ImportResult> {
    const result = await invoke<ImportResult>('import_profile_from_file', { path });
    profiles.value.push(result.profile);
    return result;
  }

  async function exportProfileToFile(profileId: string, path: string): Promise<void> {
    await invoke('export_profile_to_file', { profileId, path });
  }

  return {
    profiles,
    loading,
    loadProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    addDirectory,
    removeDirectory,
    exportProfile,
    importProfile,
    importProfileFromFile,
    exportProfileToFile,
  };
});
