
import { ContractAddress } from '@concordium/web-sdk';
import { createContext } from 'react';


export type State = {
    isConnected: boolean;
    contractAddress: ContractAddress;
}

export const state = createContext<State>( {isConnected: false, contractAddress: {index: 1075n, subindex: 0n}} );

export const CONTRACT_NAME = "tictactoe";