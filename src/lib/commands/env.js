import { invoke } from "@tauri-apps/api/core";

/**
 * @typedef {Object} EnvFile
 * @property {string} name        - Filename only, e.g. ".env.local"
 * @property {string} relPath     - Relative path from project root, e.g. "backend/.env.local"
 * @property {string} absPath     - Absolute path on disk
 * @property {boolean} inGitignore - Whether this file is listed in .gitignore
 */

/**
 * @typedef {Object} EnvFileContent
 * @property {string} relPath  - Relative path from project root
 * @property {string} content  - Raw text content of the file
 */

/**
 * @typedef {Object} CreateEnvFilePayload
 * @property {string}  relPath        - e.g. ".env.local" or "backend/.env.staging"
 * @property {string}  content        - Initial file content (can be empty string)
 * @property {boolean} addToGitignore - Whether to immediately add to .gitignore
 */

// ── Queries ───────────────────────────────────────────────────────────────

/**
 * List all .env* files found anywhere in the project.
 * Skips node_modules, .git, target, dist, .next, .svelte-kit, .takerest.
 *
 * @param {string} projectPath - Absolute path to the project folder
 * @returns {Promise<EnvFile[]>}
 */
export async function listEnvFiles(projectPath) {
  return invoke("list_env_files", { projectPath });
}

/**
 * Read the content of a specific .env file.
 *
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} relPath     - Relative path to the file from project root
 * @returns {Promise<EnvFileContent>}
 */
export async function readEnvFile(projectPath, relPath) {
  return invoke("read_env_file", { projectPath, relPath });
}

// ── Mutations ─────────────────────────────────────────────────────────────

/**
 * Overwrite the content of an existing .env file.
 * Creates parent directories if they don't exist.
 *
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} relPath     - Relative path to the file from project root
 * @param {string} content     - New file content
 * @returns {Promise<void>}
 */
export async function writeEnvFile(projectPath, relPath, content) {
  return invoke("write_env_file", { projectPath, relPath, content });
}

/**
 * Create a new .env file. Errors if the file already exists.
 * The filename must contain ".env".
 *
 * @param {string} projectPath          - Absolute path to the project folder
 * @param {CreateEnvFilePayload} payload
 * @returns {Promise<EnvFile>}          - The newly created file's metadata
 */
export async function createEnvFile(projectPath, payload) {
  return invoke("create_env_file", { projectPath, payload });
}

/**
 * Permanently delete a .env file from disk.
 * Does NOT automatically remove it from .gitignore.
 *
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} relPath     - Relative path to the file from project root
 * @returns {Promise<void>}
 */
export async function deleteEnvFile(projectPath, relPath) {
  return invoke("delete_env_file", { projectPath, relPath });
}

// ── .gitignore ────────────────────────────────────────────────────────────

/**
 * Add the given relative path to .gitignore.
 * Groups entries under a "# .env files" section.
 * No-op if the entry is already present.
 *
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} relPath     - Relative path to add, e.g. ".env.local"
 * @returns {Promise<void>}
 */
export async function addEnvToGitignore(projectPath, relPath) {
  return invoke("add_env_to_gitignore", { projectPath, relPath });
}

/**
 * Remove the given relative path from .gitignore.
 * Handles both "rel/path" and "/rel/path" variants.
 * No-op if the entry isn't present.
 *
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} relPath     - Relative path to remove, e.g. ".env.local"
 * @returns {Promise<void>}
 */
export async function removeEnvFromGitignore(projectPath, relPath) {
  return invoke("remove_env_from_gitignore", { projectPath, relPath });
}

/**
 * Toggle a .env file's presence in .gitignore.
 * Convenience wrapper — reads current state from the EnvFile object.
 *
 * @param {string}  projectPath - Absolute path to the project folder
 * @param {EnvFile} envFile     - The file to toggle (uses relPath + inGitignore)
 * @returns {Promise<void>}
 */
export async function toggleEnvGitignore(projectPath, envFile) {
  if (envFile.inGitignore) {
    return removeEnvFromGitignore(projectPath, envFile.relPath);
  } else {
    return addEnvToGitignore(projectPath, envFile.relPath);
  }
}