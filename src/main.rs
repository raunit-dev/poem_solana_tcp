use poem::{
    Route, Server, get, handler,
    listener::TcpListener,
    web::Json,
};
use solana_client::rpc_client::RpcClient;
use solana_program::{hash::hash, pubkey::Pubkey, system_instruction::transfer, account_info::Account};

use solana_sdk::{
    account_info::AccountInfo, client, commitment_config::CommitmentConfig, message::Message, native_token::LAMPORTS_PER_SOL, signature::{read_keypair_file, Keypair, Signer}, system_instruction, system_program, transaction::Transaction
};

const RPC_URL: &str = "https://api.devnet.solana.com";

use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::thread;

#[derive(Serialize, Deserialize)]
pub struct GetBalance {
    pub wallet: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetAirdrop {
    pub wallet: String,
    pub sol: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetBalanceRespose {
    pub wallet_add: String,
    pub balance_lamports: u64,
    pub balance_sol: f64,
}

#[derive(Serialize, Deserialize)]
pub struct GetAirdropRespose {
    pub wallet_add: String,
    pub previous_balance_lamports: u64,
    pub new_balance_lamports: u64,
    pub new_balance_sol: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Transfer {
    pub from_wallet: String,
    pub to_wallet: String,
    pub amount_sol: u64,
    pub from_private_key: String,
}


#[derive(Serialize, Deserialize)]
pub struct GetAccountInfo {
    pub wallet: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetAccountInfoResponse {
    pub wallet: String,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
}


#[handler]
fn get_balance(Json(data): Json<GetBalance>) -> Json<GetBalanceRespose> {
    let wallet_add = data.wallet;
    let client = RpcClient::new(RPC_URL);
    let wallet = wallet_add.parse::<Pubkey>().unwrap();
    let balance = client.get_balance(&wallet).unwrap();
    Json(GetBalanceRespose {
        wallet_add,
        balance_lamports: balance,
        balance_sol: balance as f64 / 1_000_000_000.0,
    })
}

#[handler]
fn get_airdrop(Json(data): Json<GetAirdrop>) -> Json<GetAirdropRespose> {
    let wallet_add = data.wallet;
    let airdro_amount = data.sol;
    let client = RpcClient::new(RPC_URL);
    let wallet = wallet_add.parse::<Pubkey>().unwrap();
    let old_balance = client.get_balance(&wallet).unwrap();

    match client.request_airdrop(&wallet, airdro_amount * LAMPORTS_PER_SOL) {
        Ok(s) => {
            println!("Success! Check out your TX here:");
            println!(
                "https://explorer.solana.com/tx/{}?cluster=devnet",
                s.to_string()
            );
        }
        Err(e) => println!("Opps , Kuchh to gadbad hai re baba: {}", e.to_string()),
    };

    thread::sleep(Duration::from_secs(10));

    let new_balance = client.get_balance(&wallet).unwrap();

    Json(GetAirdropRespose {
        wallet_add,
        previous_balance_lamports: old_balance,
        new_balance_lamports: new_balance,
        new_balance_sol: new_balance as f64 / 1_000_000_000.0,
    })
}

#[handler]
pub fn get_account_info(Json(data): Json<GetAccountInfo>) -> Json<GetAccountInfoResponse> {

    const MAIN_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

    let wallet_add = data.wallet;

    let wallet = wallet_add.parse::<Pubkey>().unwrap();

    let client = RpcClient::new_with_commitment(
        String::from(MAIN_RPC_URL),
        CommitmentConfig::confirmed()
    );

    let account = client.get_account(&wallet).unwrap();

    Json(GetAccountInfoResponse {
        wallet: wallet_add,
        lamports: account.lamports,
        data: account.data,
        owner: account.owner,
        executable: account.executable,
    })
}

#[handler]
pub fn transfer_sol_devnet(Json(data): Json<Transfer>) -> Json<Transfer> {
    let from_wallet = data.from_wallet;
    let to_wallet = data.to_wallet;
    let transfer_amount = data.amount_sol;
    let from_private_key = data.from_private_key;

    let from_pubkey = from_wallet.parse::<Pubkey>().unwrap();
    let to_pubkey = to_wallet.parse::<Pubkey>().unwrap();
    
    // Assuming the private key is a JSON array string (as exported by Solana CLI)
    let secret_bytes: Vec<u8> = serde_json::from_str(&from_private_key).expect("Invalid private key format");
    let from_signer = Keypair::from_bytes(&secret_bytes).expect("Failed to create keypair from bytes");

    let rpc_client = RpcClient::new(RPC_URL);

    let recent_blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");

    let transfer_ix = Transaction::new_signed_with_payer(
        &[transfer(
            &from_pubkey,
            &to_pubkey,
            transfer_amount
        )],
        Some(&from_pubkey),
        &vec![&from_signer],
        recent_blockhash
    );

    let signature = rpc_client.send_and_confirm_transaction(&transfer_ix).expect("Failed to send transaction");

    println!(
        "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
        signature
    );

    Json(Transfer {
        from_wallet,
        to_wallet,
        amount_sol: transfer_amount,
        from_private_key
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/get_balance", get(get_balance))
        .at("/get_airdrop", get(get_airdrop))
        .at("/transfer_sol_devnet", get(transfer_sol_devnet))
        .at("/get_account_info", get(get_account_info));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await
}