import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useContext } from "react";

import { WebsocketHandler } from "./WebsocketContext";
const Dashboard = () => {
  const { loaded } = useContext(WebsocketHandler);
  if (!loaded) {
    return (
      <div>
        <p>Not Loaded right now</p>
      </div>
    );
  }
  return (
    <div className="p-6 text-slate-100">
      <h1 className="mb-4 text-2xl font-bold">Dashboard</h1>
      <WalletMultiButton />

      {/**
       * Dummy sign + send transaction flow (example):
       *
       * import { useWallet, useConnection } from "@solana/wallet-adapter-react";
       * import { SystemProgram, Transaction, LAMPORTS_PER_SOL } from "@solana/web3.js";
       *
       * const { connection } = useConnection();
       * const { publicKey, sendTransaction } = useWallet();
       *
       * async function handleSignAndSendDummyTx() {
       *   if (!publicKey) throw new Error("Connect wallet first");
       *
       *   const tx = new Transaction().add(
       *     SystemProgram.transfer({
       *       fromPubkey: publicKey,
       *       toPubkey: publicKey,
       *       lamports: 1, // tiny self-transfer for testing
       *     }),
       *   );
       *
       *   const signature = await sendTransaction(tx, connection);
       *   await connection.confirmTransaction(signature, "confirmed");
       *   console.log("Dummy tx confirmed:", signature);
       * }
       */}
    </div>
  );
};

export default Dashboard;
