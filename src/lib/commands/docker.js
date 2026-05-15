import { invoke } from '@tauri-apps/api/core';

export const dockerListContainers = () =>
  invoke('docker_list_containers');

export const dockerListImages = () =>
  invoke('docker_list_images');

export const dockerListComposeFiles = (projectPath) =>
  invoke('docker_list_compose_files', { projectPath });

export const dockerContainerStart = (containerId) =>
  invoke('docker_container_start', { containerId });

export const dockerContainerStop = (containerId) =>
  invoke('docker_container_stop', { containerId });

export const dockerContainerRestart = (containerId) =>
  invoke('docker_container_restart', { containerId });

export const dockerContainerRemove = (containerId, force = false) =>
  invoke('docker_container_remove', { containerId, force });

export const dockerImageRemove = (imageId, force = false) =>
  invoke('docker_image_remove', { imageId, force });

export const dockerStartLogStream = (containerId, tail = 2000) =>
  invoke('docker_start_log_stream', { containerId, tail });

export const dockerStopLogStream = (containerId) =>
  invoke('docker_stop_log_stream', { containerId });

export const dockerComposeUp = (projectPath, composeRelPath) =>
  invoke('docker_compose_up', { projectPath, composeRelPath });

export const dockerComposeDown = (projectPath, composeRelPath) =>
  invoke('docker_compose_down', { projectPath, composeRelPath });

export const dockerPing = () =>
  invoke('docker_ping');

export const dockerStartEngine = () =>
  invoke('docker_start_engine');

export const dockerStopEngine = () =>
  invoke('docker_stop_engine');