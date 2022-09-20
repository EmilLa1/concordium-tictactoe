import React, { useState, useMemo, useEffect, useCallback } from 'react';

import { state, State } from './model';
import { Board } from './Board';

import { detectConcordiumProvider } from '@concordium/browser-wallet-api-helpers';


export default function Root() {
    const [account, setAccount] = useState<string>();
    const [isConnected, setIsConnected] = useState<boolean>(false);

    const handleGetAccount = useCallback((accountAddress: string | undefined) => {
        setAccount(accountAddress);
        setIsConnected(Boolean(accountAddress));
    }, []);

    const stateValue: State = useMemo(() => ({ isConnected }), [isConnected]);

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
    }
    );

    const [cells, updateCells] = useState<string[]>(["","","","","","","","",""]);

    function onCellClick(i: number) {
        let clls = [...cells];
        clls[i] = "X";
        updateCells(clls);
    }

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
                <Board cells={cells} onCellClick={onCellClick}></Board>
            </main>
        </state.Provider>
    )
}