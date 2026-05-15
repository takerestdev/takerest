import { invoke } from '@tauri-apps/api/core';

export const readProjectFile = (projectPath, relPath) =>
  invoke('read_project_file', { projectPath, relPath });

export const writeProjectFile = (projectPath, relPath, content) =>
  invoke('write_project_file', { projectPath, relPath, content });
