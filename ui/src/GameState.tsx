import { detectConcordiumProvider } from '@concordium/browser-wallet-api-helpers';
import React, { useContext, useState } from 'react';
import { Board } from './Board';
import { CONTRACT_NAME, state } from './model'

type GameStateProps = {
    gameId: bigint
}

export default function GameState(props: GameStateProps) {
    const { isConnected, contractAddress } = useContext(state);

    const [cells, updateCells] = useState<string[]>(["", "", "", "", "", "", "", "", ""]);


    async function updateState(): Promise<void> {
        const provider = await detectConcordiumProvider();
        const res = await provider.getJsonRpcClient().invokeContract(
            {
                method: `${CONTRACT_NAME}.view`,
                contract: contractAddress
            }
        );
        if (!res || res.tag === 'failure' || !res.returnValue) {
            throw new Error(`Expected successful invocation`);
        }

    }

    return (
        <>
            <Board cells={cells} onCellClick={(_) => {}}></Board>
            <button type="button" onClick={() => updateState()}>↻</button>
        </>
    )
}