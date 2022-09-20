
import { createContext } from 'react';


export type State = {
    isConnected: boolean;
}

export const state = createContext<State>( {isConnected: false} );