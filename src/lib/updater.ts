import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { toast } from "svelte-sonner";

export async function checkForUpdates() {
	try {
		const update = await check();
		if (!update?.available) return;

		toast(`Update ${update.version} available`, {
			description: update.body ?? "A new version of Anide is ready.",
			action: {
				label: "Install & Restart",
				onClick: async () => {
					await update.downloadAndInstall();
					await relaunch();
				},
			},
			duration: Infinity,
		});
	} catch {
		// silently ignore — update check is best-effort
	}
}
