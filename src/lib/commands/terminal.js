import { invoke } from '@tauri-apps/api/core';

export const terminalListShells = () =>
  invoke('terminal_list_shells');

export const terminalCreate = (sessionId, cwd, cols, rows, shell = null, shellArgs = null) =>
  invoke('terminal_create', { sessionId, cwd, cols, rows, shell, shellArgs });

export const terminalWrite = (sessionId, data) =>
  invoke('terminal_write', { sessionId, data });

export const terminalResize = (sessionId, cols, rows) =>
  invoke('terminal_resize', { sessionId, cols, rows });

export const terminalClose = (sessionId) =>
  invoke('terminal_close', { sessionId });
