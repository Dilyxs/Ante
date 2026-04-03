import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAccount,
} from "@solana/spl-token";

// encode number as 8-byte little-endian buffer
export function UInt64ToLE(n: number): Buffer {
  const buf = Buffer.alloc(8);
  let v = BigInt(n);
  for (let i = 0; i < 8; i++) {
    buf[i] = Number(v & BigInt(0xff));
    v >>= BigInt(8);
  }
  return buf;
}

// Ensure the provided owner ATA has at least `minAmount` tokens by calling
// the program.requestAnteTokens helper repeatedly. This function is test-only
// and mirrors the logic previously embedded in the main test file.
export async function ensureOwnerAtaHas(
  program: any,
  provider: anchor.AnchorProvider,
  ownerAta: anchor.web3.PublicKey,
  anteMintPda: anchor.web3.PublicKey,
  vaultAuthorityPda: anchor.web3.PublicKey,
  vaultAta: anchor.web3.PublicKey,
  configPda: anchor.web3.PublicKey,
  minAmount: number
) {
  const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
  const ASSOCIATED_TOKEN_PROGRAM = ASSOCIATED_TOKEN_PROGRAM_ID;

  let ownerAccount = await getAccount(provider.connection, ownerAta);
  let current = Number(ownerAccount.amount);
  while (current < minAmount) {
    // request a max chunk (8)
    await program.methods
      .requestAnteTokens(new anchor.BN(8))
      .accounts({
        admin: provider.wallet.publicKey,
        mint: anteMintPda,
        asker: provider.wallet.publicKey,
        askerAta: ownerAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    const refreshed = await getAccount(provider.connection, ownerAta);
    current = Number(refreshed.amount);
  }
}
