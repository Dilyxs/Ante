import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
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

const BOUNTY_TYPE_TO_ANCHOR = {
  OpenEnded: { openEnded: {} },
  DirectAnswer: { directAnswer: {} },
} as const;

const BOUNTY_TOPIC_TO_ANCHOR = {
  NumberTheory: { numberTheory: {} },
  CryptoPuzzle: { cryptoPuzzle: {} },
  ReverseEng: { reverseEng: {} },
  NumericalTrivial: { numericalTrivial: {} },
  PrivateKeyPuzzle: { privateKeyPuzzle: {} },
} as const;

export const verifyBountyType = (bountyType: string) => {
  const variant =
    BOUNTY_TYPE_TO_ANCHOR[bountyType as keyof typeof BOUNTY_TYPE_TO_ANCHOR];
  if (!variant) {
    throw new Error(
      "Invalid BountyType. Allowed values are: OpenEnded, DirectAnswer."
    );
  }
  return variant;
};

export const verifyBountyTopic = (bountyTopic: string) => {
  const variant =
    BOUNTY_TOPIC_TO_ANCHOR[bountyTopic as keyof typeof BOUNTY_TOPIC_TO_ANCHOR];
  if (!variant) {
    throw new Error(
      "Invalid BountyTopic. Allowed values are: NumberTheory, CryptoPuzzle, ReverseEng, NumericalTrivial, PrivateKeyPuzzle."
    );
  }
  return variant;
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

export const requestAnteToken = async (
  ctx: Awaited<ReturnType<typeof initialize_var>>,
  target_acc: anchor.web3.PublicKey,
  ante_token_count: number
) => {
  const allowedAmounts = [1, 2, 4, 8];
  if (!allowedAmounts.includes(ante_token_count)) {
    throw new Error(
      "Invalid ante_token_count. Allowed values are: 1, 2, 4, 8."
    );
  }

  const askerAta = getAssociatedTokenAddressSync(
    ctx.anteMintPda,
    target_acc,
    true,
    ctx.tokenProgram,
    ctx.associatedTokenProgram
  );

  const askerAtaInfo = await ctx.provider.connection.getAccountInfo(askerAta);
  if (!askerAtaInfo) {
    const createAskerAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        ctx.owner,
        askerAta,
        target_acc,
        ctx.anteMintPda,
        ctx.tokenProgram,
        ctx.associatedTokenProgram
      )
    );
    await ctx.provider.sendAndConfirm(createAskerAtaTx);
  }

  const tx = await (ctx.program as any).methods
    .requestAnteTokens(new anchor.BN(ante_token_count))
    .accounts({
      admin: ctx.owner,
      mint: ctx.anteMintPda,
      asker: target_acc,
      askerAta,
      vaultAuthority: ctx.vaultAuthorityPda,
      vaultAta: ctx.vaultAta,
      config: ctx.configPda,
      tokenProgram: ctx.tokenProgram,
      associatedTokenProgram: ctx.associatedTokenProgram,
      systemProgram: ctx.systemProgram,
    } as any)
    .rpc();

  // TODO: write deposit_ante_token and withdraw_ante_token (future feature).

  return { tx, askerAta };
};

export const uploadNewPoster = async (
  ctx: Awaited<ReturnType<typeof initialize_var>>,
  bounty_minimum_gain: number,
  bounty_type: string,
  bounty_topic: string,
  deadline: number | bigint,
  potential_answer: number[] | Uint8Array | null,
  submission_cost: number
) => {
  const ownerAtaInfo = await ctx.provider.connection.getAccountInfo(
    ctx.ownerAta
  );
  if (!ownerAtaInfo) {
    const createOwnerAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        ctx.owner,
        ctx.ownerAta,
        ctx.owner,
        ctx.anteMintPda,
        ctx.tokenProgram,
        ctx.associatedTokenProgram
      )
    );
    await ctx.provider.sendAndConfirm(createOwnerAtaTx);
  }

  let normalizedPotentialAnswer: number[] | null = null;
  if (potential_answer) {
    const answerArr = Array.from(potential_answer);
    if (answerArr.length !== 33) {
      throw new Error(
        "potential_answer must be exactly 33 bytes when provided."
      );
    }
    normalizedPotentialAnswer = answerArr;
  }

  const vaultState: any = await (
    ctx.program.account as any
  ).vaultGlobalState.fetch(ctx.vaultGlobalStatePda);
  const posterId = Number(vaultState.bountyCounter);
  const posterPda = ctx.derive.posterPda(posterId);
  const postingWinnerPda = ctx.derive.postWinnerPda(posterId);

  const tx = await (ctx.program as any).methods
    .uploadNewPoster(
      new anchor.BN(bounty_minimum_gain),
      verifyBountyType(bounty_type),
      verifyBountyTopic(bounty_topic),
      new anchor.BN(deadline),
      normalizedPotentialAnswer,
      new anchor.BN(submission_cost)
    )
    .accounts({
      owner: ctx.owner,
      mint: ctx.anteMintPda,
      userAta: ctx.ownerAta,
      vaultGlobalState: ctx.vaultGlobalStatePda,
      data: posterPda,
      userBalanceInfo: ctx.userBalancePda,
      vaultAuthority: ctx.vaultAuthorityPda,
      vaultAta: ctx.vaultAta,
      postingWinner: postingWinnerPda,
      tokenProgram: ctx.tokenProgram,
      associatedTokenProgram: ctx.associatedTokenProgram,
      systemProgram: ctx.systemProgram,
    } as any)
    .rpc();

  return { tx, posterId, posterPda, postingWinnerPda };
};

export const answer_poster = async (
  ctx: Awaited<ReturnType<typeof initialize_var>>,
  poster_id: number,
  answer: number[] | Uint8Array,
  answerer: anchor.web3.Keypair
) => {
  if (!Number.isInteger(poster_id) || poster_id < 0) {
    throw new Error("poster_id must be a non-negative integer.");
  }

  const answererPubkey = answerer.publicKey;
  const answerArr = Array.from(answer);
  if (answerArr.length !== 33) {
    throw new Error("answer must be exactly 33 bytes.");
  }

  const posterPda = ctx.derive.posterPda(poster_id);
  const posterResponsePda = ctx.derive.posterResponsePda(
    poster_id,
    answererPubkey
  );
  const userBalancePda = ctx.derive.userBalancePdaFor(answererPubkey);
  const answererAta = getAssociatedTokenAddressSync(
    ctx.anteMintPda,
    answererPubkey,
    true,
    ctx.tokenProgram,
    ctx.associatedTokenProgram
  );

  const answererAtaInfo = await ctx.provider.connection.getAccountInfo(
    answererAta
  );
  if (!answererAtaInfo) {
    const createAnswererAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        ctx.owner,
        answererAta,
        answererPubkey,
        ctx.anteMintPda,
        ctx.tokenProgram,
        ctx.associatedTokenProgram
      )
    );
    await ctx.provider.sendAndConfirm(createAnswererAtaTx);
  }

  const posterAccount: any = await (ctx.program.account as any).poster.fetch(
    posterPda
  );
  const posterPublisher = posterAccount.publisher as anchor.web3.PublicKey;

  const method = (ctx.program as any).methods
    .answerPoster(new anchor.BN(poster_id), answerArr)
    .accounts({
      answerer: answererPubkey,
      mint: ctx.anteMintPda,
      answererAta,
      vaultAuthority: ctx.vaultAuthorityPda,
      vaultAta: ctx.vaultAta,
      userBalanceInfo: userBalancePda,
      posterInfo: posterPda,
      posterPublisher,
      posterResponse: posterResponsePda,
      vaultGlobalState: ctx.vaultGlobalStatePda,
      tokenProgram: ctx.tokenProgram,
      associatedTokenProgram: ctx.associatedTokenProgram,
      systemProgram: ctx.systemProgram,
    } as any);

  const tx = await method.signers([answerer]).rpc();

  return {
    tx,
    posterPda,
    posterResponsePda,
    userBalancePda,
    answererAta,
    answerer: answererPubkey,
  };
};
