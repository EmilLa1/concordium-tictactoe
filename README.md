# TicTacToe dApp example

The example project in this repository is a working example of integrating a smart contract on the Cncordium blockchain. It includes a smart contract for the TicTacToe game and a frontend integrating with the Concordium browser wallet.

## Prerequisites

- Browser wallet extension must be installed in google chrome and configured with testnet JSON-RPC, in order to view smart contract details or submit transactions.
- `cargo concordium` for building the smart contract.

## Installing
### Clone the project:
```
git clone git@github.com:EmilLa1/concordium-tictactoe.git --recurse-submodules
```

### tictactoe
The smart contract implementing tic tac toe.

#### Build

Build the smart contract with:
```
cargo concordium build --out tictactoe.wasm.v1 --schema-out tictactoeschema.bin
```

The schema is required for when updating the contract with parameters.

### tictactoe-cli
A simple CLI for interacting with the smart contract. 

#### Deploy
```
cargo run --release -- --node $NODE --account $PATH_TO_EXPORTED_ACCOUNT deploy --path $PATH_TO_COMPILED_SMART_CONTRACT
```

#### Initialize the contract
```
cargo run --release -- --node $NODE --account $PATH_TO_EXPORTED_ACCOUNT init --module-ref $MODULE_REF
```
`$MODULE_REF` is written to stdout when deploying.

#### Create a game
```
cargo run --release -- --node $NODE --account $PATH_TO_EXPORTED_ACCOUNT create-game --address "<$INDEX,$SUB_INDEX>" 
``` 
The contract address consists of:
- `$INDEX` is written to stdout when initializing the contract.
- `$SUB_INDEX` is written to stdout when initializing the contract.

#### Join a game
```
cargo run --release -- --node $NODE --account $PATH_TO_EXPORED_ACCOUNT join-game --address "<$INDEX,$SUB_INDEX>" --the-game $GAME_INDEX
``` 

`$GAME_INDEX` is the index of the game created with `create-game`. This is obtainable by invoking the view function ´view-state` mentioned below.

#### Make a move
```
cargo run --release -- --node $NODE --account $PATH_TO_EXPORED_ACCOUNT move --address "<$INDEX,$SUB_INDEX>" --the-game $GAME_INDEX --the-move $THE_MOVE
```
`$THE_MOVE` is an unsigned number which determines where to put either ones circle or cross. The board simply consists of an array of size 9.
So upper left corner is index `0`, upper right corner is `2`, lower left corner is `6` and lower right corner is `8`.

#### View the whole state
```
cargo run --release -- --node $NODE --account $PATH_TO_EXPORED_ACCOUNT view-state --address "<$INDEX,$SUB_INDEX>"
```

#### View participants of the given game.

```
cargo run --release -- --node $NODE --account $PATH_TO_EXPORED_ACCOUNT view-accounts --address "<$INDEX,$SUB_INDEX>" --the-game $GAME_INDEX
```

### UI 
TBD
