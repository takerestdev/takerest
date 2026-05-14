import { invoke } from "@tauri-apps/api/core";

/** @param {string} projectPath @returns {Promise<void>} */
export async function watchProject(projectPath) {
  return invoke("watch_project", { projectPath });
}

/** @returns {Promise<void>} */
export async function unwatchProject() {
  return invoke("unwatch_project");
}
