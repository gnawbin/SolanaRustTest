
use anyhow::{Context as AnyhowContext, Result};
use argon2::{
    Argon2,PasswordHash,PasswordHasher,PasswordVerifier,
    password_hash::{SaltString,rand_core::OsRng}
};
use opentelemetry::{trace, Context as OtelContext, metrics::Meter};
use opentelemetry_sdk::trace::{SdkTracerProvider, BatchSpanProcessor};
use opentelemetry_sdk::Resource;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use rand::Rng;
use tokio::net::TcpListener;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Method;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    program_pack::Pack,
    signature::{Keypair, keypair_from_seed},
    transaction::Transaction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    pubkey,
    instruction::{AccountMeta, Instruction},
    message::Message,
    signer::Signer,
   
};
use solana_program::example_mocks::solana_sdk::system_instruction::create_account;


use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_token_interface::{id as token_program_id, instruction::{initialize_account, initialize_mint, mint_to, transfer_checked, approve_checked}, state::{Account, Mint}};
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};
use solana_transaction_status_client_types::{TransactionDetails, UiTransactionEncoding};
use base64::{engine::general_purpose, Engine};
use std::str::FromStr;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::RpcAccountInfoConfig,
};
use bincode::deserialize;
use bip39::{Language, Mnemonic, Seed, MnemonicType};
use futures::stream::StreamExt;
use std::fmt;
use tokio::sync::Semaphore;
use std::sync::Arc;
//2025年生产推荐参数
const MEMORY_COST: u32=65_536;
const ITERATIONS: u32=2;
const PARALLELISM: u32=4;
const HASH_LEN: u32=32;
pub  struct PasswordService;

async fn roll_dice(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let random_number = rand::rng().random_range(1..=6);
    Ok(Response::new(Full::new(Bytes::from(
        random_number.to_string(),
    ))))
}

async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/rolldice") => roll_dice(req).await,
        _ => Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("Not Found")))
            .unwrap()),
    }
}
impl PasswordService {
     /// 注册时：哈希密码（返回 PHC 格式字符串，可直接存数据库）
     pub fn hash(password: impl AsRef<[u8]>) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                MEMORY_COST,
                ITERATIONS,
                PARALLELISM,
                Some(HASH_LEN.try_into().map_err(|e: std::num::TryFromIntError| anyhow::Error::msg(e.to_string()))?),
            )
            .map_err(|e: argon2::Error| anyhow::Error::msg(e.to_string()))?,
        );
        // 自动生成 PHC 字符串：$argon2id$v=19$m=65536,t=2,p=4$xxxx$yyyy
        let hash = argon2
            .hash_password(password.as_ref(), &salt)
            .map_err(|e: argon2::password_hash::Error| anyhow::Error::msg(e.to_string()))?
            .to_string();

        Ok(hash)
     }
    pub fn verify(stored_hash: &str, password: impl AsRef<[u8]>) -> bool {
        let parsed = match PasswordHash::new(stored_hash) {
            Ok(h) => h,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_ref(), &parsed)
            .is_ok()
    }
    
}
pub async fn createPassword()->Result<()>{
    let password = "P@ssw0rd!2025_VeryStrong";

    // 1. 注册：生成哈希
    let hashed = PasswordService::hash(password).unwrap();
    println!("存入数据库 → {}", hashed);
    // 示例输出：
    // $argon2id$v=19$m=65536,t=2,p=4$z8z8z8z8z8z8z8w$8Z9j3fN9i3s8fN9j3fN9i3s8fN9j3fN9i3s8fA==

    // 2. 登录：验证
    assert!(PasswordService::verify(&hashed, password));
    assert!(!PasswordService::verify(&hashed, "wrong password"));

    println!("Argon2id 验证成功！");
   Ok(())
}
pub async fn createApproveChecked()->Result<()>{
    //Create connection to local validator
    let client=RpcClient::new_with_commitment(
        String::from("http://localhost:8899"),
        CommitmentConfig::confirmed());
    let  latest_blockhash=client.get_latest_blockhash().await?;
    //Generate a nwe keypair for the fee payer
    let fee_payer=Keypair::new();
    //Generate a keypair for the delegate
    let delegate=Keypair::new();
    //Airdrop 1 SOL to fee payer
    let airdrop_signature=client
    .request_airdrop(&fee_payer.pubkey(),1_000_000_000).await?;
    client.confirm_transaction(&airdrop_signature).await?;

    loop {
        let confirmed=client.confirm_transaction(&airdrop_signature).await?;
        if confirmed{
            break;
        }
    }
     // Generate keypair to use as address of mint
     let mint=Keypair::new();
     //Number of decimals form the mint
     let decimals=2;
     //Get default mint account size(in bytes),no extensions enabled
     let mint_space=Mint::LEN;
     let mint_rent=client
     .get_minimum_balance_for_rent_exemption(mint_space).await?;
     // Instruction to create new account for mint (token program)
     let create_account_instruction=create_account(
        &fee_payer.pubkey(),//payer 
        &mint.pubkey(),//new account
        mint_rent, //lamports
        mint_space as u64,//space 
        &token_program_id());//program id
        // Instruction to initialize mint account data
        let initialize_mint_instruction=initialize_mint(
            &token_program_id(),//token_program_id, 
            &mint.pubkey(),//mint,
             &fee_payer.pubkey(),//mint authority
              Some(&fee_payer.pubkey()),//freeze authority 
              decimals)?;
         // Calculate the associated token account address for fee_payer
       let associated_token_address=get_associated_token_address(
        &fee_payer.pubkey(),//owner
         &mint.pubkey());//mint

      // Instruction to create associated token account
    let create_ata_instruction = create_associated_token_account(
        &fee_payer.pubkey(), // funding address
        &fee_payer.pubkey(), // wallet address
        &mint.pubkey(),      // mint address
        &token_program_id(), // program id
    );
     // Amount of tokens to mint (100 tokens with 2 decimals)
    let amount = 100_00;
    // Create mint_to instruction to mint tokens to the associated token account
    let mint_to_instruction = mint_to(
        &token_program_id(),
        &mint.pubkey(),            // mint
        &associated_token_address, // destination
        &fee_payer.pubkey(),       // authority
        &[&fee_payer.pubkey()],    // signer
        amount,                    // amount
    )?;

    // Create transaction and add instructions
    let transaction = Transaction::new_signed_with_payer(
        &[
            create_account_instruction,
            initialize_mint_instruction,
            create_ata_instruction,
            mint_to_instruction,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );

    // Send and confirm transaction
    client.send_and_confirm_transaction(&transaction).await?;

    // Amount of tokens to approve (1 token with 2 decimals)
    let approve_amount = 1_00;

    // Create approve_checked instruction
    let approve_instruction = approve_checked(
        &token_program_id(),       // program id
        &associated_token_address, // source token account
        &mint.pubkey(),            // mint
        &delegate.pubkey(),        // delegate
        &fee_payer.pubkey(),       // owner
        &[&fee_payer.pubkey()],    // signers
        approve_amount,            // amount
        decimals,                  // decimals
    )?;

    // Create transaction for approving delegate
    let transaction = Transaction::new_signed_with_payer(
        &[approve_instruction],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        latest_blockhash,
    );

    // Send and confirm transaction
    let transaction_signature = client.send_and_confirm_transaction(&transaction).await?;

    let token = client.get_token_account(&associated_token_address).await?;

    println!("Successfully approved delegate for 1.0 token");

    println!("\nDelegate Address: {}", delegate.pubkey());

    println!("\nToken Account Address: {}", associated_token_address);
    if let Some(token) = token {
        println!("{:#?}", token);
    }
    println!("Transaction Signature: {}", transaction_signature);


        Ok(())
}
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
pub async  fn testGetBlock()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );
    let slot_number = 377261141;

    let config = solana_client::rpc_config::RpcBlockConfig {
        encoding: UiTransactionEncoding::Base58.into(),
        transaction_details: TransactionDetails::Full.into(),
        rewards: None,
        commitment: CommitmentConfig::finalized().into(),
        max_supported_transaction_version: Some(0),
    };
    let block = client.get_block_with_config(slot_number, config).await?;

    println!("Block: {:#?}", block);

    Ok(())
}

pub async fn testGetBlockHeight()->Result<()>{
    let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );
     let block_height = client.get_block_height().await?;

    println!("Block height: {:#?}", block_height);

    Ok(())
}
pub async  fn testGetBlockProduction()->Result<()>{
      let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );
    let block_production=client.get_block_production().await?;
    println!("Block production: {:#?}",block_production);
    Ok(())
}
pub async  fn testGetBlocks()->Result<()>{
      let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );
    let start_slot=377268280;
    let end_slot=377268285;
      let blocks = client.get_blocks(start_slot, Some(end_slot)).await?;
    println!("Blocks produced: {:#?}", blocks);

    Ok(())
}
pub async fn testGetBlocksWithLimit()->Result<()>{
       let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let start_slot = 377268280;
    let limit = 5;

    let blocks = client.get_blocks_with_limit(start_slot, limit).await?;

    println!("Blocks produced: {:?}", blocks);

    Ok(())
}
pub async fn testGetBlockTime()->Result<()>{
      let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let slot_number = 377268280;

    let block_time = client.get_block_time(slot_number).await?;

    println!("Blocks time: {:?}", block_time);

    Ok(())
}
pub async fn testGetClusterNodes()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let block_time = client.get_cluster_nodes().await?;

    println!("{:#?}", block_time);

    Ok(())
}
pub async fn subscribingEvent()->Result<()>{
    let wallet=Keypair::new();
    let pubkey=wallet.pubkey();
    let connection=RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    let ws_client = PubsubClient::new("ws://localhost:8900").await?;
 tokio::spawn(async move {
        let config = RpcAccountInfoConfig {
            commitment: Some(CommitmentConfig::confirmed()),
            encoding: None,
            data_slice: None,
            min_context_slot: None,
        };

        let (mut stream, _) = ws_client
            .account_subscribe(&pubkey, Some(config))
            .await
            .expect("Failed to subscribe to account updates");

        while let Some(account) = stream.next().await {
            println!("{:#?}", account);
        }
    });

    let airdrop_signature = connection
        .request_airdrop(&wallet.pubkey(), LAMPORTS_PER_SOL)
        .await?;
    loop {
        let confirmed = connection.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }
    Ok(())
}

pub async  fn testGetEpochInfo()->Result<()>{
        let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let epoch_info = client.get_epoch_info().await?;

    println!("{:#?}", epoch_info);

    Ok(())
}
pub async fn testGetEpochSchedule()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let epoch_schedule = client.get_epoch_schedule().await?;

    println!("{:#?}", epoch_schedule);

    Ok(())
}
pub async fn testFeeForMessage()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let base_64_message = "AQABAgIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEBAQAA";
    let bytes = general_purpose::STANDARD.decode(base_64_message).unwrap();
    let message: Message = deserialize(&bytes).unwrap();

    let fee = client.get_fee_for_message(&message).await?;

    println!("{:#?}", fee);

    Ok(())
}
pub async fn testGetFirstAvailableBlock()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let first_available_block = client.get_first_available_block().await?;

    println!("{:#?}", first_available_block);

    Ok(())
}
pub async fn testGetGenesisHash()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let genesis_hash = client.get_genesis_hash().await?;

    println!("{:#?}", genesis_hash);

    Ok(())
}
pub async fn testGetHealth()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let health = client.get_health().await?;

    println!("{:#?}", health);

    Ok(())
}
pub async fn testGetHighestSnapshotSlot()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let highest_snapshot_slot = client.get_highest_snapshot_slot().await?;

    println!("{:#?}", highest_snapshot_slot);

    Ok(())
}
pub async fn testGetIdentity()->Result<()>{
        let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let identity = client.get_identity().await?;

    println!("{:#?}", identity);

    Ok(())
}
pub async fn testGetInflationGovernor()->Result<()>{
   let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let inflation_govener = client.get_inflation_governor().await?;

    println!("{:#?}", inflation_govener);

    Ok(())  
}
pub async fn testGetInflationRate()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let inflation_rate = client.get_inflation_rate().await?;

    println!("{:#?}", inflation_rate);

    Ok(())
}
pub async fn testGetInflationReward()->Result<()>{
     let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let addresses = [
        pubkey!("6dmNQ5jwLeLk5REvio1JcMshcbvkYMwy26sJ8pbkvStu"),
        pubkey!("BGsqMegLpV6n6Ve146sSX2dTjUMj3M92HnU8BbNRMhF2"),
    ];

    let epoch = 2;

    let inflation_reward = client.get_inflation_reward(&addresses, Some(epoch)).await?;

    println!("{:#?}", inflation_reward);

    Ok(())
}

pub async fn createKeypair()->Result<()>{
     let keypair = Keypair::new();
    let address = keypair.pubkey();

    println!("address: {address}");
    Ok(())
}
pub async  fn validatePublicKey()->Result<()>{
        // on curve address
    let on_curve_public_key = pubkey!("5oNDL3swdJJF1g9DzJiZ4ynHXgszjAEpUkxVYejchzrY");
    println!("is on curve: {}", on_curve_public_key.is_on_curve());

    let off_curve_public_key = pubkey!("4BJXYkfvg37zEmBbsacZjeQDpTNx91KppxFJxRqrz48e");
    println!("is off curve: {}", off_curve_public_key.is_on_curve());
    Ok(())
}
pub async fn generateMnemonicsKeypairs()->Result<()>{
     let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
    let phrase = mnemonic.phrase();

    println!("phrase: {}", phrase);
    Ok(())
}
pub async fn restoringBIP39FormatMnemonics()->Result<()>{
       let phrase = "pill tomorrow foster begin walnut borrow virtual kick shift mutual shoe scatter";
    let mnemonic = Mnemonic::from_phrase(phrase, Language::English)?;

    let seed = Seed::new(&mnemonic, "");
    let keypair = keypair_from_seed(seed.as_bytes()).unwrap();

    println!("recovered address {:?}", keypair.pubkey());

    Ok(())
}
pub async fn signAndVerifyMessage()->Result<()>{
        let keypair_bytes = [
        174, 47, 154, 16, 202, 193, 206, 113, 199, 190, 53, 133, 169, 175, 31, 56, 222, 53, 138,
        189, 224, 216, 117, 173, 10, 149, 53, 45, 73, 251, 237, 246, 15, 185, 186, 82, 177, 240,
        148, 69, 241, 227, 167, 80, 141, 89, 240, 121, 121, 35, 172, 247, 68, 251, 226, 218, 48,
        63, 176, 109, 168, 89, 238, 135,
    ];
    let keypair = Keypair::try_from(&keypair_bytes[..])?;
    let message = "The quick brown fox jumps over the lazy dog";

    let signature = keypair.sign_message(message.as_bytes());
    let is_valid_signature = signature.verify(&keypair.pubkey().to_bytes(), message.as_bytes());
    println!("Verified: {:?}", is_valid_signature);

    Ok(())
}

pub async fn testopentelemetry()-> Result<(), Box<dyn std::error::Error + Send + Sync>>{
      let addr = SocketAddr::from(([127, 0, 0, 1], 8080));


    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
pub async fn testSemaphore()->Result<()>{
  //信号量（Semaphore）是控制并发访问共享资源的同步原语，用于限制同时访问特定资源的线程或任务数量。
  let semaphore=Arc::new(Semaphore::new(3));
  let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];
  // 启动 10 个任务
  for i in 0..10{
    let semaphore=semaphore.clone();
     let handle = tokio::spawn(async move {
            // 获取一个许可
            let permit = semaphore.acquire().await.unwrap();
            println!("Task {} acquired permit", i);

            // 模拟工作
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            println!("Task {} releasing permit", i);
            // 退出作用域时自动释放许可
        });
        handles.push(handle);
  }
  // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
pub async fn testAcquireWithTimeout()->Result<(),anyhow::Error>{
    //限时获取和 try_acquire
    let semaphore=Arc::new(Semaphore::new(1));
    //获取许可
    let _permit=semaphore.acquire().await.unwrap();
        // 在另一个任务中尝试获取许可
    let semaphore_clone = semaphore.clone();
    let handle = tokio::spawn(async move {
        // 限时获取许可
        match tokio::time::timeout(
            Duration::from_millis(1000),
            semaphore_clone.acquire()
        ).await {
            Ok(Ok(permit)) => {
                println!("Successfully acquired permit after timeout");
                // Drop the permit immediately since we don't need it
                drop(permit);
                Some(())
            },
            Ok(Err(_)) => {
                println!("Semaphore closed");
                None
            },
            Err(_) => {
                println!("Timeout waiting for permit");
                None
            }
        }
    });

    let _result = handle.await.unwrap();

    // 尝试立即获取许可
    match semaphore.try_acquire() {
        Ok(permit) => println!("Immediate acquisition successful"),
        Err(_) => println!("Immediate acquisition failed - no permits available"),
    }
    
    Ok(())
}
