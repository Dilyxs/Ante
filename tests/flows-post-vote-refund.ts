import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ChallengeProtocol } from "../target/types/challenge_protocol";
import { assert } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { UInt64ToLE, ensureOwnerAtaHas } from "./helpers";

describe("flows: post winner, vote, refund", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .challengeProtocol as Program<ChallengeProtocol>;

  const VAULT_AUTHORITY_SEED = Buffer.from("authority");
  const ANTE_MINT_SEED = Buffer.from("Ante");
  const CONFIG_SEED = Buffer.from("config");
  const USER_BALANCE_INFO_SEED = Buffer.from("user_balance_info");
  const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
  const ASSOCIATED_TOKEN_PROGRAM = ASSOCIATED_TOKEN_PROGRAM_ID;

  let vaultAuthorityPda: anchor.web3.PublicKey;
  let anteMintPda: anchor.web3.PublicKey;
  let configPda: anchor.web3.PublicKey;
  let vaultAta: anchor.web3.PublicKey;
  let ownerAta: anchor.web3.PublicKey;
  let userBalancePda: anchor.web3.PublicKey;

  before(async () => {
    const wallet = provider.wallet.publicKey;
    [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [VAULT_AUTHORITY_SEED],
      program.programId
    );
    [anteMintPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [ANTE_MINT_SEED],
      program.programId
    );
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [CONFIG_SEED],
      program.programId
    );

    vaultAta = getAssociatedTokenAddressSync(
      anteMintPda,
      vaultAuthorityPda,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );

    ownerAta = getAssociatedTokenAddressSync(
      anteMintPda,
      wallet,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );

    userBalancePda = anchor.web3.PublicKey.findProgramAddressSync(
      [USER_BALANCE_INFO_SEED, wallet.toBuffer()],
      program.programId
    )[0];

    // try initialize; if already initialized ignore failure
    try {
      await program.methods
        .initialize()
        .accounts({
          signer: wallet,
          vaultAuthority: vaultAuthorityPda,
          mint: anteMintPda,
          vaultAta,
          config: configPda,
          tokenProgram: TOKEN_PROGRAM,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .rpc();
    } catch (err) {
      // ignore - common when running with other tests that already initialized
    }
  });

  it("post_poster_winner: admin pays winner (happy path)", async () => {
    const wallet = provider.wallet.publicKey;

    // ensure owner ATA exists
    const ownerAtaAccount = await provider.connection.getAccountInfo(ownerAta);
    if (!ownerAtaAccount) {
      const createOwnerAtaTx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          wallet,
          ownerAta,
          wallet,
          anteMintPda,
          TOKEN_PROGRAM,
          ASSOCIATED_TOKEN_PROGRAM
        )
      );
      await provider.sendAndConfirm(createOwnerAtaTx);
    }

    // ensure owner has tokens to deposit and fund bounty
    await ensureOwnerAtaHas(
      program,
      provider,
      ownerAta,
      anteMintPda,
      vaultAuthorityPda,
      vaultAta,
      configPda,
      16
    );

    // read vault counter to derive poster PDA
    const vaultState: any = await (
      program.account as any
    ).vaultGlobalState.fetch(
      (
        await anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("vault_global_state")],
          program.programId
        )
      )[0]
    );
    const counterBefore = vaultState.bountyCounter.toNumber();

    // create poster with deadline passed so posting is allowed
    const bountyMinimumGain = 4;
    const submissionCost = 1;
    await (program as any).methods
      .uploadNewPoster(
        new anchor.BN(bountyMinimumGain),
        { openEnded: {} },
        { numberTheory: {} },
        new anchor.BN(0),
        null,
        new anchor.BN(submissionCost)
      )
      .accounts({
        owner: wallet,
        mint: anteMintPda,
        userAta: ownerAta,
        vaultGlobalState: (
          await anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_global_state")],
            program.programId
          )
        )[0],
        data: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
          program.programId
        )[0],
        userBalanceInfo: userBalancePda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
      program.programId
    )[0];

    // create answerer, their ATA, and fund them
    const answerer = anchor.web3.Keypair.generate();
    await provider.connection.requestAirdrop(answerer.publicKey, 1_000_000_000);
    const answererAta = getAssociatedTokenAddressSync(
      anteMintPda,
      answerer.publicKey,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );
    const createAnswererAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet,
        answererAta,
        answerer.publicKey,
        anteMintPda,
        TOKEN_PROGRAM,
        ASSOCIATED_TOKEN_PROGRAM
      )
    );
    await provider.sendAndConfirm(createAnswererAtaTx);

    // mint tokens to answerer ATA and deposit some to their user balance
    await program.methods
      .requestAnteTokens(new anchor.BN(8))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: answerer.publicKey,
        askerAta: answererAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    // deposit a portion into answerer's user balance
    const answererUserBalPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_balance_info"), answerer.publicKey.toBuffer()],
      program.programId
    )[0];
    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(4))
      .accounts({
        owner: answerer.publicKey,
        ownerAta: answererAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: answererUserBalPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([answerer])
      .rpc();

    // answerer posts an answer (creates poster_response)
    const answerArr = new Uint8Array(33);
    answerArr[0] = 1;
    await program.methods
      .answerPoster(new anchor.BN(counterBefore), Array.from(answerArr))
      .accounts({
        answerer: answerer.publicKey,
        mint: anteMintPda,
        answererAta: answererAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: answererUserBalPda,
        posterInfo: posterPda,
        posterPublisher: wallet,
        posterResponse: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_response"),
            Buffer.from(UInt64ToLE(counterBefore)),
            answerer.publicKey.toBuffer(),
          ],
          program.programId
        )[0],
        vaultGlobalState: (
          await anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_global_state")],
            program.programId
          )
        )[0],
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
      } as any)
      .signers([answerer])
      .rpc();

    // answerer posts decrypted answer (so they're eligible)
    const answererDecryptedAnswerPda =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("poster_answerer_decrypted_answer"),
          Buffer.from(UInt64ToLE(counterBefore)),
          answerer.publicKey.toBuffer(),
        ],
        program.programId
      )[0];
    await program.methods
      .postAnswererDecryptedAnswer(
        new anchor.BN(counterBefore),
        "answerer answer",
        "hash-xyz"
      )
      .accounts({
        answerer: answerer.publicKey,
        posterResponse: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_response"),
            Buffer.from(UInt64ToLE(counterBefore)),
            answerer.publicKey.toBuffer(),
          ],
          program.programId
        )[0],
        answererDecryptedAnswer: answererDecryptedAnswerPda,
        posterInfo: posterPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([answerer])
      .rpc();

    // prepare winner accounts
    const winner = answerer.publicKey;
    const winnerAta = getAssociatedTokenAddressSync(
      anteMintPda,
      winner,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );
    // ensure winner ATA exists (should already be created above for answerer)

    // record pre balances
    const winnerAtaBefore = await getAccount(provider.connection, winnerAta);
    const winnerUserBalBeforeNullable = await (
      program.account as any
    ).userBalance.fetchNullable(
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user_balance_info"), winner.toBuffer()],
        program.programId
      )[0]
    );
    const winnerUserBalBefore = winnerUserBalBeforeNullable
      ? Number(winnerUserBalBeforeNullable.balance)
      : 0;
    const vaultBefore = await getAccount(provider.connection, vaultAta);

    // admin posts the poster winner
    await program.methods
      .postPosterWinner(new anchor.BN(counterBefore), winner, new anchor.BN(0))
      .accounts({
        admin: wallet,
        config: configPda,
        postWinner: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("post_winner"), Buffer.from(UInt64ToLE(counterBefore))],
          program.programId
        )[0],
        posterInfo: posterPda,
        userBalanceInfo: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("user_balance_info"), winner.toBuffer()],
          program.programId
        )[0],
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        winner,
        winnerAta,
        mint: anteMintPda,
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
      } as any)
      .rpc();

    // record post balances
    const winnerAtaAfter = await getAccount(provider.connection, winnerAta);
    const winnerUserBalAfterNullable = await (
      program.account as any
    ).userBalance.fetchNullable(
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user_balance_info"), winner.toBuffer()],
        program.programId
      )[0]
    );
    const winnerUserBalAfter = winnerUserBalAfterNullable
      ? Number(winnerUserBalAfterNullable.balance)
      : 0;
    const vaultAfter = await getAccount(provider.connection, vaultAta);

    // the program transfers tokens directly to the winner ATA. Assert the ATA
    // increased by the expected bountyMinimumGain.
    assert.equal(
      Number(winnerAtaAfter.amount) - Number(winnerAtaBefore.amount),
      bountyMinimumGain
    );
    // vault should have decreased by bountyMinimumGain
    assert.equal(
      Number(vaultBefore.amount) - Number(vaultAfter.amount),
      bountyMinimumGain
    );
  });

  it("vote_for_winner: creates vote and prevents duplicates", async () => {
    const wallet = provider.wallet.publicKey;

    // derive counters and poster pda
    const vaultState: any = await (
      program.account as any
    ).vaultGlobalState.fetch(
      (
        await anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("vault_global_state")],
          program.programId
        )
      )[0]
    );
    const counterBefore = vaultState.bountyCounter.toNumber() - 1; // use last poster created above

    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
      program.programId
    )[0];

    // create a voter (we'll reuse an answerer-like flow so voter has poster_response + decrypted)
    const voter = anchor.web3.Keypair.generate();
    await provider.connection.requestAirdrop(voter.publicKey, 1_000_000_000);
    const voterAta = getAssociatedTokenAddressSync(
      anteMintPda,
      voter.publicKey,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );
    const createVoterAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet,
        voterAta,
        voter.publicKey,
        anteMintPda,
        TOKEN_PROGRAM,
        ASSOCIATED_TOKEN_PROGRAM
      )
    );
    await provider.sendAndConfirm(createVoterAtaTx);

    // mint tokens to voter so they can deposit and create poster_response
    await program.methods
      .requestAnteTokens(new anchor.BN(4))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: voter.publicKey,
        askerAta: voterAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    const voterUserBalPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_balance_info"), voter.publicKey.toBuffer()],
      program.programId
    )[0];
    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(1))
      .accounts({
        owner: voter.publicKey,
        ownerAta: voterAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: voterUserBalPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([voter])
      .rpc();

    // voter posts an answer and decrypted answer so they have poster_response and decrypted PDA
    const answerArr = new Uint8Array(33);
    answerArr[0] = 2;
    await program.methods
      .answerPoster(new anchor.BN(counterBefore), Array.from(answerArr))
      .accounts({
        answerer: voter.publicKey,
        mint: anteMintPda,
        answererAta: voterAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: voterUserBalPda,
        posterInfo: posterPda,
        posterPublisher: wallet,
        posterResponse: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_response"),
            Buffer.from(UInt64ToLE(counterBefore)),
            voter.publicKey.toBuffer(),
          ],
          program.programId
        )[0],
        vaultGlobalState: (
          await anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_global_state")],
            program.programId
          )
        )[0],
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
      })
      .signers([voter])
      .rpc();

    const posterResponsePda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("poster_response"),
        Buffer.from(UInt64ToLE(counterBefore)),
        voter.publicKey.toBuffer(),
      ],
      program.programId
    )[0];

    // create decrypted poster PDA (the program expects this to exist for voters who posted decrypted answers)
    const posterDecryptedResponsePda =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("poster_answerer_decrypted_answer"),
          Buffer.from(UInt64ToLE(counterBefore)),
          voter.publicKey.toBuffer(),
        ],
        program.programId
      )[0];
    // attempt to create it by calling postPosterSolution-like instruction if available for this key
    // Some flows create it on answerer side; try calling postAnswererDecryptedAnswer to ensure existence
    const voterDecryptedPdaAlt = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("poster_answerer_decrypted_answer"),
        Buffer.from(UInt64ToLE(counterBefore)),
        voter.publicKey.toBuffer(),
      ],
      program.programId
    )[0];
    // post answerer decrypted (will succeed if not already present)
    try {
      await program.methods
        .postAnswererDecryptedAnswer(
          new anchor.BN(counterBefore),
          "voter ans",
          "h-v"
        )
        .accounts({
          answerer: voter.publicKey,
          posterResponse: posterResponsePda,
          answererDecryptedAnswer: voterDecryptedPdaAlt,
          posterInfo: posterPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([voter])
        .rpc();
    } catch (e) {
      // ignore if already exists
    }

    // now perform vote_for_winner
    const votePda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("vote_for_winner"),
        Buffer.from(UInt64ToLE(counterBefore)),
        voter.publicKey.toBuffer(),
      ],
      program.programId
    )[0];

    // choose a winner (use voter themselves)
    await program.methods
      .voteForWinner(new anchor.BN(counterBefore), voter.publicKey)
      .accounts({
        voter: voter.publicKey,
        posterInfo: posterPda,
        posterDecryptedResponse: posterDecryptedResponsePda,
        voteForWinner: votePda,
        posterResponse: posterResponsePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([voter])
      .rpc();

    // fetch vote account and ensure stored fields are correct
    const voteAcct: any = await (program.account as any).voteForWinner.fetch(
      votePda
    );
    assert.equal(voteAcct.posterId.toString(), counterBefore.toString());
    assert.ok(voteAcct.voter.equals(voter.publicKey));
    assert.ok(voteAcct.winnerVote.equals(voter.publicKey));

    // duplicate vote should fail (account exists)
    let failed = false;
    try {
      await program.methods
        .voteForWinner(new anchor.BN(counterBefore), voter.publicKey)
        .accounts({
          voter: voter.publicKey,
          posterInfo: posterPda,
          posterDecryptedResponse: posterDecryptedResponsePda,
          voteForWinner: votePda,
          posterResponse: posterResponsePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([voter])
        .rpc();
    } catch (e: any) {
      failed = true;
      assert.isOk(e);
    }
    assert.isTrue(failed, "duplicate vote should fail");
  });

  it("refund_answerer_where_poster_didnt_post_solution: admin refunds to publisher ATA (happy)", async () => {
    const wallet = provider.wallet.publicKey;

    // create fresh poster so publisher didn't post solution
    await ensureOwnerAtaHas(
      program,
      provider,
      ownerAta,
      anteMintPda,
      vaultAuthorityPda,
      vaultAta,
      configPda,
      8
    );

    const vaultState: any = await (
      program.account as any
    ).vaultGlobalState.fetch(
      (
        await anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("vault_global_state")],
          program.programId
        )
      )[0]
    );
    const counterBefore = vaultState.bountyCounter.toNumber();

    // create poster with deadline passed
    await (program as any).methods
      .uploadNewPoster(
        new anchor.BN(2),
        { openEnded: {} },
        { numberTheory: {} },
        new anchor.BN(0),
        null,
        new anchor.BN(1)
      )
      .accounts({
        owner: wallet,
        mint: anteMintPda,
        userAta: ownerAta,
        vaultGlobalState: (
          await anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_global_state")],
            program.programId
          )
        )[0],
        data: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
          program.programId
        )[0],
        userBalanceInfo: userBalancePda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
      program.programId
    )[0];

    // create an answerer who posted a response (but publisher did not post solution)
    const answerer = anchor.web3.Keypair.generate();
    await provider.connection.requestAirdrop(answerer.publicKey, 1_000_000_000);
    const answererAta = getAssociatedTokenAddressSync(
      anteMintPda,
      answerer.publicKey,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );
    const createAnswererAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet,
        answererAta,
        answerer.publicKey,
        anteMintPda,
        TOKEN_PROGRAM,
        ASSOCIATED_TOKEN_PROGRAM
      )
    );
    await provider.sendAndConfirm(createAnswererAtaTx);

    await program.methods
      .requestAnteTokens(new anchor.BN(4))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: answerer.publicKey,
        askerAta: answererAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    const answererUserBalPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_balance_info"), answerer.publicKey.toBuffer()],
      program.programId
    )[0];
    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(1))
      .accounts({
        owner: answerer.publicKey,
        ownerAta: answererAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: answererUserBalPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([answerer])
      .rpc();

    // answerer posts a response
    const answerArr = new Uint8Array(33);
    answerArr[0] = 7;
    await program.methods
      .answerPoster(new anchor.BN(counterBefore), Array.from(answerArr))
      .accounts({
        answerer: answerer.publicKey,
        mint: anteMintPda,
        answererAta: answererAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: answererUserBalPda,
        posterInfo: posterPda,
        posterPublisher: wallet,
        posterResponse: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_response"),
            Buffer.from(UInt64ToLE(counterBefore)),
            answerer.publicKey.toBuffer(),
          ],
          program.programId
        )[0],
        vaultGlobalState: (
          await anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_global_state")],
            program.programId
          )
        )[0],
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
      } as any)
      .signers([answerer])
      .rpc();

    // record pre balances
    const posterPublisherAta = ownerAta; // publisher is wallet
    const vaultBefore = await getAccount(provider.connection, vaultAta);
    const publisherAtaBefore = await getAccount(
      provider.connection,
      posterPublisherAta
    );

    // admin triggers refund of 1 token to publisher ATA
    await program.methods
      .refundAnswererWherePosterDidntPostSolution(new anchor.BN(counterBefore))
      .accounts({
        admin: wallet,
        config: configPda,
        vaultAuthority: vaultAuthorityPda,
        mint: anteMintPda,
        vaultAta,
        posterPublisher: wallet,
        posterResponse: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_response"),
            Buffer.from(UInt64ToLE(counterBefore)),
            answerer.publicKey.toBuffer(),
          ],
          program.programId
        )[0],
        userBalanceInfo: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("user_balance_info"), wallet.toBuffer()],
          program.programId
        )[0],
        posterInfo: posterPda,
        posterPublisherAta: posterPublisherAta,
        posterAnswerer: answerer.publicKey,
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
      } as any)
      .rpc();

    const vaultAfter = await getAccount(provider.connection, vaultAta);
    const publisherAtaAfter = await getAccount(
      provider.connection,
      posterPublisherAta
    );

    assert.equal(Number(vaultBefore.amount) - Number(vaultAfter.amount), 1);
    assert.equal(
      Number(publisherAtaAfter.amount) - Number(publisherAtaBefore.amount),
      1
    );
  });
});
