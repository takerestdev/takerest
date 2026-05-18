<script>
  // @ts-nocheck
  import { onMount, onDestroy } from 'svelte';
  import { toast } from 'svelte-sonner';
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import Themetoggle from '$lib/components/themetoggle.svelte';

  let wins = $state(0), losses = $state(0), draws = $state(0);
  let board = $state(Array(9).fill(null));
  let winner = $state(null);
  let isDraw = $state(false);
  let gameOver = $state(false);
  let thinking = $state(false);
  let aiTimeout = null;

  onMount(() => {
    wins   = parseInt(localStorage.getItem('ttt_wins')   ?? '0');
    losses = parseInt(localStorage.getItem('ttt_losses') ?? '0');
    draws  = parseInt(localStorage.getItem('ttt_draws')  ?? '0');
  });

  onDestroy(() => { if (aiTimeout !== null) clearTimeout(aiTimeout); });

  function save() {
    localStorage.setItem('ttt_wins',   String(wins));
    localStorage.setItem('ttt_losses', String(losses));
    localStorage.setItem('ttt_draws',  String(draws));
  }

  const LINES = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];

  function checkWinner(b) {
    for (const [a,b1,c] of LINES) if (b[a] && b[a] === b[b1] && b[a] === b[c]) return b[a];
    return null;
  }

  function minimax(b, isMax) {
    const w = checkWinner(b);
    if (w === 'O') return 10;
    if (w === 'X') return -10;
    if (b.every(Boolean)) return 0;
    let best = isMax ? -Infinity : Infinity;
    for (let i = 0; i < 9; i++) {
      if (!b[i]) {
        b[i] = isMax ? 'O' : 'X';
        const val = minimax(b, !isMax);
        b[i] = null;
        best = isMax ? Math.max(best, val) : Math.min(best, val);
      }
    }
    return best;
  }

  function getBestMove(b) {
    let bestVal = -Infinity, bestMove = -1;
    for (let i = 0; i < 9; i++) {
      if (!b[i]) {
        b[i] = 'O';
        const val = minimax(b, false);
        b[i] = null;
        if (val > bestVal) { bestVal = val; bestMove = i; }
      }
    }
    return bestMove;
  }

  function play(i) {
    if (board[i] || gameOver || thinking) return;
    board[i] = 'X'; board = [...board];
    winner = checkWinner(board);
    if (winner === 'X') { gameOver = true; wins++; save(); toast.success('🎉 You won!'); return; }
    if (board.every(Boolean)) { isDraw = true; gameOver = true; draws++; save(); toast.info("🤝 It's a draw!"); return; }
    thinking = true;
    aiTimeout = setTimeout(() => {
      const move = getBestMove([...board]);
      if (move !== -1 && !gameOver) {
        board[move] = 'O'; board = [...board];
        winner = checkWinner(board);
        if (winner === 'O') { gameOver = true; losses++; save(); toast.error('🤖 AI wins!'); }
        else if (board.every(Boolean)) { isDraw = true; gameOver = true; draws++; save(); toast.info("🤝 It's a draw!"); }
      }
      thinking = false;
    }, 400);
  }

  function reset() {
    if (aiTimeout !== null) { clearTimeout(aiTimeout); aiTimeout = null; }
    board = Array(9).fill(null); winner = null; isDraw = false; gameOver = false; thinking = false;
    toast('Game reset', { description: "Board cleared. Your turn! ✕" });
  }

  let status = $derived(
    winner ? (winner === 'X' ? '🎉 You won!' : '🤖 AI wins!')
    : isDraw ? "🤝 It's a draw!"
    : thinking ? '🤔 AI is thinking…'
    : 'Your turn  ✕'
  );
</script>

<Card class="w-full max-w-xs">
  <CardHeader>
    <CardTitle class="flex items-center justify-between">
      <p>Tic Tac Toe</p>
      <Themetoggle />
    </CardTitle>
    <CardDescription>{status}</CardDescription>
  </CardHeader>
  <CardContent class="flex flex-col gap-4">
    <div class="grid grid-cols-3 text-center rounded-lg border divide-x overflow-hidden">
      <div class="py-2"><p class="text-lg font-bold text-green-500">{wins}</p><p class="text-[11px] text-muted-foreground">Wins</p></div>
      <div class="py-2"><p class="text-lg font-bold text-red-500">{losses}</p><p class="text-[11px] text-muted-foreground">Losses</p></div>
      <div class="py-2"><p class="text-lg font-bold text-yellow-500">{draws}</p><p class="text-[11px] text-muted-foreground">Draws</p></div>
    </div>
    <div class="grid grid-cols-3 gap-1.5">
      {#each board as cell, i}
        <button
          aria-label="Cell {i}"
          class="h-16 text-2xl font-bold border rounded-md flex items-center justify-center transition-colors hover:bg-muted disabled:opacity-40 disabled:cursor-not-allowed {cell === 'X' ? 'text-blue-500' : 'text-red-400'}"
          onclick={() => play(i)}
          disabled={!!cell || gameOver || thinking}
        >{cell ?? ''}</button>
      {/each}
    </div>
    <Button class="w-full" onclick={reset}>Reset Game</Button>
  </CardContent>
</Card>
