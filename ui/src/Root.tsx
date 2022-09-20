import React, { useState, useMemo, useEffect, useCallback } from 'react';

import { state, State } from './model';
import { Board } from './Board';

import { detectConcordiumProvider } from '@concordium/browser-wallet-api-helpers';
import GameState from './GameState';

const CONTRACT_ADDRESS = {index: 1075n, subindex: 0n};


export default function Root() {
    const [account, setAccount] = useState<string>();
    const [isConnected, setIsConnected] = useState<boolean>(false);

    const handleGetAccount = useCallback((accountAddress: string | undefined) => {
        setAccount(accountAddress);
        setIsConnected(Boolean(accountAddress));
    }, []);

    const stateValue: State = useMemo(() => ({ isConnected, contractAddress: CONTRACT_ADDRESS}), [isConnected, CONTRACT_ADDRESS]);

    useEffect(() => {
        detectConcordiumProvider()
            .then((provider) => {
                // Listen for events from the wallet.
                provider.on('accountChanged', setAccount);
                provider.on('accountDisconnected', () =>
                    provider.getMostRecentlySelectedAccount().then(handleGetAccount)
                );
                provider.getMostRecentlySelectedAccount().then(handleGetAccount);
            })
            .catch(() => setIsConnected(false));
    }, []);

    // const [cells, updateCells] = useState<string[]>(["", "", "", "", "", "", "", "", ""]);

    // function onCellClick(i: number) {
    //     let clls = [...cells];
    //     clls[i] = "X";
    //     updateCells(clls);
    // }

    return (
        <state.Provider value={stateValue}>
            <main className="tictactoe">
                <div className={`connection-banner ${isConnected ? 'connected' : ''}`}>
                    {isConnected && (
                        <>
                            Playing as {account}.
                        </>
                    )}
                    {!isConnected && (
                        <>
                            <p>No wallet connection</p>
                        </>
                    )}
                </div>
                <div>Hello world!</div>
                <GameState gameId={0n}></GameState>
            </main>
        </state.Provider>
    )
}