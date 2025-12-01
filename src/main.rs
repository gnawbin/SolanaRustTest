use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
   // solana_test::createTokenMint().await?;
    solana_test::createMintTokens().await?;
    
    Ok(())
}
