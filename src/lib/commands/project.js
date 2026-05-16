import { invoke } from "@tauri-apps/api/core";

/**
 * Ensures .anide/ folder exists in the project. Creates it if missing.
 * @param {string} projectPath - Absolute path to the project folder
 * @returns {Promise<boolean>} true if it already existed, false if just created
 */
export async function initProject(projectPath) {
  return invoke("init_project", { projectPath });
}

/**
 * Scans the project folder and returns aggregated info.
 * Uses .gitignore rules to skip ignored paths for performance.
 *
 * @param {string} projectPath - Absolute path to the project folder
 * @returns {Promise<ProjectInfo>}
 *
 * @typedef {Object} ProjectInfo
 * @property {boolean} anideInitialized - Whether .anide/ already existed
 * @property {string[]} envFiles - Relative paths to .env files
 * @property {string[]} composeFiles - Relative paths to docker-compose files
 * @property {GitInfo|null} git - Git repo info, or null if not a git repo
 * @property {FiletypeInfo|null} majorFiletype - Dominant file extension info
 *
 * @typedef {Object} GitInfo
 * @property {string} repoName - Repository name (folder name)
 * @property {string} branch - Current branch name or detached HEAD hash
 *
 * @typedef {Object} FiletypeInfo
 * @property {string} extension - Extension including dot, e.g. ".ts"
 * @property {number} count - Number of files with this extension
 */
export async function scanProject(projectPath) {
  return invoke("scan_project", { projectPath });
}


/**
 * Save the README.md file in .anide/ folder
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} content - Markdown content to save
 * @returns {Promise<void>}
 */
export async function saveReadme(projectPath, content) {
  return invoke("save_readme", { projectPath, content });
}