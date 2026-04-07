import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import * as fs from "fs";

const requireEnv = (key: string): string => {
  const value = process.env[key];
  if (!value) {
    throw new Error(`${key} is required but was not set`);
  }
  return value;
};

const loadEnv = () => {
  const rpcUrl = requireEnv("ANCHOR_PROVIDER_URL");
  const walletPath = requireEnv("ANCHOR_WALLET");
  const idlPath = requireEnv("IDL_PATH");
  const programId = new anchor.web3.PublicKey(requireEnv("PROGRAM_ID"));

  return {
    rpcUrl,
    walletPath,
    idlPath,
    programId,
  };
};

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
  const env = loadEnv();
  const idl = JSON.parse(fs.readFileSync(env.idlPath, "utf8")) as anchor.Idl & {
    address?: string;
    metadata?: { address?: string };
  };
  idl.address = env.programId.toBase58();
  if (idl.metadata) {
    idl.metadata.address = env.programId.toBase58();
  }

  const walletSecret = JSON.parse(
    fs.readFileSync(env.walletPath, "utf8")
  ) as number[];
  const payer = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(walletSecret)
  );
  const wallet = new anchor.Wallet(payer);

  const connection = new anchor.web3.Connection(
    env.rpcUrl,
    anchor.AnchorProvider.defaultOptions().commitment
  );

  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    anchor.AnchorProvider.defaultOptions()
  );

  const program = new anchor.Program(idl, provider);
  anchor.setProvider(provider);

  const owner = provider.wallet.publicKey;
  const programId = env.programId;

  const [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    programId
  );
  const [anteMintPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("Ante")],
    programId
  );
  const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    programId
  );
  const [vaultGlobalStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_global_state")],
    programId
  );
  const [userBalancePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_balance_info"), owner.toBuffer()],
    programId
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
    idl,
    program,
    provider,
    owner,
    idlPath: env.idlPath,
    programId,
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
          programId
        )[0],
      postWinnerPda: (posterId: number | bigint) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("post_winner"), u64ToLeBuffer(posterId)],
          programId
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
          programId
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
          programId
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
          programId
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
          programId
        )[0],
      userBalancePdaFor: (user: anchor.web3.PublicKey) =>
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("user_balance_info"), user.toBuffer()],
          programId
        )[0],
    },
  };
};

export const initialize = async (
  ctx: Awaited<ReturnType<typeof initialize_var>>
) => {
  return await (ctx.program as any).methods
    .initialize()
    .accounts({
      signer: ctx.owner,
      vaultAuthority: ctx.vaultAuthorityPda,
      mint: ctx.anteMintPda,
      vaultAta: ctx.vaultAta,
      config: ctx.configPda,
      vaultGlobalState: ctx.vaultGlobalStatePda,
      tokenProgram: ctx.tokenProgram,
      associatedTokenProgram: ctx.associatedTokenProgram,
      systemProgram: ctx.systemProgram,
    } as any)
    .rpc();
};
