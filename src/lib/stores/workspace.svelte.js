// @ts-nocheck

function createWorkspace() {
  let activeTool = $state(null);
  let sidebarOpen = $state(false);
  let tabs = $state([]);
  let activeTabId = $state(null);
  let folderPath = $state('');
  let envFilesVersion = $state(0);
  let gitInfo = $state(null);
  let fileChangeTicks = $state({});
  let gitRefreshTick = $state(0);
  let worktreeChangeTick = $state(0);
  let dockerRefreshTick = $state(0);
  let dirtyTabIds = $state(new Set());

  return {
    get activeTool() { return activeTool; },
    set activeTool(v) { activeTool = v; },

    get sidebarOpen() { return sidebarOpen; },
    set sidebarOpen(v) { sidebarOpen = v; },

    get tabs() { return tabs; },

    get activeTabId() { return activeTabId; },
    set activeTabId(v) { activeTabId = v; },

    get folderPath() { return folderPath; },
    set folderPath(v) { folderPath = v; },

    get envFilesVersion() { return envFilesVersion; },

    get gitInfo() { return gitInfo; },
    set gitInfo(v) { gitInfo = v; },

    get fileChangeTicks() { return fileChangeTicks; },
    get gitRefreshTick() { return gitRefreshTick; },
    get worktreeChangeTick() { return worktreeChangeTick; },
    get dockerRefreshTick() { return dockerRefreshTick; },

    get dirtyTabIds() { return dirtyTabIds; },

    setTabDirty(id, dirty) {
      const next = new Set(dirtyTabIds);
      if (dirty) next.add(id); else next.delete(id);
      dirtyTabIds = next;
    },

    openTab(tab) {
      const existing = tabs.find(t => t.id === tab.id);
      if (existing) {
        activeTabId = tab.id;
        return;
      }
      tabs = [...tabs, tab];
      activeTabId = tab.id;
    },

    closeTab(id) {
      const idx = tabs.findIndex(t => t.id === id);
      if (idx === -1) return;
      const newTabs = tabs.filter(t => t.id !== id);
      tabs = newTabs;
      if (activeTabId === id) {
        activeTabId = newTabs.length > 0
          ? newTabs[Math.min(idx, newTabs.length - 1)].id
          : null;
      }
      // clear any dirty marker for the closed tab
      if (dirtyTabIds.has(id)) {
        const next = new Set(dirtyTabIds);
        next.delete(id);
        dirtyTabIds = next;
      }
    },

    setActiveTool(tool) {
      if (activeTool === tool) {
        sidebarOpen = !sidebarOpen;
      } else {
        activeTool = tool;
        sidebarOpen = true;
      }
    },

    renameTab(id, title) {
      const tab = tabs.find(t => t.id === id);
      if (tab) tab.title = title;
    },

    refreshEnvFiles() {
      envFilesVersion++;
    },

    bumpFileTick(rel) {
      fileChangeTicks[rel] = (fileChangeTicks[rel] ?? 0) + 1;
    },

    bumpGit() {
      gitRefreshTick++;
    },

    bumpWorktree() {
      worktreeChangeTick++;
    },

    bumpDocker() {
      dockerRefreshTick++;
    },
  };
}

export const workspace = createWorkspace();
