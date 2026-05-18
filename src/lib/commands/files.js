import { invoke } from '@tauri-apps/api/core';

export const readProjectFile = (projectPath, relPath) =>
  invoke('read_project_file', { projectPath, relPath });

export const writeProjectFile = (projectPath, relPath, content) =>
  invoke('write_project_file', { projectPath, relPath, content });

export const listDocFiles = (projectPath) =>
  invoke('list_doc_files', { projectPath });

export const deleteDocFile = (projectPath, relPath) =>
  invoke('delete_doc_file', { projectPath, relPath });
