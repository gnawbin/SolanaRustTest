
use anyhow::{Ok, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_program::example_mocks::solana_sdk::system_instruction::create_account;
use solana_transaction_status_client_types::{TransactionDetails, UiTransactionEncoding};

use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, Signer},
    transaction::Transaction,
    native_token::LAMPORTS_PER_SOL, 
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction}
};
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

use spl_token_interface::{id as token_program_id, instruction::{initialize_account, initialize_mint, mint_to, transfer_checked},  state::{Account, Mint}};
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};

use std::str::FromStr;
pub async fn createTransferTokens()->Result<()>{
    // Create connection to local validator
    let client = RpcClient::new_with_commitment(
        String::from("http://localhost:8899"),
        CommitmentConfig::confirmed(),
    );
    let latest_blockhash = client.get_latest_blockhash().await?;

    // Generate a new keypair for the fee payer
    let fee_payer = Keypair::new();

    // Generate a second keypair for the token recipient
    let recipient = Keypair::new();

    // Airdrop 1 SOL to fee payer
    let airdrop_signature = client
        .request_airdrop(&fee_payer.pubkey(), 1_000_000_000)
        .await?;
    client.confirm_transaction(&airdrop_signature).await?;

    loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }

    // Airdrop 1 SOL to recipient for rent exemption
    let recipient_airdrop_signature = client
        .request_airdrop(&recipient.pubkey(), 1_000_000_000)
        .await?;
    client
        .confirm_transaction(&recipient_airdrop_signature)
        .await?;

    loop {
        let confirmed = client
            .confirm_transaction(&recipient_airdrop_signature)
            .await?;
        if confirmed {
            break;
        }
    }

    // Generate keypair to use as address of mint
    let mint = Keypair::new();

    // Get default mint account size (in bytes), no extensions enabled
    let mint_space = Mint::LEN;
    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(mint_space)
        .await?;

    // Instruction to create new account for mint (token program)
    let create_account_instruction = create_account(
        &fee_payer.pubkey(), // payer
        &mint.pubkey(),      // new account (mint)
        mint_rent,           // lamports
        mint_space as u64,   // space
        &token_program_id(), // program id
    );

    // Instruction to initialize mint account data
    let initialize_mint_instruction = initialize_mint(
        &token_program_id(),
        &mint.pubkey(),            // mint
        &fee_payer.pubkey(),       // mint authority
        Some(&fee_payer.pubkey()), // freeze authority
        2,                         // decimals
    )?;

    // Calculate the associated token account address for fee_payer
    let source_token_address = get_associated_token_address(
        &fee_payer.pubkey(), // owner
        &mint.pubkey(),      // mint
    );

    // Instruction to create associated token account for fee_payer
    let create_source_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(), // funding address
        &fee_payer.pubkey(), // wallet address
        &mint.pubkey(),      // mint address
        &token_program_id(), // program id
    );

    // Calculate the associated token account address for recipient
    let destination_token_address = get_associated_token_address(
        &recipient.pubkey(), // owner
        &mint.pubkey(),      // mint
    );

    // Instruction to create associated token account for recipient
    let create_destination_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(), // funding address
        &recipient.pubkey(), // wallet address
        &mint.pubkey(),      // mint address
        &token_program_id(), // program id
    );

    // Amount of tokens to mint (100 tokens with 2 decimal places)
    let amount = 100_00;

    // Create mint_to instruction to mint tokens to the source token account
    let mint_to_instruction = mint_to(
        &token_program_id(),
        &mint.pubkey(),         // mint
        &source_token_address,  // destination
        &fee_payer.pubkey(),    // authority
        &[&fee_payer.pubkey()], // signer
        amount,                 // amount
    )?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_account_instruction,
            initialize_mint_instruction,
            create_source_ata_instruction,
            create_destination_ata_instruction,
            mint_to_instruction,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );

    // Send and confirm transaction
    client.send_and_confirm_transaction(&transaction).await?;

    // Amount of tokens to transfer (0.50 tokens with 2 decimals)
    let transfer_amount = 50;

    // Create transfer_checked instruction to send tokens from source to destination
    let transfer_instruction = transfer_checked(
        &token_program_id(),        // program id
        &source_token_address,      // source
        &mint.pubkey(),             // mint
        &destination_token_address, // destination
        &fee_payer.pubkey(),        // owner of source
        &[&fee_payer.pubkey()],     // signers
        transfer_amount,            // amount
        2,                          // decimals
    )?;

    // Create transaction for transferring tokens
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        latest_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint_data = Mint::unpack(&mint_account.data)?;

    // Get token account balances to verify the transfer
    let source_token_account = client.get_token_account(&source_token_address).await?;
    let destination_token_account = client.get_token_account(&destination_token_address).await?;

    println!("Successfully transferred 0.50 tokens from sender to recipient");

    println!("\nMint Address: {}", mint.pubkey());
    println!("{:#?}", mint_data);

    println!("\nSource Token Account Address: {}", source_token_address);
    if let Some(source_account) = source_token_account {
        println!("TokenBalance: {}", source_account.token_amount.amount);
        println!("{:#?}", source_account);
    }

    println!(
        "\nDestination Token Account Address: {}",
        destination_token_address
    );
    if let Some(destination_account) = destination_token_account {
        println!("Token Balance: {}", destination_account.token_amount.amount);
        println!("{:#?}", destination_account);
    }

    println!("Transaction Signature: {}", transaction_signature);

    Ok(())
}
pub async fn createMintTokens()->Result<()>{
     // Create connection to local validator
     let client=RpcClient::new_with_commitment(String::from("http://localhost:8899"),CommitmentConfig::confirmed());
     let latest_blockhash=client.get_latest_blockhash().await?;
     //Generate a new keypair for the fee payer
     let fee_payer=Keypair::new();
     //Airdrop 1 SOL to fee payer
     let airdrop_signature=client.request_airdrop(&fee_payer.pubkey()
     , 1_000_000_000).await?;
     client.confirm_transaction(&airdrop_signature).await?;
       loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }
    // Generate keypair to use as address of mint
    let mint=Keypair::new();
    //Get default mint account size(in bytes),no extensions enabled
    let mint_space = Mint::LEN;
    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(mint_space)
        .await?;
     // Instruction to create new account for mint (token program)
     let create_account_instruction=create_account(&fee_payer.pubkey(),
     &mint.pubkey(),
        mint_rent,
        mint_space as u64,
        &token_program_id(),
      );
       // Instruction to initialize mint account data
       let initialize_mint_instruction=initialize_mint(
         &token_program_id(),
         &mint.pubkey(),
         &fee_payer.pubkey(),
         Some(&fee_payer.pubkey()),2
       )?;
        // Calculate the associated token account address for fee_payer
        let associated_token_address=get_associated_token_address_with_program_id(
            &fee_payer.pubkey(), // owner
        &mint.pubkey(),      // mint
        &token_program_id(), // program_id
        );
     // Instruction to create associated token account
     let create_ata_instruction=create_associated_token_account(
       &fee_payer.pubkey(),
         &fee_payer.pubkey(),
          &mint.pubkey(), 
          &token_program_id());
         // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_account_instruction,
            initialize_mint_instruction,
            create_ata_instruction,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );
     // Send and confirm transaction
     client.send_and_confirm_transaction(&transaction).await?;
     // Amount of tokens to mint (100 tokens with 2 decimal places)
     let amount=100;
      // Create mint_to instruction to mint tokens to the associated token account
      let mint_to_instruction=mint_to(
        &token_program_id(),
        &mint.pubkey(),            // mint
        &associated_token_address, // destination
        &fee_payer.pubkey(),       // authority
        &[&fee_payer.pubkey()],    // signer
        amount,                    // amount
            )?;
     // Create transaction for minting tokens
     let transaction=Transaction::new_signed_with_payer(
        &[mint_to_instruction], 
        Some(&fee_payer.pubkey()),
         &[&fee_payer], 
         latest_blockhash);
        // Send and confirm transaction
        let transaction_signature=client.send_and_confirm_transaction(&transaction).await?;
        let mint_account=client.get_account(&mint.pubkey()).await?;
        let mint_data=Mint::unpack(&mint_account.data)?;
        let token=client.get_account(&associated_token_address).await?;
        let token_data=Account::unpack(&token.data)?;

        println!("Minted 1.00 tokens to the associated token account");
    println!("\nMint Address: {}", mint.pubkey());
    println!("{:#?}", mint_data);

    println!(
        "\nAssociated Token Account Address: {}",
        associated_token_address
    );
    println!("{:#?}", token_data);

    println!("Transaction Signature: {}", transaction_signature);

     Ok(())
}

#[warn(non_snake_case)]
pub async  fn createAssociateTokenAccount()->Result<()>{
 // Create connection to local validator
 let client=RpcClient::new_with_commitment(String::from("http://localhost:8899"), CommitmentConfig::confirmed());
 let latest_blockhash=client.get_latest_blockhash().await?;
  // Generate a new keypair for the fee payer
  let fee_payer=Keypair::new();
  //Airdrop 1 SOL to fee payer
  let airdrop_signature=client.request_airdrop(&fee_payer.pubkey(), 1_000_000_000).await?;
  client.confirm_transaction(&airdrop_signature).await?;
 loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }
 // Generate keypair to use as address of mint
let mint = Keypair::new();

    // Get default mint account size (in bytes), no extensions enabled
    let mint_space = Mint::LEN;
    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(mint_space)
        .await?;

    // Instruction to create new account for mint (token program)
    let create_account_instruction = create_account(
        &fee_payer.pubkey(), // payer
        &mint.pubkey(),      // new account (mint)
        mint_rent,           // lamports
        mint_space as u64,   // space
        &token_program_id(), // program id
    );

    // Instruction to initialize mint account data
    let initialize_mint_instruction = initialize_mint(
        &token_program_id(),
        &mint.pubkey(),            // mint
        &fee_payer.pubkey(),       // mint authority
        Some(&fee_payer.pubkey()), // freeze authority
        2,                         // decimals
    )?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint_data = Mint::unpack(&mint_account.data)?;

    println!("Mint Address: {}", mint.pubkey());
    println!("{:#?}", mint_data);
    println!("Transaction Signature: {}", transaction_signature);

    // Derive the associated token account address for fee_payer
    let associated_token_account = get_associated_token_address(
        &fee_payer.pubkey(), // owner
        &mint.pubkey(),      // mint
    );

    // Instruction to create associated token account
    let create_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(), // funding address
        &fee_payer.pubkey(), // wallet address (owner)
        &mint.pubkey(),      // mint address
        &token_program_id(), // program id
    );

    // Create transaction for associated token account creation
    let transaction = Transaction::new_signed_with_payer(
        &[create_ata_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        latest_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    let token = client.get_account(&associated_token_account).await?;
    let token_data = Account::unpack(&token.data)?;

    println!(
        "\nAssociated Token Account Address: {}",
        associated_token_account
    );
    println!("{:#?}", token_data);
    println!("Transaction Signature: {}", transaction_signature);

    Ok(())
}


pub async fn createTokenAccount()->Result<()>{
    //create connect to local validator
    let client=RpcClient::new_with_commitment(String::from("http://localhost:8899"), CommitmentConfig::confirmed());
    let latest_blockhash=client.get_latest_blockhash().await?;
       // Generate a new keypair for the fee payer
       let fee_payer=Keypair::new();
       // Airdrop 1 SOL to fee payer
       let airdrop_signature=client.request_airdrop(&fee_payer.pubkey(),1_000_000_000).await?;
       loop {
           let confirmed=client.confirm_transaction(&airdrop_signature).await?;
           if confirmed{
            break;
           }
       }
       // Generate keypair to use as address of mint
       let mint=Keypair::new();
        // Get default mint account size (in bytes), no extensions enabled
       let mint_space=Mint::LEN;
       let mint_rent=client.get_minimum_balance_for_rent_exemption(mint_space).await?;
        // Instruction to create new account for mint (token program)
        let create_account_instruction
        =create_account(&fee_payer.pubkey(),//payer
            &mint.pubkey(),//new account(mint)
            mint_rent, //lamports
             mint_space as u64, //space
              &token_program_id());
          // Instruction to initialize mint account data
          let initialize_mint_instruction = initialize_mint(
        &token_program_id(),
        &mint.pubkey(),            // mint
        &fee_payer.pubkey(),       // mint authority
        Some(&fee_payer.pubkey()), // freeze authority
        2,                         // decimals
    )?;
        // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );
  // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;
    let mint_account=client.get_account(&mint.pubkey()).await?;
    let mint_data=Mint::unpack(&mint_account.data)?;
    println!("Mint Address: {}",&mint.pubkey());
     println!("{:#?}", mint_data);
    println!("Transaction Signature: {}", transaction_signature);

// Generate keypair to use as address of token account
    let token_account = Keypair::new();

    // Get token account size (in bytes)
    let token_account_space = Account::LEN;
    let token_account_rent = client
        .get_minimum_balance_for_rent_exemption(token_account_space)
        .await?;

    // Instruction to create new account for token account (token program)
    let create_token_account_instruction = create_account(
        &fee_payer.pubkey(),        // payer
        &token_account.pubkey(),    // new account (token account)
        token_account_rent,         // lamports
        token_account_space as u64, // space
        &token_program_id(),        // program id
    );
 // Instruction to initialize token account data
    let initialize_token_account_instruction = initialize_account(
        &token_program_id(),
        &token_account.pubkey(), // account
        &mint.pubkey(),          // mint
        &fee_payer.pubkey(),     // owner
    )?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_token_account_instruction,
            initialize_token_account_instruction,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &token_account],
        latest_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    let token = client.get_account(&token_account.pubkey()).await?;
    let token_data = Account::unpack(&token.data)?;

    println!("\nToken Account Address: {}", &token_account.pubkey());
    println!("{:#?}", token_data);
    println!("Transaction Signature: {}", transaction_signature);
    

 
    Ok(())
}
pub async  fn createTokenMint()->Result<()>{
   // Create connection to local validator
    let client = RpcClient::new_with_commitment(
        String::from("http://localhost:8899"),
        CommitmentConfig::confirmed(),
    );
      let latest_blockhash = client.get_latest_blockhash().await?;
      // Generate a new keypair for the fee payer
    let fee_payer = Keypair::new();

    // Airdrop 1 SOL to fee payer
    let airdrop_signature = client
        .request_airdrop(&fee_payer.pubkey(), 1_000_000_000)
        .await?;
    client.confirm_transaction(&airdrop_signature).await?;

    loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }

    // Generate keypair to use as address of mint
    let mint = Keypair::new();

    let space = Mint::LEN;
    let _rent = client.get_minimum_balance_for_rent_exemption(space).await?;

    // Create account instruction using system program
    let create_account_instruction = Instruction {
        program_id: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(mint.pubkey(), true),
        ],
        data: vec![],
    };

    // Initialize mint instruction
    let initialize_mint_instruction = initialize_mint(
        &token_program_id(),
        &mint.pubkey(),            // mint address
        &fee_payer.pubkey(),       // mint authority
        Some(&fee_payer.pubkey()), // freeze authority
        9,                         // decimals
    )?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    println!("Mint Address: {}", mint.pubkey());
    println!("\nTransaction Signature: {}", transaction_signature);

    let mint_account = client.get_account(&mint.pubkey()).await?;
    let mint = Mint::unpack(&mint_account.data)?;
    println!("\n{:#?}", mint);
  Ok(())

}

pub async fn testGetAccountInfo()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed() //承诺描述了区块在该时间点的最终确定程度。详见 配置状态承诺。
    );
    let pubkey = Pubkey::from_str("vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg")?;
    let account = client.get_account(&pubkey).await?;

    println!("{:#?}", account);

    Ok(())
}

pub async  fn testGetBalance()->Result<()>{
 
    let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );
     let pubkey = Pubkey::from_str("83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri")?;
    let balance = client.get_balance(&pubkey).await?;

    println!("{:#?} SOL", balance / LAMPORTS_PER_SOL);
    Ok(())

}
