# Sample Code Snippet

### Setup (web3 + SPL Token utils)

```rust
// lib/solana.ts
import {
  Connection,
  PublicKey,
  clusterApiUrl,
  ConfirmedSignatureInfo,
  TransactionSignature,
  Commitment,
} from "@solana/web3.js";
import {
  getAssociatedTokenAddress,
  getAccount as getSplAccount,
} from "@solana/spl-token";

// ---- connection ----
export const connection = new Connection(clusterApiUrl("mainnet-beta"), {
  commitment: "confirmed",
});

// ---- KC mint (replace with your real KC mint) ----
export const KC_MINT = new PublicKey("KC11111111111111111111111111111111111111111");

// ---- helper: find user's KC ATA ----
export async function getKcAta(user: PublicKey) {
  return getAssociatedTokenAddress(KC_MINT, user, /* allowOwnerOffCurve */ false);
}

```

## send tx → track status → refresh KC balance

```rust
// hooks/useSendAndTrack.tsx
import { useCallback, useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { connection } from "../lib/solana";
import { getKcBalance, formatKc } from "../balances/kc";
import { watchSignature } from "../tx-status/subscribe";
import type { Transaction, TransactionSignature, PublicKey } from "@solana/web3.js";

type UiState = "idle" | "sending" | "pending" | "confirmed" | "finalized" | "error";

export function useSendAndTrackKC(user: PublicKey | null) {
  const { sendTransaction, publicKey } = useWallet();
  const [ui, setUi] = useState<UiState>("idle");
  const [sig, setSig] = useState<TransactionSignature | null>(null);
  const [kcRaw, setKcRaw] = useState<bigint | null>(null);
  const [err, setErr] = useState<string | null>(null);

  const refreshBalance = useCallback(async () => {
    if (!user) return;
    const bal = await getKcBalance(user);
    setKcRaw(bal);
  }, [user]);

  // call this with a prepared Transaction (swap, stake, etc.)
  const send = useCallback(
    async (tx: Transaction) => {
      if (!publicKey) {
        setErr("Connect wallet");
        return;
      }
      setErr(null);
      setUi("sending");
      try {
        // Best practice: set a recent blockhash ourselves
        const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;
        tx.feePayer = publicKey;

        const signature = await sendTransaction(tx, connection, {
          skipPreflight: false,
          maxRetries: 3,
        });
        setSig(signature);
        setUi("pending");

        // subscribe to status
        const unsub = watchSignature(signature, (state) => {
          if (state === "err") setUi("error");
          else setUi(state as UiState);
          if (state === "finalized" || state === "confirmed") {
            // refresh KC balance when the tx hits at least confirmed
            refreshBalance();
          }
        });

        // optional cleanup after finalized
        // (you can stop listening once finalized if you want)
        return () => unsub();
      } catch (e: any) {
        setErr(e?.message ?? "Failed to send transaction");
        setUi("error");
      }
    },
    [publicKey, sendTransaction, refreshBalance]
  );

  useEffect(() => {
    refreshBalance();
  }, [refreshBalance]);

  return {
    ui,           // 'idle' | 'sending' | 'pending' | 'confirmed' | 'finalized' | 'error'
    sig,          // recent signature
    kcRaw,        // bigint
    kcFmt: kcRaw != null ? formatKc(kcRaw) : null,
    err,
    send,
    refreshBalance,
  };
}

```

## Swap Complete” + staking status

```rust
// components/SwapAndStakePanel.tsx
import React, { useMemo } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { useSendAndTrackKC } from "../hooks/useSendAndTrack";

// Button triggers a dummy tx builder you provide
import { buildSwapOrStakeTx } from "../tx-builders/swapOrStake"; // <-- implement your own

export function SwapAndStakePanel() {
  const { publicKey } = useWallet();
  const { ui, sig, kcFmt, err, send } = useSendAndTrackKC(publicKey ?? null);

  const banner = useMemo(() => {
    if (ui === "finalized") return { text: "Swap Complete ✅", tone: "success" as const };
    if (ui === "confirmed") return { text: "Confirmed ✓ Waiting to finalize...", tone: "info" as const };
    if (ui === "pending") return { text: "Pending ⏳", tone: "info" as const };
    if (ui === "sending") return { text: "Sending…", tone: "info" as const };
    if (ui === "error") return { text: "Transaction failed", tone: "error" as const };
    return null;
  }, [ui]);

  const onSwapOrStake = async () => {
    if (!publicKey) return;
    const tx = await buildSwapOrStakeTx(publicKey); // your function builds a Transaction
    await send(tx);
  };

  return (
    <div className="max-w-md w-full rounded-2xl p-4 shadow border">
      <h3 className="text-lg font-semibold">KC Swap & Stake</h3>

      <div className="mt-2 text-sm text-gray-600">
        KC Balance: <span className="font-mono">{kcFmt ?? "—"}</span>
      </div>

      {banner && (
        <div
          className={`mt-3 rounded-md p-2 text-sm ${
            banner.tone === "success"
              ? "bg-green-50 text-green-700 border border-green-200"
              : banner.tone === "error"
              ? "bg-red-50 text-red-700 border border-red-200"
              : "bg-blue-50 text-blue-700 border border-blue-200"
          }`}
        >
          {banner.text}
          {sig && (
            <div className="mt-1 text-xs opacity-75 break-all">
              Sig: <code>{sig}</code>
            </div>
          )}
        </div>
      )}

      {err && <div className="mt-2 text-sm text-red-600">{err}</div>}

      <button
        onClick={onSwapOrStake}
        disabled={!publicKey || ui === "sending" || ui === "pending"}
        className="mt-4 w-full rounded-xl py-2 border shadow hover:shadow-md disabled:opacity-50"
      >
        {publicKey ? "Swap & Stake" : "Connect Wallet"}
      </button>

      {/* Example staking status block */}
      <div className="mt-4 text-sm">
        <div className="font-medium">Staking Status</div>
        <ul className="mt-1 list-disc pl-5">
          <li>Latest Tx: {sig ? <code className="break-all">{sig}</code> : "—"}</li>
          <li>State: <code>{ui}</code></li>
        </ul>
      </div>
    </div>
  );
}

```
