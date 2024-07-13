import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ServiceMarketplace } from "../target/types/service_marketplace";

describe("service_marketplace", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ServiceMarketplace as Program<ServiceMarketplace>;

  it("Is initialized!", async () => {
    const tx = await program.rpc.initialize();
    console.log("Your transaction signature", tx);
  });

  // Add more tests for listing, purchasing, and reselling services...
});
