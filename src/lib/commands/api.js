import { invoke } from "@tauri-apps/api/core";

/**
 * @typedef {Object} KVPair
 * @property {string} key
 * @property {string} value
 * @property {boolean} enabled
 */

/**
 * @typedef {Object} AuthNone
 * @property {"none"} type
 */

/**
 * @typedef {Object} AuthBasic
 * @property {"basic"} type
 * @property {string} username
 * @property {string} password
 */

/**
 * @typedef {Object} AuthBearer
 * @property {"bearer"} type
 * @property {string} token
 */

/**
 * @typedef {Object} AuthApiKey
 * @property {"apikey"} type
 * @property {string} key
 * @property {string} value
 * @property {string} addTo - "header" or "query"
 */

/**
 * @typedef {AuthNone | AuthBasic | AuthBearer | AuthApiKey} AuthConfig
 */

/**
 * @typedef {Object} RequestData
 * @property {string} method - HTTP method (GET, POST, PUT, etc.)
 * @property {string} url - URL with optional {{variable}} templates
 * @property {KVPair[]} headers
 * @property {KVPair[]} params
 * @property {AuthConfig} auth
 * @property {string} body - Markdown body content
 */

/**
 * @typedef {Object} RequestTreeFile
 * @property {"file"} type
 * @property {string} name - Display name (without .md)
 * @property {string} path - Relative path from .takerest/requests/
 * @property {string} method - HTTP method
 */

/**
 * @typedef {Object} RequestTreeFolder
 * @property {"folder"} type
 * @property {string} name - Folder name
 * @property {string} path - Relative path from .takerest/requests/
 * @property {RequestTreeNode[]} children
 */

/**
 * @typedef {RequestTreeFile | RequestTreeFolder} RequestTreeNode
 */

// ── Commands ─────────────────────────────────────────────────────────────

/**
 * Ensures .takerest/requests/ directory exists. Creates if missing.
 * @param {string} projectPath - Absolute path to the project folder
 * @returns {Promise<boolean>} true if it already existed, false if just created
 */
export async function initRequestsDir(projectPath) {
  return invoke("init_requests_dir", { projectPath });
}

/**
 * Returns the full request tree from .takerest/requests/.
 * Folders first (alphabetical), then files (alphabetical).
 * @param {string} projectPath - Absolute path to the project folder
 * @returns {Promise<RequestTreeNode[]>}
 */
export async function getRequestTree(projectPath) {
  return invoke("get_request_tree", { projectPath });
}

/**
 * Reads a single request file and returns parsed data.
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} requestPath - Relative path from .takerest/requests/, e.g. "auth/login.md"
 * @returns {Promise<RequestData>}
 */
export async function readRequest(projectPath, requestPath) {
  return invoke("read_request", { projectPath, requestPath });
}

/**
 * Creates a new request file. Automatically appends .md if missing.
 * Fails if the file already exists.
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} requestPath - Relative path, e.g. "auth/login.md"
 * @param {RequestData} data - Request data to write
 * @returns {Promise<void>}
 */
export async function createRequest(projectPath, requestPath, data) {
  return invoke("create_request", { projectPath, requestPath, data });
}

/**
 * Updates an existing request file. Overwrites the file.
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} requestPath - Relative path, e.g. "auth/login.md"
 * @param {RequestData} data - Updated request data
 * @returns {Promise<void>}
 */
export async function updateRequest(projectPath, requestPath, data) {
  return invoke("update_request", { projectPath, requestPath, data });
}

/**
 * Deletes a request file. Cleans up empty parent folders.
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} requestPath - Relative path, e.g. "auth/login.md"
 * @returns {Promise<void>}
 */
export async function deleteRequest(projectPath, requestPath) {
  return invoke("delete_request", { projectPath, requestPath });
}

/**
 * Duplicates a request file with auto-naming (login-copy.md, login-copy-2.md, etc.)
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} requestPath - Relative path of the request to duplicate
 * @returns {Promise<string>} The new file's relative path
 */
export async function duplicateRequest(projectPath, requestPath) {
  return invoke("duplicate_request", { projectPath, requestPath });
}

/**
 * Creates a collection (folder) inside .takerest/requests/.
 * Supports nested paths, e.g. "auth/admin" creates both auth/ and auth/admin/.
 * @param {string} projectPath - Absolute path to the project folder
 * @param {string} collectionPath - Relative folder path, e.g. "auth/admin"
 * @returns {Promise<void>}
 */
export async function createCollection(projectPath, collectionPath) {
  return invoke("create_collection", { projectPath, collectionPath });
}

// ── Helpers ──────────────────────────────────────────────────────────────

/**
 * Creates a default empty RequestData object.
 * @param {string} [method="GET"] - HTTP method
 * @returns {RequestData}
 */
export function createEmptyRequest(method = "GET") {
  return {
    method,
    url: "",
    headers: [],
    params: [],
    auth: { type: "none" },
    body: "",
  };
}
