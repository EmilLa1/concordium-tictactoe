import React, { } from 'react';

interface BoardProps {
    cells : string[];
    onCellClick : (i: number) => void;
}

export const Board = ({cells, onCellClick}: BoardProps) => {
    function renderCell(i : number) {
        return (
            <button className="board-cell" onClick={() => onCellClick(i)}>{cells[i]}</button>
        )
    }
    return (
        <div>
            <div className="board-row">
                {renderCell(0)}
                {renderCell(1)}
                {renderCell(2)}
            </div>
            <div className="board-row">
                {renderCell(3)}
                {renderCell(4)}
                {renderCell(5)}
            </div>
            <div className="board-row">
                {renderCell(6)}
                {renderCell(7)}
                {renderCell(8)}
            </div>
        </div>
    )
};
