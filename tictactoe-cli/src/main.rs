//! Basic example that shows how to initialize and update a smart contract.
use anyhow::Context;
use clap::AppSettings;
use concordium_rust_sdk::{
    common::{types::TransactionTime, Get, SerdeDeserialize, SerdeSerialize},
    endpoints,
    id::types::{AccountAddress, AccountKeys},
    types::{
        smart_contracts::{
            concordium_contracts_common::{
                from_bytes, Amount, ContractAddress, OwnedContractName, OwnedReceiveName, Serialize,
            },
            ModuleRef, Parameter, WasmModule,
        },
        transactions::{send, BlockItem, InitContractPayload, UpdateContractPayload},
        AccountInfo,
    },
};
use std::path::PathBuf;
use structopt::*;

#[derive(StructOpt)]
struct App {
    #[structopt(
        long = "node",
        help = "GRPC interface of the node.",
        default_value = "http://localhost:10000"
    )]
    endpoint:  endpoints::Endpoint,
    #[structopt(long = "account", help = "Path to the account key file.")]
    keys_path: PathBuf,
    #[structopt(subcommand, help = "The action you want to perform.")]
    action:    Action,
}

#[derive(SerdeSerialize, SerdeDeserialize)]
#[serde(rename_all = "camelCase")]
/// Account address and keys that will be supplied in a JSON file.
/// The transaction will be signed with the given keys.
struct AccountData {
    account_keys: AccountKeys,
    address:      AccountAddress,
}

#[derive(StructOpt)]
enum Action {
    #[structopt(about = "Deploy the contract")]
    Deploy {
        #[structopt(long, help = "The module to deploy.")]
        path: PathBuf,
    },
    #[structopt(about = "Initialize the contract")]
    Init {
        #[structopt(long, help = "The module references to initialize.")]
        module_ref: ModuleRef,
    },
    #[structopt(about = "Create a game")]
    CreateGame {
        #[structopt(long, help = "The contract address")]
        address: ContractAddress,
    },
    #[structopt(about = "Join a game")]
    JoinGame {
        #[structopt(long, help = "The game to join")]
        the_game: u64,
        #[structopt(long, help = "The contract address")]
        address:  ContractAddress,
    },
    #[structopt(about = "Make a move")]
    Move {
        #[structopt(long, help = "The game to join")]
        the_game: u64,
        #[structopt(long, help = "where to put your piece")]
        the_move: u64,
        #[structopt(long, help = "The contract address")]
        address:  ContractAddress,
    },
    ViewState {
        #[structopt(long, help = "The contract address")]
        address: ContractAddress,
    },
    ViewAccounts {
        #[structopt(long, help = "The game to join")]
        the_game: u64,
        #[structopt(long, help = "The contract address")]
        address:  ContractAddress,
    },
}

#[derive(Serialize)]
struct JoinParams {
    game_id: u64,
}

#[derive(Serialize)]
struct MakeMoveParams {
    game_id:  u64,
    the_move: u64,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let app = {
        let app = App::clap().global_setting(AppSettings::ColoredHelp);
        let matches = app.get_matches();
        App::from_clap(&matches)
    };

    let mut client = endpoints::Client::connect(app.endpoint, "rpcadmin").await?;

    // load account keys and sender address from a file
    let keys: AccountData = serde_json::from_str(
        &std::fs::read_to_string(app.keys_path).context("Could not read the keys file.")?,
    )
    .context("Could not parse the keys file.")?;

    let consensus_info = client.get_consensus_status().await?;
    // Get the initial nonce at the last finalized block.
    let acc_info: AccountInfo =
        client.get_account_info(&keys.address, &consensus_info.last_finalized_block).await?;

    let nonce = acc_info.account_nonce;
    // set expiry to now + 5min
    let expiry: TransactionTime =
        TransactionTime::from_seconds((chrono::Utc::now().timestamp() + 300) as u64);

    match app.action {
        Action::Deploy {
            path,
        } => {
            println!("DEPLOYING TIC TAC TOE");
            let bytes = std::fs::read(path)?;
            let module: WasmModule = std::io::Cursor::new(bytes).get()?;
            let mod_ref = module.get_module_ref();
            println!("MODULE_REF: {}", mod_ref);
            let tx = send::deploy_module(&keys.account_keys, keys.address, nonce, expiry, module);
            let item = BlockItem::AccountTransaction(tx);
            // submit the transaction to the chain
            let transaction_hash = client.send_block_item(&item).await?;
            println!("Transaction {} submitted (nonce = {}).", transaction_hash, nonce,);
            let (bh, bs) = client.wait_until_finalized(&transaction_hash).await?;
            println!("Transaction finalized in block {}.", bh);
            println!("The outcome is {:#?}", bs);
        }
        Action::Init {
            module_ref: mod_ref,
        } => {
            println!("Initializing TIC TAC TOE");
            let param = Parameter::default();
            let payload = InitContractPayload {
                amount: Amount::zero(),
                mod_ref,
                init_name: OwnedContractName::new_unchecked("init_tictactoe".to_string()),
                param,
            };

            let tx = send::init_contract(
                &keys.account_keys,
                keys.address,
                nonce,
                expiry,
                payload,
                10000u64.into(),
            );
            let item = BlockItem::AccountTransaction(tx);
            // submit the transaction to the chain
            let transaction_hash = client.send_block_item(&item).await?;
            println!("Transaction {} submitted (nonce = {}).", transaction_hash, nonce,);
            let (bh, bs) = client.wait_until_finalized(&transaction_hash).await?;
            println!("Transaction finalized in block {}.", bh);
            println!("The outcome is {:#?}", bs);
        }
        Action::CreateGame {
            address,
        } => {
            // empty param
            let message = Parameter::default();
            let payload = UpdateContractPayload {
                amount: Amount::zero(),
                address,
                receive_name: OwnedReceiveName::new_unchecked("tictactoe.create_game".to_string()),
                message,
            };

            let tx = send::update_contract(
                &keys.account_keys,
                keys.address,
                nonce,
                expiry,
                payload,
                1000000u64.into(),
            );
            let item = BlockItem::AccountTransaction(tx);
            // submit the transaction to the chain
            let transaction_hash = client.send_block_item(&item).await?;
            println!("Transaction {} submitted (nonce = {}).", transaction_hash, nonce,);
            let (bh, bs) = client.wait_until_finalized(&transaction_hash).await?;
            println!("Transaction finalized in block {}.", bh);
            println!("The outcome is {:#?}", bs);
        }
        Action::JoinGame {
            the_game,
            address,
        } => {
            // the game we want to join.
            let params = JoinParams {
                game_id: the_game,
            };

            let message = Parameter::from(
                concordium_rust_sdk::types::smart_contracts::concordium_contracts_common::to_bytes(
                    &params,
                ),
            );

            let payload = UpdateContractPayload {
                amount: Amount::zero(),
                address,
                receive_name: OwnedReceiveName::new_unchecked("tictactoe.join_game".to_string()),
                message,
            };

            let tx = send::update_contract(
                &keys.account_keys,
                keys.address,
                nonce,
                expiry,
                payload,
                1000000u64.into(),
            );
            let item = BlockItem::AccountTransaction(tx);
            // submit the transaction to the chain
            let transaction_hash = client.send_block_item(&item).await?;
            println!("Transaction {} submitted (nonce = {}).", transaction_hash, nonce,);
            let (bh, bs) = client.wait_until_finalized(&transaction_hash).await?;
            println!("Transaction finalized in block {}.", bh);
            println!("The outcome is {:#?}", bs);
        }
        Action::Move {
            the_game,
            the_move,
            address,
        } => {
            // the game we want to join.
            let params = MakeMoveParams {
                game_id: the_game,
                the_move,
            };

            let message = Parameter::from(
                concordium_rust_sdk::types::smart_contracts::concordium_contracts_common::to_bytes(
                    &params,
                ),
            );

            let payload = UpdateContractPayload {
                amount: Amount::zero(),
                address,
                receive_name: OwnedReceiveName::new_unchecked("tictactoe.make_move".to_string()),
                message,
            };

            let tx = send::update_contract(
                &keys.account_keys,
                keys.address,
                nonce,
                expiry,
                payload,
                10000u64.into(),
            );
            let item = BlockItem::AccountTransaction(tx);
            // submit the transaction to the chain
            let transaction_hash = client.send_block_item(&item).await?;
            println!("Transaction {} submitted (nonce = {}).", transaction_hash, nonce,);
            let (bh, bs) = client.wait_until_finalized(&transaction_hash).await?;
            println!("Transaction finalized in block {}.", bh);
            println!("The outcome is {:#?}", bs);
        }
        Action::ViewState {
            address,
        } => {
            let message = Parameter::default();
            let ctx = concordium_rust_sdk::types::smart_contracts::ContractContext {
                invoker:   None,
                contract:  address,
                amount:    Amount::zero(),
                method:    OwnedReceiveName::new_unchecked("tictactoe.view".to_string()),
                parameter: message,
                energy:    10000000u64.into(),
            };

            match client.invoke_contract(&consensus_info.last_finalized_block, &ctx).await {
                Ok(res) => {
                    match res {
                        concordium_rust_sdk::types::smart_contracts::InvokeContractResult::Success { return_value, events: _, used_energy: _ } => {
                            if let Some(view_value) = return_value {
                                let view_state: ViewState = from_bytes(&view_value.value)?;
                                println!("{:?}",view_state);
                            }
                        },
                        concordium_rust_sdk::types::smart_contracts::InvokeContractResult::Failure { return_value: _, reason, used_energy: _ } => {
                            eprintln!("Failed invoking contract {:?}", reason);
                        },
                    }
                }
                Err(err) => eprintln!("Could not invoke contract: {}", err),
            }
        }
        Action::ViewAccounts {
            the_game,
            address,
        } => {
            let params = JoinParams {
                game_id: the_game,
            };

            let message = Parameter::from(
                concordium_rust_sdk::types::smart_contracts::concordium_contracts_common::to_bytes(
                    &params,
                ),
            );
            let ctx = concordium_rust_sdk::types::smart_contracts::ContractContext {
                invoker:   None,
                contract:  address,
                amount:    Amount::zero(),
                method:    OwnedReceiveName::new_unchecked(
                    "tictactoe.game_view_players".to_string(),
                ),
                parameter: message,
                energy:    10000000u64.into(),
            };

            match client.invoke_contract(&consensus_info.last_finalized_block, &ctx).await {
                Ok(res) => {
                    match res {
                        concordium_rust_sdk::types::smart_contracts::InvokeContractResult::Success { return_value, events: _, used_energy: _ } => {
                            if let Some(view_value) = return_value {
                                let view_state: Vec<u8> = from_bytes(&view_value.value)?;
                                println!("{:?}",view_state);
                            }
                        },
                        concordium_rust_sdk::types::smart_contracts::InvokeContractResult::Failure { return_value: _, reason, used_energy: _ } => {
                            eprintln!("Failed invoking contract {:?}", reason);
                        },
                    }
                }
                Err(err) => eprintln!("Could not invoke contract: {}", err),
            }
        }
    };

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ViewState {
    pub games: std::collections::BTreeMap<u64, Game>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Copy)]
pub enum Player {
    Cross(AccountAddress),
    Circle(AccountAddress),
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone, Copy)]
pub enum GameState {
    AwaitingOpponent,
    InProgress(Player),
    Finished(Option<Player>), // None if it was a draw, otherwise it contains the winning player.
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
enum Cell {
    Empty,
    Occupied(Player),
}
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct Board([Cell; 9]);

/// A game of tic tac toe!
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub struct Game {
    pub game_state: GameState,
    pub board:      Board,
    pub cross:      Player,
    pub circle:     Option<Player>,
}
