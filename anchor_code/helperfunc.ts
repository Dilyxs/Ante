import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";

//try to derive from os first os.process.PROGRAM_ID, make sure to have a loadEnv function enabled as well
const PROGRAM_ID = new anchor.web3.PublicKey(
  "R7tfA5vJNjYZJnEK9jy39Db8DiMwPtF95T8BiBnxbwd"
);

const u64ToLeBuffer = (n: number | bigint): Buffer => {
  const out = Buffer.alloc(8);
  let value = BigInt(n);

  for (let i = 0; i < 8; i += 1) {
    out[i] = Number(value & BigInt(0xff));
    value >>= BigInt(8);
  }

  return out;
};

export const initialize_var = async () => {
  //don't use anchor.AnchorProvider.env() instead initialize it manually by pointer to IDL_PATH once again it should be set it with env
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const owner = provider.wallet.publicKey;

  const [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    PROGRAM_ID
  );
  const [anteMintPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("Ante")],
    PROGRAM_ID
  );
  const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    PROGRAM_ID
  );
  const [vaultGlobalStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_global_state")],
    PROGRAM_ID
  );
  const [userBalancePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_balance_info"), owner.toBuffer()],
    PROGRAM_ID
  );

  const vaultAta = getAssociatedTokenAddressSync(
    anteMintPda,
    vaultAuthorityPda,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const ownerAta = getAssociatedTokenAddressSync(
    anteMintPda,
    owner,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  return {
    provider,
    owner,
    programId: PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,

    vaultAuthorityPda,
    anteMintPda,
    configPda,
    vaultGlobalStatePda,
    userBalancePda,

    vaultAta,
    ownerAta,

    derive: {
      posterPda: (posterId: number | bigint) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("poster"), u64ToLeBuffer(posterId)],
          PROGRAM_ID
        )[0],
      postWinnerPda: (posterId: number | bigint) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("post_winner"), u64ToLeBuffer(posterId)],
          PROGRAM_ID
        )[0],
      posterResponsePda: (
        posterId: number | bigint,
        answerer: anchor.web3.PublicKey
      ) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_response"),
            u64ToLeBuffer(posterId),
            answerer.toBuffer(),
          ],
          PROGRAM_ID
        )[0],
      posterDecryptedAnswerPda: (
        posterId: number | bigint,
        publisher: anchor.web3.PublicKey
      ) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_decrypted_answer"),
            u64ToLeBuffer(posterId),
            publisher.toBuffer(),
          ],
          PROGRAM_ID
        )[0],
      posterAnswererDecryptedAnswerPda: (
        posterId: number | bigint,
        answerer: anchor.web3.PublicKey
      ) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_answerer_decrypted_answer"),
            u64ToLeBuffer(posterId),
            answerer.toBuffer(),
          ],
          PROGRAM_ID
        )[0],
      voteForWinnerPda: (
        posterId: number | bigint,
        voter: anchor.web3.PublicKey
      ) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("vote_for_winner"),
            u64ToLeBuffer(posterId),
            voter.toBuffer(),
          ],
          PROGRAM_ID
        )[0],
      userBalancePdaFor: (user: anchor.web3.PublicKey) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("user_balance_info"), user.toBuffer()],
          PROGRAM_ID
        )[0],
    },
  };
};
