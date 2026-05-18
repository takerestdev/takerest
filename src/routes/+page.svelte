<script>
  // @ts-nocheck
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Resizable from "$lib/components/ui/resizable/index.js";
  import { mode } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { invoke } from "@tauri-apps/api/core";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    CardDescription,
  } from "$lib/components/ui/card/index.js";
  import * as Empty from "$lib/components/ui/empty/index.js";
  import { FolderOpen, ArrowUpRight } from "@lucide/svelte";
  import { openPath, openUrl } from "@tauri-apps/plugin-opener";
  import Themetoggle from "$lib/components/themetoggle.svelte";
  import { Minus, Square, X } from "@lucide/svelte";
  // ── Stats ───────────────────────────────────────────────
  let wins = 0;
  let losses = 0;
  let draws = 0;

  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { platform } from "@tauri-apps/plugin-os";
  import Logo from "$lib/components/logo.svelte";

  let isWindows = false;
  let appWindow;

  onMount(async () => {
    wins = parseInt(localStorage.getItem("ttt_wins") ?? "0");
    losses = parseInt(localStorage.getItem("ttt_losses") ?? "0");
    draws = parseInt(localStorage.getItem("ttt_draws") ?? "0");
    appWindow = getCurrentWindow();
    isWindows = (await platform()) === "windows";
  });

  async function isWindowsPlatform(params) {
    return (await platform()) === "windows";
  }

  async function openFolder() {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Choose a project folder",
      });
      if (!selected) return;
      const label = `folder-${Date.now()}`;
      const encoded = encodeURIComponent(selected);
      const win = new WebviewWindow(label, {
        url: `app?path=${encoded}`,
        title: selected.split(/[\\/]/).filter(Boolean).pop() ?? "Project",
        width: 1200,
        height: 800,
        transparent: true,
        decorations: false,
        resizable: true,
        focus: true,
      });

      win.once("tauri://created", () => {
        getCurrentWindow().close();
      });
      win.once("tauri://error", (e) => {
        console.error("Window creation error:", e);
        toast.error("Failed to open folder", {
          description: "Could not create the window.",
        });
      });
    } catch (e) {
      console.error("Failed to open window:", e);
      toast.error("Failed to open folder", {
        description: "An unexpected error occurred.",
      });
    }
  }

  function saveStats() {
    localStorage.setItem("ttt_wins", String(wins));
    localStorage.setItem("ttt_losses", String(losses));
    localStorage.setItem("ttt_draws", String(draws));
  }

  // ── Game state ──────────────────────────────────────────
  let board = Array(9).fill(null);
  let winner = null;
  let isDraw = false;
  let gameOver = false;
  let thinking = false;
  let aiTimeout = null;

  // ── Win check ───────────────────────────────────────────
  const LINES = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
  ];

  function checkWinner(b) {
    for (const [a, b1, c] of LINES) {
      if (b[a] && b[a] === b[b1] && b[a] === b[c]) return b[a];
    }
    return null;
  }

  // ── Minimax ─────────────────────────────────────────────
  function minimax(b, isMax) {
    const w = checkWinner(b);
    if (w === "O") return 10;
    if (w === "X") return -10;
    if (b.every(Boolean)) return 0;

    let best = isMax ? -Infinity : Infinity;
    for (let i = 0; i < 9; i++) {
      if (!b[i]) {
        b[i] = isMax ? "O" : "X";
        const val = minimax(b, !isMax);
        b[i] = null;
        best = isMax ? Math.max(best, val) : Math.min(best, val);
      }
    }
    return best;
  }

  function getBestMove(b) {
    let bestVal = -Infinity,
      bestMove = -1;
    for (let i = 0; i < 9; i++) {
      if (!b[i]) {
        b[i] = "O";
        const val = minimax(b, false);
        b[i] = null;
        if (val > bestVal) {
          bestVal = val;
          bestMove = i;
        }
      }
    }
    return bestMove;
  }

  // ── Player move ─────────────────────────────────────────
  function play(i) {
    if (board[i] || gameOver || thinking) return;

    board[i] = "X";
    board = [...board];

    winner = checkWinner(board);
    if (winner === "X") {
      gameOver = true;
      wins++;
      saveStats();
      toast.success("🎉 You won!", {
        description: "Minimax couldn't stop you!",
      });
      return;
    }
    if (board.every(Boolean)) {
      isDraw = true;
      gameOver = true;
      draws++;
      saveStats();
      toast.info("🤝 It's a draw!", {
        description: "Neither side could break through.",
      });
      return;
    }

    thinking = true;
    aiTimeout = setTimeout(() => {
      const move = getBestMove([...board]);
      if (move !== -1 && !gameOver) {
        board[move] = "O";
        board = [...board];
        winner = checkWinner(board);
        if (winner === "O") {
          gameOver = true;
          losses++;
          saveStats();
          toast.error("🤖 AI wins!", { description: "Better luck next time." });
        } else if (board.every(Boolean)) {
          isDraw = true;
          gameOver = true;
          draws++;
          saveStats();
          toast.info("🤝 It's a draw!", {
            description: "Neither side could break through.",
          });
        }
      }
      thinking = false;
    }, 400);
  }

  // ── Reset — cancel pending AI move first ─────────────────
  function reset() {
    console.log("Resetting game...");
    console.log(aiTimeout);
    if (aiTimeout !== null) {
      clearTimeout(aiTimeout);
      aiTimeout = null;
    }
    board = Array(9).fill(null);
    winner = null;
    isDraw = false;
    gameOver = false;
    thinking = false;
    toast("Game reset", { description: "Board cleared. Your turn! ✕" });
  }

  async function learnMore() {
    await openUrl("https://anide.app");
  }

  // ── Status label ────────────────────────────────────────
  $: status = winner
    ? winner === "X"
      ? "🎉 You won!"
      : "🤖 AI wins!"
    : isDraw
      ? "🤝 It's a draw!"
      : thinking
        ? "🤔 AI is thinking…"
        : "Your turn  ✕";
</script>

<div class="h-screen w-full flex flex-col text-foreground bg-background">
  <!-- ── HEADER ── -->
  <header
    data-tauri-drag-region
    class="grid grid-cols-3 w-full items-center border-b shrink-0"
  >
    <!-- Left: empty spacer -->
    <div></div>

    <!-- Center: logo + name -->
    <div
      data-tauri-drag-region
      class="flex items-center px-4 py-2.5 justify-center gap-2.5"
    >
      <Logo />
      <span class="font-semibold text-sm tracking-tight">anide</span>
    </div>

    <!-- Right: Windows controls -->
    <div data-tauri-drag-region class="flex items-center justify-end">
      {#if isWindows}
        <button
          aria-label="Minimize window"
          type="button"
          class="w-11 h-9 flex items-center justify-center hover:bg-muted transition-colors"
          onclick={() => appWindow.minimize()}
          title="Minimize"
        >
          <Minus size={14} />
        </button>
        <button
          aria-label="Maximize window"
          type="button"
          class="w-11 h-9 flex items-center justify-center hover:bg-muted transition-colors"
          onclick={() => appWindow.toggleMaximize()}
          title="Maximize"
        >
          <Square size={14} />
        </button>
        <button
          aria-label="Close window"
          type="button"
          class="w-11 h-9 flex items-center justify-center hover:bg-red-500 hover:text-white transition-colors"
          onclick={() => appWindow.close()}
          title="Close"
        >
          <X size={14} />
        </button>
      {/if}
    </div>
  </header>

  <!-- ── PANELS ── -->
  <Resizable.PaneGroup direction="horizontal" class="flex-1 overflow-hidden">
    <!-- LEFT — Empty state -->
    <Resizable.Pane defaultSize={50} minSize={25}>
      <div class="h-full flex items-center justify-center p-8 border-r">
        <Empty.Root class="border border-dashed w-full max-w-sm">
          <Empty.Header>
            <Empty.Media variant="icon">
              <FolderOpen />
            </Empty.Media>
            <Empty.Title>No Recent Folders</Empty.Title>
            <Empty.Description>
              Open a project folder to get started with your data workflows.
            </Empty.Description>
          </Empty.Header>
          <Empty.Content>
            <div class="flex flex-col gap-2 w-full">
              <Button class="w-full" onclick={openFolder}>Open Folder</Button>
              <Button
                onclick={learnMore}
                variant="outline"
                class="w-full gap-1"
              >
                Learn More <ArrowUpRight class="size-3.5" />
              </Button>
            </div>
          </Empty.Content>
        </Empty.Root>
      </div>
    </Resizable.Pane>

    <Resizable.Handle withHandle />

    <!-- RIGHT — Tic Tac Toe -->
    <Resizable.Pane defaultSize={50} minSize={25}>
      <div class="h-full flex items-center justify-center p-8">
        <Card class="w-full max-w-xs">
          <CardHeader>
            <CardTitle class="flex items-center justify-between">
              <p>Tic Tac Toe</p>
              <Themetoggle />
            </CardTitle>
            <CardDescription>{status}</CardDescription>
          </CardHeader>

          <CardContent class="flex flex-col gap-4">
            <!-- Score row -->
            <div
              class="grid grid-cols-3 text-center rounded-lg border divide-x overflow-hidden"
            >
              <div class="py-2">
                <p class="text-lg font-bold text-green-500">{wins}</p>
                <p class="text-[11px] text-muted-foreground">Wins</p>
              </div>
              <div class="py-2">
                <p class="text-lg font-bold text-red-500">{losses}</p>
                <p class="text-[11px] text-muted-foreground">Losses</p>
              </div>
              <div class="py-2">
                <p class="text-lg font-bold text-yellow-500">{draws}</p>
                <p class="text-[11px] text-muted-foreground">Draws</p>
              </div>
            </div>

            <!-- Board -->
            <div class="grid grid-cols-3 gap-1.5">
              {#each board as cell, i}
                <button
                  aria-label={`Row ${Math.floor(i / 3) + 1}, column ${(i % 3) + 1}: ${cell ?? "empty"}`}
                  class="h-16 text-2xl font-bold border rounded-md flex items-center justify-center
                       transition-colors hover:bg-muted
                       disabled:opacity-40 disabled:cursor-not-allowed
                       {cell === 'X' ? 'text-blue-500' : 'text-red-400'}"
                  onclick={() => play(i)}
                  disabled={!!cell || gameOver || thinking}
                >
                  {cell ?? ""}
                </button>
              {/each}
            </div>

            <Button class="w-full" onclick={reset}>Reset Game</Button>
          </CardContent>
        </Card>
      </div>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</div>
