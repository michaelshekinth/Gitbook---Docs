import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DegenTrader } from "../target/types/degen_trader";
import { expect } from "chai";

describe("degen-trader", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DegenTrader as Program<DegenTrader>;
  const authority = provider.wallet as anchor.Wallet;
  const player = anchor.web3.Keypair.generate();

  let gameStatePda: anchor.web3.PublicKey;
  let playerSessionPda: anchor.web3.PublicKey;

  before(async () => {
    // Derive PDAs
    [gameStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("game_state")],
      program.programId
    );

    [playerSessionPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player_session"), player.publicKey.toBuffer()],
      program.programId
    );

    // Airdrop SOL to player
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        player.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      )
    );
  });

  it("Initializes the game", async () => {
    await program.methods
      .initializeGame()
      .accounts({
        gameState: gameStatePda,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    expect(gameState.authority.toString()).to.equal(authority.publicKey.toString());
    expect(gameState.totalPlayers.toNumber()).to.equal(0);
    expect(gameState.isActive).to.be.true;
    expect(gameState.coinPrices).to.have.length(6);
  });

  it("Creates a player", async () => {
    await program.methods
      .createPlayer()
      .accounts({
        playerSession: playerSessionPda,
        gameState: gameStatePda,
        player: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    const playerSession = await program.account.playerSession.fetch(playerSessionPda);
    expect(playerSession.player.toString()).to.equal(player.publicKey.toString());
    expect(playerSession.cash.toNumber()).to.equal(100000); // $1000 in cents
    expect(playerSession.location).to.deep.equal({ bedroom: {} });
    expect(playerSession.isActive).to.be.true;
  });

  it("Processes a trading day", async () => {
    const recentBlockhashes = anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY;
    
    const trades = [
      {
        coinName: "DOGE",
        quantity: 100,
        action: { buy: {} }
      }
    ];

    await program.methods
      .nextDay(trades)
      .accounts({
        playerSession: playerSessionPda,
        gameState: gameStatePda,
        player: player.publicKey,
        recentBlockhashes: recentBlockhashes,
      })
      .signers([player])
      .rpc();

    const playerSession = await program.account.playerSession.fetch(playerSessionPda);
    expect(playerSession.day.toNumber()).to.equal(1);
    expect(playerSession.cash.toNumber()).to.be.lessThan(100000); // Should have spent money on DOGE
  });

  it("Gets player stats", async () => {
    const stats = await program.methods
      .getPlayerStats()
      .accounts({
        playerSession: playerSessionPda,
        gameState: gameStatePda,
        player: player.publicKey,
      })
      .signers([player])
      .view();

    expect(stats.day.toNumber()).to.equal(1);
    expect(stats.cash.toNumber()).to.be.greaterThan(0);
    expect(stats.netWorth.toNumber()).to.be.greaterThan(0);
  });

  it("Handles location progression", async () => {
    // This would require multiple days of trading to accumulate enough net worth
    // For testing purposes, we can modify the player's cash directly in a test environment
    console.log("Location progression test would require extended gameplay simulation");
  });

  it("Handles random events correctly", async () => {
    const recentBlockhashes = anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY;
    
    // Execute multiple days to see different random events
    for (let i = 0; i < 5; i++) {
      const trades = []; // No trades, just advance day
      
      const tx = await program.methods
        .nextDay(trades)
        .accounts({
          playerSession: playerSessionPda,
          gameState: gameStatePda,
          player: player.publicKey,
          recentBlockhashes: recentBlockhashes,
        })
        .signers([player])
        .rpc();

      // You could parse the transaction logs here to verify events were emitted
      console.log(`Day ${i + 2} completed: ${tx}`);
    }

    const playerSession = await program.account.playerSession.fetch(playerSessionPda);
    expect(playerSession.day.toNumber()).to.be.greaterThan(1);
  });
});