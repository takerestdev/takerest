<script>
  // @ts-nocheck
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
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

  // ── Stats ───────────────────────────────────────────────
  let wins = 0;
  let losses = 0;
  let draws = 0;

  onMount(() => {
    wins = parseInt(localStorage.getItem("ttt_wins") ?? "0");
    losses = parseInt(localStorage.getItem("ttt_losses") ?? "0");
    draws = parseInt(localStorage.getItem("ttt_draws") ?? "0");
  });

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
      return;
    }
    if (board.every(Boolean)) {
      isDraw = true;
      gameOver = true;
      draws++;
      saveStats();
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
        } else if (board.every(Boolean)) {
          isDraw = true;
          gameOver = true;
          draws++;
          saveStats();
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
  }

  async function learnMore() {
    await openUrl('https://takerest.dev');
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

<div class="h-screen w-full flex flex-col bg-background text-foreground">
  <!-- ── HEADER ── -->
  <header
    class="flex w-full justify-center items-center gap-2.5 px-4 py-2.5 border-b shrink-0"
  >
    <svg
      width="28"
      height="28"
      viewBox="0 0 1080 1080"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M525.004 627.079C576.926 574.106 546.103 359.464 487.74 302.287C429.377 245.111 340.009 241.837 288.086 294.811L326.483 332.427L364.879 370.043"
        fill="#AFF33E"
        stroke="#AFF33E"
        stroke-width="53.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M554.748 559.534C502.825 506.561 516.186 300.6 574.549 243.424C632.912 186.247 701.115 195.692 791.665 227.266L753.269 264.882L678.144 314.571"
        fill="#AFF33E"
        stroke="#AFF33E"
        stroke-width="53.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M554.93 501.632C554.93 427.464 488.807 367.27 407.095 367.27C325.382 367.27 259.26 427.464 259.26 501.632H313.018H366.776H474.293"
        fill="#AFF33E"
        stroke="#AFF33E"
        stroke-width="53.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M554.93 478.519C582.148 458.401 615.159 447.651 649.007 447.884C730.72 447.884 796.842 508.078 796.842 582.246H716.205H689.776H662.447H581.809"
        fill="#AFF33E"
        stroke="#AFF33E"
        stroke-width="53.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M499.547 373.69C520.226 346.899 549.039 327.536 581.662 318.509C660.261 296.174 740.326 336.001 760.608 407.343L683.043 429.384L657.621 436.608L631.333 444.078L553.768 466.119"
        fill="#AFF33E"
        stroke="#AFF33E"
        stroke-width="53.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M363.822 547.583C306.032 605.359 302 694.575 354.414 747.245L468.382 633.037L487.197 614.227L506.281 595.147L563.265 538.178C510.851 485.508 421.612 489.807 363.822 547.583Z"
        fill="#AFF33E"
        stroke="#AFF33E"
        stroke-width="53.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M547.866 501.658C564.015 582.381 477.782 889.125 450.975 953.703H580.163C644.757 776.114 564.015 566.236 547.866 501.658Z"
        fill="#AFF33E"
        stroke="#AFF33E"
        stroke-width="64.59"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <span class="font-semibold text-sm tracking-tight">takerest.dev</span>
  </header>

  <!-- ── PANELS ── -->
  <div class="flex flex-1 overflow-hidden">
    <!-- LEFT — Empty state -->
    <div class="w-1/2 border-r flex items-center justify-center p-8">
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
            <Button class="w-full">Open Folder</Button>
            <Button onclick={learnMore} variant="outline" class="w-full gap-1">
              Learn More <ArrowUpRight class="size-3.5" />
            </Button>
          </div>
        </Empty.Content>
      </Empty.Root>
    </div>

    <!-- RIGHT — Tic Tac Toe -->
    <div class="w-1/2 flex items-center justify-center p-8">
      <Card class="w-full max-w-xs">
        <CardHeader>
          <CardTitle>Tic Tac Toe</CardTitle>
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
                class="h-16 text-2xl font-bold border rounded-md flex items-center justify-center
                       transition-colors hover:bg-muted
                       disabled:opacity-40 disabled:cursor-not-allowed
                       {cell === 'X' ? 'text-blue-500' : 'text-red-400'}"
                on:click={() => play(i)}
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
  </div>
</div>
